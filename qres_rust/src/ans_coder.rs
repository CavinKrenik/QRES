use constriction::stream::model::LeakyQuantizer;
use constriction::stream::queue::{DefaultRangeDecoder, DefaultRangeEncoder};
use constriction::stream::{Decode, Encode};
use probability::distribution::Gaussian;

// QRES v4.0 "Hive-Optimized" Backend
// Strategy: Lazy Adaptive ANS with Batched Updates
// Batch Size: 64 ensures we only calculate optimal params/models once per 64 bytes
const BATCH_SIZE: usize = 64;

pub struct AnsWriter {
    encoder: DefaultRangeEncoder,
    // Welford's online statistics
    running_mean: f64,
    running_var: f64,
    count: usize,
    // Batching buffer
    buffer: Vec<i8>,
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
            buffer: Vec::with_capacity(BATCH_SIZE),
        }
    }

    pub fn write_residual(&mut self, residual: i8) {
        self.buffer.push(residual);

        if self.buffer.len() >= BATCH_SIZE {
            self.flush_buffer();
        }
    }

    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        // 1. Calculate Statistics & Create Model (Once per batch)
        let std = if self.count == 0 {
            32.0 // Initial guess
        } else {
            ((self.running_var / (self.count).max(1) as f64).sqrt()).max(1e-6)
        };

        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(self.running_mean, std));

        // 2. Encode and Update Stats Loop
        for &res in &self.buffer {
            // Encode
            self.encoder.encode_symbol(res as i32, &model).unwrap();

            // Quick Welford Update
            self.count += 1;
            let val = res as f64;
            let delta = val - self.running_mean;
            self.running_mean += delta / self.count as f64;
            // running_var += delta * (val - new_mean)
            self.running_var += delta * (val - self.running_mean);
        }

        self.buffer.clear();
    }

    pub fn finish(mut self) -> Vec<u8> {
        // Encode remaining symbols in buffer
        self.flush_buffer();

        // Seal the stream
        let compressed_words: Vec<u32> = self.encoder.into_compressed().unwrap();
        let mut result = Vec::new();
        for word in &compressed_words {
            result.extend_from_slice(&word.to_le_bytes());
        }

        result
    }
}

pub struct AnsReader {
    decoder: DefaultRangeDecoder,
    // Statistics
    running_mean: f64,
    running_var: f64,
    count: usize,
    // Decode Buffer
    buffer: Vec<i8>,
    buffer_pos: usize,
}

impl AnsReader {
    pub fn new(data: &[u8]) -> Self {
        let mut words = Vec::new();
        let mut i = 0;
        while i + 3 < data.len() {
            let word = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            words.push(word);
            i += 4;
        }

        let decoder = DefaultRangeDecoder::from_compressed(words).unwrap();

        AnsReader {
            decoder,
            running_mean: 0.0,
            running_var: 0.0,
            count: 0,
            buffer: Vec::with_capacity(BATCH_SIZE),
            buffer_pos: 0,
        }
    }

    pub fn read_residual(&mut self) -> i8 {
        // Refill if empty
        if self.buffer_pos >= self.buffer.len() {
            self.refill_buffer();
        }

        // Return next
        let res = self.buffer[self.buffer_pos];
        self.buffer_pos += 1;
        res
    }

    fn refill_buffer(&mut self) {
        self.buffer.clear();
        self.buffer_pos = 0;

        // 1. Calculate Model (Once per batch)
        let std = if self.count == 0 {
            32.0
        } else {
            ((self.running_var / (self.count).max(1) as f64).sqrt()).max(1e-6)
        };

        let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
        let model = quantizer.quantize(Gaussian::new(self.running_mean, std));

        // 2. Batch Decode (64 symbols)
        // Note: It's safe to over-decode because we rely on the caller (lib.rs)
        // to stop demanding symbols when it has enough. Over-decoded symbols (0s)
        // sit in the buffer unused.
        for _ in 0..BATCH_SIZE {
            let val = self.decoder.decode_symbol(&model).unwrap_or(0);
            let res = val as i8;
            self.buffer.push(res);

            // Update stats immediately to match encoder
            self.count += 1;
            let val_f = res as f64;
            let delta = val_f - self.running_mean;
            self.running_mean += delta / self.count as f64;
            self.running_var += delta * (val_f - self.running_mean);
        }
    }
}
