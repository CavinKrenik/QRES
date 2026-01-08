use crate::brain_aggregator::{apply_aggregated_confidence, BrainAggregator};
use crate::config::Config;
use crate::living_brain::{BrainMessage, LivingBrain};
use crate::peer_keys::PeerKeyStore;
use crate::security::{SecurityManager, SignedPayload};
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
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// Topic for brain synchronization
const BRAIN_TOPIC: &str = "qres-brain-sync";

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
    pub require_signatures: bool,
    pub aggregator: BrainAggregator,
    pub config: Config,
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

    // Shared State
    let state = Arc::new(RwLock::new(AppState {
        local_peer_id: peer_id.to_string(),
        connected_peers: HashSet::new(),
        known_peers: HashSet::new(),
        brain: LivingBrain::default(),
        peer_keys,
        security,
        require_signatures: config.security.require_signatures,
        aggregator: BrainAggregator::new(config.aggregation.clone()),
        config,
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
    let mut last_broadcast_brain: Option<LivingBrain> = None;

    loop {
        tokio::select! {
            // Periodic Brain Broadcast with Delta Encoding
            _ = interval.tick() => {
                if let Ok(content) = fs::read_to_string(brain_file) {
                    if let Some(current_brain) = LivingBrain::from_json(&content) {
                        // Update RAM state
                        state.write().await.brain = current_brain.clone();

                        // Prepare outgoing brain (potentially with privacy noise)
                        let mut outgoing_brain = current_brain.clone();
                        let privacy_config = { state.read().await.config.privacy.clone() };

                        if privacy_config.enabled {
                            let dp = qres_core::privacy::DifferentialPrivacy::new(
                                privacy_config.epsilon as f64,
                                privacy_config.delta as f64,
                                privacy_config.clipping_threshold as f64,
                            );

                            // Apply clipping and noise
                            dp.clip_update(&mut outgoing_brain.confidence);
                            if let Err(e) = dp.add_noise(&mut outgoing_brain.confidence) {
                                tracing::error!("Failed to apply privacy noise: {}", e);
                            } else {
                                tracing::debug!(epsilon = privacy_config.epsilon, "Applied differential privacy to update");
                            }
                        }

                        // Delta Encoding Logic using the (potentially noisy) outgoing brain
                        let message = if let Some(last) = &last_broadcast_brain {
                            outgoing_brain.diff(last).map(BrainMessage::Delta)
                        } else {
                            Some(BrainMessage::Full(outgoing_brain.clone()))
                        };

                        if let Some(msg) = message {
                            let topic = IdentTopic::new(BRAIN_TOPIC);
                            if let Ok(brain_payload) = serde_json::to_vec(&msg) {
                                // Sign the message if security is enabled
                                let final_payload = {
                                    let app_state = state.read().await;
                                    if let Some(sec_mgr) = &app_state.security {
                                        let signed = sec_mgr.sign(&brain_payload);
                                        serde_json::to_vec(&signed).unwrap_or(brain_payload.clone())
                                    } else {
                                        brain_payload
                                    }
                                };

                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, final_payload) {
                                    tracing::error!("P2P publish error: {:?}", e);
                                } else {
                                    match &msg {
                                        BrainMessage::Delta(d) => info!(updates = d.updates.len(), signed = state.read().await.security.is_some(), "Broadcasted Delta"),
                                        BrainMessage::Full(_) => info!(signed = state.read().await.security.is_some(), "Broadcasted Full Wisdom"),
                                    }
                                    // Update last broadcast to the one we actually sent (noisy)
                                    last_broadcast_brain = Some(outgoing_brain);
                                }
                            }
                        } else {
                            info!("No significant changes - skipping broadcast");
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
                    info!("Received Merge Candidate");

                    // Try to extract the brain message - either from SignedPayload or raw
                    let brain_data: Option<Vec<u8>> = {
                        let mut app_state = state.write().await;

                        // First, try to parse as SignedPayload
                        if let Ok(signed) = serde_json::from_slice::<SignedPayload>(&message.data) {
                            // Verify the signature if required
                            if app_state.require_signatures {
                                if let Some(ref mut sec) = app_state.security {
                                    match sec.verify(&signed) {
                                        Ok(data) => {
                                            info!(signer = %signed.signer_pubkey[..16], "Signature verified");
                                            Some(data)
                                        }
                                        Err(e) => {
                                            warn!(error = %e, signer = %signed.signer_pubkey[..16], "Rejecting message: signature verification failed");
                                            None
                                        }
                                    }
                                } else {
                                    // No SecurityManager but signatures required - reject
                                    warn!("Rejecting signed message: no SecurityManager configured");
                                    None
                                }
                            } else {
                                // Signatures not required - accept signed payload data without verification
                                info!("Accepting signed payload (verification not required)");
                                Some(signed.data)
                            }
                        } else {
                            // Not a SignedPayload - check if we require signatures
                            if app_state.require_signatures {
                                warn!("Rejecting unsigned message: signatures required");
                                None
                            } else {
                                // Accept raw message for backward compatibility
                                Some(message.data.clone())
                            }
                        }
                    };

                    // Process the brain message if we have verified data
                    if let Some(data) = brain_data {
                        if let Ok(json) = String::from_utf8(data) {
                            if let Ok(msg) = serde_json::from_str::<BrainMessage>(&json) {
                                match msg {
                                    BrainMessage::Full(remote_brain) => {
                                        // Buffer the update for robust aggregation
                                        let aggregated = {
                                            let mut app_state = state.write().await;
                                            app_state.aggregator.add_update(&remote_brain)
                                        };

                                        // If we have enough updates, apply aggregated result
                                        if let Some(agg_confidence) = aggregated {
                                            if let Ok(local_json) = fs::read_to_string(brain_file) {
                                                if let Some(mut local_brain) = LivingBrain::from_json(&local_json) {
                                                    // Apply aggregated confidence with alpha blend
                                                    apply_aggregated_confidence(&mut local_brain, &agg_confidence, 0.1);
                                                    let _ = fs::write(brain_file, local_brain.to_json());
                                                    info!("Assimilated Aggregated Knowledge (robust)");
                                                    state.write().await.brain = local_brain;
                                                }
                                            }
                                        } else {
                                            info!(buffered = state.read().await.aggregator.buffer_len(), "Buffered update for aggregation");
                                        }
                                    }
                                    BrainMessage::Delta(delta) => {
                                        // Deltas are applied immediately (already small incremental updates)
                                        if let Ok(local_json) = fs::read_to_string(brain_file) {
                                            if let Some(mut local_brain) = LivingBrain::from_json(&local_json) {
                                                local_brain.apply_delta(&delta);
                                                let _ = fs::write(brain_file, local_brain.to_json());
                                                info!(updates = delta.updates.len(), "Applied Knowledge Delta (verified)");
                                                state.write().await.brain = local_brain;
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
