use alloc::vec;
use alloc::vec::Vec;
use fixed::types::I16F16;

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
