use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use qres_core::cortex::{GeneStorage, LinearNeuron, Regime};
use rand::Rng;
use std::collections::HashMap;
use std::fs;

// --- CONFIGURATION ---
const MTU_LIMIT: usize = 1400;
const BASE_DROP_RATE: f64 = 0.02;
const GENE_SIZE_BYTES: usize = 1600; // Large gene triggers MTU fragmentation!
const NODE_COUNT: u32 = 150; // Total nodes in swarm (denser brain)

// --- BRAIN STRUCTURE CONFIGURATION ---
const BRAIN_RADIUS: f32 = 8.0; // Larger sphere so nodes spread out
const CONNECTION_DISTANCE: f32 = 3.5; // Draw connections between nearby nodes
const CONNECTION_ALPHA: f32 = 0.15; // Connection transparency
const CONNECTION_SKIP: usize = 2; // Draw every Nth connection (1 = all, 2 = half)

// --- FORCE-DIRECTED LAYOUT CONFIGURATION ---
const REPULSION_STRENGTH: f32 = 12.0; // STRONGER repulsion to spread out
const SPRING_STIFFNESS: f32 = 1.5; // Weaker springs
const SPRING_REST_LENGTH: f32 = 2.5; // Larger spacing between nodes
const DAMPING: f32 = 0.75; // Friction for stability
const CENTER_GRAVITY: f32 = 4.0; // Gentler pull to center
const MAX_VELOCITY: f32 = 6.0; // Movement speed
const SURFACE_TENSION: f32 = 2.0; // Pull to spherical surface
const NEIGHBOR_ATTRACTION: f32 = 0.5; // Weak attraction between nodes
const GLOBAL_ATTRACTION: f32 = 0.15; // Weak long-range attraction

// --- SYNAPSE CONFIGURATION ---
const SYNAPSE_DECAY_RATE: f32 = 1.5; // How fast pulses fade
const SYNAPSE_MAX_DISTANCE: f32 = 4.0; // Max distance for active synapses

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "QRES Living Brain: Neural Swarm Visualization".into(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Environment {
            noise_center: Vec2::new(0.0, 0.0), // Starts at center
            noise_radius: 6.0,
            time: 0.0,
        })
        .insert_resource(SwarmMetrics::default())
        .insert_resource(SynapseRegistry::default())
        .insert_resource(CameraController {
            yaw: 0.0,
            pitch: 0.3,
            distance: 20.0,
            auto_rotate: true,
        })
        .add_systems(Startup, (setup_swarm, setup_hud))
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
                force_directed_layout,    // 8. Organic "brain" movement
                update_visuals,           // 9. Node colors
                animate_synapses,         // 10. Pulsing connections
                update_hud,               // 11. Real-time metrics
                draw_debug_overlays,      // 12. Gizmos + Noise Zone
                orbit_camera,             // 13. Smooth camera orbit
                reset_simulation,         // 14. R key to reset
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

/// Mouse-controlled camera orbit
#[derive(Resource)]
struct CameraController {
    yaw: f32,
    pitch: f32,
    distance: f32,
    auto_rotate: bool,
}

/// Real-time metrics for HUD display
#[allow(dead_code)]
#[derive(Resource, Default)]
struct SwarmMetrics {
    total_nodes: u32,
    evolved_nodes: u32,
    storm_nodes: u32,
    active_synapses: u32,
    packets_in_flight: u32,
    entropy: f32, // 0.0 = all calm, 1.0 = all storm
}

/// Registry of active synaptic connections with pulse activity
#[derive(Resource, Default)]
struct SynapseRegistry {
    // Maps (source_id, target_id) -> activity level (0.0 to 1.0)
    connections: HashMap<(u32, u32), f32>,
}

/// Disk-based gene storage for persistent evolution
struct DiskGeneStorage {
    storage_dir: String,
}

