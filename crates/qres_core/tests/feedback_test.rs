use qres_core::adaptive::feedback_loop::FeedbackLoop;
use qres_core::adaptive::regime_detector::{RegimeChange, RegimeDetector};

#[test]
fn test_regime_change_detection() {
    // 1. Setup
    let window_size = 32;
    let mut detector = RegimeDetector::new(window_size, 0.8, 1000.0);

    println!(">> Phase 1: Training on Stable Signal (Sine Wave)");
    // Feed 100 samples of a clean sine wave
    for i in 0..100 {
        let actual = (i as f32 * 0.1).sin();
        let prediction = actual + 0.01; // Small constant error
        let residual = prediction - actual;

        let change = detector.observe(residual);
        assert_eq!(
            change,
            RegimeChange::None,
            "Should not detect drift in stable phase"
        );
    }

    println!(">> Phase 2: Injecting drift (Sudden Spike/Noise)");
    // Suddenly add massive error
    let mut detected = false;
    for _ in 0..10 {
        // Massive error: 1.0 (compared to 0.01)
        let change = detector.observe(1.0);
        if let RegimeChange::Drift {
            current_error,
            threshold,
        } = change
        {
            println!(
                "Drift Detected! Error: {:.4}, Threshold: {:.4}",
                current_error, threshold
            );
            detected = true;
            break;
        }
    }

    assert!(detected, "Failed to detect regime change!");
}

#[test]
fn test_feedback_loop_integration() {
    let mut feedback = FeedbackLoop::new(32);

    // Stable
    for _ in 0..50 {
        feedback.observe(1.0, 1.01);
    }

    // Broken
    // Taking manual observation to verify internal state isn't easily done without
    // internal access or return values, but this ensures it runs without panic.
    // In a real system, we'd check logs or events.
    feedback.observe(1.0, 5.0);
}
