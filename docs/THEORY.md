# QRES Theory: The "Living Brain" Architecture

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

## Component Logic

**The Core (Body):** A pure `no_std` Rust library (`qres_core`) that executes the compression codec using a "Zero-Copy Residual" approach. It runs on bare-metal microcontrollers (e.g., STM32, ESP32) or inside WASM sandboxes.

**The Daemon (Mind):** A background service (`qres_daemon`) that handles "Meta-Learning". It uses a PPO-based RL agent to dynamically re-weight predictors and manages the multi-layer security stack (Differential Privacy, Krum Aggregation, ZK Proofs).

## Deployment Environment (Azure)

**Infrastructure Logic:** The QRES Cloud Core operates within a dedicated Virtual Network (QRES-vnet) to ensure secure isolation of training data. The primary node (QRES) is protected by a Network Security Group (QRES-nsg) which strictly filters inbound traffic, allowing only encrypted WebSocket connections from authorized Edge Clients via the static public gateway (QRES-ip). This topology allows for scalable horizontal expansion—additional VM instances can be added to the subnet (default) without altering the public-facing entry point.
