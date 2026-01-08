# QRES API Reference (v10.5)

## 🏗️ Rust Core API (`qres_core`)

The primary interface for QRES is the Rust crate. It is `no_std` compatible and powers all other bindings.

### `compress`
```rust
pub fn compress(data: &[u8], config: CompressionConfig) -> Result<Vec<u8>, QresError>
```
Compresses a byte slice with deterministic behavior.

- **data**: Input byte slice.
- **config**: Struct containing `mode` (Standard/Tensor), `threshold` (0.0-1.0), and `window_size`.
- **Returns**: `Vec<u8>` containing the compressed bitstream.

### `decompress`
```rust
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, QresError>
```
Decompresses a QRES stream.

---

## 🐍 Python Bindings (`qres_core`)

The Python API wraps the Rust core for high-performance usage in scripts and ML pipelines.

### `qres_core.compress(data: bytes, mode: str = "standard") -> bytes`
- **data**: Bytes to compress.
- **mode**: `"standard"` (Linear/LZ77) or `"tensor"` (Tensor/SNN).

### `qres_core.decompress(data: bytes) -> bytes`
- **data**: Compressed QRES bytes.

---

## 📦 WASM / JavaScript API

The WebAssembly target allows running QRES in the browser.

### `compress_wasm(data: Uint8Array) -> Uint8Array`
Sync compression running on the main thread (or worker).

### `decompress_wasm(data: Uint8Array) -> Uint8Array`
Sync decompression.

---

## 📄 File I/O (Daemon/CLI)

For file operations, use the `qres_daemon` CLI or the `QRESFile` wrapper (if using the legacy Python helper).

### `qres_daemon`
```bash
qres_daemon compress <INPUT> [OUTPUT]
qres_daemon decompress <INPUT> [OUTPUT]
```
