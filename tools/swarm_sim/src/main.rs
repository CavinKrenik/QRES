use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use qres_core::cortex::{GeneStorage, LinearNeuron, Regime};
use rand::Rng;
use std::fs;

// --- CONFIGURATION ---
const MTU_LIMIT: usize = 1400;
const BASE_DROP_RATE: f64 = 0.02;
const GENE_SIZE_BYTES: usize = 1600; // Large gene triggers MTU fragmentation!

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "QRES Phase 3: Emergent Swarm Evolution".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Environment {
            noise_center: Vec2::new(10.0, 10.0), // Starts in middle
            noise_radius: 8.0,
            time: 0.0,
        })
        .add_systems(Startup, setup_swarm)
        .add_systems(
            Update,
            (
                move_noise_zone,          // 1. The Environment changes
                simulate_cortex_reaction, // 2. Nodes react (Calm vs Storm)
                trigger_evolution,        // 3. Random mutations ("The Spark")
                gossip_protocol,          // 4. Nodes talk (Gene Requests)
                packet_physics_system,    // 5. The Network carries (or drops) data
                process_incoming_packets, // 6. Nodes learn (Gene Install)
                persist_evolved_genes,    // 7. Save genes to disk (The Hippocampus)
                update_visuals,           // 8. God View
                draw_debug_overlays,      // 9. Gizmos + Noise Zone + Purple Web
            ),
        )
        .run();
}

// --- RESOURCES ---

#[derive(Resource)]
struct Environment {
    noise_center: Vec2,
    noise_radius: f32,
    time: f32,
}

/// Disk-based gene storage for persistent evolution
struct DiskGeneStorage {
    storage_dir: String,
}

impl DiskGeneStorage {
    fn new(dir: &str) -> Self {
        // Create directory if it doesn't exist
        let _ = fs::create_dir_all(dir);
        DiskGeneStorage {
            storage_dir: dir.to_string(),
        }
    }

    fn gene_path(&self, id: u32) -> String {
        format!("{}/gene_{}.bin", self.storage_dir, id)
    }
}

impl GeneStorage for DiskGeneStorage {
    fn save_gene(&mut self, id: u32, gene: &[u8]) -> bool {
        let path = self.gene_path(id);
        match fs::write(&path, gene) {
            Ok(_) => {
                println!("💾 Gene saved for node {}: {}", id, path);
                true
            }
            Err(e) => {
                eprintln!("Failed to save gene {}: {}", id, e);
                false
            }
        }
    }

    fn load_gene(&self, id: u32) -> Option<Vec<u8>> {
        let path = self.gene_path(id);
        match fs::read(&path) {
            Ok(gene) => {
                println!("📖 Gene loaded for node {}: {} bytes", id, gene.len());
                Some(gene)
            }
            Err(_) => None, // File doesn't exist yet
        }
    }
}

// --- COMPONENTS ---

#[derive(Component)]
struct IoTNode {
    id: u32,
    #[allow(dead_code)]
    reputation: f32,
}

#[derive(Component)]
struct Cortex {
    neuron_type: NeuronType,
    regime: Regime,
    time_in_storm: f32,     // How long have I been panicking?
    persistence_timer: f32, // Timer for gene saves
}

#[derive(Clone)]
enum NeuronType {
    #[allow(dead_code)]
    Linear(LinearNeuron), // Default: Fails in noise
    Evolved(Vec<u8>),     // Advanced: Robust in noise
}

#[derive(Component)]
struct NetworkPacket {
    target: u32, // Simple ID-based routing for sim
    payload: PacketType,
    size: usize,
    ttl: f32,
}

enum PacketType {
    #[allow(dead_code)]
    SpikeBroadcast,       // "I am surprised!"
    GeneRequest,          // "Help me!"
    GenePayload(Vec<u8>), // "Here is the cure."
}

// --- SYSTEMS ---

