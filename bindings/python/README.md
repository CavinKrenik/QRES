# QRES: Python Bindings

> **Produce. Predict. Preserve.**

**QRES** is a distributed system that solves the "Bandwidth vs. Privacy" conflict in Edge IoT. This package provides high-performance Python bindings to the `qres_core` Rust engine, enabling deterministic data consensus and adaptive compression for research and experimentation.

## Installation

```bash
pip install qres
```

## Usage

```python
from qres import qres_rust
import numpy as np

# Deterministic Compression
data = np.random.rand(100).astype(np.float32)
compressed = qres_rust.compress_adaptive(data)

print(f"Compressed: {len(compressed)} bytes")

# Decompression
recovered = qres_rust.decompress_adaptive(compressed)
assert np.allclose(data, recovered)
```

## Features

*   **Deterministic Math:** Q16.16 Fixed-Point arithmetic for bit-perfect reproducibility.
*   **Adaptive Switch:** Automatically chooses between Neural prediction and Bit-Packing.
*   **Zero-Copy:** Efficient data transfer between Python and Rust.

## License

Apache 2.0
