//! Brain Aggregator for Robust Federated Learning
//!
//! Buffers brain updates from peers and applies robust aggregation
//! algorithms (Krum, Median, Trimmed Mean) before merging.
//! Part of Phase 2 Security implementation.

use crate::config::AggregationConfig;
use crate::living_brain::LivingBrain;
use qres_core::aggregation::{aggregate_updates, AggregationMode, AggregationResult};
use std::collections::VecDeque;
use tracing::{info, warn};

/// Aggregator that buffers brain updates and applies robust aggregation
pub struct BrainAggregator {
    /// Buffered confidence vectors from remote peers (Update, PeerID)
    buffer: VecDeque<(Vec<f32>, String)>,
    /// Configuration for aggregation
    config: AggregationConfig,
    /// Derived aggregation mode
    mode: AggregationMode,
}

impl BrainAggregator {
    /// Create a new aggregator from config
    pub fn new(config: AggregationConfig) -> Self {
        let mode = Self::parse_mode(&config);
        info!(
            mode = ?config.mode,
            buffer_size = config.buffer_size,
            "Brain aggregator initialized"
        );

        Self {
            buffer: VecDeque::with_capacity(config.buffer_size),
            config,
            mode,
        }
    }

    /// Parse aggregation mode from config string  
    fn parse_mode(config: &AggregationConfig) -> AggregationMode {
        match config.mode.to_lowercase().as_str() {
            "krum" => AggregationMode::Krum {
                expected_byz: 1, // Will be calculated dynamically based on buffer size
            },
            "multi_krum" => AggregationMode::MultiKrum {
                expected_byz: 1,
                k: 3,
            },
            "trimmed_mean" | "trimmed" => AggregationMode::TrimmedMean {
                trim_fraction: config.trim_fraction,
            },
            "median" => AggregationMode::Median,
            _ => AggregationMode::SimpleMean,
        }
    }

    /// Add a brain update to the buffer
    /// Returns Some((aggregated confidence, accepted_peers, rejected_peers)) if buffer is full and ready for aggregation
    pub fn add_update(
        &mut self,
        brain: &LivingBrain,
        peer_id: String,
    ) -> Option<(Vec<f32>, Vec<String>, Vec<String>)> {
        // Add confidence vector to buffer
        self.buffer.push_back((brain.confidence.clone(), peer_id));

        // Check if we have enough updates to aggregate
        if self.buffer.len() >= self.config.buffer_size {
            Some(self.aggregate_and_clear())
        } else {
            info!(
                buffered = self.buffer.len(),
                needed = self.config.buffer_size,
                "Update buffered, waiting for more"
            );
            None
        }
    }

    /// Force aggregation with current buffer (for timeout scenarios)
    pub fn force_aggregate(&mut self) -> Option<(Vec<f32>, Vec<String>, Vec<String>)> {
        if self.buffer.is_empty() {
            return None;
        }
        Some(self.aggregate_and_clear())
    }

    /// Aggregate buffered updates and clear the buffer
    fn aggregate_and_clear(&mut self) -> (Vec<f32>, Vec<String>, Vec<String>) {
        // Separate updates and peer_ids
        let (updates, peer_ids): (Vec<Vec<f32>>, Vec<String>) = self.buffer.drain(..).unzip();
        let n = updates.len();

        // Calculate expected byzantines dynamically based on fraction
        let expected_byz = ((n as f32) * self.config.expected_byzantines_fraction).floor() as usize;

        // Create dynamic mode with calculated byz count
        let dynamic_mode = match &self.mode {
            AggregationMode::Krum { .. } => AggregationMode::Krum { expected_byz },
            AggregationMode::MultiKrum { k, .. } => AggregationMode::MultiKrum {
                expected_byz,
                k: (*k).min(n),
            },
            other => other.clone(),
        };

        let result: AggregationResult = aggregate_updates(&updates, &dynamic_mode);

        info!(
            updates = n,
            selected = result.selected_indices.len(),
            rejected = result.rejected_indices.len(),
            mode = ?self.config.mode,
            "Aggregated brain updates"
        );

        if !result.rejected_indices.is_empty() {
            warn!(
                rejected = ?result.rejected_indices,
                "Rejected potential Byzantine updates"
            );
        }

        let rejected_peers: Vec<String> = result
            .rejected_indices
            .iter()
            .map(|&idx| peer_ids[idx].clone())
            .collect();

        let accepted_peers: Vec<String> = result
            .selected_indices
            .iter()
            .map(|&idx| peer_ids[idx].clone())
            .collect();

        (result.weights, accepted_peers, rejected_peers)
    }

    /// Get current buffer size
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if using robust mode (not simple mean)
    pub fn is_robust(&self) -> bool {
        !matches!(self.mode, AggregationMode::SimpleMean)
    }
}

/// Apply aggregated confidence to a brain
pub fn apply_aggregated_confidence(brain: &mut LivingBrain, aggregated: &[f32], alpha: f32) {
    for (conf, &agg) in brain.confidence.iter_mut().zip(aggregated.iter()) {
        *conf = *conf * (1.0 - alpha) + agg * alpha;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator_buffering() {
        let config = AggregationConfig {
            mode: "mean".to_string(),
            expected_byzantines_fraction: 0.2,
            buffer_size: 3,
            trim_fraction: 0.2,
        };

        let mut agg = BrainAggregator::new(config);

        let brain1 = LivingBrain::new();
        let brain2 = LivingBrain::new();

        // First two shouldn't trigger aggregation
        assert!(agg.add_update(&brain1, "peer1".to_string()).is_none());
        assert!(agg.add_update(&brain2, "peer2".to_string()).is_none());
        assert_eq!(agg.buffer_len(), 2);

        // Third should trigger
        let result = agg.add_update(&brain1, "peer3".to_string());
        assert!(result.is_some());
        assert_eq!(agg.buffer_len(), 0);
    }

    #[test]
    fn test_krum_mode() {
        let config = AggregationConfig {
            mode: "krum".to_string(),
            expected_byzantines_fraction: 0.2,
            buffer_size: 5,
            trim_fraction: 0.2,
        };

        let agg = BrainAggregator::new(config);
        assert!(agg.is_robust());
    }
}
