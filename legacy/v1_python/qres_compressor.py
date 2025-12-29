
import sys
import os
from qres_core import encode_qres, decode_qres

def compress_file(input_path, output_path):
    with open(input_path, 'rb') as f:
        data = f.read()
    encoded = encode_qres(data)
    with open(output_path, 'w') as f:
        f.write(encoded)
    print(f"[✓] Compressed: {input_path} → {output_path}")

def decompress_file(input_path, output_path):
    with open(input_path, 'r') as f:
        encoded = f.read()
    decoded = decode_qres(encoded)
    with open(output_path, 'wb') as f:
        f.write(decoded)
    print(f"[✓] Decompressed: {input_path} → {output_path}")

def main():
    if len(sys.argv) != 4:
        print("Usage:")
        print("  python qres_compressor.py compress <input_file> <output_file.qres>")
        print("  python qres_compressor.py decompress <input_file.qres> <output_file>")
        return

    mode, input_path, output_path = sys.argv[1], sys.argv[2], sys.argv[3]

    if mode == 'compress':
        compress_file(input_path, output_path)
    elif mode == 'decompress':
        decompress_file(input_path, output_path)
    else:
        print(f"Invalid mode: {mode}")

if __name__ == "__main__":
    main()
