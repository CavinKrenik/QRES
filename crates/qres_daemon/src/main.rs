pub mod analytics;
pub mod api;
pub mod brain_aggregator;
pub mod config;
pub mod daemon;
pub mod dreaming;
pub mod living_brain;
pub mod peer_keys;
pub mod qes;
pub mod security;
pub mod stats;
pub mod swarm;
pub mod swarm_p2p;

use crate::living_brain::LivingBrain;
use clap::{Parser, Subcommand};
use qres_core::tensor::MpsCompressor;
use qres_core::{
    compress_chunk, config::QresConfig, decompress_chunk_with_state, PredictorSet, QresError,
};
// use qres_core::QresError;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use tracing::{error, info};

const DEFAULT_BRAIN_FILE: &str = "qres_brain.json";
const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

#[derive(Parser)]
#[command(name = "qres-cli")]
#[command(about = "QRES v10.0 - Neural-Symbolic Meta-Compressor")]
struct Cli {
    #[command(flatten)]
    config: QresConfig,

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
    Swarm {
        /// Path to brain file
        #[arg(long, default_value = "qres_brain.json")]
        brain: String,
        /// API Port
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Path to node private key
        #[arg(long)]
        key: Option<String>,
    },
    /// Compress structured data using Tensor MPS
    TensorCompress {
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

fn compress_file(input: &str, output: &str, config: &QresConfig) -> io::Result<()> {
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
        // Allocate buffer (worst case estimate)
        let mut comp_buffer = vec![0u8; chunk.len() + 4096];
        let compressed_result =
            compress_chunk(chunk, 0, weights_arg, Some(config), &mut comp_buffer);

        let compressed = match compressed_result {
            Ok(len) => comp_buffer[..len].to_vec(),
            Err(QresError::CompressionError(_)) => {
                // Core failed (expansion). Use Zstd fallback.
                let zstd_data = zstd::bulk::compress(chunk, 3)?;
                let ver = 0x0A;
                let flag_byte = (ver << 4) | 0x01;

                let mut out = Vec::with_capacity(5 + zstd_data.len());
                out.push(flag_byte);
                out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
                out.extend_from_slice(&zstd_data);
                out
            }
            Err(e) => return Err(io::Error::other(e.to_string())),
        };

        // Write chunk size (4 bytes) + compressed data
        output_file.write_all(&(compressed.len() as u32).to_le_bytes())?;
        output_file.write_all(&compressed)?;

        total_input += bytes_read as u64;
        total_output += compressed.len() as u64 + 4;

        // Progress indicator
        if total_input >= 1024 * 1024 && total_input.is_multiple_of(1024 * 1024) {
            let ratio = (total_output as f64 / total_input as f64) * 100.0;
            info!(
                current_input_mb = total_input as f64 / 1024.0 / 1024.0,
                current_output_mb = total_output as f64 / 1024.0 / 1024.0,
                ratio_percent = ratio,
                "Compressing..."
            );
        }
    }

    let elapsed = start.elapsed();
    let ratio = if total_input > 0 {
        (total_output as f64 / total_input as f64) * 100.0
    } else {
        0.0
    };

    info!(
        total_input_bytes = total_input,
        total_output_bytes = total_output,
        ratio_percent = ratio,
        duration_secs = elapsed.as_secs_f64(),
        throughput_mb_s = if elapsed.as_secs_f64() > 0.0 {
            (total_input as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64()
        } else {
            0.0
        },
        "Compression Complete"
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

    // OPTIMIZATION: Allocate PredictorSet ONCE (saves ~22MB allocation per chunk)
    // The PredictorSet is reset internally by decompress_chunk_with_state before each use
    let mut predictor_state = PredictorSet::new(None, None);

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

        // Decompress using reusable predictor state (eliminates ~22MB alloc/dealloc per chunk)
        let result = decompress_chunk_with_state(&compressed, 0, weights_arg, &mut predictor_state);

        let decompressed = match result {
            Ok(d) => d,
            Err(QresError::CompressionError(s)) if s.contains("Zstd") => {
                // Fallback for Zstd chunks (0x01) which Core rejected
                // We need to parse the header manually to extract Zstd payload
                // Offsets: [Header:1][UncompressedLen:4][Payload...]
                if compressed.len() < 5 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Chunk too short",
                    ));
                }
                let decomp_len = u32::from_le_bytes(compressed[1..5].try_into().unwrap()) as usize;
                let payload = &compressed[5..];
                zstd::bulk::decompress(payload, decomp_len)?
            }
            Err(e) => return Err(io::Error::other(e.to_string())),
        };

