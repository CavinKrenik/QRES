use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use qres_rust::{QresWriter, QresReader, LivingBrain};

const DEFAULT_BRAIN_FILE: &str = "qres_brain.json";

fn compress_file(input: &str, output: &str, mode_hint: u8, anomaly_threshold: Option<u8>, lossy_tolerance: Option<u8>, explain: bool, trace_file: Option<String>) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(input)?);
    let writer = BufWriter::new(File::create(output)?);
    
    // Load Brain
    let brain = if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        LivingBrain::from_json(&json).unwrap_or_else(|| LivingBrain::new())
    } else {
        LivingBrain::new()
    };

    // QresWriter handles detection internally now
    let mut qres_writer = QresWriter::new_with_brain(writer, mode_hint, brain);
    if let Some(t) = anomaly_threshold {
        qres_writer.set_anomaly_threshold(t);
    }
    if let Some(l) = lossy_tolerance {
        qres_writer.set_lossy(l);
    }
    if let Some(tf) = trace_file {
        let f = File::create(tf)?;
        qres_writer.set_trace(Box::new(f)); // Correct method name is set_trace
    }
    
    // Stream
    let start = std::time::Instant::now();
    let bytes = io::copy(&mut reader, &mut qres_writer)?;
    qres_writer.flush()?; 
    
    // Save Brain
    let new_brain = qres_writer.get_brain();
    if let Err(e) = fs::write(DEFAULT_BRAIN_FILE, new_brain.to_json()) {
        eprintln!("Warning: Failed to save brain: {}", e);
    }
    
    println!("Streamed {} bytes to {} (Mode: {}) in {:.2}s", 
        bytes, output, mode_hint, start.elapsed().as_secs_f64());

    if explain {
        println!("🧠 Neuro-Symbolic Reason: {}", qres_writer.explain_str);
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

fn brain_export() -> io::Result<()> {
    if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        println!("{}", json);
    } else {
        // If no brain exists, export a default fresh brain
        println!("{}", LivingBrain::new().to_json());
    }
    Ok(())
}

fn brain_import(file_path: &str) -> io::Result<()> {
    // Load Local
    let mut local = if let Ok(json) = fs::read_to_string(DEFAULT_BRAIN_FILE) {
        LivingBrain::from_json(&json).unwrap_or_else(|| LivingBrain::new())
    } else {
        LivingBrain::new()
    };
    
    // Load Import
    let import_json = fs::read_to_string(file_path)?;
    if let Some(imported) = LivingBrain::from_json(&import_json) {
        // Merge: New = 0.9 * Local + 0.1 * Import
        local.merge(&imported, 0.1);
        fs::write(DEFAULT_BRAIN_FILE, local.to_json())?;
        println!("🧠 Brain merged successfully. Wisdom assimilated.");
    } else {
        eprintln!("Failed to parse imported brain.");
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { 
        eprintln!("Usage: qres-cli <compress|decompress|brain-export|brain-import> ...");
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "swarm" => {
             // Subcommands: start, stop, status, run-node
             let subcmd = if args.len() > 2 { args[2].as_str() } else { "start" }; // Default to start? Or help? Let's say help or status. But user usually types `swarm start`. 
             // Actually, if no subcommand, let's show status.
             
             match subcmd {
                 "start" => {
                     let mut wan = false;
                     let mut interval = 600;
                     let mut i = 3;
                     while i < args.len() {
                         match args[i].as_str() {
                             "--wan" => { wan = true; i += 1; },
                             "--gossip-interval" => {
                                 if i + 1 < args.len() {
                                     interval = args[i+1].parse().unwrap_or(600);
                                     i += 2;
                                 } else { i += 1; }
                             },
                             _ => i += 1,
                         }
                     }
                     if let Err(e) = qres_rust::daemon::DaemonManager::start(wan, interval) {
                         eprintln!("Error starting daemon: {}", e);
                     }
                 },
                 "stop" => {
                     if let Err(e) = qres_rust::daemon::DaemonManager::stop() {
                         eprintln!("Error stopping daemon: {}", e);
                     }
                 },
                 "status" => {
                     qres_rust::daemon::DaemonManager::status();
                 },
                 "run-node" => {
                     // Internal Command - Actual Node Process
                     let mut wan = false;
                     let mut interval = 600;
                     let mut i = 3;
                      while i < args.len() {
                         match args[i].as_str() {
                             "--wan" => { wan = true; i += 1; },
                             "--gossip-interval" => {
                                 if i + 1 < args.len() {
                                     interval = args[i+1].parse().unwrap_or(600);
                                     i += 2;
                                 } else { i += 1; }
                             },
                             _ => i += 1,
                         }
                     }
                     
                     let brain_path = "qres_brain.json".to_string();
                     let config = qres_rust::swarm::SwarmConfig { wan, gossip_interval: interval };
                     
                     let rt = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    
                     println!("Daemon Process Started (Interval: {}s, WAN: {})", interval, wan);

                     rt.block_on(async {
                        if let Err(e) = qres_rust::swarm::QresSwarm::run_daemon(brain_path, config).await {
                            eprintln!("Swarm Fatal Error: {}", e);
                        }
                     });
                 },
                 _ => {
                     eprintln!("Unknown swarm command: {}. Use start, stop, status.", subcmd);
                 }
             }
        },
        "api-server" => {
            // HTTP REST API Server
            let mut port = 3030;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--port" => {
                        if i + 1 < args.len() {
                            port = args[i+1].parse().unwrap_or(3030);
                            i += 2;
                        } else { i += 1; }
                    },
                    _ => i += 1,
                }
            }
            
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            
            rt.block_on(async {
                if let Err(e) = qres_rust::api::run_api_server(port).await {
                    eprintln!("API Server Error: {}", e);
                }
            });
        },
        "compress" => {
            if args.len() < 4 { eprintln!("Usage: compress <in> <out>"); return; }
            // Parse optional flags
            let mut mode = 0;
            let mut anomaly_threshold = None;
            let mut lossy_tolerance = None;
            let mut explain = false;
            let mut trace_file = None;
            let mut auto_tune = false; 
            
            let mut i = 4;
            while i < args.len() {
                match args[i].as_str() {
                    "--mode" => {
                        if i + 1 < args.len() {
                            mode = match args[i+1].as_str() {
                                "max" => 3,
                                "fast" => 1,
                                "semantic" => 7,
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
                    "--lossy" => {
                         if i + 1 < args.len() {
                             if let Ok(t) = args[i+1].parse::<u8>() {
                                 lossy_tolerance = Some(t);
                             }
                             i += 2;
                         } else { i += 1; }
                    },
                    "--explain" => {
                        explain = true;
                        i += 1;
                    },
                    "--auto-tune" => {
                        auto_tune = true;
                        i += 1;
                    },
                    "--trace" => {
                        if i + 1 < args.len() {
                            trace_file = Some(args[i+1].clone());
                            i += 2;
                        } else { i += 1; }
                    },
                    _ => i += 1,
                }
            }
            if auto_tune {
                println!("🧠 Auto-Tune Enabled.");
            }
            compress_file(&args[2], &args[3], mode, anomaly_threshold, lossy_tolerance, explain, trace_file).unwrap()
        },
        "decompress" => {
             if args.len() < 4 { eprintln!("Usage: decompress <in> <out>"); return; }
             decompress_file(&args[2], &args[3]).unwrap()
        },
        "brain-export" => brain_export().unwrap(),
        "brain-import" => {
             if args.len() < 3 { eprintln!("Usage: brain-import <file>"); return; }
             brain_import(&args[2]).unwrap()
        },
        _ => eprintln!("Unknown command"),
    }
}
