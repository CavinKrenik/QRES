use std::collections::HashMap;
use std::collections::VecDeque;

// QRES v4.0 Predictors
// 1. SimplePredictor: Order-1 Markov Context (Fast Text)
// 2. GraphPredictor: DAG-based Long-Range Dependency Model (Telemetry/Logs)

// --- Simple Predictor (Text/Code) ---
pub struct SimplePredictor {
    prev: u8,
    context: [u8; 256], 
}

impl SimplePredictor {
    pub fn new() -> Self {
        SimplePredictor {
            prev: 0,
            context: [0; 256],
        }
    }

    pub fn predict_next(&self) -> u8 {
        self.context[self.prev as usize]
    }

    pub fn update(&mut self, actual: u8) {
        self.context[self.prev as usize] = actual;
        self.prev = actual;
    }
}

// --- Graph Predictor (Telemetry/Complex Patterns) ---
// Replaces the experimental iPEPS model with a concrete DAG-based learner.
// Captures dependencies at specific lag intervals (edges).

pub struct GraphPredictor {
    weights: Vec<f32>,
    edges: Vec<usize>,       // Lags: e.g. [1, 2, 4, 8]
    history: VecDeque<u8>,   // Sliding window
    learning_rate: f32,
}

impl GraphPredictor {
    pub fn new() -> Self {
        // Define DAG Topology: Sparse connections to past
        // Captures immediate context (1,2,3) and byte-aligned structures (4,8,16)
        let edges = vec![1, 2, 3, 4, 8, 16, 32];
        let max_lag = *edges.last().unwrap();
        
        // Initialize weights to prefer immediate neighbor
        let mut weights = vec![0.0; edges.len()];
        weights[0] = 0.9;
        
        GraphPredictor {
            weights,
            edges,
            history: VecDeque::from(vec![0; max_lag + 1]), // Pre-fill with zeros
            learning_rate: 0.01,
        }
    }

    pub fn predict_next(&self) -> u8 {
        let mut sum = 0.0;
        let hist_len = self.history.len();
        
        for (i, &lag) in self.edges.iter().enumerate() {
            // History is pushed back, so index 0 is oldest? 
            // Better to use back() as most recent.
            // history[len - 1] is t-1
            // history[len - lag] is t-lag
            if lag <= hist_len {
                let val = self.history[hist_len - lag];
                sum += (val as f32) * self.weights[i];
            }
        }
        
        sum.clamp(0.0, 255.0) as u8
    }

    pub fn update(&mut self, actual: u8) {
        let pred = self.predict_next() as f32; // Re-calculate or cache? caching is hard, recalc is cheap
        let err = actual as f32 - pred;
        
        let hist_len = self.history.len();
        
        // LMS Update for Weights
        for (i, &lag) in self.edges.iter().enumerate() {
            if lag <= hist_len {
                let val = self.history[hist_len - lag] as f32;
                // w = w + nu * error * x
                // Normalize input range (0-255) to avoid explosion
                self.weights[i] += self.learning_rate * err * (val / 255.0) * 0.01;
            }
        }
        
        // Push actual to history
        self.history.push_back(actual);
        if self.history.len() > 33 { // Keep buffer small
             self.history.pop_front();
        }
    }
}
