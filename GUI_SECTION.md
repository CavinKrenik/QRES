---

## 🎨 Quick Start - GUI (QRES Studio)

### Installation
Download the latest installer from [Releases](https://github.com/CavinKrenik/QRES/releases):
- **Windows**: `QRES-Studio-Setup.msi`
- **macOS**: `QRES-Studio.dmg`
- **Linux**: `qres-studio.AppImage`

### Usage
1. **Launch** QRES Studio
2. **Drag & Drop** any file onto the glowing ring
   - `.qres` file → Automatically decompresses (folder picker)
   - Any other file → Automatically compresses (save dialog)
3. **Watch** the ring change color as AI switches engines:
   - 🟡 Gold = Zstd (standard compression)
   - 🔵 Blue = Linear (predictable patterns)
   - 🟢 Green = iPEPS (quantum/complex signals)
   - 🟣 Purple = LSTM (time-series/drifting data)
4. **Monitor** real-time compression chart and stats

### Hive Mind Dashboard
- View bytes saved and compression statistics
- Enable/disable P2P swarm with toggle switch  
- See engine usage breakdown

---

## ⌨️ CLI Usage (Advanced)
For automation and scripting, use the command-line interface.

### 1. Installation
```bash
cargo install --path qres_rust
```

### 2. Basic Compression
