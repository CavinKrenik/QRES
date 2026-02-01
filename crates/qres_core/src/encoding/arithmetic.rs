use crate::encoding::probability::AdaptiveModel;

/// Simple range coder for compressing byte streams.
///
/// This is a basic implementation optimized for peaked distributions
/// commonly found in prediction residuals.
pub struct RangeCoder {
    low: u64,
    range: u64,
    buffer: Vec<u8>,
}

impl Default for RangeCoder {
    fn default() -> Self {
        Self::new()
    }
}

impl RangeCoder {
    // const TOP: u64 = 1 << 24;
    const BOTTOM: u64 = 1 << 16;

    pub fn new() -> Self {
        Self {
            low: 0,
            range: u64::MAX >> 8,
            buffer: Vec::new(),
        }
    }

    /// Encodes a single symbol using the provided probability model.
    pub fn encode(&mut self, symbol: u8, model: &AdaptiveModel) {
        let (start, count, total) = model.get_probability(symbol);

        self.range /= total as u64;
        self.low += start as u64 * self.range;
        self.range *= count as u64;

        while self.range < Self::BOTTOM {
            self.buffer.push((self.low >> 56) as u8);
            self.low <<= 8;
            self.range <<= 8;
        }
    }

    /// Flushes the remaining state and returns the compressed bytes.
    pub fn finish(mut self) -> Vec<u8> {
        // Flush remaining bytes
        for _ in 0..8 {
            self.buffer.push((self.low >> 56) as u8);
            self.low <<= 8;
        }
        self.buffer
    }
}

/// Range decoder for decompressing byte streams.
pub struct RangeDecoder<'a> {
    low: u64,
    range: u64,
    code: u64,
    input: &'a [u8],
    pos: usize,
}

impl<'a> RangeDecoder<'a> {
    // const TOP: u64 = 1 << 24;
    const BOTTOM: u64 = 1 << 16;

    pub fn new(input: &'a [u8]) -> Self {
        let mut decoder = Self {
            low: 0,
            range: u64::MAX >> 8,
            code: 0,
            input,
            pos: 0,
        };

        // Initialize code from first 8 bytes
        for _ in 0..8 {
            decoder.code = (decoder.code << 8) | decoder.next_byte() as u64;
        }

        decoder
    }

    fn next_byte(&mut self) -> u8 {
        if self.pos < self.input.len() {
            let b = self.input[self.pos];
            self.pos += 1;
            b
        } else {
            0
        }
    }

    /// Decodes a single symbol using the provided probability model.
    pub fn decode(&mut self, model: &AdaptiveModel) -> u8 {
        let total = model.total();
        self.range /= total as u64;

        let offset = ((self.code - self.low) / self.range) as u32;
        let symbol = model.symbol_from_count(offset.min(total - 1));

        let (start, count, _) = model.get_probability(symbol);
        self.low += start as u64 * self.range;
        self.range *= count as u64;

        while self.range < Self::BOTTOM {
            self.code = (self.code << 8) | self.next_byte() as u64;
            self.low <<= 8;
            self.range <<= 8;
        }

        symbol
    }
}

/// Compresses residual data using adaptive range coding.
///
/// This is optimal for peaked distributions (many zeros/small values) commonly
/// found in time-series prediction residuals.
///
/// Args:
///     data: The residual bytes to compress.
///
/// Returns:
///     The compressed byte stream.
pub fn compress_residuals(data: &[u8]) -> Vec<u8> {
    let mut model = AdaptiveModel::new();
    let mut encoder = RangeCoder::new();

    for &byte in data {
        encoder.encode(byte, &model);
        model.update(byte);
    }

    encoder.finish()
}

/// Decompresses data that was compressed with compress_residuals.
///
/// Args:
///     data: The compressed byte stream.
///     original_len: The expected length of the decompressed data.
///
/// Returns:
///     The original residual bytes.
pub fn decompress_residuals(data: &[u8], original_len: usize) -> Vec<u8> {
    let mut model = AdaptiveModel::new();
    let mut decoder = RangeDecoder::new(data);

    let mut output = Vec::with_capacity(original_len);

    for _ in 0..original_len {
        let symbol = decoder.decode(&model);
        output.push(symbol);
        model.update(symbol);
    }

    output
}

/// Compresses BFP-16 vector components (exponent + mantissas)
/// Format: [exponent: i8][mantissas: 16-bit BE...]
pub fn compress_bfp(exponent: i8, mantissas: &[i16]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + mantissas.len() * 2);
    // 1. Shared Exponent
    buffer.push(exponent as u8);

    // 2. Mantissas (Big Endian)
    for &m in mantissas {
        buffer.extend_from_slice(&m.to_be_bytes());
    }
    buffer
}

/// Decompresses BFP-16 vector components
pub fn decompress_bfp(
    data: &[u8],
    _valid_len: usize, // Ignored for raw format, but kept for signature compatibility if needed
) -> crate::Result<(i8, Vec<i16>)> {
    if data.is_empty() {
        return Err(crate::QresError::InvalidData("Empty BFP data".into()));
    }

    // 1. Shared Exponent
    let exponent = data[0] as i8;

    // 2. Mantissas
    let content = &data[1..];
    if !content.len().is_multiple_of(2) {
        return Err(crate::QresError::InvalidData(
            "Invalid BFP payload length".into(),
        ));
    }

    let count = content.len() / 2;
    let mut mantissas = Vec::with_capacity(count);

    let mut i = 0;
    while i < content.len() {
        let bytes = [content[i], content[i + 1]];
        mantissas.push(i16::from_be_bytes(bytes));
        i += 2;
    }

    Ok((exponent, mantissas))
}
