
import os
import sys
import logging
from qres_core import encode_qres, decode_qres

CHUNK_SIZE = 1024 * 1024  # 1MB

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

def stream_compress_qres(input_path, output_path):
    with open(input_path, 'rb') as infile, open(output_path, 'w') as outfile:
        chunk_index = 0
        while True:
            chunk = infile.read(CHUNK_SIZE)
            if not chunk:
                break
            encoded = encode_qres(chunk)
            outfile.write(f"---CHUNK:{chunk_index}---\n")
            outfile.write(encoded + '\n')
            chunk_index += 1
            logging.info(f"Compressed chunk {chunk_index}")
    logging.info(f"[✓] Chunked compression completed: {output_path}")

def stream_decompress_qres(input_path, output_path):
    with open(input_path, 'r') as infile, open(output_path, 'wb') as outfile:
        current_encoded = []
        for line in infile:
            if line.startswith('---CHUNK:'):
                if current_encoded:
                    decoded = decode_qres(''.join(current_encoded))
                    outfile.write(decoded)
                    current_encoded = []
            else:
                current_encoded.append(line.strip())
        if current_encoded:
            decoded = decode_qres(''.join(current_encoded))
            outfile.write(decoded)
    logging.info(f"[✓] Chunked decompression completed: {output_path}")

def main():
    if len(sys.argv) != 4:
        print("Usage:")
        print("  python qres_chunked_compressor.py compress <input_file> <output_file.qres>")
        print("  python qres_chunked_compressor.py decompress <input_file.qres> <output_file>")
        return

    mode, input_path, output_path = sys.argv[1], sys.argv[2], sys.argv[3]

    if not os.path.exists(input_path):
        print(f"[×] File not found: {input_path}")
        return

    if mode == 'compress':
        stream_compress_qres(input_path, output_path)
    elif mode == 'decompress':
        stream_decompress_qres(input_path, output_path)
    else:
        print(f"[×] Invalid mode: {mode}")

if __name__ == "__main__":
    main()
