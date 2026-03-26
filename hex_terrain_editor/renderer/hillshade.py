"""Elevation heightmap to hillshade computation."""

from __future__ import annotations

import numpy as np


def compute_hillshade(
    heightmap: np.ndarray,
    azimuth: float = 315.0,
    altitude: float = 45.0,
    z_factor: float = 2.0,
) -> np.ndarray:
    """Compute hillshade from a 2D heightmap array.

    Args:
        heightmap: 2D float array of elevation values.
        azimuth: Light direction in degrees (315 = NW).
        altitude: Light elevation angle in degrees.
        z_factor: Vertical exaggeration factor.

    Returns:
        2D float array of hillshade values in [0, 1].
    """
    scaled = heightmap * z_factor
    dy, dx = np.gradient(scaled)

    azimuth_rad = np.radians(360.0 - azimuth + 90.0)
    altitude_rad = np.radians(altitude)

    slope = np.arctan(np.sqrt(dx * dx + dy * dy))
    aspect = np.arctan2(-dy, dx)

    hillshade = (
        np.sin(altitude_rad) * np.cos(slope)
        + np.cos(altitude_rad) * np.sin(slope) * np.cos(azimuth_rad - aspect)
    )

    return np.clip(hillshade, 0.0, 1.0)


def apply_hillshade(
    base_rgb: np.ndarray, hillshade: np.ndarray, shadow_weight: float = 0.4
) -> np.ndarray:
    """Apply hillshade to an RGB image array.

    Args:
        base_rgb: (H, W, 3) uint8 array of base terrain colors.
        hillshade: (H, W) float array in [0, 1].
        shadow_weight: Minimum brightness (0 = full shadow possible).

    Returns:
        (H, W, 3) uint8 shaded array.
    """
    shade_factor = shadow_weight + (1.0 - shadow_weight) * hillshade
    shaded = base_rgb.astype(np.float32) * shade_factor[:, :, np.newaxis]
    return np.clip(shaded, 0, 255).astype(np.uint8)
