# Test Data

This directory contains diverse sample data for training the QRES MetaBrain and verifying compression fidelity.

## Structure

- **iot/**: Telemetry and sensor logs (Structured/Time-Series).
- **text/**: Code, prose, and structured text (Semantic).
- **images/**: Placeholder for image samples (to be added).
- **binary/**: High-entropy random data and pre-compressed archives.
- **other/**: Multimodal or edge-case files.

## Usage

Use these files for benchmarking or training:

```bash
python qres_tensor_cli.py data/iot/iot_telemetry_sample.dat --mode standard
python ai/train_compression_ppo.py --data-dir data/
```
