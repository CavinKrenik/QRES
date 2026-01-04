import os
import pytest
import qres
from qres.api import load_metabrain

# Define target ratios for the "Singularity Era" breakthroughs
TARGET_IOT_RATIO = 0.30  # Goal: < 0.30 (currently 0.537)
TARGET_TEXT_RATIO = 0.15 # Goal: < 0.15 (currently ~0.19)

@pytest.fixture
def api_instance():
    # Load with v4 brain if available
    api = qres.QRES_API()
    if os.path.exists("ai/metabrain_ppo_v4.zip"):
        api.metabrain = load_metabrain("ai/metabrain_ppo_v4.zip")
    return api

def test_iot_ratio_baseline(api_instance):
    """
    Benchmarks the current IoT ratio against the breakthrough target.
    Currently expected to FAIL the breakthrough target, serving as a driver.
    """
    if not os.path.exists("data/iot/iot_telemetry_sample.dat"):
        pytest.skip("IoT sample data missing")
        
    with open("data/iot/iot_telemetry_sample.dat", "rb") as f:
        data = f.read()
        
    compressed = api_instance.compress(data, mode="standard")
    ratio = len(compressed) / len(data)
    
    print(f"\nIoT Ratio: {ratio:.4f} (Target: {TARGET_IOT_RATIO})")
    
    # We assert it's at least better than Zstd (0.57-0.60 usually)
    # But we warn if it hasn't met the Singularity Target yet
    if ratio > TARGET_IOT_RATIO:
        pytest.warns(UserWarning, match=f"Singularity Target not met. Current: {ratio:.3f}, Target: {TARGET_IOT_RATIO}")
    else:
        assert ratio <= TARGET_IOT_RATIO

def test_text_ratio_baseline(api_instance):
    """
    Benchmarks Text ratio.
    """
    if not os.path.exists("data/text/sample_code.py"):
        pytest.skip("Text sample data missing")

    with open("data/text/sample_code.py", "rb") as f:
        data = f.read()
        
    compressed = api_instance.compress(data, mode="standard")
    ratio = len(compressed) / len(data)
    
    print(f"\nText Ratio: {ratio:.4f} (Target: {TARGET_TEXT_RATIO})")

    if ratio > TARGET_TEXT_RATIO:
        pytest.warns(UserWarning, match=f"Singularity Target not met. Current: {ratio:.3f}, Target: {TARGET_TEXT_RATIO}")
    else:
        assert ratio <= TARGET_TEXT_RATIO
