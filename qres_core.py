import re

def qres_encode(sequence):
    if not sequence:
        return None, ''
    if not all(isinstance(x, (int, float)) for x in sequence):
        raise TypeError("QRES only supports numeric sequences.")

    start = sequence[0]
    encoded = []
    prev = start
    for val in sequence[1:]:
        diff = val - prev
        if diff == 0:
            encoded.append('=')
        elif diff == 1:
            encoded.append('U')
        elif diff == -1:
            encoded.append('D')
        elif diff > 0:
            encoded.append('U' + str(diff))
        else:
            encoded.append('D' + str(-diff))
        prev = val

    encoded_str = ''.join(encoded)
    encoded_str = re.sub(r'=+', lambda m: '=' + str(len(m.group(0))) if len(m.group(0)) > 1 else '=', encoded_str)

    return start, encoded_str

def qres_decode(start, encoded):
    decoded = [start]
    current = start
    i = 0
    while i < len(encoded):
        sym = encoded[i]
        if sym == '=':
            j = i + 1
            num = ''
            while j < len(encoded) and encoded[j].isdigit():
                num += encoded[j]
                j += 1
            count = int(num) if num else 1
            decoded.extend([current] * count)
            i = j
        elif sym in ('U', 'D'):
            j = i + 1
            num = ''
            while j < len(encoded) and encoded[j].isdigit():
                num += encoded[j]
                j += 1
            delta = int(num) if num else 1
            if sym == 'D':
                delta = -delta
            current += delta
            decoded.append(current)
            i = j
        else:
            raise ValueError(f"Unknown symbol '{sym}' at position {i}")
    return decoded