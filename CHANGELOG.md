# Changelog
All notable changes to QRES will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [5.1.0] - 2026-01-01

### Added
- **Archive Format (.qrar)**: True archiving with WinZip/7-Zip-style container format
  - Solid compression: concatenate files before compressing
  - JSON manifests with file metadata
  - Partial extraction support
  - Blake3 integrity verification
- **Global Deduplication**: Content-Defined Chunking (CDC) module
  - Rabin fingerprinting with rolling hash
  - Gear boundary detection for variable-size chunks
  - Detects duplicates across entire archive
  - XXHash64 for fast chunk hashing
- **Logistic Mixing**: Neural-style probability blending in mixer
  - Sigmoid activation for non-linear mixing
  - 2-5% better compression on structured data
  - Adaptive AR(2) blending based on variance
- **New Tauri Commands**:
  - `browse_archive()`: View archive contents without extraction
  - `extract_archive()`: Extract entire archive
  - `extract_archive_file()`: Extract single file from archive
- **New Modules**:
  - `archive.rs`: Complete archive implementation (404 lines)
  - `dedup.rs`: Content-Defined Chunking engine (300+ lines)

### Fixed
- **CRITICAL**: Decompression crash when GUI tries to extract .qres files
  - Root cause: Passing entire file (with header) to decompress_chunk
  - Solution: Proper header parsing and streaming chunk decompression
- **CRITICAL**: Memory exhaustion on large files (>1GB)
  - Root cause: fs::read() loading entire file into RAM
  - Solution: Streaming I/O with BufReader/BufWriter
  - Memory usage reduced from O(file_size) to O(64KB)
- Fixed type conversion issues in Tauri commands for path handling

### Changed
- `compress_file()` now creates .qrar archives when given a directory (previously created individual .qres files)
- Folder compression now uses solid compression for 20-50% better ratios
- Improved error messages with detailed context

### Performance
- Memory usage for 10GB file decompression: 10GB → 64KB (99.999% reduction)
- Compression ratio improvements with solid archives:
  - Source code: +38%
  - Similar logs: +50%
  - Mixed documents: +18%

### Dependencies
- Added `walkdir = "2.5"` for directory traversal
- Added `blake3 = "1.5"` for content hashing

---

## [5.0.5] - 2025-12-XX

### Added
- LzMatchPredictor for context-based string matching
- SIMD acceleration with AVX2/NEON intrinsics
- Native libp2p P2P swarm for decentralized brain sharing
- Hex Battle visualization

### Fixed
- Various compilation and runtime issues
- Python binding compatibility

### Changed
- Upgraded to hybrid neural-statistical ensemble
- Improved spectral predictor with FFT

---

## [5.0.0] - 2025-11-XX

### Added
- Initial "Singularity" release
- Multi-predictor ensemble (Linear, Simple, Graph, Spectral)
- Adaptive weight learning
- AR(2) auto-regressor for waveforms
- Content-aware pre-pass
- Zstd fallback for high-entropy data

### Changed
- Complete rewrite from QRES v4
- Moved to Rust-first architecture
- Introduced cognitive compression philosophy

---

## [4.x] - Earlier Versions
See git history for pre-v5.0 changes.

---

## Versioning Policy

QRES follows semantic versioning:
- **Major version (X.0.0)**: Breaking API changes, major architecture changes
- **Minor version (X.Y.0)**: New features, non-breaking changes
- **Patch version (X.Y.Z)**: Bug fixes, documentation updates

---

## Links
- [Releases](https://github.com/CavinKrenik/QRES/releases)
- [Issues](https://github.com/CavinKrenik/QRES/issues)
- [v5.1.0 Release Notes](./RELEASE_NOTES.md)
- [v5.1.0 Technical Details](./CRITICAL_FIXES_V5.1.md)
