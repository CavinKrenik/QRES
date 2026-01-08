## other/

This folder contains edge-case and container formats used to test QRES robustness, fallback behavior, and byte-perfect fidelity.

Files here should not be assumed to contain recoverable structure.

### Contents

- **sample.pdf**: A minimal PDF document (Text+Metadata). Tests safe round-tripping of container formats (Fidelity > Ratio).
- **mixed_file.csv**: A CSV with mixed entropy (timestamps vs values) to test relational encoding rewards.
- **audio_snippet.wav**: A synthetic sine wave to test non-visual/non-text media handling.
- **compressed_archive.gz**: A pre-compressed Gzip file. Tests heuristics for "Don't compress again".

### Testing Focus

- **Fidelity Check**: `python verify_fidelity.py` should show 100% matches.
- **Tensor Mode**: `python qres_tensor_cli.py data/other/sample.pdf --mode tensor` should likely fallback to binary/spectral graph.
- **MetaBrain Training**: These files provide negative examples (high entropy or opaque structure) to teach the agent when *not* to use expensive strategies.
