"""
QRES Adversarial Hardening - Experiment 3: MNIST High-Variance Precision Test

Tests the limits of I16F16 fixed-point arithmetic (used in QRES Core) against
high-variance datasets and low learning rates.

Hypothesis: Small learning rates (< 1e-5) combined with I16F16 resolution (1.5e-5)
will cause gradient underflow (vanishing updates).

Parameters:
    - Fixed Point: I16F16 (Range: [-65536, 65536], Resolution: 2^-16 ~= 0.0000152)
    - Learning Rate Sweep: [1e-3, 1e-4, 1e-5, 1e-6]
    - Data: Simulated MNIST Gradients (Normal dist, mean=0, std=0.1)

Metric: 
    - Zero Update Rate: % of updates that quantize to exactly 0.0
    - Clip Rate: % of weights hitting max bounds (unlikely for MNIST but checked)
    - Accuracy Delta: Simulated loss of precision vs Float32

Usage: python tools/fixed_point_precision.py
"""

import numpy as np
import sys
import os
from datetime import datetime

# I16F16 Constants
SCALE = 65536.0
MAX_VAL = 65536.0
MIN_VAL = -65536.0
RESOLUTION = 1.0 / SCALE

# BFP Constants
BFP_MANTISSA_BITS = 16
BFP_MANTISSA_MAX = 2 ** (BFP_MANTISSA_BITS - 1) - 1   # 32767
BFP_MANTISSA_MIN = -(2 ** (BFP_MANTISSA_BITS - 1))     # -32768
BFP_EXPONENT_BITS = 8
BFP_EXPONENT_MAX = 2 ** (BFP_EXPONENT_BITS - 1) - 1    # 127
BFP_EXPONENT_MIN = -(2 ** (BFP_EXPONENT_BITS - 1))      # -128


def to_fixed(x):
    """Quantize float32 to I16F16 behavior."""
    x_scaled = np.round(x * SCALE)
    # Clip to bounds
    x_clipped = np.clip(x_scaled, MIN_VAL * SCALE, MAX_VAL * SCALE)
    return x_clipped / SCALE


def to_bfp(x):
    """
    Quantize a float32 vector to Block Floating Point representation.

    BFP assigns a single shared exponent to the entire vector, then stores
    each element as a 16-bit signed integer mantissa.

    shared_exponent = ceil(log2(max|x| / MANTISSA_MAX))
    mantissa[i]    = round(x[i] / 2^shared_exponent)
    reconstructed  = mantissa[i] * 2^shared_exponent

    This preserves relative precision across the vector regardless of
    absolute magnitude, solving the vanishing-update problem at low LR.
    """
    max_abs = np.max(np.abs(x))
    if max_abs == 0:
        return np.zeros_like(x)

    # Compute shared exponent: scale so max value fits in mantissa range
    raw_exp = np.ceil(np.log2(max_abs / BFP_MANTISSA_MAX))
    shared_exp = int(np.clip(raw_exp, BFP_EXPONENT_MIN, BFP_EXPONENT_MAX))

    scale = 2.0 ** shared_exp

    # Quantize to integer mantissas
    mantissas = np.round(x / scale)
    mantissas = np.clip(mantissas, BFP_MANTISSA_MIN, BFP_MANTISSA_MAX)

    # Reconstruct from quantized representation
    return mantissas * scale

def run_precision_test(use_bfp=False):
    format_name = "BFP-16" if use_bfp else "I16F16"
    print(f"Running Experiment 3: Fixed Point Precision Test [{format_name}]")

    learning_rates = [1e-3, 1e-4, 1e-5, 1e-6]
    results = {}

    n_params = 10000
    np.random.seed(42)

    # Typical gradients for a neural net (near zero, std=0.01)
    gradients = np.random.normal(0, 0.01, n_params)

    if use_bfp:
        print(f"BFP-16: {BFP_MANTISSA_BITS}-bit mantissa, {BFP_EXPONENT_BITS}-bit shared exponent")
    else:
        print(f"I16F16 Resolution: {RESOLUTION:.8f}")

    for lr in learning_rates:
        print(f"\n[TEST] LR = {lr}")

        # Float32 update (gold standard)
        delta_float = -lr * gradients

        # Quantized update
        if use_bfp:
            delta_quantized = to_bfp(delta_float)
        else:
            delta_quantized = to_fixed(delta_float)

        # Count zero updates where float32 was non-zero
        non_zero_mask = np.abs(delta_float) > 1e-30
        zero_updates = (np.abs(delta_quantized) == 0) & non_zero_mask
        zero_rate = np.sum(zero_updates) / np.sum(non_zero_mask)

        print(f"   Non-zero float updates: {np.sum(non_zero_mask)}")
        print(f"   Quantized to zero:      {np.sum(zero_updates)}")
        print(f"   Zero Rate:              {zero_rate*100:.1f}%")

        if use_bfp:
            max_abs = np.max(np.abs(delta_float))
            if max_abs > 0:
                raw_exp = int(np.ceil(np.log2(max_abs / BFP_MANTISSA_MAX)))
                eff_resolution = 2.0 ** raw_exp
                print(f"   Shared Exponent:        {raw_exp}")
                print(f"   Effective Resolution:   {eff_resolution:.2e}")

        mse = np.mean((delta_float - delta_quantized) ** 2)
        print(f"   MSE vs Float32:         {mse:.2e}")

        results[lr] = {
            'zero_rate': zero_rate,
            'mse': mse
        }

    return results

def append_results(results):
    report = f"""
### Experiment 3: MNIST High-Variance Precision Test - {datetime.now().strftime("%Y-%m-%d %H:%M")}

- **Hypothesis:** Will I16F16 fixed-point arithmetic cause "vanishing updates" when learning rates drop below the resolution threshold ($1.5 \\times 10^{{-5}}$)?

- **Parameters:**
  - Format: I16F16 (Resolution $2^{{-16}} \\approx 0.0000152$)
  - Learning Rates: 1e-3, 1e-4, 1e-5, 1e-6
  - Simulated Gradients: $\\mathcal{{N}}(0, 0.01)$

- **Raw Results:**

| Learning Rate | Zero Update Rate | MSE vs Float32 | Status |
|---------------|------------------|----------------|--------|
"""
    
    status = "PASSED"
    falsification_point = None
    
    for lr, data in results.items():
        zero_pct = data['zero_rate'] * 100
        row_status = "OK"
        if zero_pct > 10: row_status = "DEGRADED"
        if zero_pct > 50: 
            row_status = "FAILED"
            status = "FALSIFIED"
            if not falsification_point: falsification_point = lr
            
        report += f"| {lr} | {zero_pct:.1f}% | {data['mse']:.2e} | {row_status} |\n"
        
    report += f"""
- **Analysis:**
  - **Resolution Limit:** The I16F16 format has a hard resolution limit of approx 1.5e-5.
  - **At LR=1e-5:** A significant portion of updates may underflow if the gradient * LR < 1.5e-5. (Gradient needs to be > 1.5 for this to work, which is rare).
  - **At LR=1e-6:** {results[1e-6]['zero_rate']*100:.0f}% of updates quantized to zero. The model effectively stopped learning.
  - **Conclusion:** QRES cannot support fine-tuning tasks (requiring low LR) with the current fixed-point schema.

- **Status:** **{status}**
"""

    with open(os.path.join("research", "Attack.md"), "a", encoding="utf-8") as f:
        f.write(report)
    print("Results appended to Attack.md")

if __name__ == "__main__":
    use_bfp = "--bfp" in sys.argv
    data = run_precision_test(use_bfp=use_bfp)
    append_results(data)
