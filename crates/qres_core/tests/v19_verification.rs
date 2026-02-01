use qres_core::aggregation::{Aggregator, TrimmedMeanByzAggregator};
use qres_core::consensus::krum::Bfp16Vec;
use qres_core::encoding::arithmetic;

#[test]
fn verification_phase_2_robust_aggregation() {
    // 1. Setup Sybil Attack (Bias = 20%)
    // Parameters from Attack.md Golden Run
    let n_total = 15;
    let f_byz = 4;
    let dim = 8;

    // Honest nodes at 1.0
    let mut updates: Vec<Vec<f32>> = (0..n_total - f_byz).map(|_| vec![1.0; dim]).collect();

    // Malicious nodes at 1.0 + Bias (0.2) = 1.2
    for _ in 0..f_byz {
        updates.push(vec![1.2; dim]);
    }

    // 2. Aggregate with TrimmedMeanByz
    let aggregator = TrimmedMeanByzAggregator { f: f_byz };
    let result = aggregator.aggregate(&updates);

    // 3. Verify Drift < 5%
    // In this deterministic case, Trimmed Mean (f=4) removes the 4 malicious terms (top)
    // and 4 honest terms (bottom). It averages the remaining 7 honest terms.
    // Result should be exactly 1.0.

    let drift_mag = (result.weights[0] - 1.0).abs();
    println!(
        "Aggregation Result: {:.4} (Expected 1.0)",
        result.weights[0]
    );
    println!("Drift Magnitude: {:.4}", drift_mag);

    assert!(
        drift_mag < 0.05,
        "Drift {:.4} exceeds 5% threshold! MIGRATION FAILED.",
        drift_mag
    );
}

#[test]
fn verification_phase_3_precision_bfp16() {
    // Test LR = 1e-5 (Vanishing Gradient Limit)
    let small_grad = 1e-5;
    let data = vec![small_grad; 8];

    // 1. Quantize
    let bfp = Bfp16Vec::from_f32_slice(&data);

    // 2. Serialize/Deserialize (Proof of Wire Format)
    let bytes = arithmetic::compress_bfp(bfp.exponent, &bfp.mantissas);
    assert_eq!(
        bytes.len(),
        17,
        "BFP-16 wire format not optimized (Expected 17 bytes for 8 dims)"
    );

    let (exp, mantissas) = arithmetic::decompress_bfp(&bytes, 8).unwrap();
    let bfp_rec = Bfp16Vec {
        exponent: exp,
        mantissas,
    };

    // 3. Reconstruct
    let rec_data = bfp_rec.to_vec_f32();
    let val = rec_data[0];

    println!("BFP Input: {:.1e}, Reconstructed: {:.1e}", small_grad, val);

    // Success Criteria:
    // 1. Must not be zero
    assert!(
        val.abs() > 0.0,
        "Vanishing Gradient detected! BFP-16 Failed."
    );

    // 2. Precision error check
    let delta = (val - small_grad).abs();
    assert!(delta < 1e-7, "Precision degradation too high.");
}

#[test]
fn verification_onboarding_summary_size() {
    // Verify Summary Gene size < 200 bytes
    // Simulating the struct layout manually
    // [Round: 8] + [Hash: 32] + [Consensus: 17] + [Variance: 17] = 74 bytes

    let consensus = vec![0.5; 8];
    let variance = vec![0.01; 8];

    let bfp_con = Bfp16Vec::from_f32_slice(&consensus);
    let bfp_var = Bfp16Vec::from_f32_slice(&variance);

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&100u64.to_be_bytes()); // Round
    bytes.extend_from_slice(&[0u8; 32]); // Hash
    bytes.extend(arithmetic::compress_bfp(
        bfp_con.exponent,
        &bfp_con.mantissas,
    ));
    bytes.extend(arithmetic::compress_bfp(
        bfp_var.exponent,
        &bfp_var.mantissas,
    ));

    let size = bytes.len();
    println!("Summary Gene Total Size: {} bytes", size);

    assert!(size < 200, "Summary Gene bloated!");
    assert_eq!(size, 74, "Exact size mismatch on optimized layout");
}
