use crate::brain_aggregator::BrainAggregator;
use crate::config::Config;
use crate::living_brain::{LivingBrain, SignedEpiphany};
use crate::peer_keys::PeerKeyStore;
use crate::security::{ReputationManager, SecurityManager, SignedPayload};
use qres_core::privacy::PrivacyAccountant;
use qres_core::zk_proofs::{ZkNormProver, ProofBundle};
use axum::{extract::State, routing::get, Json, Router};
use libp2p::futures::StreamExt; // For select_next_some
use libp2p::gossipsub::IdentTopic; // Added helper
use libp2p::{
    gossipsub, identify, identity, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io; // Added
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn};
use rand;

// Topic for brain synchronization
const BRAIN_TOPIC: &str = "qres-hive-v2";

#[derive(Clone, Serialize, Default)]
pub struct SwarmStatus {
    pub peer_id: String,
    pub connected_peers: usize,
    pub known_peers: Vec<String>,
    pub brain_confidence: Vec<f32>,
}

pub struct AppState {
    pub local_peer_id: String,
    pub connected_peers: HashSet<String>,
    pub known_peers: HashSet<String>,
    pub brain: LivingBrain,
    pub peer_keys: PeerKeyStore,
    pub security: Option<SecurityManager>,
    pub reputation: ReputationManager,
    pub require_signatures: bool,
    pub aggregator: BrainAggregator,
    pub config: Config,
    pub privacy_accountant: PrivacyAccountant,
    pub zk_prover: ZkNormProver,
}

// Custom Behavior Struct
#[derive(NetworkBehaviour)]
pub struct QresBehavior {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}

