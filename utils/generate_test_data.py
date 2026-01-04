import os
import gzip
import wave
import struct
import math

# Ensure data/other exists
os.makedirs('data/other', exist_ok=True)

print("Generating data/other/mixed_file.csv...")
# 1. CSV
csv_content = """id,timestamp,label,value,note
1,2026-01-01T12:00:00Z,alpha,12.45,OK
2,2026-01-01T12:00:15Z,beta,13.01,EDGE_CASE
3,2026-01-01T12:00:30Z,gamma,11.98,OK
4,2026-01-01T12:00:45Z,alpha,12.50,OK
5,2026-01-01T12:01:00Z,delta,14.20,FAIL
"""
with open('data/other/mixed_file.csv', 'w') as f:
    f.write(csv_content)

print("Generating data/other/compressed_archive.gz...")
# 2. GZ
with gzip.open('data/other/compressed_archive.gz', 'wb') as f:
    f.write(b"Repeat " * 1000)

print("Generating data/other/audio_snippet.wav...")
# 3. WAV (1 sec 440Hz sine)
with wave.open('data/other/audio_snippet.wav', 'w') as w:
    w.setnchannels(1)
    w.setsampwidth(2)
    w.setframerate(44100)
    data = []
    for i in range(44100):
        value = int(32767.0 * math.sin(2 * math.pi * 440 * i / 44100))
        data.append(struct.pack('<h', value))
    w.writeframes(b''.join(data))

print("Generating data/other/sample.pdf...")
# 4. PDF (Minimal valid structure)
# Note: Offsets in xref are calculated roughly, file might not open in strict readers 
# but satisfies "container" definition for QRES testing.
pdf_content = (
    b"%PDF-1.1\n"
    b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"
    b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
    b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> /MediaBox [0 0 595 842] /Contents 4 0 R >>\nendobj\n"
    b"4 0 obj\n<< /Length 20 >>\nstream\nBT\n/F1 24 Tf\n(Hello QRES) Tj\nET\nendstream\nendobj\n"
    b"xref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000300 00000 n \n"
    b"trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n380\n%%EOF\n"
)
with open('data/other/sample.pdf', 'wb') as f:
    f.write(pdf_content)

print("Done.")
