# QRES Edge Monitor v15.3

**Real-time IoT Dashboard for the QRES Swarm**

Built with Tauri v2 (Rust) + Svelte 5 + WebAssembly.
This dashboard simulates a swarm of edge devices to demonstrate **biologically-inspired compression** on live sensor streams.

---

## 🎨 Features

### 📡 Live Telemetry Stream
- **Real-time Visualization**: D3.js scrolling charts showing Raw vs. QRES bandwidth.
- **Simulated Sensors**: Generates 10Hz vibration/temperature data to mimic industrial IoT.
- **Bit-Perfect Accuracy**: Uses the authentic WASM-compiled Rust engine.

### 🧠 Biologically-Inspired Vis
- **SNN Spike Visualizer**: Watch the "MetaBrain" neurons fire in response to data intensity.
- **Regime Change Simulation**: Trigger abrupt data anomalies and watch the neural network adapt in real-time.

### 🕸️ Swarm Network Map
- **Active Peers**: Monitor the status of connected edge nodes (Learning vs. Inferring).
- **Global Efficiency**: Track collective bandwidth savings across the swarm.

---

## 🚀 Quick Start

### Prerequisites
- **Node.js**: v18 or higher
- **Rust**: Latest stable (for native builds only)

### Development
```bash
# Install dependencies
npm install

# Run dev server (browser mode)
npm run dev

# Open http://localhost:1420
```

### Production Build
```bash
# Build native app (requires Tauri CLI)
npm install -g @tauri-apps/cli
npm run tauri build

# Output: src-tauri/target/release/bundle/
```

---

## 📖 Usage

### 1. Launch the Dashboard
Run `npm run dev` and open the local web interface.

### 2. Connect to Swarm
Click the **"Connect to Swarm"** toggle. The simulated sensors will begin streaming data, and the bandwidth chart will populate in real-time.

### 3. Trigger an Anomaly
Click the **"Trigger Regime Change"** button.
* **Observation:** The vibration values will spike (Regime Shift).
* **Result:** Watch the compression ratio momentarily drop, then recover as the SNN learns the new pattern (~20s recovery).

---

## 🏗️ Architecture

### Backend (Rust + Tauri v2)
- **Tauri Commands**: Compression, decompression, stats
- **WASM Core**: `qres_core` compiled to WebAssembly for browser-native operation
- **IPC Communication**: Secure cross-process communication

### Frontend (Svelte 5 + TypeScript)
- **App.svelte**: Main layout with sidebar navigation
- **IoTDashboard.svelte**: 3-panel streaming interface (Controls | Charts | Neural Viz)
- **LiveBandwidthChart.svelte**: D3.js real-time scrolling chart
- **SNNSpikeVisualizer.svelte**: Canvas-based neural activity display
- **NodeStatusPanel.svelte**: Edge device status monitoring
- **SensorSimulator.ts**: 10Hz telemetry generation with regime change support

### Dependencies
- **Tauri v2**: Cross-platform desktop app framework
- **Svelte 5**: Reactive UI framework
- **D3.js v7**: Data visualization library
- **qres-wasm**: WASM-compiled compression core

---

## 📁 File Structure

```
qres-studio/
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs    # Tauri commands
│   │   ├── lib.rs         # Plugin registration
│   │   └── main.rs        # App entry point
│   └── Cargo.toml         # Rust dependencies
├── src/
│   ├── App.svelte              # Main layout with sidebar nav
│   ├── components/
│   │   ├── IoTDashboard.svelte      # 3-panel streaming interface
│   │   ├── LiveBandwidthChart.svelte # D3 real-time chart
│   │   ├── SNNSpikeVisualizer.svelte # Neural activity canvas
│   │   ├── NodeStatusPanel.svelte    # Device status list
│   │   ├── SwarmConnectToggle.svelte # Connect/anomaly controls
│   │   ├── StarshipHeader.svelte     # QRES branding header
│   │   └── StarshipSidebar.svelte    # Navigation sidebar
│   ├── lib/
│   │   ├── SensorSimulator.ts  # 10Hz telemetry generator
│   │   ├── iotStore.ts         # Svelte streaming stores
│   │   └── compressionEngine.ts # WASM/Native engine bridge
│   └── routes/
│       └── +page.svelte   # SvelteKit entry
├── static/                # Static assets
├── package.json           # Node dependencies
└── svelte.config.js       # Svelte configuration
```

---

## 🔧 Configuration

### Environment Detection
- **Browser Mode**: Runs WASM compression client-side (default for `npm run dev`)
- **Native Mode**: Uses Tauri IPC to call Rust daemon (requires `npm run tauri dev`)

### Compatibility
- **QRES Backend**: Compatible with QRES v15.x
- **Node.js**: Requires v18+
- **Rust**: Latest stable (for native builds)

---

## 🐛 Troubleshooting

### WASM 403 Error
If you see "qres_wasm_bg.wasm" 403 errors:
```bash
cd ../qres_rust/qres_wasm
wasm-pack build --target web
```

### Chart not rendering
- Ensure D3.js v7 is installed: `npm install d3`
- Check browser DevTools for errors

### Simulation not starting
- Click "Connect to Swarm" to start the 10Hz stream
- Check browser console for WASM initialization logs

---

## 📚 Documentation

- **[Main QRES README](../README.md)** - Project overview
- **[Benchmarks](../docs/BENCHMARKS.md)** - Performance data
- **[Whitepaper](../docs/WHITEPAPER.md)** - Technical details

---

## 📄 License

Dual-licensed under MIT OR Apache-2.0.

---

**QRES Edge Monitor v15.3** - *Real-time IoT Compression* 🚀
