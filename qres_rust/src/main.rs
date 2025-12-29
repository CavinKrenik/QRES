use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use qres_rust::{QresWriter, QresReader};
use serde_json;

fn compress_file(input: &str, output: &str, mode_hint: u8, anomaly_threshold: Option<u8>) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(input)?);
    let writer = BufWriter::new(File::create(output)?);
    
    // QresWriter handles detection internally now
    let mut qres_writer = QresWriter::new(writer, mode_hint);
    if let Some(t) = anomaly_threshold {
        qres_writer.set_anomaly_threshold(t);
    }
    
    // Stream
    let start = std::time::Instant::now();
    let bytes = io::copy(&mut reader, &mut qres_writer)?;
    qres_writer.flush()?; 
    
    println!("Streamed {} bytes to {} (Mode: {}) in {:.2}s", 
        bytes, output, mode_hint, start.elapsed().as_secs_f64());
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
        eprintln!("Usage: qres-cli <compress|decompress> <in> <out> [--mode <auto|max|fast>] [--detect-anomalies <threshold>]");
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "compress" => {
            // Parse optional flags
            let mut mode = 0;
            let mut anomaly_threshold = None;
            
            let mut i = 4;
            while i < args.len() {
                match args[i].as_str() {
                    "--mode" => {
                        if i + 1 < args.len() {
                            mode = match args[i+1].as_str() {
                                "max" => 3,
                                "fast" => 1,
                                _ => 0,
                            };
                            i += 2;
                        } else { i += 1; }
                    },
                    "--detect-anomalies" => {
                         if i + 1 < args.len() {
                             if let Ok(t) = args[i+1].parse::<u8>() {
                                 anomaly_threshold = Some(t);
                             }
                             i += 2;
                         } else { i += 1; }
                    },
                    _ => i += 1,
                }
            }
            compress_file(&args[2], &args[3], mode, anomaly_threshold).unwrap()
        },
        "decompress" => decompress_file(&args[2], &args[3]).unwrap(),
        _ => eprintln!("Unknown command"),
    }
}
