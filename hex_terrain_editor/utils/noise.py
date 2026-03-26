"""Perlin noise implementation for terrain texture generation.

Provides both scalar and vectorized (NumPy array) versions.
"""

from __future__ import annotations

import math
import random

import numpy as np

# Permutation table for Perlin noise
_p: list[int] = []
_p_np: np.ndarray | None = None
_initialized = False


def _init_permutation(seed: int = 0) -> None:
    global _p, _p_np, _initialized
    rng = random.Random(seed)
    perm = list(range(256))
    rng.shuffle(perm)
    _p = perm + perm  # duplicate for overflow
    _p_np = np.array(_p, dtype=np.int32)
    _initialized = True


def _ensure_init() -> None:
    if not _initialized:
        _init_permutation(0)


def _fade(t: float) -> float:
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0)


def _fade_np(t: np.ndarray) -> np.ndarray:
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0)


def _grad(hash_val: int, x: float, y: float) -> float:
    h = hash_val & 3
    if h == 0:
        return x + y
    elif h == 1:
        return -x + y
    elif h == 2:
        return x - y
    else:
        return -x - y


def _grad_np(hash_arr: np.ndarray, x: np.ndarray, y: np.ndarray) -> np.ndarray:
    """Vectorized gradient computation."""
    h = hash_arr & 3
    # h==0: x+y, h==1: -x+y, h==2: x-y, h==3: -x-y
    gx = np.where((h == 0) | (h == 2), x, -x)
    gy = np.where((h == 0) | (h == 1), y, -y)
    return gx + gy


def perlin_noise_2d(x: float, y: float, seed: int = 0) -> float:
    """Compute 2D Perlin noise at the given coordinates (scalar version)."""
    _ensure_init()

    xi = int(math.floor(x)) & 255
    yi = int(math.floor(y)) & 255
    xf = x - math.floor(x)
    yf = y - math.floor(y)

    u = _fade(xf)
    v = _fade(yf)

    aa = _p[_p[xi] + yi]
    ab = _p[_p[xi] + yi + 1]
    ba = _p[_p[xi + 1] + yi]
    bb = _p[_p[xi + 1] + yi + 1]

    x1 = _lerp(_grad(aa, xf, yf), _grad(ba, xf - 1, yf), u)
    x2 = _lerp(_grad(ab, xf, yf - 1), _grad(bb, xf - 1, yf - 1), u)

    return _lerp(x1, x2, v)


def perlin_noise_2d_array(x: np.ndarray, y: np.ndarray) -> np.ndarray:
    """Compute 2D Perlin noise for arrays of coordinates (vectorized).

    Args:
        x, y: Arrays of the same shape with world coordinates.

    Returns:
        Array of noise values in [-1, 1], same shape as inputs.
    """
    _ensure_init()
    assert _p_np is not None

    xi = np.floor(x).astype(np.int32) & 255
    yi = np.floor(y).astype(np.int32) & 255
    xf = x - np.floor(x)
    yf = y - np.floor(y)

    u = _fade_np(xf)
    v = _fade_np(yf)

    # Permutation lookups (vectorized indexing)
    p = _p_np
    aa = p[p[xi] + yi]
    ab = p[p[xi] + yi + 1]
    ba = p[p[xi + 1] + yi]
    bb = p[p[xi + 1] + yi + 1]

    # Gradient dot products
    g_aa = _grad_np(aa, xf, yf)
    g_ba = _grad_np(ba, xf - 1.0, yf)
    g_ab = _grad_np(ab, xf, yf - 1.0)
    g_bb = _grad_np(bb, xf - 1.0, yf - 1.0)

    # Bilinear interpolation
    x1 = g_aa + u * (g_ba - g_aa)
    x2 = g_ab + u * (g_bb - g_ab)
    return x1 + v * (x2 - x1)


def octave_noise_2d_array(
    x: np.ndarray,
    y: np.ndarray,
    octaves: int = 4,
    persistence: float = 0.5,
    lacunarity: float = 2.0,
) -> np.ndarray:
    """Multi-octave Perlin noise for arrays (vectorized)."""
    _ensure_init()
    total = np.zeros_like(x, dtype=np.float64)
    amplitude = 1.0
    frequency = 1.0
    max_amplitude = 0.0

    for _ in range(octaves):
        total += perlin_noise_2d_array(x * frequency, y * frequency) * amplitude
        max_amplitude += amplitude
        amplitude *= persistence
        frequency *= lacunarity

    return total / max_amplitude if max_amplitude > 0 else total


def _lerp(a: float, b: float, t: float) -> float:
    return a + t * (b - a)


def octave_noise_2d(
    x: float,
    y: float,
    octaves: int = 4,
    persistence: float = 0.5,
    lacunarity: float = 2.0,
    seed: int = 0,
) -> float:
    """Multi-octave Perlin noise (scalar version)."""
    total = 0.0
    amplitude = 1.0
    frequency = 1.0
    max_amplitude = 0.0

    for _ in range(octaves):
        total += perlin_noise_2d(x * frequency, y * frequency, seed) * amplitude
        max_amplitude += amplitude
        amplitude *= persistence
        frequency *= lacunarity

    return total / max_amplitude if max_amplitude > 0 else 0.0


def reseed(seed: int) -> None:
    """Re-initialize the noise permutation table with a new seed."""
    _init_permutation(seed)
