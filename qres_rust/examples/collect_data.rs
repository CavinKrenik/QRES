use std::fs::File;
use std::io::{Read, Write};

// Import internal modules from the library
// Note: We need to use the crate name "qres_rust"
use qres_rust::mixer::{Mixer, NUM_MODELS};
use qres_rust::predictors::{GraphPredictor, LzMatchPredictor, Predictor, SimplePredictor};
use qres_rust::spectral::SpectralPredictor;

struct Features {
    entropy: f32,
    mean: f32,
    variance: f32,
    autocorr_1: f32,
}

fn calculate_features(data: &[u8]) -> Features {
    if data.is_empty() {
        return Features {
            entropy: 0.0,
            mean: 0.0,
            variance: 0.0,
            autocorr_1: 0.0,
        };
    }

    let mut counts = [0usize; 256];
    let mut sum = 0.0;
    let mut sum_sq = 0.0;

    for &b in data {
        counts[b as usize] += 1;
        sum += b as f32;
        sum_sq += (b as f32).powi(2);
    }

    let n = data.len() as f32;
    let mean = sum / n;
    let variance = sum_sq / n - mean * mean;

    let mut entropy = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f32 / n;
            entropy -= p * p.log2();
        }
    }

    // Autocorrelation Lag 1
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..data.len() - 1 {
        let diff1 = data[i] as f32 - mean;
        let diff2 = data[i + 1] as f32 - mean;
        num += diff1 * diff2;
        den += diff1 * diff1;
    }
    let autocorr_1 = if den != 0.0 { num / den } else { 0.0 };

    Features {
        entropy,
        mean,
        variance,
        autocorr_1,
    }
}

fn process_file(path: &str, output: &mut File) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Chunk size 64KB
    for chunk in buffer.chunks(64 * 1024) {
        if chunk.len() < 1024 {
            continue;
        }

        let _features = calculate_features(&chunk[0..1024]); // Features on header only? Or full chunk?
                                                             // Real-time: we only want to scan header. Let's say 512 bytes.
        let features_header = calculate_features(&chunk[0..512.min(chunk.len())]);

        // Run Compression Simulation (No encoding, just mixing)
        let mut mixer = Mixer::new(None, None);
        let mut simple = SimplePredictor::new();
        let mut graph = GraphPredictor::new();
        let mut spectral = SpectralPredictor::new(2048);
        let mut lz_match = LzMatchPredictor::new();
        let mut linear = 0u8;

        let mut preds = [0u8; NUM_MODELS];

        for &actual in chunk {
            preds[0] = linear;
            preds[1] = simple.predict_next();
            preds[2] = graph.predict_next();
            preds[3] = spectral.predict();
            preds[4] = lz_match.predict_next();

            let _mixed = mixer.mix(&preds);
            mixer.update(actual, &preds);

            linear = actual;
            simple.update(actual);
            graph.update(actual);
            spectral.update(actual);
            lz_match.update(actual);
        }

        // Final Weights
        #[cfg(target_arch = "x86_64")]
        let weights: [f32; 8] = unsafe { std::mem::transmute(mixer.weights) };
        #[cfg(not(target_arch = "x86_64"))]
        let weights = mixer.weights;

        // Write CSV Row
        // Features: entropy, mean, var, ac1
        // Labels: w0, w1, w2, w3, w4
        writeln!(
            output,
            "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            features_header.entropy,
            features_header.mean,
            features_header.variance,
            features_header.autocorr_1,
            weights[0],
            weights[1],
            weights[2],
            weights[3],
            weights[4]
        )?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Collect data from benchmarks/datasets (relative to qres_rust root)
    let datasets = std::fs::read_dir("../benchmarks/datasets")?;
    let mut out_file = File::create("../benchmarks/training_data.csv")?;

    writeln!(
        out_file,
        "entropy,mean,variance,autocorr_1,w_linear,w_simple,w_graph,w_spectral,w_lz"
    )?;

    for entry in datasets {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("Processing {:?}", path);
            process_file(path.to_str().unwrap(), &mut out_file)?;
        }
    }
    println!("Done. training_data.csv generated.");
    Ok(())
}
