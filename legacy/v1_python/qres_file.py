import zlib
import json
import struct
from datetime import datetime

MAGIC = b'QRES'
VERSION = 1

def save_qres_file(data_type, start, encoded, output_path, metadata=None):
    """
    Stores the QRES compressed data to a binary .qres format.
    Includes a header with type and timestamp.
    """
    encoded_bytes = encoded.encode()
    compressed = zlib.compress(encoded_bytes)

    header = {
        "type": data_type,
        "version": VERSION,
        "start": start,
        "timestamp": datetime.utcnow().isoformat(),
        "metadata": metadata or {}
    }

    header_json = json.dumps(header).encode()
    header_length = len(header_json)

    with open(output_path, 'wb') as f:
        f.write(MAGIC)
        f.write(struct.pack('>I', header_length))
        f.write(header_json)
        f.write(compressed)

def load_qres_file(input_path):
    """
    Loads and decompresses a .qres binary file.
    Returns: header dict, decoded string
    """
    with open(input_path, 'rb') as f:
        magic = f.read(4)
        if magic != MAGIC:
            raise ValueError("Invalid QRES file format")
        header_length = struct.unpack('>I', f.read(4))[0]
        header = json.loads(f.read(header_length))
        compressed_data = f.read()
        encoded = zlib.decompress(compressed_data).decode()
    return header, encoded