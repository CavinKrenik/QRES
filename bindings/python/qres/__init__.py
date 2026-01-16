from typing import Union, Optional, Literal
import numpy as np
import io

__version__ = "18.0.0"

# Import the Rust extension (module name fixed to qres.qres_rust)
from . import qres_rust
    
# Expose bindings directly for advanced users
encode_bytes = qres_rust.encode_bytes
decode_bytes = qres_rust.decode_bytes
get_residuals = qres_rust.get_residuals_py
compress_matrix_v1 = qres_rust.compress_matrix_v1

class QRESError(Exception):
    """Base exception for QRES errors."""
    pass

class QRES:
    """
    QRES (Quantum-Relational Encoding System) Codec.
    High-performance, bit-packed delta encoding for time-series and predictable data.
    """
    
    @staticmethod
    def compress(data: Union[bytes, bytearray, np.ndarray], predictor_id: int = 0) -> bytes:
        """
        Compress data using QRES v2 (Bit-Packed + Delta).
        Supports: bytes, bytearray, memoryview, numpy.ndarray.
        Predictor ID: 0=Previous, 1=Linear ... 255=Smart (Auto-Detect).
        """
        try:
            # Phase 9: V3 Streamable API (requires predictor_id)
            if isinstance(data, (bytes, bytearray)):
                 return encode_bytes(data, predictor_id)
            elif isinstance(data, np.ndarray):
                 return encode_bytes(data.tobytes(), predictor_id)
            else:
                 return encode_bytes(memoryview(data).tobytes(), predictor_id)
        except Exception as e:
            # Fallback for unexpected types or non-contiguous buffers
            if isinstance(data, str):
                return encode_bytes(data.encode('utf-8'), predictor_id)
            raise QRESError(f"Compression failed: {e}")

    @staticmethod
    def decompress(data: Union[bytes, bytearray], predictor_id: int = 0) -> bytes:
        """
        Decompress QRES v2 data.
        """
        if not isinstance(data, (bytes, bytearray)):
             raise TypeError(f"Unsupported type {type(data)}. Expected bytes.")
             
        try:
            return decode_bytes(data, predictor_id)
        except Exception as e:
            raise QRESError(f"Decompression failed: {e}")

# Helper aliases
compress = QRES.compress
decompress = QRES.decompress

class QRESFile(io.BufferedIOBase):
    """
    File object for reading/writing QRES compressed files.
    """
    def __init__(self, filename, mode="rb"):
        self._file = open(filename, mode)
        self._mode = mode
            
    def read(self, size=-1):
        raw = self._file.read()
        return decompress(raw)
        
    def write(self, data):
        compressed = compress(data)
        self._file.write(compressed)
        
    def close(self):
        self._file.close()
        
    def __enter__(self):
        return self
        
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

def open(filename, mode="rb"):
    """
    Open a QRES compressed file in binary mode.
    """
    return QRESFile(filename, mode)
