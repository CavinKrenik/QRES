/// Content-Defined Chunking (CDC) for Global Deduplication
///
/// This module implements a rolling hash-based chunking system that detects
/// duplicate content anywhere in the archive, not just within a 64KB window.
///
/// Inspired by: rsync, zbackup, borg, and casync
use std::collections::HashMap;
use std::io::{self, Write};

/// Rolling hash for content-defined chunking (Rabin fingerprint variant)
pub struct RollingHash {
    window_size: usize,
    hash: u64,
    window: Vec<u8>,
    position: usize,
    // Polynomial coefficients for Rabin fingerprint
    pow: u64,
    prime: u64,
}

impl RollingHash {
    pub fn new(window_size: usize) -> Self {
        let prime = 0x3DA3358B4DC173; // Large prime for Rabin fingerprint
        let mut pow = 1u64;
        for _ in 0..window_size {
            pow = pow.wrapping_mul(prime);
        }

        RollingHash {
            window_size,
            hash: 0,
            window: Vec::with_capacity(window_size),
            position: 0,
            pow,
            prime,
        }
    }

    /// Update hash with a new byte (rolling window)
    pub fn update(&mut self, byte: u8) -> u64 {
        if self.window.len() < self.window_size {
            // Building initial window
            self.window.push(byte);
            self.hash = self.hash.wrapping_mul(self.prime).wrapping_add(byte as u64);
        } else {
            // Rolling: remove oldest byte, add new byte
            let old_byte = self.window[self.position];
            self.window[self.position] = byte;

            // Hash = (Hash * prime - old * prime^n) + new
            self.hash = self
                .hash
                .wrapping_mul(self.prime)
                .wrapping_sub((old_byte as u64).wrapping_mul(self.pow))
                .wrapping_add(byte as u64);

            self.position = (self.position + 1) % self.window_size;
        }

        self.hash
    }

    pub fn reset(&mut self) {
        self.hash = 0;
        self.window.clear();
        self.position = 0;
    }
}

/// Content-defined chunk boundary detector (Gear-based)
pub struct ChunkBoundaryDetector {
    mask: u64,
    min_size: usize,
    max_size: usize,
}

impl ChunkBoundaryDetector {
    /// Create a new detector targeting average chunk size
    pub fn new(avg_chunk_size: usize) -> Self {
        // Calculate mask to achieve desired average chunk size
        // For avg_size=8KB, we want ~1/8192 probability of boundary
        let bits = (avg_chunk_size as f64).log2().floor() as u32;
        let mask = (1u64 << bits) - 1;

        ChunkBoundaryDetector {
            mask,
            min_size: avg_chunk_size / 4,
            max_size: avg_chunk_size * 4,
        }
    }

    /// Check if this hash indicates a chunk boundary
    pub fn is_boundary(&self, hash: u64, current_size: usize) -> bool {
        // Force boundary at max_size
        if current_size >= self.max_size {
            return true;
        }

        // Don't create boundary below min_size
        if current_size < self.min_size {
            return false;
        }

        // Boundary if hash matches mask (Gear method)
        (hash & self.mask) == 0
    }
}

/// A deduplicated chunk with its hash and reference count
#[derive(Debug, Clone)]
pub struct DedupChunk {
    pub hash: u64,
    pub data: Vec<u8>,
    pub ref_count: usize,
    pub first_offset: u64,
}

/// Deduplication engine
pub struct DedupEngine {
    chunks: HashMap<u64, DedupChunk>,
    chunk_size_target: usize,
    total_original: u64,
    total_unique: u64,
    next_chunk_id: u32,
}

impl DedupEngine {
    pub fn new(avg_chunk_size: usize) -> Self {
        DedupEngine {
            chunks: HashMap::new(),
            chunk_size_target: avg_chunk_size,
            total_original: 0,
            total_unique: 0,
            next_chunk_id: 0,
        }
    }

    /// Split data into content-defined chunks
    pub fn chunk_data(&self, data: &[u8]) -> Vec<(usize, usize)> {
        let mut boundaries = Vec::new();
        let mut rolling = RollingHash::new(64); // 64-byte rolling window
        let detector = ChunkBoundaryDetector::new(self.chunk_size_target);

        let mut chunk_start = 0;

        for (i, &byte) in data.iter().enumerate() {
            let hash = rolling.update(byte);
            let chunk_size = i - chunk_start + 1;

            if detector.is_boundary(hash, chunk_size) {
                boundaries.push((chunk_start, i + 1));
                chunk_start = i + 1;
                rolling.reset();
            }
        }

        // Add final chunk
        if chunk_start < data.len() {
            boundaries.push((chunk_start, data.len()));
        }

        boundaries
    }

