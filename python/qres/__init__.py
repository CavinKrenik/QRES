import importlib.util
import sys
from typing import Union, Optional, Literal
import numpy as np
import io

# Import the Rust extension
# Maturin makes it available as `qres_rust` alongside this package
try:
    import qres_rust
except ImportError:
    # Fallback for development/local builds if path isn't perfect
    import qres_rust
    
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
        Compress data using QRES v2 protocol.
        
        Args:
            data: Input data (bytes, bytearray, or numpy array). 
                  If numpy array, it must be flattened and converted to bytes.
        """
        if isinstance(data, np.ndarray):
            # Ensure 1D and contiguous layout for zero-copy efficiency if possible
            # QRES expects byte-stream, so we view as uint8
            if not data.flags['C_CONTIGUOUS']:
                data = np.ascontiguousarray(data)
            data_bytes = data.tobytes()
        elif isinstance(data, (bytes, bytearray)):
            data_bytes = data
        else:
            raise TypeError(f"Unsupported type {type(data)}. Expected bytes or numpy array.")
            
        try:
            return qres_rust.encode_bytes(data_bytes)
        except Exception as e:
            raise QRESError(f"Compression failed: {e}")

    @staticmethod
    def decompress(data: Union[bytes, bytearray]) -> bytes:
        """
        Decompress QRES v2 data.
        """
        if not isinstance(data, (bytes, bytearray)):
             raise TypeError(f"Unsupported type {type(data)}. Expected bytes.")
             
        try:
            return qres_rust.decode_bytes(data)
        except Exception as e:
            raise QRESError(f"Decompression failed: {e}")

# Helper aliases
compress = QRES.compress
decompress = QRES.decompress

class QRESFile(io.BufferedIOBase):
    """
    File object for reading/writing QRES compressed files.
    Mimics gzip.open behavior (simplified).
    """
    def __init__(self, filename, mode="rb"):
        self._file = open(filename, mode)
        self._mode = mode
        
        if "w" in mode:
            # We buffer write data specifically because QRES chunks are large (4MB)
            # But for simplicity in this Alpha, we might just write-through or buffer locally.
            # Real implementation would need a buffer to feed chunks.
            # IMPLEMENTATION NOTE: Current Rust binary handles the chunking. 
            # This Python wrapper currently wraps the Block-level API.
            # A true streaming file object needs Stream-API support in Rust lib.
            pass
            
    def read(self, size=-1):
        # Placeholder: Reading stream support requires rust-side stream exposure or 
        # reading the whole file and wrapping it (inefficient for large files).
        # For Alpha v0.3.0, we recommend using the block-based API.
        raw = self._file.read()
        return decompress(raw)
        
    def write(self, data):
        # Just write compressed blob for now (Single Chunk Assumption for context manager simplicity)
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