        output_file.write_all(&decompressed)?;

        total_output += decompressed.len() as u64;

        // Progress indicator
        if total_output >= 1024 * 1024 && total_output.is_multiple_of(1024 * 1024) {
            info!(
                decompressed_mb = total_output as f64 / 1024.0 / 1024.0,
                "Decompressing..."
            );
        }
    }

    let elapsed = start.elapsed();
    info!(
        total_output_bytes = total_output,
        duration_secs = elapsed.as_secs_f64(),
        throughput_mb_s = if elapsed.as_secs_f64() > 0.0 {
            (total_output as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64()
        } else {
            0.0
        },
        "Decompression Complete"
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
    info!(output_path = output, "Brain exported to file");
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
        info!("Brain merged successfully. Wisdom assimilated.");
    } else {
        error!("Failed to parse imported brain.");
    }
    Ok(())
}

fn swarm_mode(brain: String, port: u16, key_path: Option<String>) -> io::Result<()> {
    info!(
        brain_file = brain,
        port = port,
        key_path = ?key_path,
        "Starting QRES P2P Swarm Node (libp2p)..."
    );

    // Create Tokio Runtime for async swarm
    let rt = tokio::runtime::Runtime::new().map_err(io::Error::other)?;

    rt.block_on(async {
        if let Err(e) = crate::swarm_p2p::start_p2p_node(brain, port, key_path).await {
            error!(error = %e, "Swarm crashed");
        }
    });

    Ok(())
}

fn compress_tensor_file(
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
    info!(rows = rows, cols = cols, "Compressing Tensor Matrix...");
    let start = std::time::Instant::now();

    let compressor = MpsCompressor::new(10, threshold);
    let chunks = compressor.compress_matrix(&floats, rows, cols);

    // Write Output
    // Format: [Magic: TMPS] [Rows:8] [Cols:8] [Data...]
    let mut out_file = File::create(output)?;
    out_file.write_all(b"TMPS")?;
    out_file.write_all(&(rows as u64).to_le_bytes())?;
    out_file.write_all(&(cols as u64).to_le_bytes())?;

    let mut compressed_size = 20; // Header

    if let Some(data) = chunks.first() {
        // Simple Sparse Serialization: [Index:4, Value:8] or just [Value:8] if dense?
        // My MpsCompressor returns a dense vector with 0.0s for pruned values.
        // We should RLE or Sparse-Pack it here to realize gains.

        let mut n_zeros = 0;
        let mut packed_bytes = Vec::new();

        // Simple RLE for Zero Runs (0x00 flag)
        for val in data {
            let val: f64 = *val;
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
                packed_bytes.push(0x01); // Value flag
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
    let ratio = if len > 0 {
        (compressed_size as f64 / len as f64) * 100.0
    } else {
        0.0
    };

    info!(
        original_bytes = len,
        compressed_bytes = compressed_size,
        ratio_percent = ratio,
        duration_secs = elapsed.as_secs_f64(),
        "Tensor Compression Complete"
    );

    Ok(())
}

fn main() {
    // Initialize structured logging
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();
    println!(
        "QRES v18.0 | Predictor: {:?} | Coder: {:?}",
        cli.config.predictor, cli.config.coder
    );

    info!(
        config = ?cli.config,
        "Starting QRES with Configuration"
    );

    let result = match cli.command {
        Commands::Compress { input, output } => compress_file(&input, &output, &cli.config),
        Commands::Decompress { input, output } => decompress_file(&input, &output),
        Commands::ExportBrain { output } => brain_export_to_file(&output),
        Commands::ImportBrain { input } => brain_import(&input),
        Commands::Swarm { brain, port, key } => swarm_mode(brain, port, key),
        Commands::TensorCompress {
            input,
            output,
            rows,
            cols,
            threshold,
        } => compress_tensor_file(&input, &output, rows, cols, threshold),
    };

    if let Err(e) = result {
        error!(error = %e, "Fatal Error");
        std::process::exit(1);
    }
}
