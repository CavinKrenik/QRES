# Contributing to QRES

We welcome contributions to the Singularity! Since QRES is a hybrid Rust/Python system, there are a few things to know about the build process.

## 🛠️ Development Setup

### Prerequisites
- **Rust:** Stable toolchain (install via `rustup`).
- **Python:** 3.8+ (virtual environment recommended).
- **Node.js:** (Optional) Only for QRES Studio development.

### Architecture
1.  **`qres_rust/`**: The core engine. It compiles to a native library and a Python extension (`.pyd`/`.so`).
2.  **`python/`**: The Python wrapper (mostly types and loading logic).
3.  **`qres-studio/`**: The Tauri/Svelte GUI.

### Building the Core & Python Bindings

We use `maturin` to build the Rust core and install it into your Python environment.

```bash
# 1. Activate your venv
source .venv/bin/activate  # or .venv\Scripts\activate

# 2. Build and Install (Development Mode)
maturin develop --release --manifest-path qres_rust/Cargo.toml
```

### Running Tests

```bash
# Rust Unit Tests
cargo test --manifest-path qres_rust/Cargo.toml

# Python Integration Tests
pytest tests/
```

### Formatting & Linting
Please ensure checks pass before submitting a PR.
```bash
cargo fmt --all
cargo clippy
```

## 🧠 Core Guidelines
- **Performance:** Critical paths (predict, update) must be allocation-free. Use `lazy_static` or pre-allocated buffers.
- **Safety:** Use `unsafe` only when absolutely necessary for SIMD or raw pointer hacks, and document WHY.
- **Telepathy:** If adding a new predictor, implementing the `Predictor` trait is mandatory.
