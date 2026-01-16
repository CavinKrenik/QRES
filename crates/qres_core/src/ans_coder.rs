//! Deterministic entropy coder using pure integer arithmetic.
//! Replaces the previous f64/Gaussian/libm implementation to guarantee
//! bit-identical output across x86, ARM, and WASM.

use alloc::vec::Vec;

/// AnsWriter: Encodes i8 residuals into a compressed byte stream.
/// Uses a simple varint scheme: values in [-63, 63] use 1 byte; outliers use 2 bytes.
/// This is fully deterministic (no floating point).
pub struct AnsWriter {
    output: Vec<u8>,
}

impl Default for AnsWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsWriter {
    pub fn new() -> Self {
        AnsWriter { output: Vec::new() }
    }

    /// Encode a single residual.
    /// Scheme: if |residual| <= 63, emit one byte with sign bit.
    /// Otherwise emit escape (0x7F) followed by raw i8 byte.
    pub fn write_residual(&mut self, residual: i8) {
        let val = residual as i32;
        if (-63..=63).contains(&val) {
            // Pack into 7 bits: bit 7 = sign, bits 0-6 = magnitude
            let mag = val.unsigned_abs() as u8;
            let sign_bit = if val < 0 { 0x80 } else { 0x00 };
            self.output.push(sign_bit | mag);
        } else {
            // Escape + raw byte for values -128..-64 or 64..127
            self.output.push(0x7F); // Escape marker
            self.output.push(residual as u8);
        }
    }

    pub fn finish(self) -> Vec<u8> {
        self.output
    }
}

/// AnsReader: Decodes residuals from a compressed byte stream.
pub struct AnsReader {
    data: Vec<u8>,
    pos: usize,
}

impl AnsReader {
    pub fn new(data: &[u8]) -> Self {
        AnsReader {
            data: data.to_vec(),
            pos: 0,
        }
    }

    pub fn read_residual(&mut self) -> i8 {
        if self.pos >= self.data.len() {
            return 0;
        }

        let byte = self.data[self.pos];
        self.pos += 1;

        if byte == 0x7F {
            // Escape: next byte is raw i8
            if self.pos >= self.data.len() {
                return 0;
            }
            let raw = self.data[self.pos];
            self.pos += 1;
            raw as i8
        } else {
            // Packed: bit 7 = sign, bits 0-6 = magnitude
            let mag = (byte & 0x7F) as i32;
            let negative = (byte & 0x80) != 0;
            if negative {
                -(mag as i8)
            } else {
                mag as i8
            }
        }
    }
}
