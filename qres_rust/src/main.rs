use std::env;
use qres_rust::{compress_file, decompress_file};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage:\n  qres_rust compress <input> <output>\n  qres_rust decompress <input> <output>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "compress" => compress_file(&args[2], &args[3]).expect("Compression failed"),
        "decompress" => decompress_file(&args[2], &args[3]).expect("Decompression failed"),
        _ => eprintln!("Unknown command: {}", args[1]),
    }
}
