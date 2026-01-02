#![no_main]
use libfuzzer_sys::fuzz_target;
use qres_rust::decompress_chunk;

fuzz_target!(|data: &[u8]| {
    // Attempt to decompress arbitrary garbage
    // The goal is to ensure this NEVER panics, only returns Ok or Err.
    let _ = decompress_chunk(data, 0, None);
});
