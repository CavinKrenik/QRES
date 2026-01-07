# Contributing to QRES

Thank you for your interest in contributing to the Quantum-Relational Encoding System!

---

## Getting Started

1. **Fork** the repository
2. **Clone** your fork: `git clone https://github.com/YOUR_USERNAME/QRES.git`
3. **Create a branch**: `git checkout -b feature/your-feature`
4. **Make changes** and test
5. **Submit a Pull Request**

---

## Development Setup

# Rust Workspace (Core + Daemon)
cd qres_rust
cargo build --release --workspace

# Run tests
cargo test --workspace

# Python Bindings (Maturin)
pip install maturin
maturin develop
```

---

## Areas of Interest (v11)

We welcome contributions in these areas:

| Area | Description | Priority |
|------|-------------|----------|
| 🚀 **Portable SIMD** | ✅ Migrate `mixer.rs` to `wide` crate for ARM/WASM | **Done** |
| 🧠 **SNN Optimization** | Improve sparsity and efficiency | Medium |
| ⚛️ **FPGA Acceleration** | Verilog for Mixer offload | **High** |
| 🐝 **P2P Swarm** | Scalability and reliability | Medium |
| 📊 **Benchmarks** | New datasets and metrics | Low |
| 📝 **Documentation** | Tutorials and examples | Low |

---

## Data Contribution Guidelines (v11.1 NEW)

Help us improve QRES benchmarks by contributing test datasets!

### How to Contribute Data
1. Use `benchmarks/generate_diverse_iot.py` as a template
2. Create datasets with varied patterns (trends, anomalies, correlations)
3. Target 10-20MB file sizes
4. Run QRES compression and document ratios
5. Submit via PR to `data/community/`

### Dataset Naming Convention
- `<type>_<pattern>_<size>.dat`
- Example: `iot_sinusoidal_15mb.dat`

---

## Code Style

- **Python**: Follow PEP 8, use `ruff` for linting
- **Rust**: Run `cargo fmt` and `cargo clippy`
- **Markdown**: Use consistent formatting

---

## Commit Messages

Use conventional commits:
```
feat: Add new feature
fix: Fix a bug
docs: Update documentation
test: Add tests
chore: Maintenance
```

---

## Questions?

Open an issue or start a discussion. We're happy to help!

---

*See [ROADMAP.md](ROADMAP.md) for current priorities.*
