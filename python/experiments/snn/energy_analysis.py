def analyze_energy():
    print("====================================")
    print("⚡ QRES Energy Analysis: ANN vs SNN")
    print("====================================")
    
    # 1. Constants (Source: Horowitz et al. 45nm)
    EJ_MAC = 4.6  # pJ per MAC (32-bit float)
    EJ_AC = 0.9   # pJ per Accumulate (SNN spike add)
    
    # 2. ANN Estimations (TinyPredictor - 2 Layer MLP equivalent size)
    # Input(1) -> FC(128) -> ReLU -> FC(1)
    # MACs = (1*128 + 128) + (128*1 + 1) = 256 + 129 = 385 MACs per step
    # For a window of 32 steps (if processing sequentially) or just 1 pass if standard MLP:
    # Standard MLP takes whole window at once: Input(32) -> FC(128) -> FC(1)
    # MACs = (32*128) + (128*1) = 4096 + 128 = 4224 MACs per inference
    
    ann_macs_per_inf = 4224
    ann_energy_pj = ann_macs_per_inf * EJ_MAC
    
    # 3. SNN Actuals
    try:
        with open("snn_stats.txt", "r") as f:
            lines = f.readlines()
            test_mse = float(lines[0].strip())
            total_spikes = int(float(lines[1].strip()))
            num_samples = int(lines[2].strip())
    except FileNotFoundError:
        print("Error: snn_stats.txt not found. Run snn_predictor.py first.")
        return

    # SNN Energy = Total Spikes * AC cost
    # Average spikes per inference
    avg_spikes_per_inf = total_spikes / num_samples
    snn_energy_pj = avg_spikes_per_inf * EJ_AC
    
    # 4. Comparison
    print(f"ANN (MLP) Energy/Inf: {ann_energy_pj:.2f} pJ")
    print(f"SNN (LIF) Energy/Inf: {snn_energy_pj:.2f} pJ")
    
    ratio = ann_energy_pj / snn_energy_pj
    print(f"Improvement Factor:   {ratio:.2f}x")
    
    print("-" * 30)
    print(f"SNN MSE Accuracy:     {test_mse:.6f}")
    print("-" * 30)
    
    # Generate Markdown Report
    report = f"""# SNN Energy Analysis Report

## Experimental Setup
- **Task:** Temporal Regression (Synthetic Sine)
- **Baseline ANN:** MLP (Input 32 -> 128 -> 1)
- **SNN:** LIF Recurrent (1 -> 128 -> 1) over 32 steps

## Energy Model (45nm)
- **MAC Operation (ANN):** 4.6 pJ
- **Accumulate (SNN):** 0.9 pJ

## Results

| Metric | ANN (Baseline) | SNN (Spiking) | Improvement |
|--------|---------------|---------------|-------------|
| **Energy/Inf** | {ann_energy_pj:.2f} pJ | {snn_energy_pj:.2f} pJ | **{ratio:.1f}x** |
| Accuracy (MSE) | (Reference) | {test_mse:.6f} | N/A |

## Conclusion
The SNN demonstrates a **{ratio:.1f}x** reduction in theoretical energy consumption compared to the baseline ANN, utilizing sparse event-driven computation.
"""
    
    with open("docs/SNN_ENERGY_ANALYSIS.md", "w") as f:
        f.write(report)
        print("Report saved to docs/SNN_ENERGY_ANALYSIS.md")

if __name__ == "__main__":
    analyze_energy()
