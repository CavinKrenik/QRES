
from collections import Counter

def analyze_pattern(relations):
    """Returns frequency count of each relation type."""
    counts = Counter()
    for r in relations:
        if r.startswith("↑"):
            counts["↑"] += 1
        elif r.startswith("↓"):
            counts["↓"] += 1
        elif r.startswith("="):
            counts["=init"] += 1
        elif r == "=":
            counts["="] += 1
        else:
            counts["?"] += 1
    return counts
