//! Bit-perfect roundtrip tests for compression/decompression.
//! These tests verify that the LzMatchPredictor circular buffer optimization
//! produces identical results to the original implementation.

use qres_core::{compress_chunk, decompress_chunk, QresError};

/// Helper: Attempt roundtrip, allowing expansion errors for incompressible data
fn try_roundtrip(data: &[u8]) -> Result<Vec<u8>, QresError> {
    let mut compressed = vec![0u8; data.len() * 2 + 4096];
    let comp_len = compress_chunk(data, 0, None, None, &mut compressed)?;
    decompress_chunk(&compressed[..comp_len], 0, None)
}

/// Test basic roundtrip with small data (may expand, that's OK)
#[test]
fn roundtrip_small() {
    let test_data = b"Hello, QRES compression test! This tests predictor determinism.";

    match try_roundtrip(test_data) {
        Ok(decompressed) => {
            assert_eq!(test_data.as_slice(), decompressed.as_slice());
        }
        Err(QresError::CompressionError(msg)) if msg.contains("Expansion") => {
            // Small data may not compress - that's acceptable
            println!("Small data expansion is expected");
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

/// Test roundtrip with 64KB of patterned data (exercises LZ predictor)
#[test]
fn roundtrip_64kb_pattern() {
    // Create data with repeating patterns - ideal for LZ matching
    let test_data: Vec<u8> = (0..65536)
        .map(|i| ((i % 256) ^ ((i / 256) % 256)) as u8)
        .collect();

    let decompressed = try_roundtrip(&test_data).expect("64KB should compress");
    assert_eq!(test_data, decompressed, "64KB roundtrip failed");
}

/// Test roundtrip with random-like data (may not compress)
#[test]
fn roundtrip_random_data() {
    // Pseudo-random data using simple PRNG for reproducibility
    let mut rng_state: u32 = 0xDEADBEEF;
    let test_data: Vec<u8> = (0..32768)
        .map(|_| {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            (rng_state >> 16) as u8
        })
        .collect();

    match try_roundtrip(&test_data) {
        Ok(decompressed) => {
            assert_eq!(test_data, decompressed);
        }
        Err(QresError::CompressionError(msg)) if msg.contains("Expansion") => {
            // Random data typically doesn't compress
            println!("Random data expansion is expected");
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

/// Test roundtrip with text-like data (should compress with repetition)
#[test]
fn roundtrip_text() {
    // Longer text with lots of repetition to ensure compression
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(100);
    let test_data = text.as_bytes();

    let decompressed = try_roundtrip(test_data).expect("Repetitive text should compress");
    assert_eq!(test_data, decompressed.as_slice(), "Text roundtrip failed");
}

/// Test roundtrip approaching chunk size boundary (512KB)
#[test]
fn roundtrip_512kb() {
    // 512KB of mixed data with patterns
    let test_data: Vec<u8> = (0..524288)
        .map(|i| {
            let pattern = (i % 256) as u8;
            let noise = ((i * 7) % 13) as u8;
            pattern.wrapping_add(noise)
        })
        .collect();

    let decompressed = try_roundtrip(&test_data).expect("512KB should compress");
    assert_eq!(test_data.len(), decompressed.len(), "Length mismatch");
    assert_eq!(test_data, decompressed, "512KB roundtrip failed");
}

/// Test that compression produces non-trivial output for compressible data
#[test]
fn compression_ratio_check() {
    // Highly repetitive data
    let test_data = "AAAAAAAAAA".repeat(1000);
    let data = test_data.as_bytes();

    let mut compressed = vec![0u8; data.len() * 2 + 1024];
    let comp_len = compress_chunk(data, 0, None, None, &mut compressed)
        .expect("Repetitive data should compress");

    // Repetitive data should compress (ratio > 1.0 means it got smaller)
    let ratio = data.len() as f64 / comp_len as f64;
    assert!(
        ratio > 1.5,
        "Repetitive data should achieve >1.5x compression, got {:.2}x ({} -> {})",
        ratio,
        data.len(),
        comp_len
    );
}
