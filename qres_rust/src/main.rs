use qres_rust::{compress_chunk, decompress_chunk};
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <compress|decompress> <input> <output>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    let data = fs::read(input_path)?;

    match mode.as_str() {
        "compress" => {
            // Use Predictor ID 0 (Standard) and no external weights for the test
            let compressed = compress_chunk(&data, 0, None, None)?;
            fs::write(output_path, compressed)?;
            println!(
                "Compressed {} bytes -> {} bytes",
                data.len(),
                fs::metadata(output_path)?.len()
            );
        }
        "decompress" => {
            let restored = decompress_chunk(&data, 0, None)?;
            fs::write(output_path, restored)?;
            println!("Restored {} bytes", fs::metadata(output_path)?.len());
        }
        _ => {
            eprintln!("Unknown mode: {}", mode);
            std::process::exit(1);
        }
    }

    Ok(())
}
