// Deterministic Spectral Predictor (stub)
// The previous FFT-based predictor used f32 + libm + rustfft, which introduced
// cross-architecture drift (x86 AVX vs ARM scalar). For v18 determinism, we
// replace it with a stable, lightweight predictor that simply echoes the last
// observed value. This preserves API compatibility while guaranteeing
// identical behavior across platforms.

pub struct SpectralPredictor {
    last: u8,
}

impl SpectralPredictor {
    pub fn new(_window_size: usize) -> Self {
        SpectralPredictor { last: 0 }
    }

    pub fn update(&mut self, val: u8) {
        self.last = val;
    }

    pub fn predict(&mut self) -> u8 {
        self.last
    }

    /// Reset internal state to initial values without reallocating memory.
    pub fn reset(&mut self) {
        self.last = 0;
    }
}
