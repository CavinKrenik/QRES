# QRES Release Notes

## v5.0.5 - "Singularity Patch" (January 2026)

**Release Date:** January 1, 2026
**Codename:** Singularity Patch

### 🐛 Fixes & Improvements
*   **Build Integrity**: Resolved all `clippy` lints and formatting issues for clean CI builds.
*   **Zombie Processes**: Fixed potential zombie process creation in `swarm_sim` benchmark.
*   **API Compatibility**: Updated `probe_api` example to match `constriction` crate changes.
*   **Stability**: Addressed potential `panic` in spectral predictor loop.

---

## v5.0.0 - "Quantum Leap" (January 2026)

**Release Date:** January 1, 2026
**Codename:** Quantum Leap

### 🚀 Major Features

#### 🧠 Advanced Mixing Architectures
*   **Order-N Logistic Mixer**: Scalable context mixing with `O(N)` SIMD-accelerated weights.
*   **Vectorized Predictors**: AVX2/SSE4.1 optimizations for `Mixer` and `GraphPredictor`.
*   **LzMatchPredictor**: New dictionary-based prediction for repetitive text/binary patterns.

#### ⚡ Performance & Intelligence
*   **Smart Pre-Pass**: Intelligent header analysis for optimal engine selection.
*   **Native P2P Swarm**: Full Rust `libp2p` implementation replacing Python server.
*   **SIMD Acceleration**: Significant throughput improvements across all backends.

#### 🔧 Core Improvements
*   **v5.0 Core**: Unification of Python and Rust versioning.
*   **Enhanced Stability**: Robust error handling and adaptive fallback mechanisms.

---

## v4.2.0 - "Collective Intelligence" (January 2026)

**Release Date:** January 1, 2026  
**Codename:** Collective Intelligence

### 🎉 Major Features

#### 🌐 P2P Collective Learning
*   **Persistent Swarm Network**: Toggle survives app restarts (saved to `swarm_config.json`)
*   **Collective Learning Banner**: Visual status of swarm participation
*   **Automatic Sync**: Shares meta-brain weights with Hive when enabled
*   **Zero-Shot Adaptation**: New nodes benefit from collective knowledge

#### 📁 Folder Compression
*   **Recursive Processing**: Drag entire folders to compress all files
*   **Structure Preservation**: Maintains directory hierarchy in output
*   **Progress Tracking**: Real-time updates per file
*   **Batch Statistics**: Aggregate stats for folder operations

#### 🎓 Training Integration
*   **Auto-Detection**: Identifies trainable data files (CSV, JSON, TXT, LOG, DAT)
*   **One-Click Training**: Prompt to train meta-brain on dropped data
*   **Subprocess Execution**: Runs `train_meta.py` with file as input
*   **Result Display**: Shows training output in UI

#### ⚡ Performance Optimizations
*   **Lazy Statistics**: 128-byte batching → **2-3x speed improvement**
*   **Enhanced Spectral**: 2048-point FFT with harmonics → **60%+ compression on sine waves**
*   **Criterion Benchmarks**: Automated performance tracking

### 📊 Benchmark Results

| Optimization | Before | After | Improvement |
|--------------|--------|-------|-------------|
| **Speed** | 50-100 MB/s | 100-200 MB/s | **2-3x faster** |
| **Sine Ratio** | 46.2% | <40% | **15%+ better** |
| **Folder Support** | ❌ | ✅ | **NEW** |
| **Swarm Persistence** | ❌ | ✅ | **NEW** |

### 🎨 QRES Studio Enhancements
*   Persistent swarm toggle with Svelte stores
*   Collective learning status banner
*   File/folder name display during processing
*   Improved progress visualization

### 🔧 Technical Changes
*   Added `tauri-plugin-fs` for filesystem operations
*   Added `tauri-plugin-websocket` for P2P networking
*   Added `walkdir` for recursive directory traversal
*   Added `lazy_static` for global state management

### 📦 New Commands
*   `compress_file` - Now handles files AND folders
*   `get_swarm_status` - Returns persistent swarm state
*   `train_on_file` - Runs training subprocess

---

## v4.1.0 - "Streamlined" (January 2026)

**Release Date:** January 1, 2026  
**Codename:** Streamlined

### 🎯 Focus: Core Compression

#### Removed
*   ❌ All Ollama/LLM integration
*   ❌ AI Gen tab and related features
*   ❌ `reqwest` dependency
*   ❌ Unnecessary documentation files

#### Improved
*   ✅ Responsive Drop Zone (fits viewport without scrolling)
*   ✅ Cleaner 2-tab interface (Drop Zone, Hive Mind)
*   ✅ Native CSS visualizations (no Chart.js)
*   ✅ Faster build times (~30% reduction)

### 📐 UI Fixes
*   Drop Zone ring: `min(400px, 80vw)` × `min(400px, 60vh)`
*   Maintains perfect circle aspect ratio
*   No scrolling required on any viewport

---

## v4.0.1 - "The Hive" (December 2025)

**Release Date:** December 2025  
**Codename:** The Hive

### 🚀 Highlights
*   **Swarm Intelligence (FedProx)**: Federated averaging for instant expert performance
*   **Spectral Predictor**: FFT-based engine achieving **46.2% on Sine Waves**
*   **Graph Predictor**: DAG-based predictor for structural logs
*   **Lossy Mode (RDO)**: Rate-Distortion Optimization for smart denoising

### 📊 Benchmark Results
| Dataset | QRES v4 Ratio | Notes |
| :--- | :--- | :--- |
| **IoT Telemetry** | **74.8%** | Beats LZ4 |
| **Sine Wave** | **46.2%** | SOTA |
| **All Zeros** | **43.1%** | Fast adaptation |

### 🐛 Fixes
*   Fixed `AttributeError` in Python bindings
*   Optimized `ans_coder` with batched updates (10x throughput)
*   Resolved Windows Unicode issues in CLI

### 📦 Assets
*   `qres-cli-windows.exe`: Standalone compressor
*   `qres-*.whl`: Python bindings
*   `qres_brain.json`: Pre-trained starter brain

---

## Version History

| Version | Date | Codename | Key Features |
|---------|------|----------|--------------|
| **v4.2.0** | Jan 2026 | Collective Intelligence | P2P, Folders, Training, 2-3x speed |
| **v4.1.0** | Jan 2026 | Streamlined | Removed AI, Fixed UI, Faster builds |
| **v4.0.1** | Dec 2025 | The Hive | FedProx, Spectral, Graph predictors |
| **v3.0.1** | Nov 2025 | - | Initial stable release |

---

## Upgrade Guide

### From v4.1 to v4.2
1. Update dependencies: `cd qres-studio && npm install`
2. Rebuild: `npm run tauri dev`
3. Enable swarm: Hive Mind tab → Toggle "Swarm Network"

### From v4.0 to v4.1
1. No breaking changes
2. Ollama features removed (if used, migrate to v4.2 training)

---

## Download

**Latest Release**: [v4.2.0](https://github.com/CavinKrenik/QRES/releases/tag/v4.2.0)

**Assets**:
- `qres-cli-windows.exe` - Standalone CLI
- `qres-*.whl` - Python package
- `qres-studio-*.msi` - GUI installer (Windows)
- `qres-studio-*.dmg` - GUI installer (macOS)
- `qres-studio-*.AppImage` - GUI installer (Linux)

---

**QRES v4.2** - *Compression through Collective Intelligence* 🚀
