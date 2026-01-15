use core::fmt;
use fixed::types::I16F16;

/// Represents the operational regime of a SwarmNeuron
/// Used to determine surprise thresholds and adaptation rates
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    /// Stable operation: Low entropy, predictable signals
    /// Surprise threshold is high, low learning rate
    Calm,
    /// Elevated uncertainty: Moderate entropy, some anomalies
    /// Surprise threshold is moderate, moderate learning rate
    Alert,
    /// System-wide disruption: High entropy, many prediction errors
    /// Surprise threshold is low, high learning rate
    Storm,
}

impl Regime {
    /// Get the surprise threshold for this regime (as I16F16)
    /// Higher threshold = harder to trigger a spike
    pub fn surprise_threshold(self) -> I16F16 {
        match self {
            Regime::Calm => I16F16::from_num(0.3),   // 30% entropy
            Regime::Alert => I16F16::from_num(0.15), // 15% entropy
            Regime::Storm => I16F16::from_num(0.05), // 5% entropy
        }
    }

    /// Get the learning rate for this regime
    /// Higher rate = faster adaptation
    pub fn learning_rate(self) -> I16F16 {
        match self {
            Regime::Calm => I16F16::from_num(0.01), // Slow, conservative learning
            Regime::Alert => I16F16::from_num(0.05), // Moderate adaptation
            Regime::Storm => I16F16::from_num(0.2), // Aggressive learning
        }
    }
}

/// A spike event: The neuron detected surprise and is broadcasting to peers
/// Lightweight, Copy type for network propagation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpikeEvent {
    /// Local timestamp (tick count)
    pub tick: u32,
    /// The prediction error that triggered the spike
    pub error: u8,
    /// Surprise magnitude (as entropy estimate)
    pub surprise: u8, // 0-255 range, will be cast from I16F16
    /// Regime at time of spike
    pub regime: Regime,
}

impl SpikeEvent {
    /// Create a new spike event
    pub fn new(tick: u32, error: u8, surprise: u8, regime: Regime) -> Self {
        SpikeEvent {
            tick,
            error,
            surprise,
            regime,
        }
    }
}

impl fmt::Display for SpikeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Spike[t={}, err={}, surp={}, regime={:?}]",
            self.tick, self.error, self.surprise, self.regime
        )
    }
}

/// The "Active Neuron" trait for a distributed neural swarm
/// Each neuron processes local data, detects anomalies, and evolves via gossip
pub trait SwarmNeuron: Clone {
    /// 1. HOT PATH: Pure prediction math, must be < 50us
    ///
    /// Given a history window, predict the next value
    /// # Arguments
    /// * `history` - Recent observed values (typically 4-64 bytes)
    /// # Returns
    /// A predicted u8 value
    fn predict(&self, history: &[u8]) -> u8;

    /// 2. THE GATED SIGNAL: Returns SpikeEvent only if conditions met
    ///
    /// Detects surprise (prediction error) and checks refractory period.
    /// Only emits an event if:
    /// - The prediction error exceeds regime-specific threshold AND
    /// - The refractory counter is 0 (neuron is "ready to spike")
    ///
    /// # Arguments
    /// * `actual` - The actual observed value
    /// * `predicted` - The value predicted by `predict()`
    /// * `regime` - Current operational regime (affects thresholds)
    ///
    /// # Returns
    /// `Some(SpikeEvent)` if surprise exceeded threshold AND refractory is 0
    /// `None` otherwise
    fn check_surprise(
        &mut self,
        actual: u8,
        predicted: u8,
        regime: Regime,
        tick: u32,
    ) -> Option<SpikeEvent>;

    /// 3. PLASTICITY: Learn from peer signals and reputation
    ///
    /// Updates internal state based on recent spike signals from neighbors
    /// and their reputation scores. This is how the swarm achieves collective learning.
    ///
    /// # Arguments
    /// * `signals` - Recent spike events from peer neurons
    /// * `reputation` - Reputation scores for each signaling peer (I16F16: -1.0 to 1.0)
    fn adapt(&mut self, signals: &[SpikeEvent], reputation: &[I16F16]);

    /// 4. GENETICS: Serialize internal state for network propagation
    ///
    /// Returns a compact byte representation that can be:
    /// - Sent over the network to peers
    /// - Stored in persistent memory
    /// - Used to clone a successful neuron's strategy
    ///
    /// # Returns
    /// A Vec<u8> containing the serialized state
    fn export_gene(&self) -> alloc::vec::Vec<u8>;

    /// 5. INSTALL GENETICS: Restore internal state from a received gene
    ///
    /// Deserializes and installs a gene received from a peer.
    /// This is the "healing" mechanism: a struggling neuron adopts
    /// a successful strategy from a high-reputation neighbor.
    ///
    /// # Arguments
    /// * `gene` - Serialized state from a peer neuron
    /// # Returns
    /// `true` if gene was valid and installed, `false` otherwise
    fn install_gene(&mut self, gene: &[u8]) -> bool;

    /// Get the current refractory period remaining (in ticks)
    /// When 0, the neuron can spike again
    fn refractory_remaining(&self) -> u32;

    /// Decrement internal tick counter (called every system tick)
    /// Updates refractory period and any other time-based state
    fn tick(&mut self);
}