fn setup_swarm(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera & Light (Standard Setup)
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(0.0, 20.0, 25.0)
                .looking_at(Vec3::new(10.0, 0.0, 10.0), Vec3::Y),
            ..default()
        },
        BloomSettings::NATURAL,
    ));
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0),
        point_light: PointLight {
            intensity: 2000.0,
            range: 100.0,
            ..default()
        },
        ..default()
    });

    // Initialize gene storage (The Hippocampus)
    let storage = DiskGeneStorage::new("./swarms_memory");

    // 10x10 Grid
    let mesh = meshes.add(Mesh::from(Sphere { radius: 0.3 }));
    let mat = materials.add(StandardMaterial::from(Color::rgb(0.1, 0.1, 0.8)));

    for x in 0..10 {
        for z in 0..10 {
            let id = x * 10 + z;

            // Check if this node has a saved gene from a previous session
            let neuron_type = if let Some(gene) = storage.load_gene(id) {
                NeuronType::Evolved(gene)
            } else {
                NeuronType::Linear(LinearNeuron::new(32))
            };

            commands.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material: mat.clone(),
                    transform: Transform::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0),
                    ..default()
                },
                IoTNode {
                    id,
                    reputation: 1.0,
                },
                Cortex {
                    neuron_type,
                    regime: Regime::Calm,
                    time_in_storm: 0.0,
                    persistence_timer: 0.0,
                },
            ));
        }
    }
}

/// 1. Move the "Noise Zone" in a circle
fn move_noise_zone(time: Res<Time>, mut env: ResMut<Environment>) {
    env.time += time.delta_seconds();
    // Orbit around center (10,10)
    env.noise_center.x = 10.0 + (env.time * 0.5).sin() * 5.0;
    env.noise_center.y = 10.0 + (env.time * 0.5).cos() * 5.0;
}

/// 2. Cortex Logic: React to Environment
fn simulate_cortex_reaction(
    env: Res<Environment>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut Cortex)>,
) {
    for (transform, mut cortex) in query.iter_mut() {
        let dist =
            Vec2::new(transform.translation.x, transform.translation.z).distance(env.noise_center);
        let in_noise = dist < env.noise_radius;

        match cortex.neuron_type {
            NeuronType::Linear(_) => {
                if in_noise {
                    cortex.regime = Regime::Storm;
                    cortex.time_in_storm += time.delta_seconds();
                } else {
                    cortex.regime = Regime::Calm;
                    cortex.time_in_storm = 0.0;
                }
            }
            NeuronType::Evolved(_) => {
                // Evolved neurons handle noise perfectly
                cortex.regime = Regime::Calm;
                cortex.time_in_storm = 0.0;
            }
        }
    }
}

/// 3. The Spark: Random Mutation
fn trigger_evolution(mut query: Query<&mut Cortex>) {
    let mut rng = rand::thread_rng();
    for mut cortex in query.iter_mut() {
        // If panicking, 0.1% chance per frame to "invent" the solution
        if cortex.regime == Regime::Storm && rng.gen_bool(0.001) {
            cortex.neuron_type = NeuronType::Evolved(vec![0; GENE_SIZE_BYTES]);
            println!("✨ SPARK: A node has evolved autonomously!");
        }
    }
}

/// 4. Gossip: Request Help & Share Genes
fn gossip_protocol(
    mut commands: Commands,
    query_nodes: Query<(Entity, &IoTNode, &Cortex, &Transform)>,
    query_lookup: Query<(&IoTNode, &Transform)>, // Read-only lookups
) {
    let nodes_vec: Vec<_> = query_nodes.iter().collect();

    for (_entity, node, cortex, transform) in nodes_vec.iter() {
        // STRATEGY: If I am in Storm for too long, ask for help
        if cortex.regime == Regime::Storm && cortex.time_in_storm > 2.0 {
            // Find a calm neighbor
            for (neighbor, n_trans) in query_lookup.iter() {
                if node.id == neighbor.id {
                    continue;
                }

                if transform.translation.distance(n_trans.translation) < 3.0 {
                    // Request help!
                    commands.spawn(NetworkPacket {
                        target: neighbor.id,
                        payload: PacketType::GeneRequest,
                        size: 64, // Small packet
                        ttl: 1.0,
                    });
                }
            }
        }
    }
}

/// 5. Network Physics (The Hardware Quirk)
fn packet_physics_system(
    mut commands: Commands,
    time: Res<Time>,
    mut packets: Query<(Entity, &mut NetworkPacket)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut packet) in packets.iter_mut() {
        packet.ttl -= time.delta_seconds();
        if packet.ttl <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // NON-LINEAR DROP RATE (The Quirk)
        let drop_chance = if packet.size > MTU_LIMIT {
            // High drop rate for large genes
            0.15
        } else {
            // Low drop rate for small requests
            BASE_DROP_RATE
        };

        if rng.gen_bool(drop_chance) {
            commands.entity(entity).despawn(); // Packet lost!
        }
    }
}

