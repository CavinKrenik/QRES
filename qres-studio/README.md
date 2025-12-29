# QRES Studio - GUI Installation & Build Guide

## User Installation (Recommended)

### Windows
1. Download `QRES-Studio-Setup.msi` from [Releases](https://github.com/CavinKrenik/QRES/releases)
2. Double-click to install
3. Launch from Start Menu or Desktop shortcut

**Context Menu Integration** (Optional):
- Right-click any file → "Compress with QRES"
- Automatically opens QRES Studio with the file

### macOS
1. Download `QRES-Studio.dmg`
2. Open DMG and drag to Applications
3. Launch from Applications folder

### Linux
1. Download `qres-studio.AppImage`
2. Make executable: `chmod +x qres-studio.AppImage`
3. Run: `./qres-studio.AppImage`

---

## Developer Build (From Source)

### Prerequisites
- Rust 1.70+ (`rustup`)
- Node.js 18+ & npm
- Platform-specific:
  - **Windows**: Visual Studio Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libwebkit2gtk`, `build-essential`

### Build Steps

```bash
cd qres-studio

# Install frontend dependencies
cd ui && npm install && cd ..

# Development mode (hot reload)
npm run tauri:dev

# Production build
npm run tauri:build
```

### Output Locations
- **Windows**: `target/release/bundle/msi/QRES Studio_0.1.0_x64_en-US.msi`
- **macOS**: `target/release/bundle/dmg/QRES Studio_0.1.0_x64.dmg`
- **Linux**: `target/release/bundle/appimage/qres-studio_0.1.0_amd64.AppImage`

### Bundle Size Optimization
Target: < 10MB  
Achieved by:
- Svelte (lightweight framework)
- Direct Rust linking (no Electron overhead)
- --release optimizations

---

## Development Notes

### Project Structure
```
qres-studio/
├── src/              # Tauri Rust backend
│   ├── main.rs       # App entry
│   └── commands.rs   # Real-time compression commands
├── ui/src/           # Svelte frontend
│   ├── App.svelte
│   ├── DropZone.svelte
│   └── HiveMind.svelte
└── Cargo.toml        # Links to ../qres_rust
```

### Key Features
- **Direct qres_rust Integration**: No CLI subprocess overhead
- **Real-time Events**: Window.emit() for live progress
- **Smart Drop**: Auto-detects .qres vs regular files
- **Engine Visualization**: Ring color changes with active engine

### Troubleshooting

**Build fails with "qres_rust not found"**:
```bash
# Ensure you're in the QRES root directory
cd C:\Dev\QRES
cargo build --manifest-path qres_rust/Cargo.toml
```

**GUI doesn't start**:
- Check logs: `~/.qres/*.log`
- Verify API server isn't already running on port 3030

---

## Contributing
See main [README.md](../README.md) for contribution guidelines.
