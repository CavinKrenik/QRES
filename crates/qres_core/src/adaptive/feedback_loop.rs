use super::regime_detector::{RegimeChange, RegimeDetector};

pub struct FeedbackLoop {
    detector: RegimeDetector,
}

impl FeedbackLoop {
    pub fn new(window_size: usize) -> Self {
        Self {
            detector: RegimeDetector::new(window_size, 0.8, 1000000.0), // Default thresholds
        }
    }

    /// Observe a prediction vs actual value.
    /// Calculates deviation and checks for regime changes.
    pub fn observe(&mut self, prediction: f32, actual: f32) {
        let error = prediction - actual;
        match self.detector.observe(error) {
            RegimeChange::Drift {
                current_error: _current_error,
                threshold: _threshold,
            } => {
                // Log warning (using standard eprintln mechanism for now, or log crate if available)
                // In no_std environment this might need a different reporting mechanism
                #[cfg(feature = "std")]
                {
                    eprintln!(
                        "[FeedbackLoop] DRIFT DETECTED! Error: {:.4} > Threshold: {:.4}",
                        _current_error, _threshold
                    );
                }

                // TODO: Trigger adaptation (e.g., lower hybrid threshold)
            }
            RegimeChange::None => {}
        }
    }
}
