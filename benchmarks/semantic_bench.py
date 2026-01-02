
from qres.llm_predictor import SemanticPredictor
import time

def bench_llm_on_code(code_snippet: str):
    print(f"--- Semantic Benchmark ---")
    print(f"Context: {code_snippet[:50]}...")
    
    start = time.time()
    predictor = SemanticPredictor()
    # Warmup
    _ = predictor.predict("def test(): pass", 10)
    
    warmup = time.time()
    print(f"Model Load & Warmup: {warmup - start:.2f}s")
    
    hint = predictor.predict(code_snippet)
    elapsed = time.time() - warmup
    
    print(f"Prediction: {hint}")
    print(f"Inference Time: {elapsed:.2f}s")
    
    # Analyze gain
    original_len = len(code_snippet) + len(hint) # Approximation of what we'd need to encode
    # "Matches"
    # Logic: if hint matches reality, we save bytes.
    # For now just printing metrics.
    
if __name__ == "__main__":
    snippet = """
def quicksort(arr):
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
"""
    bench_llm_on_code(snippet)
