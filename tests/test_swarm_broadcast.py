
import os
import time
import pytest
import shutil
from pathlib import Path

# Path Hack
import sys
sys.path.append(os.getcwd())

@pytest.fixture
def clean_env():
    # Setup
    if os.path.exists("quantum_outbox"):
        shutil.rmtree("quantum_outbox")
    os.makedirs("quantum_outbox")
    yield
    # Teardown
    if os.path.exists("quantum_outbox"):
        shutil.rmtree("quantum_outbox")

def test_cli_broadcast(clean_env):
    print("\n[Test] CLI Broadcast Integration")
    
    # 1. Create dummy input
    val = b"Remote Swarm Data"
    with open("test_data.bin", "wb") as f:
        f.write(val)
        
    # 2. Run CLI with --broadcast
    ret = os.system("python qres_quantum_cli.py test_data.bin --broadcast")
    assert ret == 0
    
    # 3. Verify Outbox has content
    files = os.listdir("quantum_outbox")
    assert len(files) == 1
    assert files[0].endswith(".qres")
    assert files[0].startswith("qv7_")
    
    print(f"Verified broadcast file: {files[0]}")
    
    # Cleanup local source
    os.remove("test_data.bin")

if __name__ == "__main__":
    pytest.main([__file__])
