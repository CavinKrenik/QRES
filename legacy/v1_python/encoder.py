
def byte_to_transitions(data):
    """Convert byte array to transition representation."""
    transitions = []
    prev = None
    for byte in data:
        if prev is None:
            transitions.append(f"={byte}")
        else:
            diff = byte - prev
            if diff == 0:
                transitions.append("=")
            elif diff > 0:
                transitions.append(f"↑{diff}")
            else:
                transitions.append(f"↓{-diff}")
        prev = byte
    return transitions

def transitions_to_bytes(transitions):
    """Convert transitions back to byte array."""
    bytes_out = []
    prev = None
    for t in transitions:
        if t.startswith("="):
            val = int(t[1:]) if len(t) > 1 else (prev if prev is not None else 0)
        elif t.startswith("↑"):
            val = prev + int(t[1:])
        elif t.startswith("↓"):
            val = prev - int(t[1:])
        elif t == "=":
            val = prev
        else:
            raise ValueError(f"Unknown transition: {t}")
        bytes_out.append(val)
        prev = val
    return bytes(bytes_out)
