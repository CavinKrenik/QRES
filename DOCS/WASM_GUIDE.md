# 🕸️ QRES WASM Build Guide

QRES Core can be compiled to WebAssembly to run in the browser without a server.

## Prerequisites
- Rust 1.80+
- `wasm-pack`: `cargo install wasm-pack`

## Build Instructions
1. Navigate to `qres_rust`:
   ```bash
   cd qres_rust
   ```
2. Build with `wasm-pack`:
   ```bash
   wasm-pack build --target web --no-default-features
   ```
   *Note: We disable default features (swarms/tokio) because browser doesn't support raw TCP sockets.*

## Usage in JS
```javascript
import init, { compress, decompress } from './pkg/qres_rust.js';

async function run() {
    await init();
    
    const data = new Uint8Array([1, 2, 3, 4, 1, 2, 3, 4]); // ...
    const compressed = compress(data);
    const restored = decompress(compressed);
    
    console.log("Compressed Size:", compressed.length);
}
run();
```
