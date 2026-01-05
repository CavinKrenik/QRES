# QRES Python API Reference (v10.0)

## Core Class: `QRES`

The main entry point for compression and decompression.

```python
from qres import QRES, QRESError
import numpy as np
```

### `QRES.compress(data: Union[bytes, bytearray, np.ndarray], predictor_id: int = 0) -> bytes`

Compresses data using the QRES v10 engine (Bit-Packed + Delta).

- **data**: The input data. Supports `bytes`, `bytearray`, or `numpy.ndarray`.
- **predictor_id**: 
  - `0`: **Previous** (Best for constant data)
  - `1`: **Linear** (Best for sequences/timestamps)
  - `255`: **Auto-Detect** (Smart mode - slower but optimal)
- **Returns**: A `bytes` object containing the compressed stream.
- **Raises**: `QRESError` if compression fails.

### `QRES.decompress(data: Union[bytes, bytearray], predictor_id: int = 0) -> bytes`

Decompresses a QRES stream.

- **data**: The compressed bytes.
- **predictor_id**: Must match the ID used during compression (usually handled automatically by header).
- **Returns**: The original `bytes`.

---

## File I/O: `QRESFile`

A file-like object for reading and writing compressed files transparently.

### `qres.open(filename: str, mode: str = "rb") -> QRESFile`

Opens a QRES compressed file.

- **filename**: Path to the file.
- **mode**: File mode (`rb`, `wb`, etc.).

**Example:**
```python
import qres

# Write compressed
with qres.open("data.qres", "wb") as f:
    f.write(b"Hello Quantum World")

# Read compressed
with qres.open("data.qres", "rb") as f:
    content = f.read()
```

---

## Constants

- `VERSION`: `10.0.0`
- `QRES_MAGIC`: `b'QRES'`
