# QRES

> A neural compression engine for time-series data

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18194636.svg)](https://doi.org/10.5281/zenodo.18194636)
[![ORCID](https://img.shields.io/badge/ORCID-0009--0008--9183--1278-green.svg)](https://orcid.org/0009-0008-9183-1278)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/CavinKrenik/QRES/release.yml?style=flat)](https://github.com/CavinKrenik/QRES/actions)
[![Version](https://img.shields.io/badge/version-v15.4.0-brightgreen)](https://github.com/CavinKrenik/QRES/releases)

**Paper:** [Download PDF](paper/QRES__Biologically_Inspired_Secure_Federated_Learning_for_Edge_IoT_Devices.pdf) | **DOI:** [10.5281/zenodo.18194636](https://doi.org/10.5281/zenodo.18194636)

## The Core Idea

What if we compressed data the way brains compress memories—by predicting patterns and encoding only surprises?

QRES started with a simple question: **Can you compress data using only three comparisons: `<`, `>`, `=`?**

That question led to the exploration of predictive compression, deterministic neural networks, and distributed model synchronization.

## How It Works

**Traditional compression:** "How do I represent this data in fewer bits?"  
**QRES approach:** "How often does this data differ from what I expected?"

When encoder and decoder share the same predictor, you only need to transmit the surprises—the moments when reality diverged from expectation.

```mermaid
graph LR
    A[Sensor Data] --> B[Predictor]
    B --> C[Expected Value]
    A --> D[Compare]
    C --> D
    D --> E{Surprise?}
    E -->|No| F[Send 0 bit]
    E -->|Yes| G[Encode Residual]
```

## Key Features

- **Deterministic Architecture**: Q16.16 fixed-point math ensures bit-perfect reproducibility across x86, ARM, RISC-V, WASM
- **Federated Learning**: Nodes share models peer-to-peer without transmitting raw data
- **Edge-Native**: `no_std` compatible, runs on bare-metal embedded devices
- **Hybrid Runtime**: Seamlessly switch between native performance and portable WebAssembly

## When to Use QRES

| Use QRES For | Don't Use QRES For |
|----------------|----------------------|
| Real-time IoT Telemetry (Sensor Streams) | High-entropy data (encryption, random) |
| Bandwidth-Constrained Edge Networks | Existing archives (.zip, .jpg, .mp4) |
| Structured Logs (Timestamps, patterns) | Files < 1KB (header overhead) |
| Regime Change Adaptation (Drifting signals) | Maximum speed (use LZ4) |

## Quick Start

Experience the **QRES Edge Dashboard** running the compression engine in real-time.

```bash
# 1. Clone the repo
git clone https://github.com/CavinKrenik/QRES.git
cd QRES/qres-studio

# 2. Install dependencies
npm install

# 3. Launch the IoT Swarm Simulator
npm run dev
# Open http://localhost:1420 to watch live telemetry compression
```

[Full installation docs →](docs/guides/P2P_IMPLEMENTATION.md)

## Hardware-in-the-Loop Simulation: "Director's Cut"
To demonstrate QRES capabilities on real-world chaotic data, this repository includes a **Weather Replay Engine** powered by the [Jena Climate Dataset](https://www.bgc-jena.mpg.de/wetter/).

We curated a specific "Calm → Storm" narrative to demonstrate **Bio-inspired Adaptation**:

### Phase 1: The Calm (Inference Mode)
During high-pressure stable weather, the swarm operates in `INFERRING` mode (Green).
- **Compression Ratio:** High (~10:1) as predictions are accurate.
- **Neural State:** Cool colors (Blue/Cyan) indicating low computational stress.

![Calm Phase](docs/images/qres_calm_preview.png)

### Phase 2: The Storm (Learning Mode)
At the 5-minute mark, the atmospheric pressure drops significantly ("Regime Change"). The Swarm detects this anomaly immediately.
- **Trigger:** The sensor node (`ESP32-01`) detects the anomaly first and switches to high-alert.
- **Response:** The Swarm enters `LEARNING` mode (Red), retraining the global model to adapt to the new high-entropy pattern.
- **Neural State:** Nodes turn Gold/Orange to visualize the intense peer-to-peer gradient sync.

![Storm Phase](docs/images/qres_storm_preview.png)

To run this simulation yourself:
```bash
# 1. Fetch the curated "Director's Cut" data
python3 scripts/fetch_weather_replay.py

# 2. Launch real-time dashboard
cd qres-studio && npm run dev
```

## The Journey

This project evolved through 15 major iterations:

| Version | Milestone |
|---------|-----------|
| v1 | Simple ternary encoding (left/right/equal) |
| v2 | Added delta prediction |
| v3 | Experimented with neural predictors |
| v4 | Switched to SNNs for edge compatibility |
| v8 | P2P swarm architecture |
| v10 | Tensor network correlators, Q16.16 determinism |
| v12 | Federated swarms, zero-bandwidth synchronization |
| v13 | Security hardening: ed25519 signatures, Krum aggregation |
| v14 | Robust aggregation: Multi-Krum, Trimmed Mean, Median |
| v15 | Privacy: Differential Privacy, Secure Aggregation, ZK Proofs |
| v15.2 | Publication Enhancement: Benchmarks, Reproducibility, Paper Draft |

[Read the full story →](docs/PHILOSOPHY.md)

---

## Publication

**QRES: Biologically-Inspired Secure Federated Learning for Edge IoT Devices**

**Author:** Cavin Krenik [![ORCID](https://img.shields.io/badge/ORCID-0009--0008--9183--1278-green.svg)](https://orcid.org/0009-0008-9183-1278)  
**Affiliation:** Olympic College, Shelton, WA, USA  
**Published:** January 2026  
**DOI:** [10.5281/zenodo.18194636](https://doi.org/10.5281/zenodo.18194636)

### Key Results

| Metric | Result |
|--------|--------|
| Compression | 48:1 synthetic, 22:1 IoT telemetry |
| Privacy | DP (ε=1.0) + Secure Aggregation + ZK Proofs |
| Byzantine Tolerance | Up to 45% malicious nodes (Krum) |
| Overhead | 3.1× runtime for full security stack |
| Scalability | 10-100 nodes, >85% success rate |

### Citation

```bibtex
@software{krenik2026qres,
  author       = {Krenik, Cavin},
  title        = {{QRES: Biologically-Inspired Secure
                   Federated Learning for Edge IoT Devices}},
  month        = jan,
  year         = 2026,
  publisher    = {Zenodo},
  version      = {v15.2.0},
  doi          = {10.5281/zenodo.18194636},
  url          = {https://doi.org/10.5281/zenodo.18194636}
}
```

---

## System Architecture: The "Living Brain"

QRES adopts a bio-mimetic architecture that separates deterministic execution (**The Body**) from adaptive learning (**The Mind**). This ensures bit-perfect reproducibility while allowing the system to "dream" and adapt to new data regimes.

```mermaid
graph TD
    %% Nodes
    IoT[Raw IoT Data] --> Core
    
    subgraph Body ["The Core (Body)"]
        style Body fill:#fff9c4,stroke:#fbc02d,stroke-width:2px
        Core[qres_core<br>No_Std Rust Library]
        
        subgraph Predictors [Predictor Ensemble]
            style Predictors fill:#ffffff,stroke:#fbc02d,stroke-width:1px,stroke-dasharray: 5 5
            SNN[SNN Predictor]
            Linear[Linear Predictor]
            Graph[Graph Predictor]
        end
        
        Core --- Predictors
    end
    
    Core -->|Residuals & Surprises| Daemon
    
    subgraph Mind ["The Daemon (Mind)"]
        style Mind fill:#e1f5fe,stroke:#0277bd,stroke-width:2px
        Daemon[qres_daemon<br>Async Service]
        MetaBrain[MetaBrain RL Agent]
        
        subgraph Security ["Security Stack"]
            style Security fill:#ffffff,stroke:#0277bd,stroke-width:1px,stroke-dasharray: 5 5
            L1[Layer 1: Differential Privacy]
            L2[Layer 2: Secure Aggregation]
            L3[Layer 3: ZK Proofs]
            
            L1 --> L2 --> L3
        end
        
        Daemon --- MetaBrain
        Daemon --- Security
    end

    Security -->|Signed Updates| Swarm[P2P Swarm]
    Swarm -->|Aggregated Model| Cloud[Cloud / Aggregator]
```

### Component Logic

- **The Core (Body):** A pure `no_std` Rust library (`qres_core`) that executes the compression codec using a "Zero-Copy Residual" approach. It runs on bare-metal microcontrollers (e.g., STM32, ESP32) or inside WASM sandboxes.

- **The Daemon (Mind):** A background service (`qres_daemon`) that handles "Meta-Learning". It uses a PPO-based RL agent to dynamically re-weight predictors and manages the multi-layer security stack (Differential Privacy, Krum Aggregation, ZK Proofs).

[See paper PDF for technical details →](paper/QRES__Biologically_Inspired_Secure_Federated_Learning_for_Edge_IoT_Devices.pdf)

### Deployment Environment (Azure)

![QRES Network Topology](assets/Networkimg.png)
<br>
**Figure 2:** The QRES Cloud Core architecture deployed on Azure.

**Infrastructure Logic**
The QRES Cloud Core operates within a dedicated Virtual Network (`QRES-vnet`) to ensure secure isolation of training data. The primary node (`QRES`) is protected by a Network Security Group (`QRES-nsg`) which strictly filters inbound traffic, allowing only encrypted WebSocket connections from authorized Edge Clients via the static public gateway (`QRES-ip`). This topology allows for scalable horizontal expansion—additional VM instances can be added to the subnet (default) without altering the public-facing entry point.

#### Diagram Representation
```mermaid
graph TD
    subgraph Azure ["Azure Cloud Resource Group"]
        style Azure fill:#f9f9f9,stroke:#333,stroke-width:2px
        
        Gateway(Static Public IP<br>QRES-ip) -->|WebSocket :443| NSG
        
        subgraph VNet ["Virtual Network (QRES-vnet)"]
            style VNet fill:#e3f2fd,stroke:#2196f3,stroke-width:2px,stroke-dasharray: 5 5
            
            NSG(Network Security Group<br>QRES-nsg) -->|Allow| VM
            
            subgraph Subnet ["Default Subnet"]
                style Subnet fill:#ffffff,stroke:#90caf9,stroke-width:1px
                VM[Virtual Machine<br>Node: QRES]
            end
        end
    end

    Client[Edge Client] -->|Encrypted Traffic| Gateway
```

## Performance

Performance metrics are now visualized in real-time via the **Edge Dashboard**.

Historical benchmarks on Intel Ice Lake (v15.2):

| Dataset | Ratio | Speed |
|---------|-------|-------|
| **Sensor Stream** | ~0.045 (22:1) | 85 MB/s |
| **Synthetic Wave** | ~0.02 (48:1) | 120 MB/s |

To validate these results on your hardware, run the simulation:
`npm run dev`

## Implementation Status

| Status | Components |
|--------|------------|
| **Production Ready** | Core engine, Python bindings, WASM decoder |
| **Experimental** | Federated dreaming, regime adaptation |
| **Security Complete** | Authentication (v13), Robust Aggregation (v14), Privacy (v15) |
| **Roadmap** | Arithmetic coding, FPGA acceleration |

[Detailed status →](docs/IMPLEMENTATION_STATUS.md)

## Project Structure

- `qres_rust/` – Rust workspace containing core engine and daemon
  - `qres_core`: High-performance, `no_std` compression library
  - `qres_daemon`: P2P node and API service
- `python/` – Python bindings and experimental ML models
- `qres-studio/` – Cross-platform GUI (Tauri/Svelte)
- `docs/` – Documentation hub

## Documentation

- [Implementation Status](docs/IMPLEMENTATION_STATUS.md)
- [Technical Deep Dives](docs/TECHNICAL_DEEP_DIVES.md)
- [Contributing Guide](docs/CONTRIBUTING.md)

## Frequently Asked Questions

**Q: Is this production-ready?**  
A: The core engine is stable and deterministic. Federated learning is experimental. Security features (authentication, Byzantine-tolerant aggregation, privacy) are **complete** as of v15.2. See [Implementation Status](docs/IMPLEMENTATION_STATUS.md).

**Q: How does this compare to Zstd/Gzip?**  
A: QRES is specialized for repetitive time-series data. For general-purpose compression, use Zstd. QRES shines when data patterns are predictable and bandwidth is constrained.

**Q: Does this use quantum computing?**  
A: No. QRES runs on classical hardware. Early versions had misleading naming that has been removed.

**Q: What's the learning curve?**  
A: For basic use (compress/decompress): low, similar to any compression tool. For federated swarm deployment: moderate, requires understanding of distributed systems.

**Q: Can I use this in production?**  
A: The compression engine is solid. The P2P/federated features now include ed25519 authentication (v13), Byzantine-tolerant Krum aggregation (v14), and differential privacy with ZK proofs (v15). Safe for most environments; production hardening is ongoing.

## License

Apache 2.0 – See [LICENSE](LICENSE)

---

## What I Learned

**Before:** I thought compression was about clever bit-packing  
**After:** Compression is prediction. Better predictions = better compression

**Challenge that surprised me:** Handling regime changes gracefully. Static predictors fail when patterns shift.

**What I'd do differently:** Start with security/Byzantine tolerance from day one. Adding it retroactively is painful.

[Read more about the development journey →](docs/PHILOSOPHY.md)
