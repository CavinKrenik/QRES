# QRES Studio v10.1

**Cross-platform GUI for QRES v10 (Hybrid Engine)**

Built with Tauri v2 (Rust) + Svelte 5 + WebAssembly.

---

## 🎨 Features

### 🌐 Hybrid Runtime (New in v10.5)
- **Native Mode (Default):** Uses the Rust Daemon for maximum performance and P2P Swarm access.
- **WASM Mode (Browser):** Runs `qres_core` entirely in the frontend thread. Zero system dependencies, perfect for quick client-side checks.

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

### 🌐 Hybrid Runtime (New in v10.5)
- **Native Mode (Default):** Uses the Rust Daemon for maximum performance and P2P Swarm access.
- **WASM Mode (Browser):** Runs `qres_core` entirely in the browser thread. Zero system dependencies, perfect for quick client-side checks.

### 🌐 P2P Networking
- **Persistent State**: Swarm toggle survives app restarts
- **Automatic Sync**: Shares learnings with Hive when enabled
- **Zero-Shot Adaptation**: Benefit from collective intelligence

### 📊 Knowledge Graph
- **Interactive Visualization**: D3.js powered force-directed graph
- **Zoom & Pan**: Navigate complex neural relationships
- **Auto-fit**: Automatically centers and scales the graph
- **Real-time Updates**: Reflects current brain state

---

## 🚀 Quick Start

### Prerequisites
- **Rust**: Latest stable version
- **Node.js**: v18 or higher
- **Tauri CLI**: `npm install -g @tauri-apps/cli`

### Development
```bash
# Install dependencies
npm install

# Run static analysis check
npm run check

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

### Backend (Rust + Tauri v2)
- **Tauri Commands**: Compression, decompression, stats, swarm, training
- **P2P State**: Global state with `Arc<Mutex<P2PState>>`
- **Persistent Storage**: JSON files in app data directory
- **IPC Communication**: Secure cross-process communication

### Frontend (Svelte 5 + TypeScript)
- **App.svelte**: Main layout with tab navigation and global state
- **DropZone.svelte**: Drag-drop interface with folder support
- **HiveMind.svelte**: Analytics dashboard with persistent swarm toggle
- **KnowledgeGraph.svelte**: Interactive D3.js visualization
- **Environment Guards**: Browser vs Tauri mode detection

### Dependencies
- **Tauri v2**: Cross-platform desktop app framework
- **Svelte 5**: Reactive UI framework with runes
- **D3.js v7**: Data visualization library
- **@zerodevx/svelte-toast**: Notification system

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
│   ├── App.svelte         # Main layout with tab navigation
│   ├── DropZone.svelte    # Compression interface
│   ├── HiveMind.svelte    # Analytics dashboard
│   ├── KnowledgeGraph.svelte # Neural graph visualization
│   ├── SwarmView.svelte   # P2P network view
│   ├── ArchiveView.svelte # Archive browser
│   ├── components/
│   │   ├── StarshipHeader.svelte # Header with stats
│   │   └── StarshipSidebar.svelte # Sidebar controls
│   └── types.d.ts         # TypeScript declarations
├── static/                # Static assets
├── package.json           # Node dependencies
├── jsconfig.json          # JavaScript config
└── svelte.config.js       # Svelte configuration
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

### Compatibility
- **QRES Backend**: Compatible with QRES v8.x
- **Tauri**: Built with Tauri v2 API
- **Node.js**: Requires v18+
- **Rust**: Latest stable recommended

---

## 🎯 Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Switch to Drop Zone | `Ctrl+1` |
| Switch to Hive Mind | `Ctrl+2` |
| Switch to Knowledge Graph | `Ctrl+3` |
| Toggle Swarm | `Ctrl+S` |
| Refresh Stats | `F5` |

---

## 🐛 Troubleshooting

### Import Resolution Errors
- **Tauri v2 Migration**: Ensure all imports use `@tauri-apps/api/core` instead of `@tauri-apps/api/tauri`
- **Run Check**: Use `npm run check` to verify TypeScript/svelte-check passes
- **Clear Cache**: Delete `node_modules` and `package-lock.json` if issues persist

### "invoke is not defined" Error
- **Environment Guards**: Check that Tauri API calls are wrapped in `window.__TAURI__` checks
- **Browser Mode**: App should work in browser for development (limited functionality)

### Swarm toggle doesn't persist
- Check app data directory permissions
- Ensure `swarm_config.json` is writable

### Folder compression fails
- Verify destination folder exists
- Check file permissions
- Try smaller folders first

### Graph visualization issues
- **D3.js**: Ensure D3 v7 is properly installed
- **Zoom/Pan**: Check browser console for JavaScript errors
- **Auto-fit**: Graph should center automatically on load

---

## 📚 Documentation

- **[P2P_IMPLEMENTATION.md](P2P_IMPLEMENTATION.md)** - v4.2 P2P guide
- **[STREAMLINED_RELEASE.md](STREAMLINED_RELEASE.md)** - v4.1 release notes
- **[../README.md](../README.md)** - Main QRES project README
- **[../ROADMAP.md](../ROADMAP.md)** - Development roadmap

---

## 🤝 Contributing

See [../CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

**Areas of Interest**:
- UI/UX improvements
- Performance optimization
- Cross-platform testing
- Documentation
- Graph visualization enhancements

---

## 📄 License

Dual-licensed under MIT OR Apache-2.0.

---

**QRES Studio v10.1** - *Compression through Collective Intelligence* 🚀
