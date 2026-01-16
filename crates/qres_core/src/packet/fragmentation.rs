//! MTU-aware packet fragmentation with CRC32 integrity.
//! Splits large payloads into SAFE_MTU-sized fragments for constrained links.

use alloc::vec::Vec;

/// Safe MTU for ESP32 Wi-Fi / LoRa after headers.
pub const SAFE_MTU: usize = 1024;

/// Header layout (bytes):
/// - sequence_id: u32 (4)
/// - fragment_index: u16 (2)
/// - total_fragments: u16 (2)
/// - checksum: u32 (4) — CRC32 of payload
pub const HEADER_SIZE: usize = 12;
pub const MAX_PAYLOAD: usize = SAFE_MTU - HEADER_SIZE;

#[derive(Debug, Clone)]
pub struct Fragment {
    pub sequence_id: u32,
    pub fragment_index: u16,
    pub total_fragments: u16,
    pub checksum: u32,
    pub payload: Vec<u8>,
}

pub struct Fragmenter;

impl Fragmenter {
    /// Split a payload into MTU-safe fragments with sequence metadata and CRC32.
    pub fn split(data: &[u8], sequence_id: u32) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        let chunks = data.chunks(MAX_PAYLOAD);
        let total = chunks.len() as u16;

        for (i, chunk) in chunks.enumerate() {
            let checksum = Self::crc32(chunk);

            fragments.push(Fragment {
                sequence_id,
                fragment_index: i as u16,
                total_fragments: total,
                checksum,
                payload: chunk.to_vec(),
            });
        }

        fragments
    }

    /// Reassemble fragments; returns None if missing or corrupted.
    pub fn reassemble(mut fragments: Vec<Fragment>) -> Option<Vec<u8>> {
        if fragments.is_empty() {
            return None;
        }

        fragments.sort_by_key(|f| f.fragment_index);

        let expected_total = fragments[0].total_fragments as usize;
        if fragments.len() != expected_total {
            return None;
        }

        let mut buffer = Vec::new();
        for frag in fragments {
            if Self::crc32(&frag.payload) != frag.checksum {
                return None;
            }
            buffer.extend_from_slice(&frag.payload);
        }

        Some(buffer)
    }

    /// Minimal CRC32 (IEEE) for no_std contexts.
    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg() & 0xEDB8_8320;
                crc = (crc >> 1) ^ mask;
            }
        }
        !crc
    }
}
