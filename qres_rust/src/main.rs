use clap::{Parser, Subcommand};
use qres_rust::{compress_chunk, decompress_chunk, LivingBrain};
use std::fs::{self, File};
use std::io::{self, Read, Write};

const DEFAULT_BRAIN_FILE: &str = "qres_brain.json";
const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

#[derive(Parser)]
#[command(name = "qres-cli")]
#[command(about = "QRES v3.0.1 - Neural-Symbolic Meta-Compressor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a file
    Compress {
        /// Input file path
        input: String,
        /// Output file path
        output: String,
    },
    /// Decompress a file
    Decompress {
        /// Input file path
        input: String,
        /// Output file path
        output: String,
    },
    /// Export brain to JSON
    ExportBrain {
        /// Output JSON file path
        output: String,
    },
    /// Import brain from JSON
    ImportBrain {
        /// Input JSON file path
        input: String,
    },
    /// Run swarm node
    Swarm,
}

fn compress_file(input: &str, output: &str) -> io::Result<()> {
    let mut input_file = File::open(input)?;
    let mut output_file = File::create(output)?;

    // Load Living Brain for Initialization
    let brain = if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        LivingBrain::from_json(&json).unwrap_or_default()
    } else {
        LivingBrain::default()
    };

    // Prepare weights buffer (Init + Global)
    let mut w_bytes = Vec::with_capacity(80);
    // 1. Initial Weights
    for &f in &brain.confidence {
        w_bytes.extend_from_slice(&f.to_le_bytes());
    }
    // 2. Global Weights (FedProx)
    if let Some(g) = &brain.global_confidence {
        for &f in g {
            w_bytes.extend_from_slice(&f.to_le_bytes());
        }
    }
    let weights_arg = if w_bytes.is_empty() { None } else { Some(w_bytes.as_slice()) };

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
        let compressed = compress_chunk(chunk, 0, weights_arg, None)?;

        // Write chunk size (4 bytes) + compressed data
        output_file.write_all(&(compressed.len() as u32).to_le_bytes())?;
        output_file.write_all(&compressed)?;

        total_input += bytes_read as u64;
        total_output += compressed.len() as u64 + 4;

        // Progress indicator
        if total_input.is_multiple_of(1024 * 1024) {
            let ratio = (total_output as f64 / total_input as f64) * 100.0;
            eprint!(
                "\rCompressed: {:.2} MB -> {:.2} MB ({:.1}%)",
                total_input as f64 / 1024.0 / 1024.0,
                total_output as f64 / 1024.0 / 1024.0,
                ratio
            );
        }
    }

    let elapsed = start.elapsed();
    let ratio = (total_output as f64 / total_input as f64) * 100.0;

    eprintln!(
        "\n[Done] Compressed {} bytes to {} bytes ({:.2}%) in {:.2}s",
        total_input,
        total_output,
        ratio,
        elapsed.as_secs_f64()
    );
    eprintln!(
        "  Throughput: {:.2} MB/s",
        (total_input as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64()
    );

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
            Ok(_) => {}
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
        if total_output.is_multiple_of(1024 * 1024) {
            eprint!(
                "\rDecompressed: {:.2} MB",
                total_output as f64 / 1024.0 / 1024.0
            );
        }
    }

    let elapsed = start.elapsed();
    eprintln!(
        "\n[Done] Decompressed {} bytes in {:.2}s",
        total_output,
        elapsed.as_secs_f64()
    );
    eprintln!(
        "  Throughput: {:.2} MB/s",
        (total_output as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64()
    );

    Ok(())
}

fn brain_export_to_file(output: &str) -> io::Result<()> {
    let json = if let Ok(content) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        content
    } else {
        LivingBrain::new().to_json()
    };
    fs::write(output, json)?;
    eprintln!("[Brain] Brain exported to {}", output);
    Ok(())
}

fn brain_import(file_path: &str) -> io::Result<()> {
    let mut local = if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        LivingBrain::from_json(&json).unwrap_or_default()
    } else {
        LivingBrain::new()
    };

    let import_json = fs::read_to_string(file_path)?;
    if let Some(imported) = LivingBrain::from_json(&import_json) {
        // V4: Hive Sync (Python) handles the merging logic (FedProx).
        // CLI just applies the result (Overwrite confidence, keep stats).
        local.merge(&imported, 1.0);
        fs::write(DEFAULT_BRAIN_FILE, local.to_json())?;
        eprintln!("[Brain] Brain merged successfully. Wisdom assimilated.");
    } else {
        eprintln!("Failed to parse imported brain.");
    }
    Ok(())
}

fn swarm_mode() -> io::Result<()> {
    eprintln!("[Swarm] Starting QRES Swarm Node...");
    // For now, just a placeholder - actual swarm logic in daemon.rs
    eprintln!("Swarm mode not implemented in CLI yet. Use daemon.");
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Compress { input, output } => compress_file(&input, &output),
        Commands::Decompress { input, output } => decompress_file(&input, &output),
        Commands::ExportBrain { output } => brain_export_to_file(&output),
        Commands::ImportBrain { input } => brain_import(&input),
        Commands::Swarm => swarm_mode(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
