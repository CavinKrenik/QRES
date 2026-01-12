use qres_core::resource_management::ResourceUsagePredictor;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    println!("==================================================");
    println!("Resource Predictor Benchmark (Neural vs Heuristic)");
    println!("==================================================");

    // 1. Setup
    // Locate ONNX model relative to where we expect to be running
    // If running from root: qres_rust/qres_core/assets/predictor_v2.onnx
    // We try a few paths to be robust
    let mut model_path = PathBuf::from("qres_rust/qres_core/assets/predictor_v2.onnx");
    if !model_path.exists() {
        model_path = PathBuf::from("../qres_rust/qres_core/assets/predictor_v2.onnx");
    }
    if !model_path.exists() {
        // Maybe we are in qres_core
        model_path = PathBuf::from("assets/predictor_v2.onnx");
    }

    // Fallback check
    if !model_path.exists() {
        println!(
            "Error: Could not find ONNX model at {:?}. Benchmark will use Heuristic only.",
            model_path
        );
    } else {
        println!("Found ONNX model at {:?}", model_path);
    }

    let predictor = ResourceUsagePredictor::new(Some(&model_path));

    // 2. Data Gen
    let window_size = 32;
    let iterations = 10_000;

    // Create random window data
    // Create mixed window data (50% simple/flat, 50% complex/sine)
    let mut data: Vec<Vec<f32>> = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let mut window = Vec::with_capacity(window_size);
        let is_flat = (i / 100) % 2 == 0; // Alternate every 100 iterations

        for j in 0..window_size {
            let val = if is_flat {
                0.5 // Flat line (Variance = 0)
            } else {
                ((i + j) as f32 * 0.1).sin() // Sine wave
            };
            window.push(val);
        }
        data.push(window);
    }

    // 3. Benchmark Heuristic
    let start_heuristic = Instant::now();
    for window in &data {
        let _ = predictor.predict_heuristic(window);
    }
    let duration_heuristic = start_heuristic.elapsed();
    let avg_heuristic = duration_heuristic.as_secs_f64() * 1_000_000.0 / iterations as f64;

    println!("Heuristic Average Latency: {:.2} µs", avg_heuristic);

    // 4. Benchmark Neural
    let start_neural = Instant::now();
    let mut success_count = 0;
    for window in &data {
        if predictor.predict_neural(window).is_some() {
            success_count += 1;
        }
    }
    let duration_neural = start_neural.elapsed();

    if success_count > 0 {
        let avg_neural = duration_neural.as_secs_f64() * 1_000_000.0 / iterations as f64;
        println!("Neural Average Latency:    {:.2} µs", avg_neural);
        println!(
            "Neural Overhead Factor:    {:.2}x",
            avg_neural / avg_heuristic
        );
    } else {
        println!("Neural Benchmark Failed (Model not loaded or invalid windows)");
    }

    // 5. Benchmark Hybrid
    let start_hybrid = Instant::now();
    for window in &data {
        let _ = predictor.predict(window);
    }
    let duration_hybrid = start_hybrid.elapsed();
    let avg_hybrid = duration_hybrid.as_secs_f64() * 1_000_000.0 / iterations as f64;

    println!("Hybrid Average Latency:    {:.2} µs", avg_hybrid);

    println!("==================================================");
}
