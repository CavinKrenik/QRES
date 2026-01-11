# QRES Edge Monitor v15.3.0

**Release Date:** January 11, 2026  
**Codename:** Edge Visualization

---

## Overview

v15.3.0 transforms QRES Studio into a **Real-Time IoT Edge Dashboard**, replacing the file compression interface with live sensor stream visualization.

---

## New Features

### 📡 Real-time IoT Streaming
- **Live Telemetry**: 10Hz sensor data from simulated ESP32, Pi-4 Cluster, and Jetson-Nano
- **D3.js Bandwidth Chart**: Scrolling Raw vs. QRES compressed bandwidth visualization
- **Connect to Swarm Toggle**: One-click sensor stream activation

### 🧠 Neural Graph Visualization
- **Interactive Topology**: Force-directed graph (Swarm, Mixer, Root, QNN, SNN nodes)
- **Real-time Updates**: Graph reflects current MetaBrain activity

### ⚡ MetaBrain State Monitoring
- **SNN Spike Visualizer**: Canvas-based spiking neuron animation
- **Regime Change Detection**: Visual feedback for neural adaptation

---

## Improvements

- **Simplified Header**: "QRES Edge" branding with clean badge
- **Navigation-Only Sidebar**: Icon-based (📡 🕸️ 🧠) replacing file controls
- **Window Size**: 1200x800 default for better visibility
- **Flexbox Layout**: More reliable viewport sizing

---

## Bug Fixes

- WebSocket connection stability for long-running streams
- TypeScript status mapping (`IDLE` → `OFFLINE`)
- A11y form label compliance

---

## Known Issues

| Feature | Native Mode | Browser Mode |
|---------|-------------|--------------|
| Swarm Toggle | ✅ Full | ⚠️ Simulated |
| Hive Mind Sync | ✅ Full | ❌ Disabled |

> **Note:** Browser sandboxing prevents P2P socket connections. Use `npm run tauri dev` for full functionality.

---

## Upgrade

```bash
git pull origin main
cd qres-studio && npm install
npm run dev
```

---

*Full changelog: [v15.2.0...v15.3.0](https://github.com/CavinKrenik/QRES/compare/v15.2.0...v15.3.0)*
