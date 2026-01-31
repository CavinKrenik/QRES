#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum QresError {
    InvalidInput(String),
    InvalidData(String),
    CompressionError(String),
    Other(String),
}

impl core::fmt::Display for QresError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            QresError::InvalidInput(s) => write!(f, "InvalidInput: {}", s),
            QresError::InvalidData(s) => write!(f, "InvalidData: {}", s),
            QresError::CompressionError(s) => write!(f, "CompressionError: {}", s),
            QresError::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for QresError {
    fn from(err: std::io::Error) -> Self {
        QresError::Other(err.to_string())
    }
}

#[cfg(feature = "std")]
impl From<QresError> for std::io::Error {
    fn from(err: QresError) -> Self {
        std::io::Error::other(err.to_string())
    }
}

pub type Result<T> = core::result::Result<T, QresError>;

pub mod adaptive;
pub mod aggregation;
pub mod ans_coder;
#[cfg(feature = "std")]
pub mod archive;
#[cfg(feature = "std")]
pub mod compression;
pub mod config;
pub mod consensus;
pub mod cortex;
#[cfg(feature = "std")]
pub mod dedup;
#[cfg(feature = "std")]
pub mod encoding;
#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(feature = "std")]
pub mod inference;
pub mod meta_brain;
pub mod mixer;
#[cfg(feature = "std")]
pub mod multivariate;
pub mod packet;
pub mod predictors;
pub mod privacy;
#[cfg(feature = "python")]
pub mod python_api;
#[cfg(feature = "std")]
pub mod resource_management;
pub mod secure_agg;
pub mod spectral;
pub mod tensor;
pub mod transformer;
pub mod zk_proofs;

use crate::ans_coder::{AnsReader, AnsWriter};
use crate::mixer::{Mixer, NUM_MODELS};
use crate::predictors::{GraphPredictor, LzMatchPredictor, Predictor, SimplePredictor};
use crate::spectral::SpectralPredictor;
use transformer::TransformerPredictor;

// ============================================================================
// PredictorSet: Reusable predictor bundle to avoid ~22MB reallocation per chunk
// ============================================================================

/// A bundle of all predictors used in v4 encoding/decoding.
///
/// This struct allows reusing predictor memory across multiple chunks,
/// eliminating ~22MB of heap allocation overhead per chunk (SimplePredictor
/// uses 16MB, LzMatchPredictor uses ~5MB).
///
/// # Usage
/// ```ignore
/// let mut state = PredictorSet::new(None, None);
/// for chunk in chunks {
///     state.reset(None, None);
///     let decoded = decompress_chunk_with_state(&chunk, 0, None, &mut state)?;
/// }
/// ```
pub struct PredictorSet {
    pub linear: u8,
    pub simple: SimplePredictor,
    pub graph: GraphPredictor,
    pub spectral: SpectralPredictor,
    pub lz_match: LzMatchPredictor,
    pub transformer: TransformerPredictor,
    pub mixer: Mixer,
}

impl PredictorSet {
    /// Create a new PredictorSet, allocating all predictor memory.
    ///
    /// This allocates approximately 22MB:
    /// - SimplePredictor: 16MB (order-3 context table)
    /// - LzMatchPredictor: ~5MB (1MB history + 4MB hash table)
    /// - Other predictors: negligible
    pub fn new(init_weights: Option<&[i32]>, global_weights: Option<&[i32]>) -> Self {
        PredictorSet {
            linear: 0,
            simple: SimplePredictor::new(),
            graph: GraphPredictor::new(),
            spectral: SpectralPredictor::new(2048),
            lz_match: LzMatchPredictor::new(),
            transformer: TransformerPredictor::new(),
            mixer: Mixer::new(init_weights, global_weights),
        }
    }

    /// Reset all predictors to their initial state without reallocating memory.
    ///
    /// This uses `.fill(0)` on internal buffers instead of dropping and
    /// reallocating, saving ~22MB of allocation overhead per call.
    pub fn reset(&mut self, init_weights: Option<&[i32]>, global_weights: Option<&[i32]>) {
        self.linear = 0;
        self.simple.reset();
        self.graph.reset();
        self.spectral.reset();
        self.lz_match.reset();
        self.transformer.reset();
        // Mixer is small (~100 bytes), recreate it with the new weights
        self.mixer = Mixer::new(init_weights, global_weights);
    }
}

