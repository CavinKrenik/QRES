import qres as qres_rust

# Test adaptive ANS compression
data = b'Hello World! ' * 100
print(f'Original: {len(data)} bytes')

compressed = qres_rust.encode_bytes(data, 0, None)
print(f'Compressed: {len(compressed)} bytes')
print(f'Compression Ratio: {len(compressed)/len(data):.2%}')

decompressed = qres_rust.decode_bytes(compressed, 0, None)
print(f'Decompressed: {len(decompressed)} bytes')

if data == decompressed:
    print('✅ Round-trip PASSED')
else:
    print('❌ Round-trip FAILED')
    print(f'Data mismatch: original {len(data)} bytes, got {len(decompressed)} bytes')
