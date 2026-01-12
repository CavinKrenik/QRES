#![cfg_attr(not(feature = "std"), no_std)]

use alloc::vec::Vec;

/// ZigZag encode a signed 16-bit integer to unsigned.
/// Maps: 0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3, 2 -> 4, ...
#[inline]
fn zigzag_encode(n: i16) -> u16 {
    ((n << 1) ^ (n >> 15)) as u16
}

/// Calculate minimum bits needed to represent a value.
#[inline]
fn bits_needed(max_val: u16) -> u8 {
    if max_val == 0 {
        1
    } else {
        16 - max_val.leading_zeros() as u8
    }
}

/// Block size for bit-packing
pub const BLOCK_SIZE: usize = 256;

/// Compress float data to bytes using the "Golden" Pipeline:
/// Delta Encoding -> ZigZag -> Bit-Packing
///
/// This pipeline is optimized for time-series data, removing trends (Delta)
/// and efficiently storing the residuals (Bit-Packing).
///
/// Returns a byte vector format:
/// [Bit-Packed Blocks]
/// Each block: [Bit-Width (1 byte)] [Packed Data]
pub fn compress_golden(data: &[f32]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    // Step 1: Delta encode and scale to i16
    let mut deltas: Vec<i16> = Vec::with_capacity(data.len());

    // First value as baseline
    let first_scaled = (data[0] * 100.0).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
    deltas.push(first_scaled);

    // Delta for remaining values
    for i in 1..data.len() {
        let delta = (data[i] - data[i - 1]) * 100.0;
        let delta_scaled = delta.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        deltas.push(delta_scaled);
    }

    // Step 2: ZigZag encode all values
    let zigzag_values: Vec<u16> = deltas.iter().map(|&d| zigzag_encode(d)).collect();

    // Step 3: Bit-pack per block
    let mut result = Vec::with_capacity(data.len()); // Optimistic size estimation

    for chunk in zigzag_values.chunks(BLOCK_SIZE) {
        // Find max value in block to determine bit width
        let max_val = *chunk.iter().max().unwrap_or(&0);
        let bit_width = bits_needed(max_val);

        // Write block header: bit_width (1 byte)
        result.push(bit_width);

        // Pack values using bit_width bits each
        let mut bit_buffer: u64 = 0;
        let mut bits_in_buffer: u8 = 0;

        for &val in chunk {
            bit_buffer |= (val as u64) << bits_in_buffer;
            bits_in_buffer += bit_width;

            // Flush complete bytes
            while bits_in_buffer >= 8 {
                result.push(bit_buffer as u8);
                bit_buffer >>= 8;
                bits_in_buffer -= 8;
            }
        }

        // Flush remaining bits
        if bits_in_buffer > 0 {
            result.push(bit_buffer as u8);
        }
    }

    result
}
