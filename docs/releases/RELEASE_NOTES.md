# QRES v5.1.0 "Singularity" Release Notes

**Date:** January 1, 2026
**Codename:** Singularity

## 🌟 Highlights

### 🧠 Neural-Symbolic "Telepathy"
The compression engine now features a "Living Brain" that autonomously selects between 4 distinct predictor models (Linear, Simple, Graph, Spectral) for every byte of data. This allows QRES to adapt its strategy in real-time, functioning like a telepathic link that anticipates the next symbol before it arrives.

### 📦 Deduplication Engine (CDC)
We have introduced a Content-Defined Chunking (CDC) layer using a Gear-based rolling hash.
- **Cross-File Deduplication:** Identical chunks across different files are stored only once.
- **Reference Chunks:** The archive format now supports `0x03` Reference Flags, pointing to existing hashes in the dictionary.
- **Solid Compression:** Archive streams are now solid by default, maximizing redundancy elimination.

### 🐝 Native Swarm P2P
The Python-based Hive Server has been replaced with a high-performance **Rust** implementation using `libp2p`.
- **GossipSub:** Efficient model weight synchronization across the swarm.
- **Distributed Learning:** Nodes share "epiphanies" (highly successful model weights) without sharing the actual private data.

### 🖥️ QRES Studio (GUI)
A new Taur + Svelte frontend provides a beautiful interface for the engine.
- **Archive Browser:** Open `.qrar` files to view contents, sizes, and compression ratios without extracting.
- **Drag & Drop:** Intelligent handling of PDF, Images, and legacy `.qres` files.
- **Visualizations:** Real-time feedback on the active neural engine.

## ⚠️ Breaking Changes
- **Archive Extension:** The default extension for solid archives is now `.qrar`. `.qres` is reserved for single-file compressed streams.
- **CLI Commands:** `qres create` has been renamed to `qres archive` to better reflect its function.
- **Manifest:** The internal manifest format has been updated to version 5.1 (Flag `0x05`). Checksums are now `xxhash64`.
