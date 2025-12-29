import numpy as np
import requests
import qres
# from qres_rust import meta_brain_debug
# I have to replicate the prediction logic or trust the rust outcome.
# I'll just rely on `qres` to tell me what mode it picked if I could.
# Qres output format has a header. I can inspect the header to see what ID was picked!
# `qres_inspect.py` exists!

def download_shakespeare():
    url = "https://www.gutenberg.org/files/100/100-0.txt"
    try:
        r = requests.get(url)
        return r.content[:4096] # Just 4KB
    except:
        return b" " * 4096

data = download_shakespeare()
chunk = np.frombuffer(data, dtype=np.uint8)

mean = np.mean(chunk)
var = np.var(chunk)
counts = np.bincount(chunk, minlength=256)
probs = counts[counts > 0] / len(chunk)
entropy = -np.sum(probs * np.log2(probs))
diffs = np.diff(chunk.astype(np.int16))
zcr = np.sum(np.abs(diffs) > 10) / len(chunk) 

print(f"Features: Mean={mean:.4f}, Var={var:.4f}, Entropy={entropy:.4f}, ZCR={zcr:.4f}")
print("Run 'qres-cli compress ... --report' ? No report logic removed.")
print("But I can compress a small file and inspect it.")

with open("temp_shake.txt", "wb") as f:
    f.write(data)
