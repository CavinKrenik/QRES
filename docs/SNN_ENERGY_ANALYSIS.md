# SNN Energy Analysis Report

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
| **Energy/Inf** | 19430.40 pJ | 886.05 pJ | **21.9x** |
| Accuracy (MSE) | (Reference) | 0.415954 | N/A |

## Conclusion
The SNN demonstrates a **21.9x** reduction in theoretical energy consumption compared to the baseline ANN, utilizing sparse event-driven computation.
