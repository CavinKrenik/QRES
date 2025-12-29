use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use qres_rust::{QresWriter, QresReader};
use serde_json;

fn compress_file(input: &str, output: &str, mode_hint: u8, report_path: Option<String>) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(input)?);
    let writer = BufWriter::new(File::create(output)?);
    
    // QresWriter handles detection internally now
    let mut qres_writer = QresWriter::new(writer, mode_hint);
    
    // Stream
    let start = std::time::Instant::now();
    let bytes = io::copy(&mut reader, &mut qres_writer)?;
    qres_writer.flush()?; 
    
    println!("Streamed {} bytes to {} (Mode: {}) in {:.2}s", 
        bytes, output, mode_hint, start.elapsed().as_secs_f64());
        
    // Dump Report
    if let Some(path) = report_path {
        if let Some(stats) = &qres_writer.race_stats {
            let json = serde_json::to_string_pretty(stats)?;
            let mut f = File::create(&path)?;
            f.write_all(json.as_bytes())?;
            println!("Race Report saved to {}", path);
        } else {
            println!("No Race Stats available (Buffer too small or forced mode).");
        }
    }
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
        eprintln!("Usage: qres-cli <compress|decompress> <in> <out> [--mode <auto|max|fast>] [--report <stats.json>]");
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "compress" => {
            // Parse optional flags
            let mut mode = 0;
            let mut report_path = None;
            
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
                    "--report" => {
                         if i + 1 < args.len() {
                             report_path = Some(args[i+1].clone());
                             i += 2;
                         } else { i += 1; }
                    },
                    _ => i += 1,
                }
            }
            compress_file(&args[2], &args[3], mode, report_path).unwrap()
        },
        "decompress" => decompress_file(&args[2], &args[3]).unwrap(),
        _ => eprintln!("Unknown command"),
    }
}
