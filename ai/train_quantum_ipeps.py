import numpy as np
import struct
import os

def fast_tanh(x):
    return x / (1.0 + np.abs(x))

def generate_ipeps_weights():
    print("Generating iPEPS (Infinite Projected Entangled Pair States) Weights...")
    
    # Architecture: 
    # Input Vector (Context): [p1, p2, p3, p4] -> 4 floats
    # Hidden Layer (Bond Dimension): 8 floats
    # Output: 1 val
    
    # W1: 4 -> 8
    # W2: 8 -> 1
    # B1: 8
    # B2: 1
    
    np.random.seed(42)
    
    # Initialize with orthogonal matrices for better signal propagation (Quantum Style)
    # W1 (4x8)
    X = np.random.randn(4, 8)
    u, _, vh = np.linalg.svd(X, full_matrices=False)
    w1 = u @ vh
    
    # W2 (8x1)
    Y = np.random.randn(8, 1)
    u2, _, vh2 = np.linalg.svd(Y, full_matrices=False)
    w2 = u2 @ vh2
    
    # Bias
    b1 = np.zeros(8, dtype=np.float32)
    b2 = np.zeros(1, dtype=np.float32)
    
    # Flatten
    w1_flat = w1.flatten().astype(np.float32)
    w2_flat = w2.flatten().astype(np.float32)
    
    # Serialization
    # Layout: W1 (32 floats) | B1 (8 floats) | W2 (8 floats) | B2 (1 float)
    # Total: 32 + 8 + 8 + 1 = 49 floats * 4 bytes = 196 bytes
    
    buffer = bytearray()
    buffer.extend(w1_flat.tobytes())
    buffer.extend(b1.tobytes())
    buffer.extend(w2_flat.tobytes())
    buffer.extend(b2.tobytes())
    
    os.makedirs("qres_rust/assets", exist_ok=True)
    with open("qres_rust/assets/ipeps.qnn", "wb") as f:
        f.write(buffer)
        
    print(f"Generated qres_rust/assets/ipeps.qnn ({len(buffer)} bytes)")

if __name__ == "__main__":
    generate_ipeps_weights()
