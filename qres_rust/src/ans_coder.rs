use constriction::ans::Coder;
use constriction::stream::model::LeakyQuantizer;
use probability::distribution::Gaussian;

// QRES v3.0 Entropy Backend
// Strategy: Range Coding (Queue) with Gaussian Modeling.
// REPLACES: The 'bincode' fallback.

pub struct AnsWriter {
    encoder: DefaultRangeEncoder,
}

impl AnsWriter {
    pub fn new() -> Self {
        AnsWriter {
            encoder: DefaultRangeEncoder::new(),
        }
    }

    pub fn write_residual(&mut self, residual: i8) {
        // Model: Quantized Gaussian centered at 0 with std dev = 1.0.
        // This effectively compresses small residuals (errors) into very few bits.
        // The loop in lib.rs runs FORWARD, so we use RangeEncoder (FIFO).
        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(0.0, 1.0));
        // Map i8 residual to i32 symbol space for constriction
        self.encoder.encode_symbol(residual as i32, &model).unwrap();
    }

    pub fn finish(mut self) -> Vec<u8> {
        // Range Coding requires 'sealing' to flush the final bits
        let compressed_words: Vec<u32> = self.encoder.into_compressed().unwrap();
        let mut result = Vec::new();
        for &word in &compressed_words {
            result.extend_from_slice(&word.to_le_bytes());
        }
        result
    }
}

pub struct AnsReader {
    decoder: DefaultRangeDecoder,
}

impl AnsReader {
    pub fn new(data: &[u8]) -> Self {
        // Convert bytes to u32 words
        let mut words = Vec::new();
        let mut i = 0;
        while i + 3 < data.len() {
            let word = u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]);
            words.push(word);
            i += 4;
        }
        
        // Initialize decoder from the compressed byte stream
        let decoder = DefaultRangeDecoder::from_compressed(words).unwrap();
        
        AnsReader { decoder }
    }

    pub fn read_residual(&mut self) -> i8 {
        // Define the same model used for encoding
        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(0.0, 1.0));
        
        // Decode next symbol
        // If the stream is exhausted or invalid, we default to 0 (no residual)
        let val = self.decoder.decode_symbol(&model).unwrap_or(0);
        
        // Clamp to i8 range
        val as i8
    }
}