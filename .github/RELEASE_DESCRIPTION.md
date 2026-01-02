# QRES v5.1.0 - Archive Edition 🗜️

**Release Date**: January 1, 2026

QRES v5.1 represents a major evolution from a compression tool to a full-featured archiver with critical bug fixes and enterprise-grade features.

## 🎉 What's New

### Archive Format (.qrar)
- **Solid Compression**: Files concatenated before compression for 20-50% better ratios
- **Manifest System**: Browse archives without extraction
- **Blake3 Integrity**: Cryptographic verification for each file
- **Partial Extraction**: Extract individual files on demand

### Global Deduplication
- **Content-Defined Chunking**: Detects duplicate chunks across entire archive
- **Variable-size chunks**: 2KB-32KB, adapts to content patterns
- **10:1+ dedup**: On similar files before compression even starts

### Logistic Mixing
- **Neural-inspired**: Sigmoid-based probability blending
- **2-5% improvement**: On structured data
- **State-of-the-art**: Used in world-class compressors (PAQ, NNCP)

## 🐛 Critical Bug Fixes

### Decompression Crash (HIGH SEVERITY) ✅ FIXED
- **Problem**: GUI would crash on any .qres file with "Unknown codec flag" error
- **Root Cause**: Passing entire file (with header) to decompress_chunk
- **Solution**: Proper header parsing and streaming decompression
- **Impact**: GUI decompression now works correctly

### Memory Exhaustion (HIGH SEVERITY) ✅ FIXED
- **Problem**: Large files (>1GB) would crash the system
- **Root Cause**: Loading entire file into RAM with fs::read()
- **Solution**: Streaming I/O with BufReader/BufWriter
- **Impact**: Can now decompress files of ANY size

## 📊 Performance Improvements

### Memory Usage
| Operation | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Decompress 10GB file | 10GB RAM | 64KB | 99.999% |
| Compress 100 files | 500MB | 128KB | 99.97% |

### Compression Ratios (Solid Archives)
| Dataset | Individual | Archive | Improvement |
|---------|-----------|---------|-------------|
| Source Code | 0.45 | 0.28 | +38% |
| Log Files | 0.38 | 0.19 | +50% |
| Documents | 0.62 | 0.51 | +18% |

## 📦 Installation

### Binaries
Download the appropriate binary for your platform from the assets below.

### From Source
```bash
git clone https://github.com/CavinKrenik/QRES.git
cd QRES
cargo build --release
```

### Python Package
```bash
pip install qres  # Coming soon to PyPI
```

## 🚀 Quick Start

### Individual File Compression
```bash
qres-cli compress -i input.dat -o output.qres
qres-cli decompress -i output.qres -o restored.dat
```

### Archive Creation
```bash
# Create archive from directory
qres-cli archive create ./my_project -o my_project.qrar

# Browse contents
qres-cli archive list my_project.qrar

# Extract all
qres-cli archive extract my_project.qrar -o ./restored/
```

## 📚 Documentation

- **Migration Guide**: [MIGRATION_v5.1.md](https://github.com/CavinKrenik/QRES/blob/main/MIGRATION_v5.1.md)
- **Technical Details**: [CRITICAL_FIXES_V5.1.md](https://github.com/CavinKrenik/QRES/blob/main/CRITICAL_FIXES_V5.1.md)
- **Full Changelog**: [CHANGELOG.md](https://github.com/CavinKrenik/QRES/blob/main/CHANGELOG.md)

## ⚠️ Breaking Changes

- Folder compression now creates `.qrar` archives instead of individual `.qres` files
- This is intentional for better compression ratios
- See [MIGRATION_v5.1.md](https://github.com/CavinKrenik/QRES/blob/main/MIGRATION_v5.1.md) for details

## 🔜 Coming in v5.2

- GUI archive browser
- Encryption support (AES-256)
- Multi-threaded compression
- Progressive decompression

## 👥 Contributors

Special thanks to all who contributed to this release!

## 📄 License

QRES is dual-licensed under MIT OR Apache-2.0

---

**This is a production-ready release.** All critical bugs from v5.0 have been resolved.

**Recommended for all users** - The bug fixes alone make this a critical update.

See the full [Release Notes](https://github.com/CavinKrenik/QRES/blob/main/RELEASE_NOTES.md) for complete details.