impl DiskGeneStorage {
    fn new(dir: &str) -> Self {
        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("⚠️ WARNING: Failed to initialize persistence layer: {}", e);
        }
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

/// Velocity for force-directed physics
#[derive(Component)]
struct Velocity(Vec3);

/// Marker for HUD text elements
#[derive(Component)]
struct HudText;

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum HudMetricType {
    Title,
    NodeCount,
    Entropy,
    Bandwidth,
    Timestamp,
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
    Evolved(Vec<u8>), // Advanced: Robust in noise
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
    SpikeBroadcast, // "I am surprised!"
    GeneRequest,          // "Help me!"
    GenePayload(Vec<u8>), // "Here is the cure."
}

// --- SYSTEMS ---

fn setup_swarm(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera & Light - positioned for brain view
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(18.0, 4.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BloomSettings::NATURAL,
    ));
    // Multiple lights for better brain illumination
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(15.0, 10.0, 15.0),
        point_light: PointLight {
            intensity: 3000.0,
            range: 50.0,
            color: Color::rgb(1.0, 0.95, 0.9),
            ..default()
        },
        ..default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(-15.0, -5.0, -10.0),
        point_light: PointLight {
            intensity: 1500.0,
            range: 50.0,
            color: Color::rgb(0.7, 0.8, 1.0), // Cool fill light
            ..default()
        },
        ..default()
    });

    // Initialize gene storage (The Hippocampus)
    let storage = DiskGeneStorage::new("./swarms_memory");

    // Low-poly sphere for organic "brain cell" look
    let mesh = meshes.add(Sphere { radius: 0.3 }.mesh().ico(1).unwrap());
    let mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.15, 0.2, 0.9),
        emissive: Color::rgb(0.02, 0.03, 0.15),
        ..default()
    });

    // Spawn nodes on a spherical shell (brain surface) with slight depth variation
    let mut rng = rand::thread_rng();
    for id in 0..NODE_COUNT {
        // Fibonacci sphere distribution for even spacing
        let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let i = id as f32;
        let theta = std::f32::consts::TAU * i / golden_ratio;
        let phi = (1.0 - 2.0 * (i + 0.5) / NODE_COUNT as f32).acos();

        // Radius varies between 0.7 and 1.0 of BRAIN_RADIUS for depth
        let r = BRAIN_RADIUS * (0.7 + rng.gen::<f32>() * 0.3);
        let x = r * phi.sin() * theta.cos();
        let y = r * phi.sin() * theta.sin();
        let z = r * phi.cos();

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
                transform: Transform::from_xyz(x, y, z).with_scale(Vec3::splat(0.4)),
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
            Velocity(Vec3::ZERO),
        ));
    }

    // Setup HUD overlay
    commands.spawn((
        TextBundle::from_section(
            "Entropy: 0.00\nNodes: 0\nBandwidth: 0 B/s",
            TextStyle {
                font_size: 24.0,
                color: Color::rgba(0.0, 1.0, 0.5, 0.9),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        HudText,
    ));
}

/// 1. Move the "Noise Zone" through the brain to trigger storms
fn move_noise_zone(time: Res<Time>, mut env: ResMut<Environment>) {
    env.time += time.delta_seconds();
    // Orbit around brain center (0,0) - passes through the brain
    env.noise_center.x = (env.time * 0.3).sin() * 4.0;
    env.noise_center.y = (env.time * 0.3).cos() * 4.0;
    env.noise_radius = 5.0; // Large enough to catch nodes
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
    mut registry: ResMut<SynapseRegistry>,
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
                    // Register synapse activity (for visualization)
                    registry.connections.insert((node.id, neighbor.id), 1.0);

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

/// 8. God View Visuals - colorful nodes with glow
fn update_visuals(
    mut query: Query<(
        &Cortex,
        &IoTNode,
        &mut Handle<StandardMaterial>,
        &mut Transform,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_seconds();

    for (cortex, node, mut mat, mut transform) in query.iter_mut() {
        // Color based on state with variety
        let hue_offset = (node.id as f32 * 0.1).sin() * 0.1; // Slight color variation per node

        let (base_color, emissive) = match cortex.neuron_type {
            NeuronType::Evolved(_) => {
                // Evolved: vibrant colors cycling through purple/magenta/cyan
                let hue = 0.8 + hue_offset + (t * 0.2 + node.id as f32 * 0.05).sin() * 0.1;
                let pulse = ((t * 3.0 + node.id as f32).sin() * 0.5 + 0.5) * 0.3;
                (
                    Color::hsl(hue * 360.0, 0.9, 0.6),
                    Color::rgb(0.4 + pulse, 0.1, 0.6 + pulse), // Glowing
                )
            }
            NeuronType::Linear(_) => match cortex.regime {
                Regime::Storm => (
                    Color::hsl(0.0 + hue_offset * 30.0, 0.9, 0.5), // Red
                    Color::rgb(0.4, 0.05, 0.0),                    // Red glow
                ),
                Regime::Calm => (
                    Color::hsl(210.0 + hue_offset * 30.0, 0.7, 0.4), // Blue
                    Color::rgb(0.0, 0.02, 0.08),                     // Subtle blue glow
                ),
                _ => (
                    Color::hsl(45.0, 0.8, 0.5), // Orange/yellow
                    Color::rgb(0.15, 0.1, 0.0),
                ),
            },
        };

        // Evolved nodes pulse in size
        let scale = if matches!(cortex.neuron_type, NeuronType::Evolved(_)) {
            0.5 + (t * 2.0 + node.id as f32).sin() * 0.08
        } else {
            0.4
        };
        transform.scale = Vec3::splat(scale);

        *mat = materials.add(StandardMaterial {
            base_color,
            emissive,
            ..default()
        });
    }
}

/// 8. Gizmo Overlays: Neural connections + noise zone
fn draw_debug_overlays(
    mut gizmos: Gizmos,
    env: Res<Environment>,
    cortex_query: Query<(&Transform, &Cortex, &IoTNode)>,
) {
    use bevy::math::primitives::Direction3d;

    // Draw the Noise Zone as a subtle red sphere indicator
    let noise_pos = Vec3::new(env.noise_center.x, 0.0, env.noise_center.y);
    gizmos.circle(
        noise_pos,
        Direction3d::Y,
        env.noise_radius,
        Color::rgba(1.0, 0.2, 0.1, 0.3),
    );
    gizmos.circle(
        noise_pos + Vec3::Y * 2.0,
        Direction3d::Y,
        env.noise_radius * 0.8,
        Color::rgba(1.0, 0.1, 0.0, 0.2),
    );
    gizmos.circle(
        noise_pos - Vec3::Y * 2.0,
        Direction3d::Y,
        env.noise_radius * 0.8,
        Color::rgba(1.0, 0.1, 0.0, 0.2),
    );

    // Collect node data
    let all_nodes: Vec<(Vec3, bool, u32)> = cortex_query
        .iter()
        .map(|(t, c, n)| {
            (
                t.translation,
                matches!(c.neuron_type, NeuronType::Evolved(_)),
                n.id,
            )
        })
        .collect();

    let mut connection_count = 0;
    for i in 0..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            let (a, a_evolved, _a_id) = all_nodes[i];
            let (b, b_evolved, _b_id) = all_nodes[j];
            let dist = a.distance(b);

            if dist < CONNECTION_DISTANCE {
                connection_count += 1;
                let both_evolved = a_evolved && b_evolved;

                // Always draw evolved connections, skip some normal ones
                if !both_evolved && (connection_count % CONNECTION_SKIP != 0) {
                    continue;
                }

                // Fade connection based on distance
                let alpha = CONNECTION_ALPHA * (1.0 - dist / CONNECTION_DISTANCE);

                // Color based on node types
                let color = if a_evolved && b_evolved {
                    // Evolved-to-evolved: bright purple/magenta
                    Color::rgba(0.95, 0.2, 0.9, alpha * 2.5)
                } else if a_evolved || b_evolved {
                    // Mixed: cyan glow
                    Color::rgba(0.1, 0.9, 0.95, alpha * 2.0)
                } else {
                    // Normal: subtle blue
                    Color::rgba(0.3, 0.5, 0.8, alpha * 1.2)
                };

                gizmos.line(a, b, color);
            }
        }
    }
}

/// Setup HUD elements (separate from 3D swarm)
fn setup_hud() {
    // HUD is spawned inline in setup_swarm for simplicity
}

/// 8. Force-Directed Layout: Organic "brain" physics
fn force_directed_layout(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &IoTNode, &Cortex)>,
) {
    let dt = time.delta_seconds().min(0.05); // Clamp to avoid physics explosion on lag

    // Collect all node positions first (borrow checker workaround)
    let positions: Vec<(Entity, Vec3, bool)> = query
        .iter()
        .map(|(e, t, _, _, c)| {
            (
                e,
                t.translation,
                matches!(c.neuron_type, NeuronType::Evolved(_)),
            )
        })
        .collect();

    for (entity, mut transform, mut velocity, _node, cortex) in query.iter_mut() {
        let pos = transform.translation;
        let mut force = Vec3::ZERO;

        // 1. Repulsion from all other nodes
        for &(other_entity, other_pos, _) in &positions {
            if entity == other_entity {
                continue;
            }
            let delta = pos - other_pos;
            let dist = delta.length().max(0.5); // Prevent division by zero
            let repulsion = delta.normalize_or_zero() * REPULSION_STRENGTH / (dist * dist);
            force += repulsion;
        }

        // 2. Attraction to ALL nearby nodes (keeps brain cohesive)
        for &(other_entity, other_pos, other_evolved) in &positions {
            if entity == other_entity {
                continue;
            }
            let delta = other_pos - pos;
            let dist = delta.length();
            let is_evolved = matches!(cortex.neuron_type, NeuronType::Evolved(_));

            // Global attraction - ALL nodes pull towards each other (weaker at distance)
            let global_pull = delta.normalize_or_zero() * GLOBAL_ATTRACTION / (1.0 + dist * 0.1);
            force += global_pull;

            // Stronger spring attraction for nearby nodes
            if dist < CONNECTION_DISTANCE * 1.5 {
                // Stronger attraction for evolved pairs
                let strength = if is_evolved && other_evolved {
                    SPRING_STIFFNESS * 1.5
                } else {
                    NEIGHBOR_ATTRACTION
                };

                let displacement = dist - SPRING_REST_LENGTH;
                let spring_force = delta.normalize_or_zero() * strength * displacement;
                force += spring_force;
            }
        }

        // 3. Gravity towards center
        force += -pos * CENTER_GRAVITY;

        // 4. Surface tension - push nodes towards ideal brain radius
        let current_dist = pos.length();
        if current_dist > 0.1 {
            let target_dist = BRAIN_RADIUS * 0.85;
            let surface_force = (target_dist - current_dist) * SURFACE_TENSION;
            force += pos.normalize() * surface_force;
        }

        // 5. Apply force to velocity
        velocity.0 += force * dt;
        velocity.0 *= DAMPING;
        velocity.0 = velocity.0.clamp_length_max(MAX_VELOCITY);

        // 5. Apply velocity to position
        transform.translation += velocity.0 * dt;
    }
}

