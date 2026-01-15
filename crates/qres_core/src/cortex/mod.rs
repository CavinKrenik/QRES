/// The "Cortex" module: Active neural processing for the swarm
///
/// This module replaces passive predictors with active neurons that:
/// 1. Predict the next value (hot path)
/// 2. Detect surprise (anomaly detection via refractory logic)
/// 3. Adapt based on peer signals (collective learning)
/// 4. Export/import genes (behavior propagation)
///
/// Design constraints:
/// - No std: All types use core::* and alloc
/// - No String: All communication is Copy or Vec<u8>
/// - Fixed-point math: I16F16 for determinism across platforms
pub mod linear;
pub mod neuron;
pub mod storage;

pub use linear::LinearNeuron;
pub use neuron::{Regime, SpikeEvent, SwarmNeuron};
pub use storage::GeneStorage;
