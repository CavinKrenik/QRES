from qres_core import qres_encode, qres_decode

def qres_encode_text(text):
    return qres_encode([ord(c) for c in text])

def qres_decode_text(start, encoded):
    return ''.join(chr(v) for v in qres_decode(start, encoded))