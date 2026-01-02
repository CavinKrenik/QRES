# QRES Python API Reference (v5.1)

## Core Functions

### `encode_bytes(data: bytes, predictor_id: int = 0, weights: Optional[bytes] = None) -> bytes`
Compresses a raw byte stream using the QRES v5 engine.

- **data**: The input bytes to compress.
- **predictor_id**: Legacy parameter (0=Auto).
- **weights**: Optional pre-trained weight tensor (40 bytes) for the neural mixer.
- **Returns**: A compressed byte object containing the QRES v5 header and solid stream.

### `decode_bytes(data: bytes, predictor_id: int = 0, weights: Optional[bytes] = None) -> bytes`
Decompresses a QRES v5 stream.

- **data**: The compressed QRES bytes.
- **weights**: Must match the weights used during compression (if any).
- **Returns**: The original raw bytes.

### `SemanticPredictor` (Experimental v6.0)
Located in `qres.llm_predictor`. Uses local LLMs to estimate entropy for text/code blocks.

```python
from qres.llm_predictor import SemanticPredictor
llm = SemanticPredictor("models/codellama-7b.gguf")
perplexity = llm.predict_block("def main():")
```

## Training

### `train_model(file_pattern: str, chunk_size: int = 65536) -> bytes`
Trains the "Living Brain" on a set of files to discover optimal weights.

- **file_pattern**: Glob pattern (e.g., `logs/*.log`).
- **chunk_size**: The attention span for the learner.
- **Returns**: A 40-byte weight tensor that can be passed to `encode_bytes`.

## Constants

- `QRES_MAGIC`: `b'QRES'`
- `QRAR_MAGIC`: `b'QRAR'` (Solid Archive)
- `VERSION`: `5.1.0`
