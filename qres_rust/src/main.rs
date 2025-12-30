use qres_rust::{compress_chunk, decompress_chunk, LivingBrain};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};

const DEFAULT_BRAIN_FILE: &str = "qres_brain.json";
const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

fn compress_file(input: &str, output: &str) -> io::Result<()> {
    let mut input_file = File::open(input)?;
    let mut output_file = File::create(output)?;
    
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_input = 0u64;
    let mut total_output = 0u64;
    let start = std::time::Instant::now();
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        let chunk = &buffer[..bytes_read];
        let compressed = compress_chunk(chunk, 0, None, None)?;
        
        // Write chunk size (4 bytes) + compressed data
        output_file.write_all(&(compressed.len() as u32).to_le_bytes())?;
        output_file.write_all(&compressed)?;
        
        total_input += bytes_read as u64;
        total_output += compressed.len() as u64 + 4;
        
        // Progress indicator
        if total_input % (1024 * 1024) == 0 {
            let ratio = (total_output as f64 / total_input as f64) * 100.0;
            eprint!("\rCompressed: {:.2} MB -> {:.2} MB ({:.1}%)", 
                   total_input as f64 / 1024.0 / 1024.0,
                   total_output as f64 / 1024.0 / 1024.0,
                   ratio);
        }
    }
    
    let elapsed = start.elapsed();
    let ratio = (total_output as f64 / total_input as f64) * 100.0;
    
    eprintln!("\n✓ Compressed {} bytes to {} bytes ({:.2}%) in {:.2}s", 
             total_input, total_output, ratio, elapsed.as_secs_f64());
    eprintln!("  Throughput: {:.2} MB/s", 
             (total_input as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64());
    
    Ok(())
}

fn decompress_file(input: &str, output: &str) -> io::Result<()> {
    let mut input_file = File::open(input)?;
    let mut output_file = File::create(output)?;
    
    let mut total_output = 0u64;
    let start = std::time::Instant::now();
    
    loop {
        // Read chunk size
        let mut size_buf = [0u8; 4];
        match input_file.read_exact(&mut size_buf) {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        
        let chunk_size = u32::from_le_bytes(size_buf) as usize;
        
        // Read compressed chunk
        let mut compressed = vec![0u8; chunk_size];
        input_file.read_exact(&mut compressed)?;
        
        // Decompress
        let decompressed = decompress_chunk(&compressed, 0, None)?;
        output_file.write_all(&decompressed)?;
        
        total_output += decompressed.len() as u64;
        
        // Progress indicator
        if total_output % (1024 * 1024) == 0 {
            eprint!("\rDecompressed: {:.2} MB", total_output as f64 / 1024.0 / 1024.0);
        }
    }
    
    let elapsed = start.elapsed();
    eprintln!("\n✓ Decompressed {} bytes in {:.2}s", total_output, elapsed.as_secs_f64());
    eprintln!("  Throughput: {:.2} MB/s", 
             (total_output as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64());
    
    Ok(())
}

fn brain_export() -> io::Result<()> {
    if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        println!("{}", json);
    } else {
        println!("{}", LivingBrain::new().to_json());
    }
    Ok(())
}

fn brain_import(file_path: &str) -> io::Result<()> {
    let mut local = if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        LivingBrain::from_json(&json).unwrap_or_else(|| LivingBrain::new())
    } else {
        LivingBrain::new()
    };
    
    let import_json = fs::read_to_string(file_path)?;
    if let Some(imported) = LivingBrain::from_json(&import_json) {
        local.merge(&imported, 0.1);
        fs::write(DEFAULT_BRAIN_FILE, local.to_json())?;
        eprintln!("🧠 Brain merged successfully. Wisdom assimilated.");
    } else {
        eprintln!("Failed to parse imported brain.");
    }
    Ok(())
}

fn print_usage() {
    eprintln!("QRES v3.0 - Adaptive Neural-Symbolic Compression");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  qres-cli compress <input> <output>");
    eprintln!("  qres-cli decompress <input> <output>");
    eprintln!("  qres-cli brain-export");
    eprintln!("  qres-cli brain-import <file>");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  qres-cli compress data.bin data.qres");
    eprintln!("  qres-cli decompress data.qres data.bin");
    eprintln!("  qres-cli brain-export > my_brain.json");
    eprintln!("  qres-cli brain-import peer_brain.json");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }
    
    let result = match args[1].as_str() {
        "compress" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: qres-cli compress <input> <output>");
                std::process::exit(1);
            }
            compress_file(&args[2], &args[3])
        }
        "decompress" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: qres-cli decompress <input> <output>");
                std::process::exit(1);
            }
            decompress_file(&args[2], &args[3])
        }
        "brain-export" => brain_export(),
        "brain-import" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file argument");
                eprintln!("Usage: qres-cli brain-import <file>");
                std::process::exit(1);
            }
            brain_import(&args[2])
        }
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_usage();
            std::process::exit(1);
        }
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
