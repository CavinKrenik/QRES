# Contributing to QRES

Thank you for your interest in contributing to QRES! This document provides guidelines for contributors.

## Code of Conduct

Please be respectful and constructive in all interactions.

## How to Contribute

1. **Fork the repository** on GitHub
2. **Create a feature branch** from `main`
3. **Make your changes** following the guidelines below
4. **Test thoroughly** - run `cargo test`, `cargo clippy`, and benchmarks
5. **Submit a pull request** with a clear description

## Development Setup

```bash
git clone https://github.com/CavinKrenik/QRES.git
cd QRES/qres_rust
cargo build --release
```

## Coding Standards

- **Rust**: Follow standard Rust conventions
- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Ensure `cargo clippy` passes with no warnings
- **Testing**: Add tests for new functionality
- **Documentation**: Update docs for public APIs

## Commit Messages

Use clear, descriptive commit messages:

```
feat: add order-1 context to SimplePredictor
fix: resolve clippy warnings in mixer.rs
docs: update README with v3.0.1 features
```

## Areas for Contribution

- **Predictor Improvements**: Enhance neural models for better compression
- **Performance Optimization**: Speed up encoding/decoding
- **Swarm Intelligence**: Improve peer-to-peer learning
- **GUI Enhancements**: Extend qres-studio features
- **Documentation**: Improve docs and tutorials
- **Testing**: Add more comprehensive tests

## Reporting Issues

- Use GitHub Issues for bugs and feature requests
- Provide clear reproduction steps
- Include system information and QRES version

## License

By contributing, you agree to license your work under the same terms as the project (MIT OR Apache-2.0).