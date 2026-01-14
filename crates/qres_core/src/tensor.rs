use alloc::vec;
use alloc::vec::Vec;
use fixed::types::I16F16;
use fixed::FixedI16;

/// Q8.8 Fixed Point Type (16-bit total: 8 integer, 8 fractional)
pub type I8F8 = FixedI16<fixed::types::extra::U8>;

/// Fixed-Point Tensor Structure for QRES
/// Supports both I16F16 (Calm Mode) and I8F8 (Storm Mode) precision levels
#[derive(Debug, Clone)]
pub struct FixedTensor {
    pub data: Vec<I16F16>,
}

impl FixedTensor {
    pub fn new(data: Vec<I16F16>) -> Self {
        Self { data }
    }

    /// Create FixedTensor from I16F16 bytes (4 bytes per value)
    pub fn from_i16f16_bytes(bytes: &[u8]) -> Self {
        let data: Vec<I16F16> = bytes
            .chunks(4)
            .filter_map(|chunk| {
                if chunk.len() == 4 {
                    let bits = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    Some(I16F16::from_bits(bits as i32))
                } else {
                    None
                }
            })
            .collect();
        Self::new(data)
    }

    /// Create FixedTensor from I8F8 bytes (2 bytes per value)
    pub fn from_i8f8_bytes(bytes: &[u8]) -> Self {
        let i8f8_data: Vec<I8F8> = bytes
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    let bits = u16::from_le_bytes([chunk[0], chunk[1]]);
                    Some(I8F8::from_bits(bits as i16))
                } else {
                    None
                }
            })
            .collect();
        Self::from_i8f8(&i8f8_data)
    }

    /// Downcast to I8F8 (Storm Mode): Halves precision and bandwidth
    /// Saturates values outside I8F8 range to prevent overflow
    pub fn quantize_to_i8f8(&self) -> Vec<I8F8> {
        self.data.iter().map(|&val| {
            // Convert to f32 for range checking, then quantize
            let f32_val = val.to_num::<f32>();
            // I8F8 range: -128.0 to 127.996 (approximately)
            let clamped = f32_val.clamp(-128.0, 127.996);
            I8F8::from_num(clamped)
        }).collect()
    }

    /// Upcast from I8F8 (Restore from Storm Mode)
    /// Fills lower precision bits with zeros (lossy but deterministic)
    pub fn from_i8f8(data: &[I8F8]) -> Self {
        let data_i16f16 = data.iter().map(|&val| {
            // Convert I8F8 to f32, then to I16F16
            let f32_val = val.to_num::<f32>();
            I16F16::from_num(f32_val)
        }).collect();
        Self::new(data_i16f16)
    }
}

/// Tensor Network MPS (Matrix Product State) Compressor
/// Breaks a high-dimensional tensor into a chain of low-rank tensors (cores).
///
/// Compression comes from truncating the "Bond Dimension" (chis) via SVD.
///
/// Current implementation:
/// - Input: flattened byte stream treated as a Vector/Tensor.
/// - Output: List of compressed cores.
pub struct MpsCompressor {
    pub bond_dim: usize,
    pub threshold: I16F16,
}

impl MpsCompressor {
    pub fn new(bond_dim: usize, threshold: f64) -> Self {
        MpsCompressor {
            bond_dim,
            threshold: I16F16::from_num(threshold),
        }
    }

    /// Compress a 2D matrix (rows x cols) into MPS cores using "Haar Wavelet Tensor Train"
    /// Uses Q16.16 Fixed Point arithmetic for determinism.
    pub fn compress_matrix(&self, data: &[f64], rows: usize, cols: usize) -> Vec<Vec<f64>> {
        // Validation
        if data.len() != rows * cols {
            return Vec::new(); // Error
        }

        // 1. Convert to Fixed Point Matrix
        let mut matrix: Vec<I16F16> = Vec::with_capacity(rows * cols);
        for &val in data {
            matrix.push(I16F16::from_num(val));
        }

        // Implementation: 2D Haar Wavelet Transform (Lossy)
        // 1. Row transform
        // 2. Column transform
        // 3. Thresholding (Tensor sparsity)

        // Row steps
        for r in 0..rows {
            self.haar_1d(&mut matrix, r * cols, cols);
        }

        // Col steps
        // Transpose
        let mut transposed = vec![I16F16::ZERO; rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                transposed[c * rows + r] = matrix[r * cols + c];
            }
        }

        // Transform Columns
        for c in 0..cols {
            self.haar_1d(&mut transposed, c * rows, rows);
        }

        // Thresholding (Sparse approximation of Wavelet Coefficients)
        let mut flattened_sparse = Vec::new();
        for val in transposed {
            if val.abs() > self.threshold {
                flattened_sparse.push(val.to_num::<f64>());
            } else {
                flattened_sparse.push(0.0);
            }
        }

        vec![flattened_sparse]
    }

    fn haar_1d(&self, data: &mut [I16F16], start: usize, len: usize) {
        let mut temp = vec![I16F16::ZERO; len];
        let mut h = len;

        // Pre-calculate constants in Fixed Point
        let frac_sqrt_2 = I16F16::from_num(core::f64::consts::FRAC_1_SQRT_2);

        while h > 1 {
            let half = h / 2;
            for i in 0..half {
                // Safety: Use checked arithmetic to prevent panics on overflow
                let a = data.get(start + 2 * i).copied().unwrap_or(I16F16::ZERO);
                let b = data.get(start + 2 * i + 1).copied().unwrap_or(I16F16::ZERO);

                let sum = a.checked_add(b).unwrap_or(I16F16::MAX);
                let diff = a.checked_sub(b).unwrap_or(I16F16::MAX);

                // temp[i] = sum * frac_sqrt_2
                temp[i] = sum.checked_mul(frac_sqrt_2).unwrap_or(I16F16::MAX);

                // temp[half + i] = diff * frac_sqrt_2
                if let Some(idx) = temp.get_mut(half + i) {
                    *idx = diff.checked_mul(frac_sqrt_2).unwrap_or(I16F16::MAX);
                }
            }
            // Copy back
            if let Some(dest_slice) = data.get_mut(start..start + h) {
                if let Some(src_slice) = temp.get(..h) {
                    dest_slice.copy_from_slice(src_slice);
                }
            }
            h = half;
        }
    }
}
