// Constriction-based ANS Coding for QRES residuals

use constriction::stream::stack::DefaultAnsCoder;
use constriction::stream::Decode;
use constriction::stream::model::DefaultLeakyQuantizer;
use probability::distribution::Gaussian;

pub struct AnsWriter {
    residuals: Vec<i8>,
}

impl AnsWriter {
    pub fn new() -> Self {
        AnsWriter { residuals: Vec::new() }
    }

    pub fn write_residual(&mut self, residual: i8) {
        self.residuals.push(residual);
    }

    pub fn finish(self) -> Vec<u8> {
        if self.residuals.is_empty() {
            return vec![];
        }

        let mut coder = DefaultAnsCoder::new();

        // Map i8 to usize: -128..127 -> 0..255
        let symbols: Vec<usize> = self.residuals.iter().map(|&r| (r as i16 + 128) as usize).collect();

        // Entropy model favoring small residuals (Laplacian-like)
        let mut probabilities: Vec<f64> = (0..256).map(|i| {
            let residual = (i as i16) - 128;
            1.0 / (residual.abs() + 1) as f64
        }).collect();
        // Normalize
        let sum: f64 = probabilities.iter().sum();
        probabilities.iter_mut().for_each(|p| *p /= sum);
        let model: ContiguousCategoricalEntropyModel<u32, _, 24> = ContiguousCategoricalEntropyModel::from_floating_point_probabilities_fast(&probabilities, None).unwrap();

        // Encode in reverse order (ANS stack semantics)
        coder.encode_symbols_reverse(symbols.iter().map(|&s| (s, &model))).unwrap();

        // Get compressed as u32 words
        let compressed_words: Vec<u32> = coder.into_compressed().unwrap();

        // Convert to bytes, little endian
        let mut result = Vec::new();
        for &word in &compressed_words {
            result.extend_from_slice(&word.to_le_bytes());
        }
        result
    }
}

pub struct AnsReader<'a> {
    residuals: std::vec::IntoIter<i8>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> AnsReader<'a> {
    pub fn new(data: &[u8], num_residuals: usize) -> Self {
        if num_residuals == 0 {
            return AnsReader {
                residuals: vec![].into_iter(),
                _marker: std::marker::PhantomData,
            };
        }

        // Convert bytes to u32 words, little endian
        let mut words = Vec::new();
        let mut i = 0;
        while i + 3 < data.len() {
            let word = u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]);
            words.push(word);
            i += 4;
        }

        // Create coder from compressed data
        let mut coder = DefaultAnsCoder::from_compressed(words).unwrap();

        // Same model
        let mut probabilities: Vec<f64> = (0..256).map(|i| {
            let residual = (i as i16) - 128;
            1.0 / (residual.abs() + 1) as f64
        }).collect();
        // Normalize
        let sum: f64 = probabilities.iter().sum();
        probabilities.iter_mut().for_each(|p| *p /= sum);
        let model: ContiguousCategoricalEntropyModel<u32, _, 24> = ContiguousCategoricalEntropyModel::from_floating_point_probabilities_fast(&probabilities, None).unwrap();

        // Decode exactly num_residuals symbols
        let decoded_symbols: Vec<usize> = coder.decode_symbols(std::iter::repeat(&model).take(num_residuals)).map(|r: Result<usize, _>| r.unwrap()).collect();

        // Map back to i8
        let residuals: Vec<i8> = decoded_symbols.iter().map(|&s| ((s as i16) - 128) as i8).collect();

        AnsReader {
            residuals: residuals.into_iter(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn read_residual(&mut self) -> i8 {
        self.residuals.next().unwrap_or(0)
    }
}