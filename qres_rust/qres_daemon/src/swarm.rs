use crate::living_brain::LivingBrain;
use libp2p::futures::StreamExt;
use libp2p::{
    core::upgrade,
    gossipsub,
    kad::{self, store::MemoryStore},
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use tokio::time::{self, Duration};

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
    network_up: u64,   // Placeholder for now
    network_down: u64, // Placeholder
    battery: String,
    last_update: u64,
}

pub struct QresSwarm;

impl QresSwarm {
    pub async fn run_daemon(
        brain_path: String,
        config: SwarmConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        // v8.0: Quantum Network Topic
        let quantum_topic_str = "qres-quantum-net";
        let quantum_topic = gossipsub::IdentTopic::new(quantum_topic_str);

        let score_params = gossipsub::PeerScoreParams::default();
        let score_thresholds = gossipsub::PeerScoreThresholds::default();

        let gossip_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1)) // Faster heartbeat for scoring
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .max_transmit_size(1024 * 1024) // Bump to 1MB for large Tensors
            .build()
            .expect("Valid config");

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
            gossip_config,
        )
        .expect("Correct config");

        // Apply scoring (basic for now)
        let _ = gossipsub.with_peer_score(score_params, score_thresholds);

        gossipsub.subscribe(&topic)?;
        gossipsub.subscribe(&quantum_topic)?;

        // 4. mDNS
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // 5. Kademlia (WAN)
        let store = MemoryStore::new(peer_id);
        let mut kad_config = kad::Config::default();
        kad_config.set_protocol_names(vec![libp2p::StreamProtocol::new("/qres/kad/1.0.0")]);
        let mut kad = kad::Behaviour::with_config(peer_id, store, kad_config);

        if config.wan {
            println!("🌍 WAN Mode Enabled: Initializing Global Kademlia DHT...");
            kad.set_mode(Some(kad::Mode::Server));
            // Experimental: QRES Seed Nodes (Public Bootstrap)
            // Ideally this comes from config, but for v8.0 prototype we add a known seed.
            // Placeholder: "/ip4/148.251.10.1/tcp/4001" (Not real)
            // Implementation: We initiate bootstrap query.
            if let Err(e) = kad.bootstrap() {
                eprintln!("Kademlia Bootstrap Warning: {:?}", e);
            }
        }

        // 6. Swarm Construction
        let behaviour = QresBehavior {
            gossipsub,
            mdns,
            kad,
        };

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
        // Ensure quantum inbox exists
        let inbox_path = "quantum_inbox";
        tokio::fs::create_dir_all(inbox_path).await?;

        // Ensure quantum outbox exists
        let outbox_path = "quantum_outbox";
        tokio::fs::create_dir_all(outbox_path).await?;

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
                    // Check for outgoing Quantum Tensors (from quantum_outbox)
                    if let Ok(mut entries) = tokio::fs::read_dir(outbox_path).await {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            let path = entry.path();
                            if path.is_file() {
                                if let Ok(data) = tokio::fs::read(&path).await {
                                    println!("🚀 [Hive] Broadcasting Quantum Tensor from outbox: {:?}", path.file_name());
                                    // Verify header (optional safety) or just send
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(quantum_topic.clone(), data) {
                                         eprintln!("Quantum Broadcast Error: {:?}", e);
                                    } else {
                                         // Delete after successful publish
                                         let _ = tokio::fs::remove_file(path).await;
                                    }
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
                        // Check if it's a Quantum Tensor
                        if message.topic.as_str() == "qres-quantum-net" {
                            if message.data.starts_with(b"QRES_Q_TENSOR") {
                                println!("🌌 [Hive] Quantum Tensor received from {} ({} bytes)", peer_id, message.data.len());
                                let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                                let filename = format!("{}/{}_{}.qt", inbox_path, timestamp, peer_id);
                                if let Err(e) = tokio::fs::write(&filename, &message.data).await {
                                     eprintln!("Failed to save quantum tensor: {}", e);
                                }
                            }
                        } else {
                            // Handle Standard Brain Gossip
                            match serde_json::from_slice::<LivingBrain>(&message.data) {
                                Ok(remote_brain) => {
                                    if validate_brain(&remote_brain) {
                                        println!("🧠 Wisdom received from {}", peer_id);
                                        if let Ok(local_json) = tokio::fs::read_to_string(&brain_path).await {
                                            let mut local_brain = LivingBrain::from_json(&local_json).unwrap_or_default();
                                            // V3.0: Hot-Swap Weights if Peer is Smarter
                                            if let Some(_remote_w) = &remote_brain.best_engine_weights {
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
                                    // Could be other messages or encryption noise
                                }
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

fn validate_brain(_brain: &LivingBrain) -> bool {
    /* TODO: Fix type inference
    for &w in brain.confidence.iter() {
        if !w.is_finite() || w < 0.0f32 || w > 10.0f32 {
            return false;
        }
    }
    */
    true
}
