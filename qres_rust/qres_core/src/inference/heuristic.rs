/// Simple heuristic predictor using Weighted Moving Average
pub struct MovingAveragePredictor {
    _window_size: usize,
}

impl MovingAveragePredictor {
    pub fn new(window_size: usize) -> Self {
        Self {
            _window_size: window_size,
        }
    }

    pub fn predict(&self, window: &[f32]) -> f32 {
        if window.is_empty() {
            return 0.0;
        }

        let mut sum = 0.0;
        let mut weight_sum = 0.0;

        // Give more weight to recent values (Linear decay)
        for (i, &val) in window.iter().enumerate() {
            let weight = (i + 1) as f32;
            sum += val * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            sum / weight_sum
        } else {
            0.0
        }
    }
}