impl Default for PredictorSet {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[allow(dead_code)]
const CHUNK_SIZE: usize = 1024 * 1024;
#[allow(dead_code)]
const QRES_MAGIC: &[u8] = b"QRES";
const QRES_PROTOCOL_VERSION: u8 = 10;

#[allow(dead_code)]
const PREDICTOR_ID_DEFAULT: u8 = 0;
#[allow(dead_code)]
const PREDICTOR_ID_NEURAL: u8 = 1;
const PREDICTOR_ID_SPLIT: u8 = 2;

const NUM_PREDICTORS: usize = 6;
const WEIGHTS_LEN: usize = NUM_PREDICTORS * 4;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QresHeader {
    pub version: u8,
    pub flags: u8,
    pub predictor_id: u8,
    pub timestamp: i64,
    pub original_size: u64,
    pub compressed_size: u64,
    pub file_name: String,
    pub chunk_compressed_sizes: Vec<u32>,
}

#[cfg(feature = "std")]
#[allow(dead_code)]
fn calculate_sample_entropy(data: &[u8]) -> f32 {
    let mut counts = [0usize; 256];
    let step = if data.len() > 4096 { 4 } else { 1 };
    let mut total = 0;

    for i in (0..data.len()).step_by(step) {
        counts[data[i] as usize] += 1;
        total += 1;
    }

    let total_f = total as f32;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let p = count as f32 / total_f;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Internal encoding function using a pre-allocated PredictorSet.
/// The caller MUST call state.reset() before calling this function.
fn predictive_encode_v4_with_state(
    data: &[u8],
    config: Option<&crate::config::QresConfig>,
    state: &mut PredictorSet,
    output: &mut [u8],
) -> Result<usize> {
    #[cfg(feature = "std")]
    println!("DEBUG: Running Optimized Encoder");

    const UPDATE_BATCH_SIZE: usize = 32;

    let mut ans = AnsWriter::new();

    let q_factor = if let Some(cfg) = config {
        match cfg.mode {
            crate::config::CompressionMode::Lossy => 5,
            _ => 1,
        }
    } else {
        1
    };

    let mut preds = [0u8; 6];
    let mut batch_counter = 0usize;

    for &actual in data {
        preds[0] = state.linear;
        preds[1] = state.simple.predict_next();
        preds[2] = state.graph.predict_next();
        preds[3] = state.spectral.predict();
        preds[4] = state.lz_match.predict_next();
        preds[5] = state.transformer.predict_next();

        let mixed_prediction = state.mixer.mix(&preds);

        let base_residual = actual.wrapping_sub(mixed_prediction) as i8;

        let residual = if q_factor > 1 {
            (base_residual / q_factor) * q_factor
        } else {
            base_residual
        };

        ans.write_residual(residual);

        let reconstructed = mixed_prediction.wrapping_add(residual as u8);

        batch_counter += 1;
        if batch_counter >= UPDATE_BATCH_SIZE {
            state
                .mixer
                .update_lazy(UPDATE_BATCH_SIZE, reconstructed, &preds);
            batch_counter = 0;
        }

        state.linear = reconstructed;
        state.simple.update(reconstructed);
        state.graph.update(reconstructed);
        state.spectral.update(reconstructed);
        state.lz_match.update(reconstructed);
        state.transformer.update(reconstructed);
    }

    if batch_counter > 0 {
        state.mixer.update_lazy(batch_counter, state.linear, &preds);
    }

    let compressed_data = ans.finish();

    if compressed_data.len() > output.len() {
        return Err(QresError::CompressionError(String::from(
            "Expansion detected",
        )));
    }

    output[..compressed_data.len()].copy_from_slice(&compressed_data);
    Ok(compressed_data.len())
}

fn predictive_encode_v4(
    data: &[u8],
    config: Option<&crate::config::QresConfig>,
    weights: Option<&[u8]>,
    output: &mut [u8],
) -> Result<usize> {
    // Parse weights
    let mut safe_weights_vec = Vec::new();
    if let Some(w_bytes) = weights {
        for chunk in w_bytes.chunks_exact(4) {
            safe_weights_vec.push(i32::from_le_bytes(chunk.try_into().unwrap()));
        }
    }

    let (init_w, global_w) = if !safe_weights_vec.is_empty() {
        let wc = safe_weights_vec.len();
        if wc >= 2 * NUM_MODELS {
            (
                Some(&safe_weights_vec[0..NUM_MODELS]),
                Some(&safe_weights_vec[NUM_MODELS..2 * NUM_MODELS]),
            )
        } else if wc >= NUM_MODELS {
            (Some(&safe_weights_vec[0..NUM_MODELS]), None)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Create temporary PredictorSet (backward compatibility wrapper)
    let mut state = PredictorSet::new(init_w, global_w);
    predictive_encode_v4_with_state(data, config, &mut state, output)
}

/// Internal decoding function using a pre-allocated PredictorSet.
/// The caller MUST call state.reset() before calling this function.
fn predictive_decode_v4_with_state(
    compressed_words: &[u8],
    decoded_len: usize,
    state: &mut PredictorSet,
) -> Vec<u8> {
    const UPDATE_BATCH_SIZE: usize = 32;

    let mut ans = AnsReader::new(compressed_words);

    let mut out = Vec::with_capacity(decoded_len);
    let mut preds = [0u8; 6];
    let mut batch_counter = 0usize;

    for _ in 0..decoded_len {
        preds[0] = state.linear;
        preds[1] = state.simple.predict_next();
        preds[2] = state.graph.predict_next();
        preds[3] = state.spectral.predict();
        preds[4] = state.lz_match.predict_next();
        preds[5] = state.transformer.predict_next();

        let mixed_prediction = state.mixer.mix(&preds);

        let residual = ans.read_residual();

        let actual = mixed_prediction.wrapping_add(residual as u8);
        out.push(actual);

        batch_counter += 1;
        if batch_counter >= UPDATE_BATCH_SIZE {
            state.mixer.update_lazy(UPDATE_BATCH_SIZE, actual, &preds);
            batch_counter = 0;
        }

        state.linear = actual;
        state.simple.update(actual);
        state.graph.update(actual);
        state.spectral.update(actual);
        state.lz_match.update(actual);
        state.transformer.update(actual);
    }

    if batch_counter > 0 {
        state.mixer.update_lazy(batch_counter, state.linear, &preds);
    }

    out
}

fn predictive_decode_v4(
    compressed_words: &[u8],
    decoded_len: usize,
    weights: Option<&[u8]>,
) -> Vec<u8> {
    // Parse weights
    let mut safe_weights_vec = Vec::new();
    if let Some(w_bytes) = weights {
        for chunk in w_bytes.chunks_exact(4) {
            safe_weights_vec.push(i32::from_le_bytes(chunk.try_into().unwrap()));
        }
    }

    let (init_w, global_w) = if !safe_weights_vec.is_empty() {
        let wc = safe_weights_vec.len();
        if wc >= 2 * NUM_MODELS {
            (
                Some(&safe_weights_vec[0..NUM_MODELS]),
                Some(&safe_weights_vec[NUM_MODELS..2 * NUM_MODELS]),
            )
        } else if wc >= NUM_MODELS {
            (Some(&safe_weights_vec[0..NUM_MODELS]), None)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Create temporary PredictorSet (backward compatibility wrapper)
    let mut state = PredictorSet::new(init_w, global_w);
    predictive_decode_v4_with_state(compressed_words, decoded_len, &mut state)
}

pub fn compress_chunk(
    chunk: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
    config: Option<&crate::config::QresConfig>,
    output: &mut [u8],
) -> Result<usize> {
    if _predictor_id > PREDICTOR_ID_SPLIT {
        return Err(QresError::InvalidInput(format!(
            "Unsupported Predictor ID: {}",
            _predictor_id
        )));
    }

    let mut effective_weights = Vec::new();
    let mut is_neural = false;
    let mut stored_init_weights = Vec::new();

    if let Some(w) = _weights {
        let take = w.len().min(WEIGHTS_LEN);
        effective_weights.extend_from_slice(&w[0..take]);
        if take > 0 {
            is_neural = true;
            stored_init_weights.extend_from_slice(&w[0..take]);
            while stored_init_weights.len() < WEIGHTS_LEN {
                stored_init_weights.push(0);
            }
        }
    }

    if let Some(w) = _weights {
        if w.len() >= WEIGHTS_LEN * 2 {
            effective_weights.extend_from_slice(&w[WEIGHTS_LEN..WEIGHTS_LEN * 2]);
        }
    }

    let w_arg = if effective_weights.is_empty() {
        None
    } else {
        Some(effective_weights.as_slice())
    };

    let mode = if is_neural { 0x02 } else { 0x00 };
    let ver = QRES_PROTOCOL_VERSION & 0x0F;
    let flag_byte = (ver << 4) | mode;

    let header_size = 1
        + 4
        + if is_neural {
            stored_init_weights.len()
        } else {
            0
        };

    if output.len() < header_size {
        return Err(QresError::Other(String::from(
            "Buffer too small for header",
        )));
    }

    let mut cursor = 0;
    output[cursor] = flag_byte;
    cursor += 1;

    let chunk_len_u32 = chunk.len() as u32;
    output[cursor..cursor + 4].copy_from_slice(&chunk_len_u32.to_le_bytes());
    cursor += 4;

    if is_neural {
        output[cursor..cursor + stored_init_weights.len()].copy_from_slice(&stored_init_weights);
        cursor += stored_init_weights.len();
    }

    let compressed_len = predictive_encode_v4(chunk, config, w_arg, &mut output[cursor..])?;
    cursor += compressed_len;

    if cursor < chunk.len() {
        Ok(cursor)
    } else {
        Err(QresError::CompressionError(String::from(
            "Expansion detected",
        )))
    }
}

pub fn decompress_chunk(
    compressed: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
) -> Result<Vec<u8>> {
    if compressed.len() < 5 {
        return Err(QresError::InvalidData(String::from("Chunk too short")));
    }

    let flag_byte = compressed[0];
    let version = (flag_byte >> 4) & 0x0F;
    let codec_mode = flag_byte & 0x0F;

    if version != (QRES_PROTOCOL_VERSION & 0x0F) {
        return Err(QresError::InvalidData(format!(
            "Version Mismatch: File v{} != Library v{}",
            version, QRES_PROTOCOL_VERSION
        )));
    }

    let decomp_len = u32::from_le_bytes(
        compressed[1..5]
            .try_into()
            .map_err(|_| QresError::InvalidData(String::from("Invalid Header")))?,
    ) as usize;

    match codec_mode {
        0x00 => Ok(predictive_decode_v4(&compressed[5..], decomp_len, _weights)),
        0x01 => {
            // Zstd fallback - return error to trigger daemon's Zstd handler
            Err(QresError::CompressionError(String::from(
                "Zstd fallback chunk - handle externally",
            )))
        }
        0x02 => {
            let header_size = 5 + WEIGHTS_LEN;
            if compressed.len() < header_size {
                return Err(QresError::InvalidData(String::from(
                    "Chunk too short for Neural Header",
                )));
            }
            let init_w_bytes = &compressed[5..header_size];

            let mut w_vec = Vec::with_capacity(WEIGHTS_LEN * 2);
            w_vec.extend_from_slice(init_w_bytes);

            if let Some(w) = _weights {
                if w.len() >= WEIGHTS_LEN * 2 {
                    w_vec.extend_from_slice(&w[WEIGHTS_LEN..WEIGHTS_LEN * 2]);
                }
            }

            let w_arg = if w_vec.is_empty() {
                None
            } else {
                Some(w_vec.as_slice())
            };
            Ok(predictive_decode_v4(
                &compressed[header_size..],
                decomp_len,
                w_arg,
            ))
        }
        0x03 => {
            // Split logic omitted for brevity (unchanged)
            Err(QresError::Other(String::from(
                "Split not reimplemented yet",
            )))
        }
        _ => Err(QresError::InvalidData(format!(
            "Unknown codec mode: {:#x}",
            codec_mode
        ))),
    }
}

/// Decompress a chunk using a pre-allocated PredictorSet.
///
/// This function eliminates ~22MB of allocation overhead per chunk by reusing
/// the predictor memory from a `PredictorSet`. The caller is responsible for
/// calling `state.reset()` before each chunk to ensure bit-perfect compatibility.
///
/// # Arguments
/// * `compressed` - The compressed chunk data including header
/// * `_predictor_id` - Predictor ID (currently unused, kept for API compatibility)
/// * `_weights` - Optional weight bytes for neural codec modes
/// * `state` - A pre-allocated PredictorSet that will be reset and reused
///
/// # Example
/// ```ignore
/// let mut state = PredictorSet::new(None, None);
/// for chunk in chunks {
///     // Reset is handled internally by this function
///     let decoded = decompress_chunk_with_state(&chunk, 0, None, &mut state)?;
/// }
/// ```
pub fn decompress_chunk_with_state(
    compressed: &[u8],
    _predictor_id: u8,
    _weights: Option<&[u8]>,
    state: &mut PredictorSet,
) -> Result<Vec<u8>> {
    if compressed.len() < 5 {
        return Err(QresError::InvalidData(String::from("Chunk too short")));
    }

    let flag_byte = compressed[0];
    let version = (flag_byte >> 4) & 0x0F;
    let codec_mode = flag_byte & 0x0F;

    if version != (QRES_PROTOCOL_VERSION & 0x0F) {
        return Err(QresError::InvalidData(format!(
            "Version Mismatch: File v{} != Library v{}",
            version, QRES_PROTOCOL_VERSION
        )));
    }

    let decomp_len = u32::from_le_bytes(
        compressed[1..5]
            .try_into()
            .map_err(|_| QresError::InvalidData(String::from("Invalid Header")))?,
    ) as usize;

    match codec_mode {
        0x00 => {
            // Standard predictive codec
            // Parse weights and reset state
            let mut safe_weights_vec = Vec::new();
            if let Some(w_bytes) = _weights {
                for chunk in w_bytes.chunks_exact(4) {
                    safe_weights_vec.push(i32::from_le_bytes(chunk.try_into().unwrap()));
                }
            }

            let (init_w, global_w) = if !safe_weights_vec.is_empty() {
                let wc = safe_weights_vec.len();
                if wc >= 2 * NUM_MODELS {
                    (
                        Some(&safe_weights_vec[0..NUM_MODELS]),
                        Some(&safe_weights_vec[NUM_MODELS..2 * NUM_MODELS]),
                    )
                } else if wc >= NUM_MODELS {
                    (Some(&safe_weights_vec[0..NUM_MODELS]), None)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            state.reset(init_w, global_w);
            Ok(predictive_decode_v4_with_state(
                &compressed[5..],
                decomp_len,
                state,
            ))
        }
        0x01 => {
            // Zstd fallback - return error to trigger daemon's Zstd handler
            Err(QresError::CompressionError(String::from(
                "Zstd fallback chunk - handle externally",
            )))
        }
        0x02 => {
            let header_size = 5 + WEIGHTS_LEN;
            if compressed.len() < header_size {
                return Err(QresError::InvalidData(String::from(
                    "Chunk too short for Neural Header",
                )));
            }
            let init_w_bytes = &compressed[5..header_size];

            let mut w_vec = Vec::with_capacity(WEIGHTS_LEN * 2);
            w_vec.extend_from_slice(init_w_bytes);

            if let Some(w) = _weights {
                if w.len() >= WEIGHTS_LEN * 2 {
                    w_vec.extend_from_slice(&w[WEIGHTS_LEN..WEIGHTS_LEN * 2]);
                }
            }

            // Parse weights for state reset
            let mut safe_weights_vec = Vec::new();
            for chunk in w_vec.chunks_exact(4) {
                safe_weights_vec.push(i32::from_le_bytes(chunk.try_into().unwrap()));
            }

            let (init_w, global_w) = if !safe_weights_vec.is_empty() {
                let wc = safe_weights_vec.len();
                if wc >= 2 * NUM_MODELS {
                    (
                        Some(&safe_weights_vec[0..NUM_MODELS]),
                        Some(&safe_weights_vec[NUM_MODELS..2 * NUM_MODELS]),
                    )
                } else if wc >= NUM_MODELS {
                    (Some(&safe_weights_vec[0..NUM_MODELS]), None)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            state.reset(init_w, global_w);
            Ok(predictive_decode_v4_with_state(
                &compressed[header_size..],
                decomp_len,
                state,
            ))
        }
        0x03 => {
            // Split logic omitted for brevity (unchanged)
            Err(QresError::Other(String::from(
                "Split not reimplemented yet",
            )))
        }
        _ => Err(QresError::InvalidData(format!(
            "Unknown codec mode: {:#x}",
            codec_mode
        ))),
    }
}

// RESTORED FUNCTIONS

#[cfg(feature = "std")]
pub fn compress_with_callback<F>(src_path: &str, dest_path: &str, callback: F) -> Result<()>
where
    F: Fn(f32, f32, &str),
{
    use std::io::{Read, Write};

    let mut input_file = std::fs::File::open(src_path)?;
    let mut output_file = std::fs::File::create(dest_path)?;

    let metadata = input_file.metadata()?;
    let total_size = metadata.len();
    let mut processed_size = 0u64;

    let mut buffer = vec![0u8; CHUNK_SIZE];

    loop {
        let n = input_file.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        // Output buffer safety margin
        let mut out_buf = vec![0u8; n * 2 + 1024];

        let compressed_len =
            compress_chunk(&buffer[..n], PREDICTOR_ID_DEFAULT, None, None, &mut out_buf)?;

        output_file.write_all(&out_buf[..compressed_len])?;

        processed_size += n as u64;
        let progress = if total_size > 0 {
            (processed_size as f32 / total_size as f32) * 100.0
        } else {
            100.0
        };
        let ratio = if processed_size > 0 {
            // Estimate ratio based on current file pos vs processed
            // This is rough. `output_file.metadata()` might be slow.
            // Let's use written bytes estimate if we tracked it.
            // For now 1.0 is fine or we can track compressed_size.
            1.0
        } else {
            1.0
        };

        callback(progress, ratio, "NeuralMixed");
    }

    Ok(())
}
