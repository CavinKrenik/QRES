import importlib.util
import sys
from typing import Union, Optional, Literal
import numpy as np
import io

# Import the Rust extension
# It is now built as a submodule: qres.qres_rust
try:
    from . import qres_rust
except ImportError:
    # Fallback/Debug
    import qres_rust
    
# Expose bindings directly for advanced users
encode_buffer = qres_rust.encode_buffer
decode_bytes = qres_rust.decode_bytes

class QRESError(Exception):
    """Base exception for QRES errors."""
    pass

class QRES:
    """
    QRES (Quantum-Relational Encoding System) Codec.
    High-performance, bit-packed delta encoding for time-series and predictable data.
    """
    
    @staticmethod
    def compress(data: Union[bytes, bytearray, np.ndarray]) -> bytes:
        """
        Compress data using QRES v2 (Bit-Packed + Delta).
        Supports: bytes, bytearray, memoryview, numpy.ndarray.
        """
        # Phase 5 Optimization: Direct Buffer Protocol passing
        # Rust handles the buffer pointer directly.
        try:
            return encode_buffer(data)
        except Exception as e:
            # Fallback for unexpected types or non-contiguous buffers
            if isinstance(data, str):
                return encode_buffer(data.encode('utf-8'))
            raise QRESError(f"Compression failed: {e}")

    @staticmethod
    def decompress(data: Union[bytes, bytearray]) -> bytes:
        """
        Decompress QRES v2 data.
        """
        if not isinstance(data, (bytes, bytearray)):
             raise TypeError(f"Unsupported type {type(data)}. Expected bytes.")
             
        try:
            return decode_bytes(data)
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
