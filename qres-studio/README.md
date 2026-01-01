# QRES Studio v4.2

**Cross-platform GUI for QRES compression with P2P collective learning**

Built with Tauri (Rust) + Svelte for maximum performance and minimal bundle size.

---

## 🎨 Features

### 🎯 Drop Zone
- **Drag-Drop Interface**: Files or entire folders
- **Real-time Progress**: Color-coded engine visualization
  - 🟡 Gold = ZSTD
  - 🔵 Blue = LINEAR  
  - 🟢 Green = IPEPS
  - 🟣 Purple = LSTM
- **Training Detection**: Auto-prompt for data files (CSV, JSON, TXT)

### 🐝 Hive Mind
- **Persistent Swarm Toggle**: Enable collective learning
- **Live Statistics**: Bytes saved, compression ratio, file count
- **Engine Usage**: Visual breakdown of predictor selection
- **Collective Learning Banner**: Shows swarm participation status

### 🌐 P2P Networking
- **Persistent State**: Swarm toggle survives app restarts
- **Automatic Sync**: Shares learnings with Hive when enabled
- **Zero-Shot Adaptation**: Benefit from collective intelligence

---

## 🚀 Quick Start

### Development
```bash
# Install dependencies
npm install

# Run dev server
npm run tauri dev
```

### Production Build
```bash
# Build for your platform
npm run tauri build

# Output: src-tauri/target/release/bundle/
```

---

## 📖 Usage

### Compress a File
1. Drag file onto circular Drop Zone
2. Choose save location via dialog
3. Watch real-time progress
4. Stats auto-update

### Compress a Folder
1. Drag folder onto Drop Zone
2. Choose destination folder
3. All files compressed recursively
4. Directory structure preserved

### Enable Swarm Network
1. Go to **Hive Mind** tab
2. Toggle **"Swarm Network"** ON
3. Status changes to 🟢 Connected
4. All compressions now sync with Hive

### Train on Data
1. Drop CSV/JSON/TXT file
2. Click **"Yes"** when prompted
3. Meta-brain trains on your data
4. Results displayed in alert

---

## 🏗️ Architecture

### Backend (Rust)
- **Tauri Commands**: Compression, decompression, stats, swarm, training
- **P2P State**: Global state with `Arc<Mutex<P2PState>>`
- **Persistent Storage**: JSON files in app data directory

### Frontend (Svelte)
- **App.svelte**: Main layout with tab navigation
- **DropZone.svelte**: Drag-drop with folder support
- **HiveMind.svelte**: Stats dashboard with persistent swarm toggle

---

## 📁 File Structure

```
qres-studio/
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs    # Tauri commands (P2P, folders, training)
│   │   ├── lib.rs         # Plugin registration
│   │   └── main.rs        # App entry point
│   └── Cargo.toml         # Rust dependencies
├── src/
│   ├── App.svelte         # Main layout
│   ├── DropZone.svelte    # Compression interface
│   └── HiveMind.svelte    # Analytics dashboard
├── static/                # Static assets
└── package.json           # Node dependencies
```

---

## 🔧 Configuration

### Persistent Files
Located in app data directory:
- **Windows**: `%APPDATA%\qres-studio\`
- **macOS**: `~/Library/Application Support/qres-studio/`
- **Linux**: `~/.local/share/qres-studio/`

**Files**:
- `stats.json` - Compression statistics
- `swarm_config.json` - Swarm enabled/disabled state

---

## 🎯 Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Switch to Drop Zone | `Ctrl+1` |
| Switch to Hive Mind | `Ctrl+2` |
| Toggle Swarm | `Ctrl+S` |
| Refresh Stats | `F5` |

---

## 🐛 Troubleshooting

### "Ollama connection failed"
- **This is expected!** Ollama was removed in v4.1
- Use the new training integration instead

### Swarm toggle doesn't persist
- Check app data directory permissions
- Ensure `swarm_config.json` is writable

### Folder compression fails
- Verify destination folder exists
- Check file permissions
- Try smaller folders first

---

## 📚 Documentation

- **[P2P_IMPLEMENTATION.md](P2P_IMPLEMENTATION.md)** - v4.2 P2P guide
- **[STREAMLINED_RELEASE.md](STREAMLINED_RELEASE.md)** - v4.1 release notes
- **[../README.md](../README.md)** - Main project README
- **[../ROADMAP.md](../ROADMAP.md)** - Development roadmap

---

## 🤝 Contributing

See [../CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

**Areas of Interest**:
- UI/UX improvements
- Performance optimization
- Cross-platform testing
- Documentation

---

## 📄 License

Dual-licensed under MIT OR Apache-2.0.

---

**QRES Studio v4.2** - *Compression through Collective Intelligence* 🚀
