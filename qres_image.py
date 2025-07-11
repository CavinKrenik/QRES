import numpy as np
from qres_core import qres_encode, qres_decode

def qres_encode_image(image: np.ndarray):
    channels = [qres_encode(image[..., c].flatten()) for c in range(image.shape[-1])]
    return channels

def qres_decode_image(channels, shape):
    decoded_channels = [np.array(qres_decode(s, e)).reshape(shape[:-1]) for s, e in channels]
    return np.stack(decoded_channels, axis=-1)