# QRES Studio

**QRES Studio** is the official desktop GUI for the Quantum-Relational Encoding System. Built with **Tauri v2** and **Svelte 5**, it provides a beautiful, real-time visualization of the compression process.

![QRES Studio](../DOCS/studio_screenshot_mockup.png)

## ✨ Features

- **Drag-and-Drop Compression**: Simply drop standard files or `.qres` archives to process them.
- **Real-Time Visualization**: Watch the Neural Selector switch engines (Gold/Zstd, Blue/Linear, Green/iPEPS, Purple/LSTM) in real-time.
- **Hive Mind Integration**: Control the local swarm daemon directly from the UI. Toggle "Swarm Mode" to share learnings.
- **Stats Dashboard**: Track total bytes saved and "Hive Wisdom" accumulation.

## 🛠️ Development

### Prerequisites
- **Node.js**: v18+
- **Rust**: v1.75+
- **Tauri CLI**: `cargo install tauri-cli`

### Setup
```bash
cd qres-studio
npm install
```

### Run Locally (Dev Mode)
This starts the Svelte web server and the Tauri native window.
```bash
npm run tauri dev
```

## 📦 Build

To build the optimized release binary (installers):

```bash
npm run tauri build
```
Artifacts will be in `src-tauri/target/release/bundle/`.

## 🏗️ Architecture

- **Frontend**: Svelte 5 (Vite)
- **Backend**: Rust (Tauri Host)
- **Communication**: Tauri IPC (`invoke` / `listen`)

The frontend remains strictly a view layer. All heavy compression logic happens in the Rust backend (`src-tauri/src/lib.rs`), which spawns async tasks to keep the UI smooth.
