import os

# Read the file
with open(r"c:\Dev\QRES\crates\qres_core\src\lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

# Define the unsafe block to replace (used in encode)
unsafe_block_encode = """    // FIXED: Strict Q16.16 loading
    let (init_w, global_w) = if let Some(w_bytes) = weights {
        let word_count = w_bytes.len() / 4;
        if word_count > 0 {
            let ptr = w_bytes.as_ptr() as *const i32;
            // SAFETY: Caller ensures alignment and byte length is valid for i32s.
            let slice = unsafe { core::slice::from_raw_parts(ptr, word_count) };

            if word_count >= 2 * NUM_MODELS {
                (
                    Some(&slice[0..NUM_MODELS]),
                    Some(&slice[NUM_MODELS..2 * NUM_MODELS]),
                )
            } else if word_count >= NUM_MODELS {
                (Some(&slice[0..NUM_MODELS]), None)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let mut mixer = Mixer::new(init_w, global_w);"""

# Define the unsafe block to replace (used in decode)
unsafe_block_decode = """    // FIXED: Strict Q16.16 loading
    let (init_w, global_w) = if let Some(w_bytes) = weights {
        let word_count = w_bytes.len() / 4;
        if word_count > 0 {
            let ptr = w_bytes.as_ptr() as *const i32; // <--- TYPE MIGRATION: i32
            let slice = unsafe { core::slice::from_raw_parts(ptr, word_count) };

            if word_count >= 2 * NUM_MODELS {
                (
                    Some(&slice[0..NUM_MODELS]),
                    Some(&slice[NUM_MODELS..2 * NUM_MODELS]),
                )
            } else if word_count >= NUM_MODELS {
                (Some(&slice[0..NUM_MODELS]), None)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let mut mixer = Mixer::new(init_w, global_w);"""

# Define the replacement (safe and deterministic)
safe_block = """    // FIXED: Safe and Deterministic Q16.16 loading
    let mut safe_weights_vec = Vec::new();
    if let Some(w_bytes) = weights {
        for chunk in w_bytes.chunks_exact(4) {
            safe_weights_vec.push(i32::from_le_bytes(chunk.try_into().unwrap()));
        }
    }

    let (init_w_slice, global_w_slice) = if !safe_weights_vec.is_empty() {
        let wc = safe_weights_vec.len();
        if wc >= 2 * NUM_MODELS {
            (
                Some(&safe_weights_vec[0..NUM_MODELS]),
                Some(&safe_weights_vec[NUM_MODELS..2 * NUM_MODELS]),
            )
        } else if wc >= NUM_MODELS {
            (Some(&safe_weights_vec[0..NUM_MODELS]), None)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let mut mixer = Mixer::new(init_w_slice, global_w_slice);"""

# Apply replacement (order matters or replace all occurrences?)
# The blocks are slightly different (comments), so we replace both individually.
# encode block
if unsafe_block_encode in content:
    content = content.replace(unsafe_block_encode, safe_block)
else:
    print("Could not find encode block")

# decode block
if unsafe_block_decode in content:
    content = content.replace(unsafe_block_decode, safe_block)
else:
    print("Could not find decode block")

# Write back
with open(r"c:\Dev\QRES\crates\qres_core\src\lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
