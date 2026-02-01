# QRES BFT Paper Assets

IEEE conference paper: **"QRES: Deterministic Byzantine Fault Tolerance for Resource-Constrained Edge Learning"**

## Files

| File | Description |
|------|-------------|
| `paper.tex` | LaTeX source (IEEEtran format) |
| `figures/` | All figures for the paper |

## Figures

| Figure | Filename | Description |
|--------|----------|-------------|
| **Fig 1** | `fig1_attack_rejection.png` | Basic attack rejection demo |
| **Fig 2** | `fig2_three_scenarios.png` | Three attack scenarios (A, B, C) |
| **Fig 3** | `fig3_tolerance_curve.png` | Byzantine tolerance threshold |
| **Fig 4** | `fig4_temporal_evolution.png` | Temporal evolution (T=0,10,20,50) |

## Building

### Overleaf
1. Upload `paper.tex` and `figures/` folder
2. Set compiler to pdfLaTeX
3. Compile

### Local
```bash
pdflatex paper.tex
pdflatex paper.tex  # Run twice for references
```

## Regenerating Figures

All visualization scripts are in `../../tools/`:

```bash
python ../../tools/visualize_attack.py          # Fig 1
python ../../tools/visualize_robustness.py      # Fig 2
python ../../tools/visualize_tolerance.py       # Fig 3
python ../../tools/figure3_static_evolution.py  # Fig 4
```

## Citation

```bibtex
@inproceedings{krenik2026qres,
  title={QRES: Deterministic Byzantine Fault Tolerance for Resource-Constrained Edge Learning},
  author={Krenik, Cavin},
  booktitle={Proc. IEEE Conference},
  year={2026}
}
```