/// 10. Animate synaptic pulses between nodes
fn animate_synapses(
    time: Res<Time>,
    mut registry: ResMut<SynapseRegistry>,
    mut gizmos: Gizmos,
    query: Query<(&Transform, &IoTNode)>,
) {
    // Decay all connections
    let dt = time.delta_seconds();
    registry.connections.retain(|_, activity| *activity > 0.01);
    for (_, activity) in registry.connections.iter_mut() {
        *activity -= SYNAPSE_DECAY_RATE * dt;
        *activity = activity.max(0.0);
    }

    // Build position lookup
    let positions: HashMap<u32, Vec3> = query.iter().map(|(t, n)| (n.id, t.translation)).collect();

    // Draw active synapses with pulsing glow
    for (&(src, dst), &activity) in registry.connections.iter() {
        if let (Some(&src_pos), Some(&dst_pos)) = (positions.get(&src), positions.get(&dst)) {
            let pulse = (time.elapsed_seconds() * 8.0).sin() * 0.3 + 0.7;
            let alpha = activity * pulse;
            gizmos.line(src_pos, dst_pos, Color::rgba(0.2, 1.0, 0.8, alpha * 0.8));
        }
    }
}

/// 11. Update HUD with real-time metrics
fn update_hud(
    cortex_query: Query<&Cortex>,
    packet_query: Query<&NetworkPacket>,
    registry: Res<SynapseRegistry>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    let total = cortex_query.iter().count();
    let evolved = cortex_query
        .iter()
        .filter(|c| matches!(c.neuron_type, NeuronType::Evolved(_)))
        .count();
    let storm = cortex_query
        .iter()
        .filter(|c| c.regime == Regime::Storm)
        .count();
    let packets = packet_query.iter().count();
    let synapses = registry.connections.len();

    let entropy = if total > 0 {
        storm as f32 / total as f32
    } else {
        0.0
    };

    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!(
            "🧠 NEURAL SWARM METRICS\n\
             ├─ Nodes: {} (Evolved: {})\n\
             ├─ Entropy: {:.1}% (Storm: {})\n\
             ├─ Synapses: {} active\n\
             └─ Packets: {} in-flight",
            total,
            evolved,
            entropy * 100.0,
            storm,
            synapses,
            packets
        );
    }
}

