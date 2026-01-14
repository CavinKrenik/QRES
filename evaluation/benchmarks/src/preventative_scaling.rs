use qres_core::resource_management::{ResourceUsagePredictor, WorkerPool};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    println!("--------------------------------------------------");
    println!("Preventative Scaling Simulation (Neural Control)");
    println!("--------------------------------------------------");

    // 1. Setup
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

    let predictor = ResourceUsagePredictor::new(Some(&model_path));
    let mut pool = WorkerPool::new();

    println!("Model loaded from: {:?}", model_path);
    println!("Initial Pool Size: {} threads", pool.current_capacity);

    // 2. Generate Workload Stream (Sine wave rising)
    // We simulate a stream of data points.
    let window_size = 32;
    let total_steps = 100;

    let mut history = vec![0.1; window_size];

    // Main Control Loop
    for t in 0..total_steps {
        // Generate actual load for NEXT step (t+1) to compare against prediction
        // Load Pattern: Rise from 0.1 to 0.9 then drop
        let progress = t as f32 / total_steps as f32;
        let actual_load = if progress < 0.5 {
            0.1 + (progress * 1.6) // Rise to 0.9
        } else {
            0.9 - ((progress - 0.5) * 1.6) // Fall back to 0.1
        };
        // Add noise
        let noise = (rand::random::<f32>() - 0.5) * 0.05;
        let current_val = actual_load + noise;

        history.push(current_val);

        // Prepare window for prediction
        let window_start = history.len() - window_size;
        let window = &history[window_start..];

        // PREDICT
        let prediction = predictor.predict(window);

        // ACT
        let old_size = pool.current_capacity;
        let new_size = pool.adjust_capacity(prediction);

        // REPORT
        // formatting: [Step XX] Load: 0.XX, Pred: 0.XX -> Resize: XX -> XX
        let arrow = if new_size > old_size {
            "UP"
        } else if new_size < old_size {
            "DOWN"
        } else {
            "--"
        };

        println!(
            "[Step {:03}] Load: {:.2}, Pred: {:.2} -> Pool: {:02} ({})",
            t, current_val, prediction, new_size, arrow
        );

        // Sleep for visual effect (optional, keep it fast though)
        thread::sleep(Duration::from_millis(20));
    }

    println!("--------------------------------------------------");
    println!("Simulation Complete.");
}
