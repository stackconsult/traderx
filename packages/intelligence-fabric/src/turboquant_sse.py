"""TurboQuant 3.5-bit for SSE4.2 (CPU mode)"""
import numpy as np

class TurboQuantMSE:
    def __init__(self, dim=128, bits=3):
        self.dim = dim
        self.bits = bits
        self.n_clusters = 2 ** bits
        self.centroids = np.linspace(-1, 1, self.n_clusters)
    
    def quantize(self, x):
        norms = np.linalg.norm(x, axis=-1, keepdims=True)
        x_unit = x / (norms + 1e-10)
        indices = np.argmin(np.abs(x_unit[..., None] - self.centroids), axis=-1)
        return indices.astype(np.uint8), norms.squeeze()
    
    def dequantize(self, indices, norms):
        values = self.centroids[indices]
        return values * norms[..., None]
    
    def compress_ratio(self):
        original_bits = 32  # float32
        return original_bits / self.bits
