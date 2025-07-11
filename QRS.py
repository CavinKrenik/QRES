def binary_to_qres(binary_str):
    """Converts a binary string to QRES-style relational pattern."""
    relations = []
    for i in range(len(binary_str) - 1):
        current = int(binary_str[i])
        next_val = int(binary_str[i + 1])
        if next_val > current:
            relations.append("↑")
        elif next_val < current:
            relations.append("↓")
        else:
            relations.append("=")
    return relations

def analyze_pattern(relations):
    """Basic analysis of the pattern."""
    counts = {"↑": 0, "↓": 0, "=": 0}
    for r in relations:
        if r in counts:
            counts[r] += 1
    return counts

def compress_qres_pattern(relations):
    """Compresses repeated patterns using run-length encoding logic."""
    if not relations:
        return []

    compressed = []
    count = 1
    prev = relations[0]

    for current in relations[1:]:
        if current == prev:
            count += 1
        else:
            if count > 1:
                compressed.append(f"repeat({prev},{count})")
            else:
                compressed.append(prev)
            prev = current
            count = 1

    # Append the final group
    if count > 1:
        compressed.append(f"repeat({prev},{count})")
    else:
        compressed.append(prev)

    return compressed

def main():
    binary_input = input("Enter a binary string (e.g., 1010110): ").strip()
    qres_output = binary_to_qres(binary_input)
    print(f"QRES Relations: {' '.join(qres_output)}")

    pattern_stats = analyze_pattern(qres_output)
    print("Pattern Analysis:")
    for k, v in pattern_stats.items():
        print(f"  {k}: {v} times")

    compressed_output = compress_qres_pattern(qres_output)
    print(f"Compressed QRES: {' '.join(compressed_output)}")

if __name__ == "__main__":
    main()

