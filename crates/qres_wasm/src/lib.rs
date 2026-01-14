use qres_core::{compress_chunk, decompress_chunk};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_hooks() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn compress_bytes(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Bridges JS Uint8Array <-> Rust Vec<u8>
    // Using default args: predictor_id=0, weights=None, lossy=None
    // WASM: Allocate buffer.
    let capacity = data.len() + 4096;
    let mut buffer = vec![0u8; capacity];

    let len = compress_chunk(data, 0, None, None, &mut buffer)
        .map_err(|e| JsValue::from_str(&format!("Compression failed: {:?}", e)))?;

    Ok(buffer[..len].to_vec())
}

#[wasm_bindgen]
pub fn decompress_bytes(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Using default args: predictor_id=0, weights=None
    decompress_chunk(data, 0, None)
        .map_err(|e| JsValue::from_str(&format!("Decompression failed: {:?}", e)))
}