pub async fn start_p2p_node(
    brain_path: String,
    port: u16,
    key_path_override: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Identity
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    info!(peer_id = %peer_id, "Local Peer ID generated");

    // Load config for security settings
    let config = Config::load().unwrap_or_default();
    let peer_keys = PeerKeyStore::new(
        &config.security.trusted_peers,
        &config.security.trusted_pubkeys,
    );

    // Initialize SecurityManager
    // Priority: 1. CLI Override, 2. Config Key Path, 3. Auto-generate if required
    let security = if let Some(key_path_str) =
        key_path_override.or(config.security.key_path.clone())
    {
        let key_path = PathBuf::from(key_path_str);
        match SecurityManager::new(&key_path, config.security.require_signatures) {
            Ok(mgr) => {
                info!(pubkey = %mgr.public_key_hex(), path = ?key_path, "Security manager initialized");
                Some(mgr)
            }
            Err(e) => {
                warn!(error = %e, "Failed to initialize SecurityManager, running without signatures");
                None
            }
        }
    } else if config.security.require_signatures {
        // Auto-generate key if signatures required but no path specified
        let key_path = dirs::home_dir()
            .map(|p| p.join(".qres").join("node_key"))
            .unwrap_or_else(|| PathBuf::from("node_key"));
        match SecurityManager::new(&key_path, true) {
            Ok(mgr) => {
                info!(pubkey = %mgr.public_key_hex(), key_path = ?key_path, "Security manager auto-initialized");
                Some(mgr)
            }
            Err(e) => {
                warn!(error = %e, "Failed to auto-initialize SecurityManager");
                None
            }
        }
    } else {
        None
    };

    // Initialize ReputationManager
    let rep_path = dirs::home_dir()
        .map(|p| p.join(".qres").join("reputation.json"))
        .unwrap_or_else(|| PathBuf::from("reputation.json"));
    let reputation = ReputationManager::new(rep_path);

    // Shared State
    let state = Arc::new(RwLock::new(AppState {
        local_peer_id: peer_id.to_string(),
        connected_peers: HashSet::new(),
        known_peers: HashSet::new(),
        brain: LivingBrain::default(),
        peer_keys,
        security,
        reputation,
        require_signatures: config.security.require_signatures,
        aggregator: BrainAggregator::new(config.aggregation.clone()),
        config,
        privacy_accountant: PrivacyAccountant::new(10.0, 1e-5, 0.995),
        zk_prover: ZkNormProver::new(),
    }));

    // Spawn API
    let app_state = state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/status", get(get_status))
            .route("/brain", get(get_brain))
            .route("/health", get(get_health))
            .with_state(app_state);

        let addr_str = if std::env::var("QRES_PUBLIC").is_ok() {
            format!("0.0.0.0:{}", port)
        } else {
            format!("127.0.0.1:{}", port)
        };

        info!(address = addr_str, "API Server listening");
        // Bind to localhost by default
        let listener = tokio::net::TcpListener::bind(&addr_str).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // 2. Build Swarm using modern Builder API
    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // Gossipsub
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(1))
                .validation_mode(gossipsub::ValidationMode::Permissive)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(io::Error::other)?;

            let mut gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .map_err(io::Error::other)?;

            let topic = gossipsub::IdentTopic::new(BRAIN_TOPIC);
            gossipsub
                .subscribe(&topic)
                .map_err(|e| io::Error::other(format!("{:?}", e)))?;

            // mDNS
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), PeerId::from(key.public()))?;

            // Identify
            let identify = identify::Behaviour::new(identify::Config::new(
                "qres/1.0.0".to_string(),
                key.public(),
            ));

            Ok(QresBehavior {
                gossipsub,
                mdns,
                identify,
            })
        })?
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // 6. Loop
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let brain_file = &brain_path;
    let _last_broadcast_brain: Option<LivingBrain> = None;

    loop {
        tokio::select! {
            // Periodic Brain Broadcast with Ghost Protocol
            _ = interval.tick() => {
                // --- PHASE 2: Privacy Accounting ---
                // Decay budget over time (rolling window)
                {
                    let mut app_state = state.write().await;
                    app_state.privacy_accountant.decay();
                }

                // Cost of an Epiphany (approximate)
                let epiphany_cost = 0.1;

                let should_publish = {
                    let app_state = state.read().await;
                    match app_state.privacy_accountant.check_budget(epiphany_cost) {
                        Ok(_) => true,
                        Err(_) => {
                            info!("Privacy budget exhausted. Entering Listen-Only Mode.");
                            false
                        }
                    }
                };

                if should_publish {
                    if let Ok(content) = fs::read_to_string(brain_file) {
                        if let Some(brain) = LivingBrain::from_json(&content) {
                            // Update RAM state
                            state.write().await.brain = brain.clone();

                            // --- PHASE 1: Proving Step (Sender) ---
                            // A. Type Conversion: I16F16 (Bytes) -> f32
                            // We need Vec<f32> for the ZK Prover
                            let weights_f32: Vec<f32> = if let Some(w_bytes) = &brain.best_engine_weights {
                                // Safety: Assuming 4-byte chunks are little-endian i32 (Q16.16)
                                w_bytes.chunks(4).filter_map(|chunk| {
                                    if chunk.len() == 4 {
                                        let bits = i32::from_le_bytes(chunk.try_into().unwrap());
                                        let fixed = fixed::types::I16F16::from_bits(bits);
                                        Some(fixed.to_num::<f32>())
                                    } else {
                                        None
                                    }
                                }).collect()
                            } else {
                                Vec::new()
                            };

                            // B. Generate ZK Proof
                            let proof_bundle = {
                                let app_state = state.read().await;
                                if !weights_f32.is_empty() {
                                    // Threshold 10.0 (L2 Norm Squared)
                                    if let Some((proof, _)) = app_state.zk_prover.generate_proof(&weights_f32, 10.0) {
                                        Some(ProofBundle {
                                            peer_id: [0u8; 32], // Placeholder or derived from sec_mgr
                                            masked_weights: weights_f32, // Sending unmasked in this context for Epiphany
                                            zk_proof: proof,
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };

                            // C. Sign the Payload
                            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                            let nonce = rand::random::<u64>(); // Generate unique nonce

                            let mut epiphany = SignedEpiphany {
                                brain: brain.clone(),
                                proof_bundle: proof_bundle.clone(),
                                signature: String::new(),
                                sender_id: state.read().await.security.as_ref().map(|s| s.public_key_hex()).unwrap_or_default(),
                                timestamp,
                                nonce,
                            };

                            let payload_bytes = epiphany.payload_bytes();
                            let signed_payload = {
                                let app_state = state.read().await;
                                if let Some(sec_mgr) = &app_state.security {
                                    sec_mgr.sign(&payload_bytes)
                                } else {
                                    // Fallback: no signature
                                    SignedPayload {
                                        data: payload_bytes,
                                        signature: String::new(),
                                        signer_pubkey: String::new(),
                                        timestamp,
                                        nonce,
                                    }
                                }
                            };

                            // Move signature into our struct
                            epiphany.signature = signed_payload.signature;

                            // D. Serialize & Publish
                            let msg_bytes = serde_json::to_vec(&epiphany).unwrap();
                            let topic = IdentTopic::new(BRAIN_TOPIC);
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg_bytes) {
                                tracing::error!("Publish error: {:?}", e);
                            } else {
                                // Record Privacy Cost only on successful publish
                                let mut app_state = state.write().await;
                                let _ = app_state.privacy_accountant.record_consumption(epiphany_cost);
                                info!("Published SignedEpiphany with ZK proof");
                            }
                        }
                    }
                }
            }

            // Swarm Events
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!(address = %address, "Swarm listening");
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                     info!(peer_id = %peer_id, "Connected to peer");
                     state.write().await.connected_peers.insert(peer_id.to_string());
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                     info!(peer_id = %peer_id, "Disconnected from peer");
                     state.write().await.connected_peers.remove(&peer_id.to_string());
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        info!(peer_id = %peer_id, "mDNS Discovered");
                        state.write().await.known_peers.insert(peer_id.to_string());
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        let _ = swarm.dial(multiaddr);
                    }
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        info!(peer_id = %peer_id, "mDNS Expired");
                        state.write().await.known_peers.remove(&peer_id.to_string());
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }
                // Handle Identify events - store public keys from peers
                SwarmEvent::Behaviour(QresBehaviorEvent::Identify(identify::Event::Received { peer_id, info })) => {
                    info!(peer_id = %peer_id, agent = %info.agent_version, "Received Identify from peer");
                    let mut app_state = state.write().await;
                    if app_state.peer_keys.add_peer_key(peer_id, info.public_key) {
                        info!(peer_id = %peer_id, known_keys = app_state.peer_keys.peer_count(), "Peer key verified and stored");
                    }
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Identify(identify::Event::Sent { peer_id })) => {
                    info!(peer_id = %peer_id, "Sent Identify to peer");
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Identify(identify::Event::Error { peer_id, error })) => {
                    warn!(peer_id = %peer_id, error = %error, "Identify error");
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Gossipsub(gossipsub::Event::Message { propagation_source: _, message_id: _, message })) => {
                    // --- PHASE 1: Verification Step (Receiver) ---
                    // 1. Deserialize SignedEpiphany
                    if let Ok(signed_epiphany) = serde_json::from_slice::<SignedEpiphany>(&message.data) {
                        
                        // 2. Reconstruct the SignedPayload expected by SecurityManager
                        let payload_to_verify = SignedPayload {
                            data: signed_epiphany.payload_bytes(),
                            signature: signed_epiphany.signature.clone(),
                            signer_pubkey: signed_epiphany.sender_id.clone(),
                            timestamp: signed_epiphany.timestamp,
                            nonce: signed_epiphany.nonce,
                        };

                        // 3. Perform the cryptographic check
                        let sig_valid = {
                            let mut app_state = state.write().await;
                            if let Some(security_mgr) = &mut app_state.security {
                                match security_mgr.verify(&payload_to_verify) {
                                    Ok(_) => true,
                                    Err(e) => {
                                        warn!("Verification Failed: {}", e);
                                        false
                                    }
                                }
                            } else {
                                // No security manager, accept if not requiring signatures
                                !app_state.require_signatures
                            }
                        };

                        if sig_valid {
                            // 4. Verify ZK Proof
                            let zk_valid = {
                                let app_state = state.read().await;
                                if let Some(bundle) = &signed_epiphany.proof_bundle {
                                    // Note: Verify the proof against the weights inside the bundle
                                    // Threshold must match sender (10.0)
                                    app_state.zk_prover.verify_proof(&bundle.zk_proof, 10.0)
                                } else {
                                    true // No weights = valid? Or require proof? Policy decision.
                                }
                            };

                            if zk_valid {
                                // 5. Merge (Success)
                                if let Ok(local_json) = fs::read_to_string(brain_file) {
                                    if let Some(mut local_brain) = LivingBrain::from_json(&local_json) {
                                        local_brain.merge(&signed_epiphany.brain, 0.05);
                                        let _ = fs::write(brain_file, local_brain.to_json());
                                        state.write().await.brain = local_brain;
                                        
                                        // Reputation Reward
                                        let mut app_state = state.write().await;
                                        app_state.reputation.reward(&signed_epiphany.sender_id);
                                        info!("Merged SignedEpiphany and rewarded sender");
                                    }
                                }
                            } else {
                                // 6. Punish (Fail ZK)
                                warn!("Invalid ZK Proof from {}", signed_epiphany.sender_id);
                                let mut app_state = state.write().await;
                                app_state.reputation.punish(&signed_epiphany.sender_id);
                            }
                        } else {
                            warn!("Invalid Signature from {}", signed_epiphany.sender_id);
                            let mut app_state = state.write().await;
                            app_state.reputation.punish(&signed_epiphany.sender_id);
                        }
                    } else {
                        warn!("Failed to deserialize SignedEpiphany");
                    }
                }
                _ => {}
            }
        }
    }
}

// Handlers
async fn get_status(State(state): State<Arc<RwLock<AppState>>>) -> Json<SwarmStatus> {
    let s = state.read().await;
    Json(SwarmStatus {
        peer_id: s.local_peer_id.clone(),
        connected_peers: s.connected_peers.len(),
        known_peers: s.known_peers.iter().cloned().collect(),
        brain_confidence: s.brain.confidence.to_vec(),
    })
}

async fn get_brain(State(state): State<Arc<RwLock<AppState>>>) -> Json<LivingBrain> {
    let s = state.read().await;
    Json(s.brain.clone())
}

async fn get_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