/// 13. Interactive camera - mouse drag to orbit, scroll to zoom, Space to toggle auto-rotate
fn orbit_camera(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut controller: ResMut<CameraController>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    // Toggle auto-rotate with Space
    if keyboard.just_pressed(KeyCode::Space) {
        controller.auto_rotate = !controller.auto_rotate;
    }

    // Mouse drag to orbit
    if mouse_button.pressed(MouseButton::Left) {
        controller.auto_rotate = false; // Stop auto-rotate on interaction
        for ev in mouse_motion.read() {
            controller.yaw -= ev.delta.x * 0.005;
            controller.pitch -= ev.delta.y * 0.005;
            controller.pitch = controller.pitch.clamp(-1.4, 1.4); // Limit vertical
        }
    } else {
        mouse_motion.clear();
    }

    // Scroll to zoom
    for ev in scroll.read() {
        controller.distance -= ev.y * 1.5;
        controller.distance = controller.distance.clamp(8.0, 50.0);
    }

    // Auto-rotate when enabled
    if controller.auto_rotate {
        controller.yaw += time.delta_seconds() * 0.15;
    }

    // Apply camera transform
    for mut transform in query.iter_mut() {
        let x = controller.distance * controller.pitch.cos() * controller.yaw.cos();
        let y = controller.distance * controller.pitch.sin() + 2.0;
        let z = controller.distance * controller.pitch.cos() * controller.yaw.sin();
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

/// 14. Reset simulation - press R to restart with fresh nodes
fn reset_simulation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Cortex>,
    mut registry: ResMut<SynapseRegistry>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        println!("🔄 RESET: Clearing all evolved genes and restarting simulation...");

        // Clear saved gene files
        if let Ok(entries) = fs::read_dir("./swarms_memory") {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .extension()
                    .map(|e| e == "bin")
                    .unwrap_or(false)
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        // Reset all nodes to Linear (unevolved) state
        for mut cortex in query.iter_mut() {
            cortex.neuron_type = NeuronType::Linear(LinearNeuron::new(32));
            cortex.regime = Regime::Calm;
            cortex.time_in_storm = 0.0;
        }

        // Clear synapse activity
        registry.connections.clear();

        println!("✅ Reset complete! Watch the swarm evolve again.");
    }
}
