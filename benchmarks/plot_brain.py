import sys
import numpy as np
import matplotlib.pyplot as plt
import qres

def plot_residuals(file_path):
    # Read Raw Data
    try:
        raw_data = np.fromfile(file_path, dtype=np.uint8)
        # Limit to 64KB for visualization (same as race sample)
        sample = raw_data[:65536]
    except Exception as e:
        print(f"Error: {e}")
        return

    # Get Residuals
    print("Computing Residuals...")
    bytes_sample = sample.tobytes()
    res_linear = np.array(qres.get_residuals(bytes_sample, 1, None))
    
    # Load Weights Helper
    def load_weights(path):
        try:
            with open(path, 'rb') as f:
                return f.read()
        except:
            return None

    w_lstm = load_weights("qres_rust/src/models/lstm.qnn")
    w_tensor = load_weights("qres_rust/src/models/tensor.qnn")
    
    res_lstm = np.zeros_like(res_linear)
    res_tensor = np.zeros_like(res_linear)

    if w_lstm:
        res_lstm = np.array(qres.get_residuals(bytes_sample, 3, w_lstm))
    if w_tensor:
        res_tensor = np.array(qres.get_residuals(bytes_sample, 4, w_tensor))

    # Plot
    fig, axs = plt.subplots(4, 1, figsize=(12, 10), sharex=True)
    
    # 1. Raw Wave
    axs[0].plot(sample[:1000], color='black', alpha=0.7)
    axs[0].set_title("Raw Signal (First 1000 samples)")
    axs[0].set_ylabel("Amplitude")

    # 2. Linear Residuals
    axs[1].plot(res_linear[:1000], color='cyan')
    axs[1].set_title(f"Linear Residuals (Std: {np.std(res_linear):.2f})")
    
    # 3. Tensor Residuals
    col_t = 'yellow' if w_tensor else 'gray'
    axs[2].plot(res_tensor[:1000], color=col_t)
    axs[2].set_title(f"Tensor Residuals (Std: {np.std(res_tensor):.2f})")

    # 4. LSTM Residuals
    col_l = 'magenta' if w_lstm else 'gray'
    axs[3].plot(res_lstm[:1000], color=col_l)
    axs[3].set_title(f"LSTM Residuals (Std: {np.std(res_lstm):.2f})")
    
    plt.tight_layout()
    output_file = "brain_residuals.png"
    plt.savefig(output_file)
    print(f"Plot saved to {output_file}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python plot_brain.py <input_file>")
    else:
        plot_residuals(sys.argv[1])
