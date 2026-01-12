use crate::multivariate::correlation::PearsonCorrelation;
use std::collections::{HashMap, HashSet};

/// A group of streams identified as highly correlated.
#[derive(Debug, Clone)]
pub struct StreamGroup {
    /// The primary stream ID (the "leader" or "base").
    pub leader: String,
    /// Streams that are correlated with the leader.
    pub members: Vec<String>,
}

pub struct MultivariateManager;

impl MultivariateManager {
    /// Scans a set of streams and groups them by correlation.
    ///
    /// This is an O(N^2) operation intended to be run periodically, not on every sample.
    ///
    /// Args:
    ///     streams: Map of StreamID -> Data History.
    ///     threshold: Correlation score (0.0 to 1.0) required to group.
    ///
    /// Returns:
    ///     Vec<StreamGroup>: List of detected groups.
    pub fn find_groups(streams: &HashMap<String, Vec<f32>>, threshold: f32) -> Vec<StreamGroup> {
        let mut groups = Vec::new();
        let mut assigned = HashSet::new();

        // Sort keys for deterministic behavior
        let mut ids: Vec<&String> = streams.keys().collect();
        ids.sort();

        for (i, leader_id) in ids.iter().enumerate() {
            if assigned.contains(*leader_id) {
                continue;
            }

            let mut members = Vec::new();
            let leader_data = &streams[*leader_id];

            // Compare against all subsequent streams
            for other_id in ids.iter().skip(i + 1) {
                if assigned.contains(*other_id) {
                    continue;
                }

                let other_data = &streams[*other_id];
                let score = PearsonCorrelation::calculate(leader_data, other_data);

                if score.abs() >= threshold {
                    members.push(other_id.to_string());
                    assigned.insert(other_id.to_string());
                }
            }

            if !members.is_empty() {
                assigned.insert(leader_id.to_string());
                groups.push(StreamGroup {
                    leader: leader_id.to_string(),
                    members,
                });
            }
        }

        groups
    }
}
