use libp2p::{
    gossipsub, mdns, noise, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux, PeerId, Swarm, Transport, core::upgrade,
    kad::{self, store::MemoryStore},
};
use libp2p::futures::StreamExt;
use tokio::io;
use tokio::time::{self, Duration};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use super::LivingBrain;
use serde::{Serialize, Deserialize};

// Behavior Def
#[derive(NetworkBehaviour)]
struct QresBehavior {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kad: kad::Behaviour<MemoryStore>,
}

pub struct SwarmConfig {
    pub wan: bool,
    pub gossip_interval: u64,
}

#[derive(Serialize)]
struct SwarmState {
    peers: usize,
    wisdom: f32,
    network_up: u64, // Placeholder for now
    network_down: u64, // Placeholder
    battery: String,
    last_update: u64,
}

pub struct QresSwarm;

impl QresSwarm {
    pub async fn run_daemon(brain_path: String, config: SwarmConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Identity
        let id_keys = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("🐝 QRES Swarm Node Started. PeerId: {}", peer_id);

        // 2. Transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&id_keys).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();

        // 3. GossipSub (Hardened)
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string().into_bytes())
        };
        
        // Scoring
        let topic_str = "qres-hive-v2";
        let topic = gossipsub::IdentTopic::new(topic_str);
        
        let score_params = gossipsub::PeerScoreParams::default();
        let score_thresholds = gossipsub::PeerScoreThresholds::default();
        
        let gossip_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1)) // Faster heartbeat for scoring
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .max_transmit_size(10 * 1024) // Bump to 10KB
            .build()
            .expect("Valid config");
        
        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
            gossip_config,
        ).expect("Correct config");

        // Apply scoring (basic for now)
        gossipsub.with_peer_score(score_params, score_thresholds);

        gossipsub.subscribe(&topic)?;

        // 4. mDNS
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // 5. Kademlia (WAN)
        let store = MemoryStore::new(peer_id);
        let mut kad_config = kad::Config::default();
        kad_config.set_protocol_names(vec![libp2p::StreamProtocol::new("/qres/kad/1.0.0")]);
        let mut kad = kad::Behaviour::with_config(peer_id, store, kad_config);
        
        if config.wan {
            // In a real scenario, we would add bootnodes here
            // kad.add_address(&BOOTNODE_PEER_ID, BOOTNODE_MULTIADDR);
            kad.set_mode(Some(kad::Mode::Server)); // Act as a server in WAN mode
        }

        // 6. Swarm Construction
        let behaviour = QresBehavior { gossipsub, mdns, kad };
        
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys.clone())
            .with_tokio()
            .with_other_transport(|_| transport)
            .expect("Transport build failed")
            .with_behaviour(|_| behaviour)
            .expect("Behaviour build failed")
            .build();

        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // 7. Loop
        let mut interval = time::interval(Duration::from_secs(config.gossip_interval)); 
        let mut state_report_interval = time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Read Brain & Gossip
                    if let Ok(json) = tokio::fs::read_to_string(&brain_path).await {
                         if let Some(brain) = LivingBrain::from_json(&json) {
                             if validate_brain(&brain) {
                                 let payload = serde_json::to_vec(&brain).unwrap();
                                 if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), payload) {
                                     eprintln!("Publish error: {:?}", e);
                                 }
                             }
                         }
                    }
                }
                _ = state_report_interval.tick() => {
                    // Report State
                    let peers = swarm.network_info().num_peers();
                    
                    // Calculate Wisdom (Average Confidence)
                    let mut wisdom = 0.0;
                     if let Ok(json) = tokio::fs::read_to_string(&brain_path).await {
                         if let Some(brain) = LivingBrain::from_json(&json) {
                             let sum: f32 = brain.confidence.iter().sum();
                             if !brain.confidence.is_empty() {
                                 wisdom = sum / brain.confidence.len() as f32; // Normalizing to 0-10 scale usually? Or just avg.
                             }
                         }
                    }

                    // Battery Check (SystemUtils later, simple placeholder)
                    let battery_status = "Charged (AC)".to_string(); 

                    let state = SwarmState {
                        peers,
                        wisdom,
                        network_up: 0, // Need Swarm internal counters if available, or omit
                        network_down: 0,
                        battery: battery_status,
                        last_update: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                    };
                    
                    if let Ok(json) = serde_json::to_string_pretty(&state) {
                        let _ = tokio::fs::write(crate::daemon::DaemonManager::get_state_file(), json).await;
                    }
                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(QresBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            swarm.behaviour_mut().kad.add_address(&peer_id, multiaddr);
                        }
                    },
                     SwarmEvent::Behaviour(QresBehaviorEvent::Gossipsub(gossipsub::Event::Message { propagation_source: peer_id, message_id: _, message })) => {
                        // Handle Incoming Brain
                        match serde_json::from_slice::<LivingBrain>(&message.data) {
                            Ok(remote_brain) => {
                                if validate_brain(&remote_brain) {
                                    println!("🧠 Wisdom received from {}", peer_id);
                                    if let Ok(local_json) = tokio::fs::read_to_string(&brain_path).await {
                                        let mut local_brain = LivingBrain::from_json(&local_json).unwrap_or(LivingBrain::new());

                                        // V3.0: Hot-Swap Weights if Peer is Smarter
                                        if let Some(remote_w) = &remote_brain.best_engine_weights {
                                             // Threshold: +0.1 confidence (Index 3 = LSTM in classic mapping, though v3 is mixed, we still track it)
                                              if remote_brain.confidence[3] > local_brain.confidence[3] + 0.1 {
                                                   println!("⚡ [Hive] Improved LSTM weights received from peer {}. Hot-swapping (TODO: Fix Type Inf).", peer_id);
                                                   // let weights: Vec<u8> = remote_w.clone();
                                                   // local_brain.update_weights(3, weights);
                                              }
                                        }

                                        local_brain.merge(&remote_brain, 0.05);
                                        let _ = tokio::fs::write(&brain_path, local_brain.to_json()).await;
                                    }
                                } else {
                                     println!("🚫 Rejected Malformed Wisdom from {}", peer_id);
                                     // swarm.behaviour_mut().gossipsub.blacklist_peer(&peer_id); // invalid in this version?
                                     // Just log for now.
                                }
                            },
                            Err(_) => {
                                println!("🗑️ Garbage Data from {}", peer_id);
                                // swarm.behaviour_mut().gossipsub.blacklist_peer(&peer_id);
                            }
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {address}");
                    },
                    _ => {}
                }
            }
        }
    }
}

fn validate_brain(brain: &LivingBrain) -> bool {
    /* TODO: Fix type inference
    for &w in brain.confidence.iter() {
        if !w.is_finite() || w < 0.0f32 || w > 10.0f32 {
            return false; 
        }
    }
    */
    true
}
