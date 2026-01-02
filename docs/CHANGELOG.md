# Changelog

All notable changes to the QRES project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [5.1.0] - 2026-01-01
### Added
- **Telepathy Engine:** "Living Brain" predictor selection (Linear/Simple/Graph/Spectral) for adaptive entropy coding.
- **Solid Archive Support:** New `.qrar` format with deduplication and solid stream compression.
- **Reference Chunks (0x03):** Archive format now supports cross-file deduplication references.
- **QRES Studio:** Desktop GUI (Tauri+Svelte) for drag-and-drop compression and archive browsing.
- **Native Swarm:** Replaced Python Hive server with Rust `libp2p` implementation (GossipSub).
- **SIMD Optimization:** AVX2/NEON vectorization for the Mixer and Neural layers.

### Changed
- **CLI Renaming:** `qres create` -> `qres archive`.
- **Docs:** Consolidated all documentation into `docs/` directory.
- **Python API:** `encode_bytes` now handles headers internally; removed `bincode` dependency for core stream.

### Fixed
- **Performance:** Fixed regression in text compression by re-enabling Graph predictor.
- **Stability:** Resolved "Archive Doctsests" failures in `archive.rs`.

## [5.0.0] - 2025-12-30
- Initial "Singularity" architecture release.
- Introduction of `MetaBrain` and Neural Predictors.

## [4.0.0] - 2025-11-15
- Stable release of QRES v4 (Context Mixing).
