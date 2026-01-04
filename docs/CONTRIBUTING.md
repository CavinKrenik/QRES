# Contributing to QRES

We welcome contributions to the Singularity! Since QRES is a hybrid Rust/Python system, there are a few things to know about the build process.

## Development Setup

### Prerequisites
*   **Rust:** Stable toolchain (install via `rustup`).
*   **Python:** 3.8+ (virtual environment recommended).
*   **Node.js:** (Optional) Only for QRES Studio development.
*   **RL/ML Libs:** Stable Baselines3, Gymnasium (for MetaBrain training).

### Architecture
*   `qres_rust/`: The core engine. It compiles to a native library and a Python extension (`.pyd` / `.so`).
*   `python/`: The Python wrapper (mostly types and loading logic).
*   `ai/`: Training scripts for MetaBrain PPO agent.
*   `qres-studio/`: The Tauri/Svelte GUI.
*   `data/`: Sample datasets for testing/training.

## Building the Core & Python Bindings
```bash
maturin develop --release --manifest-path qres_rust/Cargo.toml
```

## Training MetaBrain
```bash
python ai/train_compression_ppo.py --data-dir data/  # Add custom data to data/
```

## Running Tests
```bash
cargo test --manifest-path qres_rust/Cargo.toml
pytest tests/
python verify_fidelity.py  # Fidelity checks
```

## Formatting & Linting
```bash
cargo fmt --all
cargo clippy
```

## Core Guidelines
1.  **Performance:** Critical paths allocation-free.
2.  **Safety:** Document `unsafe` blocks.
3.  **Telepathy:** New predictors must implement `Predictor` trait.
4.  **RL Contributions:** Ensure env observations match (e.g., 257-dim for v4).
5.  **Multimodal:** Test on `data/` folder; maintain >0.98 fidelity.
