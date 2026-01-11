# 📄 QRES v15.2.0 - Publication Release

**Published research paper** with implementation, benchmarks, and reproducibility artifacts.

## 📖 Paper

**[Download PDF →](https://github.com/CavinKrenik/QRES/raw/main/paper/QRES__Biologically_Inspired_Secure_Federated_Learning_for_Edge_IoT_Devices.pdf)**

- **Title:** QRES: Biologically-Inspired Secure Federated Learning for Edge IoT Devices
- **Author:** Cavin Krenik (Olympic College)
- **Pages:** 5
- **Keywords:** Federated Learning, Edge AI, Spiking Neural Networks, Secure Aggregation, Differential Privacy, Zero-Knowledge Proofs, IoT Compression

## 🔬 Key Contributions

1. **Fixed-point SNN ensemble** for deterministic temporal prediction (Q16.16 arithmetic)
2. **Complete privacy stack:** Differential Privacy + Secure Aggregation + Zero-Knowledge Proofs
3. **Empirical evaluation:** Azure VMs (10-100 nodes), real IoT datasets

## 📊 Experimental Results

- **Compression:** 48:1 on synthetic data, 22:1 on IoT telemetry
- **Privacy overhead:** 3.1× runtime for full security stack (DP + Secure Agg + ZK)
- **Byzantine tolerance:** Up to 45% malicious nodes (Krum aggregation)
- **Regime change recovery:** ~20 seconds (5-12 rounds depending on severity)
- **Scalability:** 10-100 nodes on Azure B1s VMs, >85% success rate

## 🛠️ Implementation

- **Language:** Rust (no_std compatible for bare-metal)
- **Security:** ed25519 signatures, ECDH masking, Pedersen commitments
- **Networking:** libp2p for P2P swarm coordination
- **Deployment:** Docker + Azure VM scripts included

## 📦 Contents
```
paper/
├── QRES__...pdf (Publication PDF)
├── paper.tex (LaTeX source)
├── references.bib (14+ citations)
└── figures/ (5 publication-quality figures)

qres_rust/
├── qres_core/ (Rust implementation)
├── qres_daemon/ (P2P coordinator)
└── security modules (DP, Secure Agg, ZK)

reproducibility/
├── Dockerfile
└── scripts/
```

## 🎯 Categories

- cs.DC, cs.LG, cs.CR

## 📚 Citation
```bibtex
@software{krenik2026qres,
  author       = {Krenik, Cavin},
  title        = {{QRES: Biologically-Inspired Secure 
                   Federated Learning for Edge IoT Devices}},
  month        = jan,
  year         = 2026,
  publisher    = {Zenodo},
  version      = {v15.2.0},
  doi          = {10.5281/zenodo.18194636},
  url          = {https://doi.org/10.5281/zenodo.18194636}
}
```

## 📧 Contact

- **Author:** Cavin Krenik
- **Institution:** Olympic College, Shelton, WA, USA
- **GitHub:** https://github.com/CavinKrenik/QRES

---

**🚀 Published research with reproducibility artifacts on Zenodo!**
