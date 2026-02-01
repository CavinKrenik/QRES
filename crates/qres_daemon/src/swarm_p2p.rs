use crate::brain_aggregator::{BrainAggregator, FederatedAverager};
use crate::config::Config;
use crate::living_brain::{LivingBrain, SignedEpiphany};
use crate::peer_keys::PeerKeyStore;
use crate::security::{ReputationManager, SecurityManager, SignedPayload};
use crate::stats::SingularityMetrics;
use axum::{extract::State, routing::get, Json, Router};
use fixed::types::I16F16;
use libp2p::futures::StreamExt; // For select_next_some
use libp2p::gossipsub::IdentTopic; // Added helper
use libp2p::{
    gossipsub, identify, identity, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use qres_core::adaptive::regime_detector::{Regime, RegimeDetector};
use qres_core::consensus::krum::Bfp16Vec; // v19.0 Bfp16Vec
use qres_core::privacy::PrivacyAccountant;
use qres_core::tensor::{FixedTensor, I8F8};
use qres_core::zk_proofs::{ProofBundle, ZkNormProver};
use rand;
use serde::{Deserialize, Serialize}; // Added Deserialize
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn};

// Topic for brain synchronization
const BRAIN_TOPIC: &str = "qres-hive-v2";

// v19.0: Summary Gene for Fast Onboarding
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SummaryGene {
    pub round_index: u64,
    pub history_hash: [u8; 32],
    pub consensus: Bfp16Vec,
    pub variance: Bfp16Vec,
}

impl SummaryGene {
    pub fn new(round: u64, hash: [u8; 32], consensus: &[f32], variance: &[f32]) -> Self {
        Self {
            round_index: round,
            history_hash: hash,
            consensus: Bfp16Vec::from_f32_slice(consensus),
            variance: Bfp16Vec::from_f32_slice(variance),
        }
    }

    /// Serialize to optimized binary format for packet size verification
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(200);
        bytes.extend_from_slice(&self.round_index.to_be_bytes());
        bytes.extend_from_slice(&self.history_hash);

        // Manual BFP compression using qres_core arithmetic module
        bytes.extend(qres_core::encoding::arithmetic::compress_bfp(
            self.consensus.exponent,
            &self.consensus.mantissas,
        ));
        bytes.extend(qres_core::encoding::arithmetic::compress_bfp(
            self.variance.exponent,
            &self.variance.mantissas,
        ));

