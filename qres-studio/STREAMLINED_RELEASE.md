# QRES Studio v4.1 - Streamlined Release

## 🎯 Major Revamp: Focus on Core Compression

QRES Studio has been completely revamped to focus on what it does best: **neural compression**. All AI/Ollama features have been removed for a cleaner, faster, more focused experience.

## ✅ What Changed

### Removed
- ❌ **AI Gen Tab** - Completely removed
- ❌ **Ollama Integration** - All LLM code stripped out
- ❌ **reqwest dependency** - No longer needed
- ❌ **LMInsights.svelte** - Deleted
- ❌ **query_lm, run_training, save_ai_data commands** - Removed from backend

### Improved
- ✅ **Responsive Drop Zone** - Ring now fits viewport (max 80vh, 80vw)
- ✅ **Cleaner UI** - Two-tab interface (Drop Zone, Hive Mind)
- ✅ **Better Stats** - Simplified cards with clear metrics
- ✅ **Engine Usage Bars** - Native CSS bars instead of Chart.js
- ✅ **Faster Build** - Fewer dependencies, quicker compilation

## 🎨 Current Features

### Drop Zone Tab
- **Circular Drag-Drop Interface**
  - Responsive sizing: `min(400px, 80vw)` × `min(400px, 60vh)`
  - Fits viewport without scrolling
  - Box icon (📦) with "Drop file here" prompt
  
- **Smart File Detection**
  - `.qres` files → Decompress mode
  - Other files → Compress mode
  - Tauri dialog for save location
  
- **Real-time Progress**
  - Pulsing ring animation
  - Color-coded by engine:
    - 🟡 Gold = ZSTD
    - 🔵 Blue = LINEAR
    - 🟢 Green = IPEPS
    - 🟣 Purple = LSTM
  - Shows: Progress %, Engine name, Compression ratio

### Hive Mind Tab
- **Stats Cards**
  - Bytes Saved Today (MB)
  - Hive Wisdom (% efficiency)
  - Total Compressions
  
- **Swarm Network Toggle**
  - On/Off checkbox
  - Status indicator: 🟢 Connected / ⚪ Offline
  - Runs `hive_sync.py` when enabled
  
- **Engine Usage**
  - Native CSS progress bars
  - Shows count and percentage per engine
  - Auto-updates every 5 seconds

### Top Stats Bar
- 💾 Saved: X.XMB
- 📦 Files: N

## 🔧 Technical Stack

### Backend (Rust)
```toml
[dependencies]
tauri = "2"
tauri-plugin-opener = "2"
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
qres_rust = { path = "../../qres_rust", default-features = false }
```

**Commands:**
- `compress_file(src, dest)` - Compress with progress events
- `decompress_file(src, dest)` - Decompress .qres files
- `get_stats()` - Load persistent stats from JSON
- `toggle_swarm(enabled)` - Run hive_sync.py subprocess

### Frontend (Svelte)
**Components:**
- `App.svelte` - Main layout, 2 tabs
- `DropZone.svelte` - Drag-drop compression UI
- `HiveMind.svelte` - Stats dashboard

**No external chart libraries** - Pure CSS for visualizations

## 📊 Stats Persistence

Stats are stored in JSON at:
- **Windows**: `%APPDATA%\qres-studio\stats.json`
- **macOS**: `~/Library/Application Support/qres-studio/stats.json`
- **Linux**: `~/.local/share/qres-studio/stats.json`

**Structure:**
```json
{
  "bytes_saved": 0,
  "total_compressions": 0,
  "avg_ratio": 0.0,
  "engines_used": {
    "zstd": 0,
    "linear": 0,
    "ipeps": 0,
    "lstm": 0
  }
}
```

## 🚀 Build Instructions

### Development
```bash
cd qres-studio
npm install
npm run tauri dev
```

### Production
```bash
npm run tauri build
```

### Check Types
```bash
npm run check
```

## 🎯 Usage Workflow

### Compress a File
1. Open QRES Studio
2. Go to **Drop Zone** tab
3. Drag any file onto the circular target
4. Choose save location in dialog
5. Watch real-time progress
6. Stats auto-update in top bar

### Decompress a File
1. Drag a `.qres` file onto Drop Zone
2. Choose save location
3. File is decompressed instantly

### View Analytics
1. Go to **Hive Mind** tab
2. See compression stats
3. Toggle Swarm Network (optional)
4. View engine usage breakdown

## 🔒 Security & Privacy

- ✅ All file operations use Tauri's secure dialog
- ✅ Stats stored locally (no cloud)
- ✅ No network requests (except optional Hive sync)
- ✅ No telemetry or tracking

## 📐 Responsive Design

The Drop Zone ring is now fully responsive:
```css
width: min(400px, 80vw);
height: min(400px, 60vh);
aspect-ratio: 1;
```

This ensures:
- Never exceeds 80% of viewport width
- Never exceeds 60% of viewport height
- Maintains perfect circle (1:1 aspect ratio)
- No scrolling required

## 🎨 Theme

**Dark Futuristic:**
- Background: Linear gradient `#0a0e27` → `#1a1f3a`
- Glassmorphism: `backdrop-filter: blur(10px)`
- Accent: Indigo to Purple gradient
- Text: `#e0e7ff` (light blue-white)

## 📝 File Changes

**Modified:**
- `src-tauri/Cargo.toml` - Removed reqwest
- `src-tauri/src/commands.rs` - Removed AI commands
- `src-tauri/src/lib.rs` - Updated handler registration
- `src/App.svelte` - Removed AI Gen tab
- `src/DropZone.svelte` - Fixed responsive sizing
- `src/HiveMind.svelte` - Simplified with CSS bars

**Deleted:**
- `src/LMInsights.svelte` - AI component removed

## 🐛 Known Limitations

1. **Hive Sync** requires Python environment with `hive_sync.py`
2. **Stats** reset if JSON file is deleted
3. **Engine tracking** only works if qres_rust reports engine names

## 🚧 Future Enhancements

- [ ] Batch compression (multiple files)
- [ ] Compression presets (lossy/lossless)
- [ ] Export stats to CSV
- [ ] Dark/Light theme toggle
- [ ] Compression history log

---

**Version**: 4.1.0
**Status**: ✅ Production Ready
**Build Time**: ~2-3 minutes (clean)
**Bundle Size**: ~15MB (Windows .msi)
**Last Updated**: January 1, 2026

**QRES Studio - Neural Compression, Simplified.** 🚀
