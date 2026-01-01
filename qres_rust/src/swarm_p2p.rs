use libp2p::{
    gossipsub, mdns, identity, identify, noise, tcp, yamux,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, Transport, SwarmBuilder,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use libp2p::futures::StreamExt; // For select_next_some
use crate::LivingBrain;
use std::fs;
use std::io; // Added
use libp2p::gossipsub::IdentTopic; // Added helper

// Topic for brain synchronization
const BRAIN_TOPIC: &str = "qres-brain-sync";

// Custom Behavior Struct
#[derive(NetworkBehaviour)]
pub struct QresBehavior {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}

pub async fn start_p2p_node(brain_path: String) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Identity
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    eprintln!("[Swarm] Local Peer ID: {}", peer_id);

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
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let mut gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let topic = gossipsub::IdentTopic::new(BRAIN_TOPIC);
            gossipsub.subscribe(&topic).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

            // mDNS
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(), 
                PeerId::from(key.public())
            )?;

            // Identify
            let identify = identify::Behaviour::new(identify::Config::new(
                "qres/1.0.0".to_string(),
                key.public(),
            ));

            Ok(QresBehavior { gossipsub, mdns, identify })
        })?
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // 6. Loop
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let brain_file = &brain_path;

    loop {
        tokio::select! {
            // Periodic Brain Broadcast
            _ = interval.tick() => {
                if let Ok(content) = fs::read_to_string(brain_file) {
                    let topic = IdentTopic::new(BRAIN_TOPIC);
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, content.as_bytes()) {
                         eprintln!("[Swarm] Publish error: {:?}", e);
                    } else {
                         eprintln!("[Swarm] Broadcasted local wisdom.");
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
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        eprintln!("[Swarm] mDNS Discovered: {:?}", peer_id);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        let _ = swarm.dial(multiaddr);
                    }
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        eprintln!("[Swarm] mDNS Expired: {:?}", peer_id);
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(QresBehaviorEvent::Gossipsub(gossipsub::Event::Message { propagation_source: _, message_id: _, message })) => {
                    eprintln!("[Swarm] Received Merge Candidate");
                    if let Ok(json) = String::from_utf8(message.data) {
                        if let Some(remote_brain) = LivingBrain::from_json(&json) {
                            // Merge
                            if let Ok(local_json) = fs::read_to_string(brain_file) {
                                if let Some(mut local_brain) = LivingBrain::from_json(&local_json) {
                                    // FedProx Merge: Only if we are less experienced? Or always?
                                    // P2P Swarm Logic: Average?
                                    // Let's use weighted average with 0.1 alpha
                                    local_brain.merge(&remote_brain, 0.1);
                                    let _ = fs::write(brain_file, local_brain.to_json());
                                    eprintln!("[Swarm] Assimilated knowledge from peer.");
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
