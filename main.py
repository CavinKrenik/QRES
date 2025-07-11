from qres_core import qres_encode, qres_decode
from qres_text import qres_encode_text, qres_decode_text
from qres_image import qres_encode_image, qres_decode_image
import numpy as np
import json
import argparse
import os
from datetime import datetime

def save_log(entry, log_path='qres_log.jsonl'):
    with open(log_path, 'a') as f:
        json.dump(entry, f)
        f.write('\n')

def compress_text(text, output_path):
    start, encoded = qres_encode_text(text)
    timestamp = datetime.utcnow().isoformat()
    entry = {
        "type": "text",
        "timestamp": timestamp,
        "start": start,
        "encoded": encoded
    }
    save_log(entry)
    with open(output_path, 'w') as f:
        f.write(json.dumps(entry, indent=2))

def decompress_text(input_path):
    with open(input_path, 'r') as f:
        data = json.load(f)
    return qres_decode_text(data["start"], data["encoded"])

def compress_numpy_array(input_path, output_path):
    array = np.load(input_path)
    shape = array.shape
    channels = qres_encode_image(array)
    timestamp = datetime.utcnow().isoformat()
    entry = {
        "type": "image",
        "timestamp": timestamp,
        "shape": shape,
        "channels": channels
    }
    save_log(entry)
    with open(output_path, 'w') as f:
        json.dump(entry, f, indent=2)

def decompress_numpy_array(input_path, output_path):
    with open(input_path, 'r') as f:
        data = json.load(f)
    image = qres_decode_image(data["channels"], tuple(data["shape"]))
    np.save(output_path, image)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="QRES v4 Compression Tool")
    parser.add_argument('mode', choices=['compress', 'decompress'], help='compress or decompress')
    parser.add_argument('--type', choices=['text', 'image'], required=True, help='Type of data')
    parser.add_argument('--input', required=True, help='Input file path')
    parser.add_argument('--output', required=True, help='Output file path')

    args = parser.parse_args()

    if args.type == 'text':
        if args.mode == 'compress':
            with open(args.input, 'r') as f:
                text = f.read()
            compress_text(text, args.output)
        else:
            decoded_text = decompress_text(args.input)
            with open(args.output, 'w') as f:
                f.write(decoded_text)
    elif args.type == 'image':
        if args.mode == 'compress':
            compress_numpy_array(args.input, args.output)
        else:
            decompress_numpy_array(args.input, args.output)