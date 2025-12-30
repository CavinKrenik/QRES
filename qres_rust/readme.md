# QRES v3.0.1 - Neural-Symbolic Meta-Compressor

QRES is a cutting-edge lossless data compression tool that combines adaptive ANS (Asymmetric Numeral Systems) encoding with intelligent zstd fallback, iPEPS predictors, LivingBrain swarm intelligence, and Python bindings.

## 🚀 Key Features

- **Adaptive ANS Encoding**: Real-time distribution tracking using Welford's online statistics
- **Intelligent Fallback**: Automatic zstd compression for incompressible data
- **Neural Predictors**: iPEPS tensor networks, LSTM, and simple order-1 context models
- **Swarm Intelligence**: LivingBrain for distributed weight sharing and evolution
- **High Performance**: 10 MB/s compression, 7 MB/s decompression
- **Python Bindings**: Seamless integration with Python ecosystem
- **CLI Tool**: Chunk-based streaming with progress indicators

## 📊 Performance Benchmarks

| Dataset | Ratio | Throughput | Notes |
|---------|-------|------------|-------|
| Repetitive Text | 90.5% | 10 MB/s | Adaptive ANS excels |
| Sine Waves | 85.2% | 10 MB/s | Neural predictors shine |
| Constant Data | 77.7% | 10 MB/s | Near-optimal compression |
| Random Data | 101.5% | 10 MB/s | Zstd fallback prevents expansion |

## 🛠️ Installation

### From Source
```bash
git clone https://github.com/CavinKrenik/QRES.git
cd QRES/qres_rust
cargo build --release
```

### Python Package
```bash
pip install qres-rust
```

## 📖 Usage

### CLI
```bash
# Compress
./target/release/qres-cli compress input.bin output.qres

# Decompress
./target/release/qres-cli decompress output.qres restored.bin

# Swarm Mode
./target/release/qres-cli swarm
```

### Python
```python
import qres_rust

# Compress bytes
compressed = qres_rust.encode_bytes(data)

# Decompress
original = qres_rust.decode_bytes(compressed)
```

## 🧠 Swarm Intelligence

QRES features a distributed LivingBrain that evolves compression strategies:

- **Peer-to-Peer Learning**: Nodes share predictor weights via gossip protocol
- **Adaptive Evolution**: Brains merge and mutate based on performance
- **Persistent State**: Brains saved as JSON for continuity

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## 📄 License

MIT OR Apache-2.0