/// 6. Process Incoming Packets
fn process_incoming_packets(
    mut commands: Commands,
    mut packets: Query<(Entity, &NetworkPacket)>,
    mut nodes: Query<(&IoTNode, &mut Cortex)>,
) {
    for (p_entity, packet) in packets.iter_mut() {
        // Find target node (naive O(N) lookup for sim)
        for (node, mut cortex) in nodes.iter_mut() {
            if node.id == packet.target {
                match &packet.payload {
                    PacketType::GeneRequest => {
                        // If I am evolved, send the cure
                        if let NeuronType::Evolved(gene) = &cortex.neuron_type {
                            // Reply with the Payload (Subject to MTU drops!)
                            commands.spawn(NetworkPacket {
                                target: node.id, // Should reply to sender, simplified here
                                payload: PacketType::GenePayload(gene.clone()),
                                size: GENE_SIZE_BYTES,
                                ttl: 1.0,
                            });
                        }
                    }
                    PacketType::GenePayload(gene) => {
                        // INSTALL THE CURE
                        cortex.neuron_type = NeuronType::Evolved(gene.clone());
                    }
                    _ => {}
                }
                commands.entity(p_entity).despawn(); // Consumed
            }
        }
    }
}

/// 7. Persistence: Save evolved genes to disk (The Hippocampus)
fn persist_evolved_genes(time: Res<Time>, mut query: Query<(&IoTNode, &mut Cortex)>) {
    let mut storage = DiskGeneStorage::new("./swarms_memory");

    for (node, mut cortex) in query.iter_mut() {
        cortex.persistence_timer += time.delta_seconds();

        // Every 5 seconds, if this node is evolved AND calm, save its gene
        if cortex.persistence_timer >= 5.0 {
            cortex.persistence_timer = 0.0;

            if cortex.regime == Regime::Calm {
                if let NeuronType::Evolved(ref gene) = cortex.neuron_type {
                    let _ = storage.save_gene(node.id, gene);
                }
            }
        }
    }
}

/// 8. God View Visuals
fn update_visuals(
    mut query: Query<(&Cortex, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (cortex, mut mat) in query.iter_mut() {
        let color = match cortex.neuron_type {
            NeuronType::Evolved(_) => Color::rgb(0.6, 0.1, 1.0), // Purple: The Cure
            NeuronType::Linear(_) => match cortex.regime {
                Regime::Calm => Color::rgb(0.1, 0.1, 0.8),  // Blue
                Regime::Storm => Color::rgb(0.9, 0.1, 0.1), // Red
                _ => Color::rgb(1.0, 1.0, 0.0),             // Orange
            },
        };
        *mat = materials.add(StandardMaterial::from(color));
    }
}

/// 8. Gizmo Overlays: Noise Zone + Purple Web
fn draw_debug_overlays(
    mut gizmos: Gizmos,
    env: Res<Environment>,
    cortex_query: Query<(&Transform, &Cortex)>,
) {
    use bevy::math::primitives::Direction3d;

    // 1. Draw the Noise Zone (Red Force Field) - Concentric circles at env.noise_center
    let base_pos = Vec3::new(env.noise_center.x, 0.0, env.noise_center.y);
    gizmos.circle(
        base_pos,
        Direction3d::Y,
        env.noise_radius,
        Color::rgba(1.0, 0.3, 0.0, 0.8),
    );
    gizmos.circle(
        base_pos,
        Direction3d::Y,
        env.noise_radius * 0.95,
        Color::rgba(1.0, 0.0, 0.0, 0.4),
    );

    // 2. Draw "Connection Lines" between Evolved Nodes (The Purple Web)
    let evolved_nodes: Vec<Vec3> = cortex_query
        .iter()
        .filter(|(_, c)| matches!(c.neuron_type, NeuronType::Evolved(_)))
        .map(|(t, _)| t.translation)
        .collect();

    for i in 0..evolved_nodes.len() {
        for j in (i + 1)..evolved_nodes.len() {
            let a = evolved_nodes[i];
            let b = evolved_nodes[j];
            if a.distance(b) < 3.0 {
                gizmos.line(a, b, Color::rgba(0.8, 0.2, 1.0, 0.6));
            }
        }
    }
}
