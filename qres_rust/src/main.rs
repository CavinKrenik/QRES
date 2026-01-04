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
    #[cfg(feature = "swarm")]
    Swarm {
        /// Path to brain file
        #[arg(long, default_value = "qres_brain.json")]
        brain: String,
        /// API Port
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Compress structured data using Quantum MPS
    QuantumCompress {
        /// Input file path (Raw f64 binary)
        input: String,
        /// Output file path
        output: String,
        /// Matrix rows
        #[arg(long)]
        rows: usize,
        /// Matrix cols
        #[arg(long)]
        cols: usize,
        /// Approximation Threshold
        #[arg(long, default_value = "1.0")]
        threshold: f64,
    },
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
    let weights_arg = if w_bytes.is_empty() {
        None
    } else {
        Some(w_bytes.as_slice())
    };

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

    // Load Living Brain for Initialization (Must match Encoder!)
    let brain = if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        LivingBrain::from_json(&json).unwrap_or_default()
    } else {
        LivingBrain::default()
    };

    // Prepare weights buffer
    let mut w_bytes = Vec::with_capacity(80);
    for &f in &brain.confidence {
        w_bytes.extend_from_slice(&f.to_le_bytes());
    }
    if let Some(g) = &brain.global_confidence {
        for &f in g {
            w_bytes.extend_from_slice(&f.to_le_bytes());
        }
    }
    let weights_arg = if w_bytes.is_empty() {
        None
    } else {
        Some(w_bytes.as_slice())
    };

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
        let decompressed = decompress_chunk(&compressed, 0, weights_arg)?;
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

#[cfg(feature = "swarm")]
fn swarm_mode(brain: String, port: u16) -> io::Result<()> {
    eprintln!("[Swarm] Starting QRES P2P Swarm Node (libp2p)...");
    eprintln!("[Swarm] Brain File: {}", brain);
    eprintln!("[Swarm] API Port: {}", port);

    // Create Tokio Runtime for async swarm
    let rt = tokio::runtime::Runtime::new().map_err(io::Error::other)?;

    rt.block_on(async {
        if let Err(e) = qres_rust::swarm_p2p::start_p2p_node(brain, port).await {
            eprintln!("Swarm crashed: {}", e);
        }
    });

    Ok(())
}

use qres_rust::quantum::MpsCompressor;

// ...

fn compress_quantum_file(
    input: &str,
    output: &str,
    rows: usize,
    cols: usize,
    threshold: f64,
) -> io::Result<()> {
    let mut file = File::open(input)?;
    let metadata = file.metadata()?;
    let len = metadata.len();

    // Validate size (must be rows*cols*8)
    if len != (rows * cols * 8) as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "File size {} does not match rows*cols*8 ({})",
                len,
                rows * cols * 8
            ),
        ));
    }

    // Read all data as f64 (unsafe/transmute for speed, or byte-by-byte conversion)
    // For simplicity: Read bytes, convert to f64 safely.
    let mut buffer = Vec::with_capacity(len as usize);
    file.read_to_end(&mut buffer)?;

    // Convert to Vec<f64>
    // Assuming Little Endian (standard)
    let mut floats = Vec::with_capacity(rows * cols);
    for chunk in buffer.chunks_exact(8) {
        let val = f64::from_le_bytes(chunk.try_into().unwrap());
        floats.push(val);
    }

    // Compress
    eprintln!("[Quantum] Compressing {}x{} Matrix...", rows, cols);
    let start = std::time::Instant::now();

    let compressor = MpsCompressor::new(10, threshold);
    let chunks = compressor.compress_matrix(&floats, rows, cols);

    // Write Output
    // Format: [Magic: QMPS] [Rows:8] [Cols:8] [Data...]
    let mut out_file = File::create(output)?;
    out_file.write_all(b"QMPS")?;
    out_file.write_all(&(rows as u64).to_le_bytes())?;
    out_file.write_all(&(cols as u64).to_le_bytes())?;

    let mut compressed_size = 20; // Header

    if let Some(data) = chunks.first() {
        // Simple Sparse Serialization: [Index:4, Value:8] or just [Value:8] if dense?
        // My MpsCompressor returns a dense vector with 0.0s for pruned values.
        // We should RLE or Sparse-Pack it here to realize gains.

        let mut n_zeros = 0;
        let mut packed_bytes = Vec::new();

        // Simple RLE for Zero Runs
        // Flag byte: 0x00 = Run of Zeros (Next byte = count), 0x01 = Value follows (8 bytes)
        // Optimization: Just count non-zeros for the "Breakthrough" metrics?
        // No, user wants usable CLI. Let's do a trivial packing.

        for &val in data {
            if val.abs() < 1e-9 {
                n_zeros += 1;
                while n_zeros >= 255 {
                    packed_bytes.push(0x00);
                    packed_bytes.push(255);
                    n_zeros -= 255;
                }
            } else {
                if n_zeros > 0 {
                    packed_bytes.push(0x00);
                    packed_bytes.push(n_zeros as u8);
                    n_zeros = 0;
                }
                packed_bytes.push(0x01);
                packed_bytes.extend_from_slice(&val.to_le_bytes());
            }
        }
        // Flush zeros
        if n_zeros > 0 {
            packed_bytes.push(0x00);
            packed_bytes.push(n_zeros as u8);
        }

        out_file.write_all(&packed_bytes)?;
        compressed_size += packed_bytes.len();
    }

    let elapsed = start.elapsed();
    let ratio = (compressed_size as f64 / len as f64) * 100.0;

    eprintln!(
        "[Done] Quantum Compressed {} bytes to {} bytes ({:.2}%) in {:.2}s",
        len,
        compressed_size,
        ratio,
        elapsed.as_secs_f64()
    );

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Compress { input, output } => compress_file(&input, &output),
        Commands::Decompress { input, output } => decompress_file(&input, &output),
        Commands::ExportBrain { output } => brain_export_to_file(&output),
        Commands::ImportBrain { input } => brain_import(&input),
        #[cfg(feature = "swarm")]
        Commands::Swarm { brain, port } => swarm_mode(brain, port),
        Commands::QuantumCompress {
            input,
            output,
            rows,
            cols,
            threshold,
        } => compress_quantum_file(&input, &output, rows, cols, threshold),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
