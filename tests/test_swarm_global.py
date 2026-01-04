"""
Tests for Global P2P Swarm functionality.
"""

import os
import sys
import pytest
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))
from qres.swarm_cli import SwarmClient

@pytest.fixture
def swarm_client():
    """Create a test swarm client."""
    client = SwarmClient(node_id="test_node")
    return client

def test_outbox_creation(swarm_client):
    """Test that outbox directory is created."""
    assert Path("quantum_outbox").exists()

def test_inbox_creation(swarm_client):
    """Test that inbox directory is created."""
    assert Path("quantum_inbox").exists()

def test_broadcast_epiphany(swarm_client):
    """Test broadcasting an epiphany to the outbox."""
    fake_weights = b'\x42' * 256
    filename = swarm_client.broadcast_epiphany(fake_weights, fidelity_score=0.95)
    
    assert Path(filename).exists()
    
    # Verify content
    with open(filename, 'rb') as f:
        data = f.read()
    
    assert data.startswith(b"QRES_EPIPHANY_V1")
    assert b"test_node" in data
    
    # Cleanup
    Path(filename).unlink()

def test_delta_compression(swarm_client):
    """Test Fed2Com-style delta compression."""
    previous = b'\x00' * 100
    current = b'\x00' * 50 + b'\xFF' * 50  # 50% change
    
    delta = swarm_client.delta_compress(current, previous)
    
    # Should use delta if efficient
    # In this case, exactly 50% zeros, may or may not trigger
    assert len(delta) <= len(current) + 6  # +6 for DELTA: prefix at most

def test_delta_decompress(swarm_client):
    """Test delta decompression roundtrip."""
    previous = b'\x00' * 100
    current = b'\x00' * 80 + b'\xFF' * 20  # 80% same
    
    delta = swarm_client.delta_compress(current, previous)
    
    if delta.startswith(b"DELTA:"):
        reconstructed = swarm_client.delta_decompress(delta, previous)
        assert reconstructed == current

def test_receive_empty_inbox(swarm_client):
    """Test receiving from empty inbox."""
    received = swarm_client.receive_epiphanies()
    # Should not crash, returns list (possibly empty)
    assert isinstance(received, list)
