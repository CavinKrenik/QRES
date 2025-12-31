#!/usr/bin/env python3
"""
QRES Release Preparation Script
Trains the neural meta-selector and prepares for release.
"""

import subprocess
import sys
import os

def main():
    print("🚀 QRES Release Preparation")
    print("=" * 40)

    # Check if in venv
    if not hasattr(sys, 'real_prefix') and sys.base_prefix == sys.prefix:
        print("❌ Not in a virtual environment. Please activate venv first.")
        sys.exit(1)

    # Train the meta-selector
    print("\n🧠 Training Neural Meta-Selector...")
    result = subprocess.run([sys.executable, "ai/train_meta.py"], cwd=os.getcwd())
    if result.returncode != 0:
        print("❌ Training failed")
        sys.exit(1)

    print("\n✅ Release preparation complete!")
    print("Next: cargo build --release")

if __name__ == "__main__":
    main()