# QRES v4 - Extended with .qres Binary Format

### New: .qres Format
You can now save compressed output to a binary `.qres` file with headers and compression.

## Format Structure:
- `MAGIC` bytes: `QRES`
- `Header Length`: 4 bytes
- `Header`: JSON (type, version, start value, timestamp, metadata)
- `Body`: zlib-compressed transition string

## Usage Example

```python
from qres_core import qres_encode, qres_decode
from qres_file import save_qres_file, load_qres_file

# Encode and save
seq = [100, 100, 105, 90]
start, transitions = qres_encode(seq)
save_qres_file("numbers", start, transitions, "example.qres")

# Load and decode
header, encoded = load_qres_file("example.qres")
decoded_seq = qres_decode(header["start"], encoded)
```# QRES
