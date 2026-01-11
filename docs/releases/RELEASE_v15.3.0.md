# QRES Edge Monitor v15.3.0 Release Notes

**Release Date:** January 11, 2026  
**Codename:** Edge Visualization

---

## 🎉 New Features

### Real-time IoT Streaming Dashboard
- **Live Telemetry Visualization**: 10Hz sensor data from simulated edge devices (ESP32, Pi-4 Cluster, Jetson-Nano)
- **D3.js Bandwidth Chart**: Scrolling line chart comparing Raw vs. QRES compressed bandwidth in real-time
- **Connect to Swarm Toggle**: One-click activation of sensor stream simulation

### Neural Graph Visualization
- **Interactive Topology**: Force-directed graph showing Swarm, Mixer, Root, QNN, and SNN node connections
- **Zoom & Pan**: Navigate complex neural relationships with D3-powered controls
- **Real-time State Reflection**: Graph updates to reflect current MetaBrain activity

### MetaBrain State Monitoring
- **SNN Spike Visualizer**: Canvas-based animation of spiking neurons reacting to incoming data intensity
- **Packets Processed Counter**: Live metric tracking compression throughput
- **Regime Change Detection**: Visual feedback when neural network adapts to signal drift

---

## 🔧 Improvements

### UI/UX Enhancements
- **Streamlined Header**: Simplified to "QRES Edge" branding with clean badge
- **Navigation-Only Sidebar**: Icon-based navigation (📡 🕸️ 🧠) replacing legacy file controls
- **Cyberpunk Dark Theme**: Refined color palette (#050510 background, #00ffcc/#ff4444 accents)
- **Monospace Data Values**: JetBrains Mono font for all telemetry readouts

### Architecture
- **Flexbox Layout**: Replaced CSS Grid for more reliable viewport sizing
- **WASM-First Design**: Browser-native compression via `qres_wasm` module
- **Window Size Increase**: Default 1200x800 for better dashboard visibility

---

## 🐛 Bug Fixes

- **WebSocket Stability**: Improved connection handling for long-running streams
- **Type Safety**: Fixed TypeScript status mapping (`IDLE` → `OFFLINE`)
- **A11y Compliance**: Resolved form label accessibility warnings

---

## ⚠️ Known Issues

### Browser Mode Limitations

| Feature | Native Mode | Browser Mode |
|---------|-------------|--------------|
| Swarm Toggle | ✅ Full | ⚠️ Simulated |
| Hive Mind Sync | ✅ Full | ❌ Disabled |
| P2P Networking | ✅ Full | ❌ Disabled |

**Explanation:** Browser sandboxing prevents direct P2P socket connections required for Swarm synchronization. The "Swarm Toggle" in browser mode runs a client-side simulation only. For full Hive Mind functionality, use the native Tauri build (`npm run tauri dev`).

### WASM Loading
- Some environments may show 403 errors for `qres_wasm_bg.wasm`
- **Workaround:** Rebuild WASM with `wasm-pack build --target web` in `qres_rust/qres_wasm/`

---

## 📦 Upgrade Instructions

```bash
# Pull latest
git pull origin main

# Install dependencies
cd qres-studio
npm install

# Launch dashboard
npm run dev
```

---

## 📊 Metrics

| Metric | v15.2.0 | v15.3.0 |
|--------|---------|---------|
| Startup Time | ~2s | ~1.6s |
| Bundle Size | 1.2MB | 1.4MB |
| Stream Latency | N/A | <2ms/packet |

---

**Full Changelog:** [v15.2.0...v15.3.0](https://github.com/CavinKrenik/QRES/compare/v15.2.0...v15.3.0)
