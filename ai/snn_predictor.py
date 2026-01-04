import torch
import torch.nn as nn
import numpy as np

class LeakyIntegrateAndFire(nn.Module):
    """
    Simulates a Spiking Neural Network (SNN) with Leaky Integrate-and-Fire (LIF) neurons.
    Used for temporal, sparse inputs (Brain-Like Compression).
    """
    def __init__(self, input_size=256, hidden_size=128, output_size=256, beta=0.9):
        super(LeakyIntegrateAndFire, self).__init__()
        self.input_size = input_size
        self.hidden_size = hidden_size
        self.output_size = output_size
        self.beta = beta  # Decay rate for membrane potential

        # Synaptic Weights
        self.fc1 = nn.Linear(input_size, hidden_size)
        self.fc2 = nn.Linear(hidden_size, output_size)

        # State initialization
        self.reset_state()

    def reset_state(self):
        self.mem1 = torch.zeros(1, self.hidden_size)
        self.mem2 = torch.zeros(1, self.output_size)
        self.spk1 = torch.zeros(1, self.hidden_size)
        self.spk2 = torch.zeros(1, self.output_size)

    def forward(self, x):
        """
        Forward pass for one time step.
        x: Input spike rate or direct value [Batch, Input_Size]
        """
        # Layer 1: LIF
        cur1 = self.fc1(x)
        self.mem1 = self.beta * self.mem1 + cur1
        self.spk1 = self.fire(self.mem1)
        self.mem1 = self.reset_potential(self.mem1, self.spk1)

        # Layer 2: LIF (Output)
        cur2 = self.fc2(self.spk1)
        self.mem2 = self.beta * self.mem2 + cur2
        self.spk2 = self.fire(self.mem2) # Output spikes
        self.mem2 = self.reset_potential(self.mem2, self.spk2)

        return self.spk2, self.mem2

    def fire(self, mem):
        """Heaviside step function for spiking."""
        return (mem > 1.0).float()

    def reset_potential(self, mem, spikes):
        """Hard reset membrane potential after spike."""
        return mem * (1.0 - spikes)

class SNNPredictor:
    """
    Wrapper for usage in QRES MetaBrain.
    Encodes bytes as one-hot spike inputs, predicts next byte probability via membrane potential.
    """
    def __init__(self, model_path=None):
        self.device = torch.device("cpu")
        self.model = LeakyIntegrateAndFire().to(self.device)
        if model_path:
            self.load(model_path)
        
        self.model.eval()
        self.context_window = []

    def load(self, path):
        # In a real scenario, load state_dict
        pass

    def predict_next(self, context_bytes):
        """
        Takes last byte, converts to one-hot, runs SNN step.
        Returns probabilities (softmax of membrane potential).
        """
        if not context_bytes:
            return np.ones(256) / 256.0

        last_byte = context_bytes[-1]
        
        # One-hot encoding
        inp = torch.zeros(1, 256)
        inp[0, last_byte] = 1.0
        
        with torch.no_grad():
            _, mem_out = self.model(inp)
        
        # In SNNs, output membrane potential is proxy for probability
        probs = torch.softmax(mem_out, dim=1).numpy()[0]
        return probs

    def update(self, byte):
        # Online learning (STDP) could happen here
        pass
