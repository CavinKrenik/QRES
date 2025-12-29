use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use qres_rust::{QresWriter, QresReader};

fn compress_file(input: &str, output: &str) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(input)?);
    let writer = BufWriter::new(File::create(output)?);
    
    // Create the QRES Stream Wrapper
    let mut qres_writer = QresWriter::new(writer, 1); // Default to Linear (1)
    
    // Stream data (this handles buffering and chunking automatically)
    let bytes = io::copy(&mut reader, &mut qres_writer)?;
    
    // Note: qres_writer needs to be flushed/finished when it drops, or we might miss the last chunk if logic isn't in Drop.
    // The QresWriter in lib.rs has a 'finish' method in previous versions, but standard Write trait reliance implies explicit flush or Drop.
    // The lib.rs implementation for QresWriter doesn't implement Drop, but it implements Flush.
    // io::copy calls write_all, so we should call flush manually at the end if we don't consume the writer.
    qres_writer.flush()?; 
    
    println!("Streamed {} bytes to {}", bytes, output);
    Ok(())
}

fn decompress_file(input: &str, output: &str) -> io::Result<()> {
    let reader = BufReader::new(File::open(input)?);
    let mut writer = BufWriter::new(File::create(output)?);
    
    // Create the QRES Stream Wrapper
    let mut qres_reader = QresReader::new(reader);
    
    // Stream data (handles reading frames and decompressing automatically)
    let bytes = io::copy(&mut qres_reader, &mut writer)?;
    
    println!("Restored {} bytes from {}", bytes, input);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 { 
        eprintln!("Usage: qres_rust <compress|decompress> <in> <out>");
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "compress" => compress_file(&args[2], &args[3]).unwrap(),
        "decompress" => decompress_file(&args[2], &args[3]).unwrap(),
        _ => eprintln!("Unknown command"),
    }
}
