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

```bash
# Python environment
python -m venv .venv
source .venv/bin/activate  # or .venv\Scripts\activate on Windows
pip install -e .

# Rust core
cd qres_rust
cargo build --release

# Run tests
pytest tests/
cargo test
```

---

## Areas of Interest

We welcome contributions in these areas:

| Area | Description |
|------|-------------|
| 🧠 **SNN Optimization** | Improve sparsity and efficiency |
| ⚛️ **QNN Circuits** | Better variational designs |
| 🐝 **P2P Swarm** | Scalability and reliability |
| 📊 **Benchmarks** | New datasets and metrics |
| 📝 **Documentation** | Tutorials and examples |

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
