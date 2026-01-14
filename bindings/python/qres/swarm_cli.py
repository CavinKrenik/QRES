"""
QRES Swarm CLI - Python wrapper for distributed Hive Mind operations.
Provides interface for broadcasting Epiphanies and receiving peer updates.
"""

import os
import json
import time
import struct
from pathlib import Path

# Constants
OUTBOX_PATH = "quantum_outbox"
INBOX_PATH = "quantum_inbox"
EPIPHANY_TOPIC = "qres/v1/epiphany"

class SwarmClient:
    """
    High-level interface for QRES P2P operations.
    Uses filesystem-based outbox/inbox for Rust swarm daemon interop.
    """
    
    def __init__(self, node_id: str = "python_node"):
        self.node_id = node_id
        
        # Ensure directories exist
        Path(OUTBOX_PATH).mkdir(exist_ok=True)
        Path(INBOX_PATH).mkdir(exist_ok=True)
    
    def broadcast_epiphany(self, weights_bytes: bytes, fidelity_score: float = 0.99):
        """
        Queues an Epiphany (model weights) for broadcast via the Rust swarm daemon.
        
        Format: QRES_EPIPHANY_V1 | timestamp | fidelity | node_id_len | node_id | weights
        """
        timestamp = int(time.time() * 1000)
        
        # Header
        header = b"QRES_EPIPHANY_V1"
        
        # Payload
        node_id_bytes = self.node_id.encode('utf-8')
        payload = struct.pack(
            f"<QfH{len(node_id_bytes)}s",
            timestamp,
            fidelity_score,
            len(node_id_bytes),
            node_id_bytes
        )
        
        # Full message
        message = header + payload + weights_bytes
        
        # Write to outbox (Rust daemon polls this)
        filename = f"{OUTBOX_PATH}/epiphany_{timestamp}_{self.node_id}.bin"
        with open(filename, 'wb') as f:
            f.write(message)
        
        print(f"[Swarm] Epiphany queued for broadcast: {len(message)} bytes")
        return filename
    
    def receive_epiphanies(self) -> list:
        """
        Reads incoming Epiphanies from the inbox (deposited by Rust daemon).
        Returns list of (node_id, fidelity, weights_bytes) tuples.
        """
        received = []
        
        for entry in Path(INBOX_PATH).glob("*.bin"):
            try:
                with open(entry, 'rb') as f:
                    data = f.read()
                
                # Parse header
                if not data.startswith(b"QRES_EPIPHANY_V1"):
                    continue
                
                # Skip header
                offset = 16
                
                # Parse struct
                timestamp, fidelity, node_id_len = struct.unpack_from("<QfH", data, offset)
                offset += struct.calcsize("<QfH")
                
                node_id = data[offset:offset + node_id_len].decode('utf-8')
                offset += node_id_len
                
                weights = data[offset:]
                
                received.append({
                    "node_id": node_id,
                    "fidelity": fidelity,
                    "timestamp": timestamp,
                    "weights": weights
                })
                
                # Archive processed file
                entry.rename(f"{INBOX_PATH}/processed_{entry.name}")
                
            except Exception as e:
                print(f"[Swarm] Error parsing {entry}: {e}")
        
        if received:
            print(f"[Swarm] Received {len(received)} Epiphanies from peers")
        
        return received

    def delta_compress(self, current: bytes, previous: bytes) -> bytes:
        """
        Fed2Com-style delta compression: XOR diff for efficient transmission.
        """
        if len(current) != len(previous):
            # Fallback to full send if sizes differ
            return current
        
        delta = bytes(a ^ b for a, b in zip(current, previous))
        
        # Check if delta is more compressible (higher sparsity)
        zeros = delta.count(0)
        if zeros > len(delta) * 0.5:  # >50% zeros = use delta
            return b"DELTA:" + delta
        else:
            return current
    
    def delta_decompress(self, delta_or_full: bytes, previous: bytes) -> bytes:
        """
        Reconstructs full weights from delta.
        """
        if delta_or_full.startswith(b"DELTA:"):
            delta = delta_or_full[6:]
            return bytes(a ^ b for a, b in zip(delta, previous))
        else:
            return delta_or_full

def run_swarm_demo():
    """Demo function to test swarm operations."""
    client = SwarmClient(node_id="demo_node")
    
    # Simulate broadcasting
    fake_weights = b'\x00' * 1024  # Placeholder
    client.broadcast_epiphany(fake_weights, fidelity_score=0.98)
    
    # Check for incoming
    received = client.receive_epiphanies()
    print(f"Received: {len(received)} epiphanies")

if __name__ == "__main__":
    run_swarm_demo()
