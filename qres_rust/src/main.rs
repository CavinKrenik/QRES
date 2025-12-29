use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use qres_rust::{QresWriter, QresReader};

fn compress_file(input: &str, output: &str, brain_path: Option<String>) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(input)?);
    let writer = BufWriter::new(File::create(output)?);
    
    // Config based on flags
    let (predictor_id, weights) = if let Some(path) = brain_path {
        println!("🧠 Neural Mode Activated! Loading Brain from {}...", path);
        let mut f = File::open(path)?;
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?; 
        // Expect "QNN1", but simplified training script might not match exactly if I changed it. 
        // Trainer wrote "QNN1". Good.
        // Detect Magic
        if &magic == b"QNN1" {
             let mut w = Vec::new();
             f.read_to_end(&mut w)?;
             (2, Some(w))
        } else if &magic == b"LSTM" {
             println!("🧠 LSTM Mode Detected!");
             let mut w = Vec::new();
             f.read_to_end(&mut w)?;
             (3, Some(w))
        } else {
             return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Brain file (Unknown Magic)"));
        }
    } else {
        (1, None) // Default Linear
    };

    let mut qres_writer = QresWriter::new(writer, predictor_id, weights);
    
    // Stream
    let start = std::time::Instant::now();
    let bytes = io::copy(&mut reader, &mut qres_writer)?;
    qres_writer.flush()?; 
    
    println!("Streamed {} bytes to {} (Predictor: {}) in {:.2}s", 
        bytes, output, predictor_id, start.elapsed().as_secs_f64());
    Ok(())
}

fn decompress_file(input: &str, output: &str) -> io::Result<()> {
    // Decoding is auto-configured via header
    let reader = BufReader::new(File::open(input)?);
    let mut writer = BufWriter::new(File::create(output)?);
    
    let mut qres_reader = QresReader::new(reader);
    let start = std::time::Instant::now();
    let bytes = io::copy(&mut qres_reader, &mut writer)?;
    
    println!("Restored {} bytes from {} in {:.2}s", bytes, input, start.elapsed().as_secs_f64());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 { 
        eprintln!("Usage: qres-cli <compress|decompress> <in> <out> [--brain <model.qnn>]");
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "compress" => {
            let brain = if args.len() > 5 && args[4] == "--brain" {
                Some(args[5].clone())
            } else {
                None
            };
            compress_file(&args[2], &args[3], brain).unwrap()
        },
        "decompress" => decompress_file(&args[2], &args[3]).unwrap(),
        _ => eprintln!("Unknown command"),
    }
}
