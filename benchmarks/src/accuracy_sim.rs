use qres_core::resource_management::ResourceUsagePredictor;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    println!("Running Accuracy Sim...");

    // 1. Load Model
    let mut model_path = PathBuf::from("qres_rust/qres_core/assets/predictor_v2.onnx");
    if !model_path.exists() {
        model_path = PathBuf::from("../qres_rust/qres_core/assets/predictor_v2.onnx");
    }
    // Fallback if running from benchmarks dir
    if !model_path.exists() {
        model_path = PathBuf::from("../qres_rust/qres_core/assets/predictor_v2.onnx");
    }
    // One more try for absolute path
    if !model_path.exists() {
        model_path = PathBuf::from("C:/Dev/QRES/qres_rust/qres_core/assets/predictor_v2.onnx");
    }

    println!("Loading model from: {:?}", model_path);
    let predictor = ResourceUsagePredictor::new(Some(&model_path));

    // 2. Generate Synthetic Data
    // Pattern: Sine wave with varying frequency + random noise + occasional spikes
    let total_steps = 1000;
    let window_size = 32;
    let mut values = Vec::with_capacity(total_steps + window_size);

    for t in 0..total_steps + window_size {
        let t_f = t as f32;
        // Base: Sine wave
        let base = (t_f * 0.1).sin();
        // Noise
        let noise = (rand::random::<f32>() - 0.5) * 0.1;
        // Spike (occasional)
        let spike = if rand::random::<f32>() > 0.98 {
            (rand::random::<f32>() - 0.5) * 2.0
        } else {
            0.0
        };
        values.push(base + noise + spike);
    }

    // 3. Run Predictions & Log
    let output_path = "accuracy_results.csv";
    let mut file = File::create(output_path)?;
    writeln!(file, "step,actual_value,heuristic_pred,neural_pred")?;

    for i in 0..total_steps {
        let window = &values[i..i + window_size];
        let actual_next = values[i + window_size];

        let heuristic = predictor.predict_heuristic(window);

        // Neural can fail if model not loaded or internal error, handle gracefully
        let neural = predictor.predict_neural(window).unwrap_or(0.0);

        writeln!(file, "{},{},{},{}", i, actual_next, heuristic, neural)?;
    }

    println!("Simulation complete. Results written to {}", output_path);
    Ok(())
}
