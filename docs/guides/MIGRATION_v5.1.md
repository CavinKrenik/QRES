# Migration Guide: v4.x to v5.1

## Overview
QRES v5.1 introduces a fundamental shift from simple stream compression to a comprehensive archival system with deduplication. This guide covers the necessary changes for CLI users and library integrators.

## CLI Changes

### 1. Archiving Folders
**Old (v4):**
```bash
qres create ./my_folder output.qres
```

**New (v5.1):**
```bash
qres archive --dir ./my_folder --out output.qrar
```
*Note: The output extension is now `.qrar` for archives.*

### 2. Single File Compression
**Old (v4):**
```bash
qres encode file.txt file.qres
```

**New (v5.1):**
```bash
qres compress file.txt --out file.qres
```

## Python API Changes

### 1. Header Handling
The `encode_bytes` function now internally manages the QRES header (Magic, Version, Flags). You no longer need to manually prepend lengths or flags.

```python
import qres

# v5.1 - Safe and Simple
compressed = qres.encode_bytes(b"data") 
decompressed = qres.decode_bytes(compressed)
```

### 2. Training
The `train_model` function now accepts a `chunk_size` parameter to tune the "Living Brain's" attention span.

```python
# Train on a specific file pattern
weights = qres.train_model("logs/*.log", chunk_size=65536)
```

## Archive Format Internals
If you are writing a custom decoder:
- **Magic Bytes:** `QRAR` (Archives) vs `QRES` (Streams).
- **Chunking:** Archives now use variable-sized chunks determined by a rolling hash (CDC), rather than fixed 64KB blocks.
- **References:** Be prepared to handle Chunk Flag `0x03`, which indicates a reference to a previously decoded chunk hash. You must maintain a hash map of decoded chunks to resolve these references.
