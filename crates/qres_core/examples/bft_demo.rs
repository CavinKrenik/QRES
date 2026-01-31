//! BFT Defense Demo - Run to see the visual log output
//!
//! Usage: cargo run --example bft_demo --features std

use fixed::types::I16F16;
use qres_core::consensus::aggregate_krum;

fn main() {
    println!("\n🧪 QRES Byzantine Fault Tolerance Demo");
    println!("=====================================\n");

    // Create 4 honest vectors + 1 malicious outlier
    let honest_vectors = vec![
        vec![I16F16::from_num(1.0), I16F16::from_num(1.1)],
        vec![I16F16::from_num(0.9), I16F16::from_num(1.0)],
        vec![I16F16::from_num(1.05), I16F16::from_num(0.95)],
        vec![I16F16::from_num(1.0), I16F16::from_num(1.0)],
    ];

    let malicious = vec![I16F16::from_num(100.0), I16F16::from_num(100.0)];

    let mut all_vectors = honest_vectors.clone();
    all_vectors.push(malicious);

    println!("📊 Input Vectors:");
    for (i, v) in all_vectors.iter().enumerate() {
        let label = if i == 4 {
            "MALICIOUS ⚠️"
        } else {
            "Honest    ✓"
        };
        println!(
            "   [{i}] {label}: [{:.2}, {:.2}]",
            v[0].to_num::<f32>(),
            v[1].to_num::<f32>()
        );
    }

    // Calculate what naive Mean would produce
    let n = all_vectors.len() as f32;
    let mean: Vec<f32> = (0..2)
        .map(|i| {
            all_vectors
                .iter()
                .map(|v| v[i].to_num::<f32>())
                .sum::<f32>()
                / n
        })
        .collect();

    println!("\n❌ Naive Mean (COMPROMISED):");
    println!("   [{:.2}, {:.2}]", mean[0], mean[1]);
    println!("   (Malicious node poisoned the average!)");

    // Run Krum (f=1 Byzantine tolerance)
    let krum_result = aggregate_krum(&all_vectors, 1).expect("Krum should succeed");
    let krum_f32: Vec<f32> = krum_result.iter().map(|v| v.to_num()).collect();

    println!("\n🛡️ BFT DEFENSE ACTIVE: Malicious outlier rejected");
    println!(
        "   Krum (PROTECTED): [{:.2}, {:.2}]",
        krum_f32[0], krum_f32[1]
    );

    // Calculate difference
    let diff: f32 = mean
        .iter()
        .zip(krum_f32.iter())
        .map(|(m, k)| (m - k).abs())
        .sum();

    println!("\n📈 Defense Impact:");
    println!("   Mean (Compromised):  [{:.2}, {:.2}]", mean[0], mean[1]);
    println!(
        "   Krum (Protected):    [{:.2}, {:.2}]",
        krum_f32[0], krum_f32[1]
    );
    println!("   Total Correction:    {:.2}", diff);
    println!("\n✅ Byzantine attack mitigated using I16F16 deterministic arithmetic.\n");
}