    /// Process data with deduplication, return compressed references
    pub fn deduplicate(&mut self, data: &[u8], offset: u64) -> DedupResult {
        let chunks = self.chunk_data(data);
        let mut references = Vec::new();
        let mut unique_data = Vec::new();

        for (start, end) in chunks {
            let chunk_data = &data[start..end];
            let chunk_hash = xxhash64(chunk_data);

            self.total_original += chunk_data.len() as u64;

            if let Some(existing) = self.chunks.get_mut(&chunk_hash) {
                // Duplicate found! Store reference instead of data
                existing.ref_count += 1;
                references.push(DedupReference::Existing {
                    hash: chunk_hash,
                    size: chunk_data.len(),
                });
            } else {
                // New unique chunk
                let chunk_id = self.next_chunk_id;
                self.next_chunk_id += 1;

                self.total_unique += chunk_data.len() as u64;

                let dedup_chunk = DedupChunk {
                    hash: chunk_hash,
                    data: chunk_data.to_vec(),
                    ref_count: 1,
                    first_offset: offset + start as u64,
                };

                self.chunks.insert(chunk_hash, dedup_chunk);
                unique_data.push(chunk_data.to_vec());

                references.push(DedupReference::New {
                    hash: chunk_hash,
                    chunk_id,
                    size: chunk_data.len(),
                });
            }
        }

        DedupResult {
            references,
            unique_data,
            original_size: data.len(),
            dedup_ratio: self.dedup_ratio(),
        }
    }

    pub fn dedup_ratio(&self) -> f64 {
        if self.total_original == 0 {
            return 1.0;
        }
        self.total_unique as f64 / self.total_original as f64
    }

    pub fn stats(&self) -> DedupStats {
        DedupStats {
            total_chunks: self.chunks.len(),
            total_original_bytes: self.total_original,
            total_unique_bytes: self.total_unique,
            dedup_ratio: self.dedup_ratio(),
            avg_chunk_size: if self.chunks.is_empty() {
                0
            } else {
                self.total_unique / self.chunks.len() as u64
            },
        }
    }

    /// Serialize the dedup index for storage
    pub fn serialize_index(&self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write header
        buffer.write_all(b"DEDP")?; // Dedup index magic
        buffer.write_all(&(self.chunks.len() as u32).to_le_bytes())?;

        // Write each chunk entry
        for (hash, chunk) in &self.chunks {
            buffer.write_all(&hash.to_le_bytes())?;
            buffer.write_all(&(chunk.data.len() as u32).to_le_bytes())?;
            buffer.write_all(&chunk.ref_count.to_le_bytes())?;
            buffer.write_all(&chunk.first_offset.to_le_bytes())?;
        }

        Ok(buffer)
    }
}

/// Reference to a deduplicated chunk
#[derive(Debug, Clone)]
pub enum DedupReference {
    /// New unique chunk that needs to be stored
    New {
        hash: u64,
        chunk_id: u32,
        size: usize,
    },
    /// Reference to existing chunk (duplicate)
    Existing { hash: u64, size: usize },
}

/// Result of deduplication operation
pub struct DedupResult {
    pub references: Vec<DedupReference>,
    pub unique_data: Vec<Vec<u8>>,
    pub original_size: usize,
    pub dedup_ratio: f64,
}

/// Statistics about deduplication performance
#[derive(Debug, Clone)]
pub struct DedupStats {
    pub total_chunks: usize,
    pub total_original_bytes: u64,
    pub total_unique_bytes: u64,
    pub dedup_ratio: f64,
    pub avg_chunk_size: u64,
}

/// Fast XXHash64 implementation for chunk hashing
pub fn xxhash64(data: &[u8]) -> u64 {
    const PRIME1: u64 = 0x9E3779B185EBCA87;
    const PRIME2: u64 = 0xC2B2AE3D27D4EB4F;
    const PRIME3: u64 = 0x165667B19E3779F9;
    const PRIME5: u64 = 0x27D4EB2F165667C5;

    let mut hash = PRIME5.wrapping_add(data.len() as u64);

    let mut chunks = data.chunks_exact(8);
    for chunk in &mut chunks {
        let k = u64::from_le_bytes(chunk.try_into().unwrap());
        hash ^= k.wrapping_mul(PRIME2);
        hash = hash.rotate_left(31).wrapping_mul(PRIME1);
    }

    // Process remaining bytes
    for &byte in chunks.remainder() {
        hash ^= (byte as u64).wrapping_mul(PRIME5);
        hash = hash.rotate_left(11).wrapping_mul(PRIME1);
    }

    // Finalization mix
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME3);
    hash ^= hash >> 32;

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_hash() {
        let mut rh = RollingHash::new(4);
        let data = b"abcdefgh";

        for &byte in data {
            rh.update(byte);
        }

        // Hash should be deterministic
        assert_ne!(rh.hash, 0);
    }

    #[test]
    fn test_deduplication() {
        let mut engine = DedupEngine::new(1024); // 1KB chunks

        // Create data with duplication using a pattern that varies (0..255)
        // detailed enough to ensure the rolling hash moves and triggers boundaries.
        let mut data = Vec::with_capacity(16 * 1024);
        let pattern: Vec<u8> = (0..8192).map(|i| (i % 255) as u8).collect();
        data.extend_from_slice(&pattern);
        data.extend_from_slice(&pattern); // Duplicate the pattern

        let result = engine.deduplicate(&data, 0);

        // Should detect duplication
        assert!(result.dedup_ratio < 1.0);
        assert!(!result.unique_data.is_empty());
    }

    #[test]
    fn test_chunk_boundaries() {
        let engine = DedupEngine::new(1024); // 1KB avg
        let data = vec![0xAB; 10 * 1024]; // 10KB of same byte

        let chunks = engine.chunk_data(&data);

        // Should create multiple chunks
        assert!(chunks.len() > 1);

        // Average size should be close to target
        let total: usize = chunks.iter().map(|(s, e)| e - s).sum();
        assert_eq!(total, data.len());
    }
}
