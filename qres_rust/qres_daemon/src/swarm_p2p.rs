use crate::living_brain::{LivingBrain, BrainMessage};
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
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

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
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Identity
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    eprintln!("[Swarm] Local Peer ID: {}", peer_id);

    // Shared State
    let state = Arc::new(RwLock::new(AppState {
        local_peer_id: peer_id.to_string(),
        connected_peers: HashSet::new(),
        known_peers: HashSet::new(),
        brain: LivingBrain::default(),
    }));

    // Spawn API
    let app_state = state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/status", get(get_status))
            .route("/brain", get(get_brain))
            .with_state(app_state);

        eprintln!("[API] Server listening on http://0.0.0.0:{}", port);
        // Bind to 0.0.0.0 to allow external access
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();
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

                        // Delta Encoding Logic
                        let message = if let Some(last) = &last_broadcast_brain {
                            if let Some(delta) = current_brain.diff(last) {
                                Some(BrainMessage::Delta(delta))
                            } else {
                                None // No significant change - skip broadcast
                            }
                        } else {
                            Some(BrainMessage::Full(current_brain.clone()))
                        };

                        if let Some(msg) = message {
                            let topic = IdentTopic::new(BRAIN_TOPIC);
                            if let Ok(payload) = serde_json::to_vec(&msg) {
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, payload) {
                                     eprintln!("[Swarm] Publish error: {:?}", e);
                                } else {
                                     match &msg {
                                         BrainMessage::Delta(d) => eprintln!("[Swarm] Broadcasted Delta ({} updates).", d.updates.len()),
                                         BrainMessage::Full(_) => eprintln!("[Swarm] Broadcasted Full Wisdom."),
                                     }
                                     last_broadcast_brain = Some(current_brain);
                                }
                            }
                        } else {
                            eprintln!("[Swarm] No significant changes - skipping broadcast.");
                        }
                    }
                }
            }

            // Swarm Events
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    eprintln!("[Swarm] Listening on {:?}", address);
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                     eprintln!("[Swarm] Connected to {:?}", peer_id);
                     state.write().await.connected_peers.insert(peer_id.to_string());
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                     eprintln!("[Swarm] Disconnected from {:?}", peer_id);
                     state.write().await.connected_peers.remove(&peer_id.to_string());
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        eprintln!("[Swarm] mDNS Discovered: {:?}", peer_id);
                        state.write().await.known_peers.insert(peer_id.to_string());
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        let _ = swarm.dial(multiaddr);
                    }
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        eprintln!("[Swarm] mDNS Expired: {:?}", peer_id);
                        state.write().await.known_peers.remove(&peer_id.to_string());
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Gossipsub(gossipsub::Event::Message { propagation_source: _, message_id: _, message })) => {
                    eprintln!("[Swarm] Received Merge Candidate");
                    if let Ok(json) = String::from_utf8(message.data) {
                        // Try to parse as BrainMessage (supports both Full and Delta)
                        if let Ok(msg) = serde_json::from_str::<BrainMessage>(&json) {
                            match msg {
                                BrainMessage::Full(remote_brain) => {
                                    if let Ok(local_json) = fs::read_to_string(brain_file) {
                                        if let Some(mut local_brain) = LivingBrain::from_json(&local_json) {
                                            local_brain.merge(&remote_brain, 0.1);
                                            let _ = fs::write(brain_file, local_brain.to_json());
                                            eprintln!("[Swarm] Assimilated Full Knowledge.");
                                            state.write().await.brain = local_brain;
                                        }
                                    }
                                }
                                BrainMessage::Delta(delta) => {
                                    if let Ok(local_json) = fs::read_to_string(brain_file) {
                                        if let Some(mut local_brain) = LivingBrain::from_json(&local_json) {
                                            local_brain.apply_delta(&delta);
                                            let _ = fs::write(brain_file, local_brain.to_json());
                                            eprintln!("[Swarm] Applied Knowledge Delta ({} updates).", delta.updates.len());
                                            state.write().await.brain = local_brain;
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
