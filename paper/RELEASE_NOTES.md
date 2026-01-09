# 📄 TensorSwarm: Biologically-Inspired Secure Federated Learning for Edge IoT

**Complete publication release** including academic paper, implementation, benchmarks, and reproducibility artifacts.

## 📖 Paper

**[Download PDF →](https://github.com/CavinKrenik/QRES/raw/main/paper/TensorSwarm__Biologically_Inspired_Secure_Federated_Learning_for_Edge_IoT_Devices.pdf)**

- **Title:** TensorSwarm: Biologically-Inspired Secure Federated Learning for Edge IoT Devices
- **Author:** Cavin Krenik (Olympic College)
- **Pages:** 14
- **Keywords:** Federated Learning, Edge AI, SNNs, Differential Privacy, Secure Aggregation

## 🔬 Key Contributions

1. **Fixed-point SNN ensemble** for deterministic temporal prediction (Q16.16 arithmetic)
2. **Complete privacy stack:** Differential Privacy + Secure Aggregation + Zero-Knowledge Proofs
3. **Empirical evaluation:** Azure VMs (10-100 nodes), real IoT datasets

## 📊 Experimental Results

- **Compression:** 22:1 ratio on time-series sensor data
- **Privacy overhead:** 20-30% for full security stack
- **Byzantine tolerance:** Up to 49% malicious nodes (Krum aggregation)
- **Regime change recovery:** 3-4 rounds average
- **Scalability:** Tested up to 100 Azure B1s VMs

## 🛠️ Implementation

- **Language:** Rust (no_std compatible for bare-metal)
- **Security:** ed25519 signatures, ECDH masking, Pedersen commitments
- **Networking:** libp2p for P2P swarm coordination
- **Deployment:** Docker + Azure VM scripts included

## 📦 Contents
```
paper/
├── TensorSwarm__...pdf (Publication PDF)
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

## 🎯 Target Venues

- **Primary:** FLICS 2026 (June 9-12, Valencia, Spain)
- **Categories:** cs.DC, cs.LG, cs.CR

## 📚 Citation
```bibtex
@software{krenik2026tensorswarm,
  author       = {Krenik, Cavin},
  title        = {{TensorSwarm: Biologically-Inspired Secure 
                   Federated Learning for Edge IoT Devices}},
  month        = jan,
  year         = 2026,
  publisher    = {Zenodo},
  version      = {v15.2-paper},
  doi          = {10.5281/zenodo.18193906},
  url          = {https://doi.org/10.5281/zenodo.18193906}
}
```

## 📧 Contact

- **Author:** Cavin Krenik
- **Institution:** Olympic College, Shelton, WA, USA
- **GitHub:** https://github.com/CavinKrenik/QRES

---

**🚀 Ready for submission to FLICS 2026 and archival on Zenodo!**