        bytes
    }
}

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
    pub federated_averager: FederatedAverager,
    pub config: Config,
    pub privacy_accountant: PrivacyAccountant,
    pub zk_prover: ZkNormProver,
    pub regime_detector: RegimeDetector,
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
        federated_averager: FederatedAverager::new(50, 300.0), // Buffer 50 updates, 5min half-life
        config,
        privacy_accountant: PrivacyAccountant::new(10.0, 1e-5, 0.995),
        zk_prover: ZkNormProver::new(),
        regime_detector: RegimeDetector::new(100, 0.8, 1000000.0), // window=100, entropy_thresh=0.8, throughput_thresh=1MB/s
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
    let mut federation_epoch = tokio::time::interval(Duration::from_secs(5)); // Every 5 seconds
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
                        if let Some(mut brain) = LivingBrain::from_json(&content) {
                            // Update RAM state
                            state.write().await.brain = brain.clone();

                            // Check current regime
                            let current_regime = state.read().await.regime_detector.current_regime();
                            let is_storm = matches!(current_regime, Regime::Storm);

                            // Adaptive quantization
                            if is_storm {
                                // Storm Mode: Quantize to I8F8 to halve bandwidth
                                if let Some(w_bytes) = &brain.best_engine_weights {
                                    let i16f16_weights: Vec<I16F16> = w_bytes.chunks(4).filter_map(|chunk| {
                                        if chunk.len() == 4 {
                                            let bits = i32::from_le_bytes(chunk.try_into().unwrap());
                                            Some(I16F16::from_bits(bits))
                                        } else {
                                            None
                                        }
                                    }).collect();
                                    let fixed_tensor = FixedTensor::new(i16f16_weights);
                                    let i8f8_weights = fixed_tensor.quantize_to_i8f8();
                                    // Convert back to bytes
                                    let quantized_bytes: Vec<u8> = i8f8_weights.iter().flat_map(|&w| w.to_le_bytes()).collect();
                                    brain.best_engine_weights = Some(quantized_bytes);
                                }
                            }

                            // --- PHASE 1: Proving Step (Sender) ---
                            // A. Type Conversion: Weights (Bytes) -> f32 for ZK (only in Calm mode)
                            let weights_f32: Vec<f32> = if !is_storm {
                                if let Some(w_bytes) = &brain.best_engine_weights {
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
                                }
                            } else {
                                Vec::new() // No ZK in storm mode
                            };

                            // B. Generate ZK Proof (only in Calm mode)
                            let proof_bundle = if !is_storm {
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
                            } else {
                                None // No proof in storm mode
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
                                is_storm_mode: is_storm,
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
                            let outgoing_bytes = msg_bytes.len() as u64;
                            let topic = IdentTopic::new(BRAIN_TOPIC);
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg_bytes) {
                                tracing::error!("Publish error: {:?}", e);
                            } else {
                                // Record Privacy Cost only on successful publish
                                let mut app_state = state.write().await;
                                let _ = app_state.privacy_accountant.record_consumption(epiphany_cost);

                                // Update regime detector
                                let entropy = calculate_brain_entropy(&brain);
                                app_state.regime_detector.update(entropy, 10000, outgoing_bytes); // elapsed_ms=10s

                                info!("Published SignedEpiphany (mode: {})", if is_storm { "Storm" } else { "Calm" });
                            }
                        }
                    }
                }
            }

            // Federated Learning Epoch
            _ = federation_epoch.tick() => {
                let mut app_state = state.write().await;
                if app_state.federated_averager.should_aggregate() {
                    // Clone reputation data to avoid borrowing issues
                    let reputation_clone = app_state.reputation.clone();
                    if let Some((aggregated_weights, aggregated_confidence)) =
                        app_state.federated_averager.aggregate(&reputation_clone) {

                        // Load current brain
                        if let Ok(local_json) = fs::read_to_string(brain_file) {
                            if let Some(mut local_brain) = LivingBrain::from_json(&local_json) {
                                // Apply aggregated weights
                                local_brain.best_engine_weights = Some(aggregated_weights);

                                // Apply aggregated confidence with learning rate
                                for (local_conf, &agg_conf) in local_brain.confidence.iter_mut().zip(aggregated_confidence.iter()) {
                                    *local_conf = *local_conf * 0.9 + agg_conf * 0.1; // 10% learning rate
                                }

                                // Check for Singularity
                                let global_error_rate = 1.0 - (aggregated_confidence.iter().sum::<f32>() / aggregated_confidence.len() as f32);
                                if global_error_rate < 0.01 {
                                    info!("🎯 SINGULARITY ACHIEVED! Global error rate: {:.6}", global_error_rate);
                                    // Emit SystemEvent::SingularityReached (would be sent to monitoring system)
                                    // Switch to inference-only mode (conceptual - would disable training)
                                }

                                // Export metrics to CSV
                                let local_loss = 1.0 - (local_brain.confidence.iter().sum::<f32>() / local_brain.confidence.len() as f32);
                                let swarm_variance = aggregated_confidence.iter()
                                    .map(|&c| (c - global_error_rate).powi(2))
                                    .sum::<f32>() / aggregated_confidence.len() as f32;
                                let active_peers = app_state.connected_peers.len();

                                let metrics = SingularityMetrics::new(local_loss, swarm_variance.sqrt(), active_peers);
                                if let Err(e) = metrics.export_csv() {
                                    warn!("Failed to export singularity metrics: {}", e);
                                }

                                // Save updated brain
                                let _ = fs::write(brain_file, local_brain.to_json());
                                app_state.brain = local_brain;

                                info!("Applied federated aggregation. Global error rate: {:.4}", global_error_rate);
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

                     // v19.0: Serve Summary Gene (Mid-Flight Join)
                     // Upon connection, we simulate pushing the compact Summary Gene to the new peer
                     // instead of the full event log.
                     {
                         let state_read = state.read().await;
                         let brain = &state_read.brain;

                         // Create Summary Gene from current state
                         // Using first 8 dims of confidence for demo/header if brain is large
                         let dims = brain.confidence.len().min(8);
                         let consensus = &brain.confidence[..dims];
                         let variance = vec![0.0; dims]; // Placeholder variance

                         let summary = SummaryGene::new(
                             1900, // v19.0 epoch
                             [0xAA; 32], // Mock hash
                             consensus,
                             &variance
                         );

                         let bytes = summary.to_bytes();
                         info!(
                             peer_id = %peer_id,
                             size_bytes = bytes.len(),
                             "served_summary_gene" = true,
                             "mid_flight_join" = "active",
                             "Serving Summary Gene instead of Event Log"
                         );
                     }
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
                            // 4. Verify ZK Proof or Trust High-Reputation Peers
                            let proof_valid = {
                                let app_state = state.read().await;
                                if let Some(bundle) = &signed_epiphany.proof_bundle {
                                    // Verify proof if present
                                    app_state.zk_prover.verify_proof(&bundle.zk_proof, 10.0)
                                } else {
                                    // No proof: Accept if high reputation (>80) or storm mode
                                    let reputation_score = app_state.reputation.get_trust(&signed_epiphany.sender_id);
                                    signed_epiphany.is_storm_mode || reputation_score > 80.0
                                }
                            };

                            if proof_valid {
                                // 5. Handle Storm Mode Upcasting
                                let mut processed_brain = signed_epiphany.brain.clone();
                                if signed_epiphany.is_storm_mode {
                                    // Upcast I8F8 weights back to I16F16
                                    if let Some(w_bytes) = &processed_brain.best_engine_weights {
                                        let i8f8_weights: Vec<I8F8> = w_bytes.chunks(2).filter_map(|chunk| {
                                            if chunk.len() == 2 {
                                                let bits = i16::from_le_bytes(chunk.try_into().unwrap());
                                                Some(I8F8::from_bits(bits))
                                            } else {
                                                None
                                            }
                                        }).collect();
                                        let fixed_tensor = FixedTensor::from_i8f8(&i8f8_weights);
                                        // Convert back to bytes
                                        let i16f16_bytes: Vec<u8> = fixed_tensor.data.iter().flat_map(|&w| w.to_le_bytes()).collect();
                                        processed_brain.best_engine_weights = Some(i16f16_bytes);
                                    }
                                }

                                // 6. Buffer for Federated Learning (instead of immediate merge)
                                let mut app_state = state.write().await;
                                app_state.federated_averager.add_update(signed_epiphany.clone());

                                // Update regime detector with incoming bytes
                                let incoming_bytes = message.data.len() as u64;
                                let entropy = calculate_brain_entropy(&processed_brain);
                                app_state.regime_detector.update(entropy, 10000, incoming_bytes);

                                // Reputation Reward
                                app_state.reputation.reward(&signed_epiphany.sender_id);
                                info!("Buffered SignedEpiphany (mode: {}) for federated averaging", if signed_epiphany.is_storm_mode { "Storm" } else { "Calm" });
                            } else {
                                // 7. Punish (Fail Proof or Low Reputation)
                                warn!("Rejected SignedEpiphany from {}: missing/invalid proof and low reputation", signed_epiphany.sender_id);
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

/// Calculate entropy from brain confidence (Shannon entropy)
fn calculate_brain_entropy(brain: &LivingBrain) -> f32 {
    let mut entropy = 0.0;
    for &conf in &brain.confidence {
        if conf > 0.0 {
            entropy -= conf * conf.ln(); // Assuming normalized
        }
    }
    entropy
}
