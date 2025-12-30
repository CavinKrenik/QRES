use constriction::stream::model::LeakyQuantizer;
use constriction::stream::queue::{DefaultRangeDecoder, DefaultRangeEncoder};
use constriction::stream::{Decode, Encode};
use probability::distribution::Gaussian;

// QRES v3.0 Adaptive Entropy Backend
// Strategy: Range Coding with Adaptive Gaussian Modeling
// Uses Welford's online algorithm to track residual distribution in real-time

pub struct AnsWriter {
    encoder: DefaultRangeEncoder,
    // Welford's online statistics for adaptive modeling
    running_mean: f64,
    running_var: f64,
    count: usize,
}

impl Default for AnsWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsWriter {
    pub fn new() -> Self {
        AnsWriter {
            encoder: DefaultRangeEncoder::new(),
            running_mean: 0.0,
            running_var: 0.0,
            count: 0,
        }
    }

    pub fn write_residual(&mut self, residual: i8) {
        // Calculate adaptive std based on running statistics
        // Initial std=32.0 based on empirical analysis (actual std ~36.1)
        let std = if self.count == 0 {
            32.0
        } else {
            ((self.running_var / (self.count - 1) as f64).sqrt()).max(1e-6)
        };
        
        // Use adaptive Gaussian model
        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(self.running_mean, std));
        
        // Encode residual
        self.encoder.encode_symbol(residual as i32, model).unwrap();
        
        // Update statistics AFTER encoding (for next symbol)
        // Welford's algorithm for numerical stability
        let res_f = residual as f64;
        self.count += 1;
        let delta = res_f - self.running_mean;
        self.running_mean += delta / self.count as f64;
        let delta2 = res_f - self.running_mean;
        self.running_var += delta * delta2;
    }

    pub fn finish(self) -> Vec<u8> {
        // Range Coding requires 'sealing' to flush the final bits
        let compressed_words: Vec<u32> = self.encoder.into_compressed().unwrap();
        let mut result = Vec::new();
        for word in &compressed_words {
            result.extend_from_slice(&word.to_le_bytes());
        }
        
        // Optional: Log final statistics for debugging
        #[cfg(debug_assertions)]
        if self.count > 0 {
            let final_std = (self.running_var / (self.count - 1) as f64).sqrt();
            eprintln!("[ANS] Encoded {} residuals: mean={:.2}, std={:.2}", 
                     self.count, self.running_mean, final_std);
        }
        
        result
    }
}

pub struct AnsReader {
    decoder: DefaultRangeDecoder,
    // Mirror encoder's statistics for symmetric decoding
    running_mean: f64,
    running_var: f64,
    count: usize,
}

impl AnsReader {
    pub fn new(data: &[u8]) -> Self {
        // Convert bytes to u32 words
        let mut words = Vec::new();
        let mut i = 0;
        while i + 3 < data.len() {
            let word = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            words.push(word);
            i += 4;
        }
        
        // Initialize decoder from the compressed byte stream
        let decoder = DefaultRangeDecoder::from_compressed(words).unwrap();
        
        AnsReader {
            decoder,
            running_mean: 0.0,
            running_var: 0.0,
            count: 0,
        }
    }

    pub fn read_residual(&mut self) -> i8 {
        // Calculate adaptive std (MUST match encoder exactly)
        let std = if self.count == 0 {
            32.0
        } else {
            ((self.running_var / (self.count - 1) as f64).sqrt()).max(1e-6)
        };
        
        // Use same adaptive Gaussian model as encoder
        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(self.running_mean, std));
        
        // Decode symbol
        let val = self.decoder.decode_symbol(model).unwrap_or(0);
        let residual = val as i8;
        
        // Update statistics with DECODED residual (symmetric to encoder)
        let res_f = residual as f64;
        self.count += 1;
        let delta = res_f - self.running_mean;
        self.running_mean += delta / self.count as f64;
        let delta2 = res_f - self.running_mean;
        self.running_var += delta * delta2;
        
        residual
    }
}
