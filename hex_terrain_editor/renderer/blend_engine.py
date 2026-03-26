"""Cross-hex seamless blending engine — fully vectorized with NumPy.

Produces continuous terrain by interpolating elevation and color across hex
boundaries.  All per-pixel work is done as NumPy array operations for speed.
"""

from __future__ import annotations

import math
from typing import Any

import numpy as np

from ..model.hex_grid import HexGrid
from ..model.terrain_types import Climate, Elevation
from ..utils.hex_math import hex_to_pixel, pixel_to_hex
from ..utils.noise import perlin_noise_2d_array, octave_noise_2d_array
from ..utils.color_palettes import (
    CLIMATE_PALETTE,
    SHORE_COLOR,
    WATER_PALETTE,
)


# ---------------------------------------------------------------------------
# Pre-build look-up tables so we never touch Python dicts in the inner loop.
# ---------------------------------------------------------------------------

# Water colour LUT:  index 0..3 → (R,G,B) float
_WATER_LUT = np.array(
    [WATER_PALETTE[i]["base"] for i in range(4)], dtype=np.float32
)  # shape (4, 3)

# Land colour LUT per climate: climate_index → [plains, hills, mountain] → RGB
_CLIMATE_NAMES = ["BW", "BS", "Cs", "Cw", "Cf", "Df", "Am", "Af"]
_CLIMATE_NAME_TO_IDX = {n: i for i, n in enumerate(_CLIMATE_NAMES)}

_LAND_PLAINS = np.zeros((8, 3), dtype=np.float32)
_LAND_HILLS = np.zeros((8, 3), dtype=np.float32)
_LAND_MOUNTAIN = np.zeros((8, 3), dtype=np.float32)
_NOISE_AMP = np.zeros(8, dtype=np.float32)

for _i, _cn in enumerate(_CLIMATE_NAMES):
    _pal = CLIMATE_PALETTE[_cn]
    _LAND_PLAINS[_i] = _pal["plains"]
    _LAND_HILLS[_i] = _pal["hills"]
    _LAND_MOUNTAIN[_i] = _pal["mountain"]
    _NOISE_AMP[_i] = _pal.get("texture_noise_amplitude", 15)


def _climate_to_idx(climate: Climate) -> int:
    return _CLIMATE_NAME_TO_IDX.get(climate.name, 4)  # default Cf


# ---------------------------------------------------------------------------
# Vectorised helpers
# ---------------------------------------------------------------------------

def _pixel_to_hex_arrays(
    wx: np.ndarray, wy: np.ndarray, size: float
) -> tuple[np.ndarray, np.ndarray]:
    """Vectorised pixel→hex (axial round)."""
    q_frac = (2.0 / 3.0 * wx) / size
    sqrt3_inv3 = math.sqrt(3.0) / 3.0
    r_frac = (-1.0 / 3.0 * wx + sqrt3_inv3 * wy) / size
    return _axial_round_arrays(q_frac, r_frac)


def _axial_round_arrays(
    q_frac: np.ndarray, r_frac: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    s_frac = -q_frac - r_frac
    qi = np.round(q_frac).astype(np.int32)
    ri = np.round(r_frac).astype(np.int32)
    si = np.round(s_frac).astype(np.int32)

    qd = np.abs(qi - q_frac)
    rd = np.abs(ri - r_frac)
    sd = np.abs(si - s_frac)

    mask_q = (qd > rd) & (qd > sd)
    mask_r = (~mask_q) & (rd > sd)

    qi = np.where(mask_q, -ri - si, qi)
    ri = np.where(mask_r, -qi - si, ri)
    return qi, ri


# ---------------------------------------------------------------------------
# Main tile renderer  (replaces the old double-for-loop)
# ---------------------------------------------------------------------------

def render_terrain_tile(
    world_x0: float,
    world_y0: float,
    tile_size: int,
    scale: float,
    grid: HexGrid,
) -> np.ndarray:
    """Render a (tile_size, tile_size, 3) uint8 terrain tile.

    Fully vectorized: builds coordinate grids, looks up hex properties via
    NumPy fancy-indexing, blends colours with IDW, adds noise texture, and
    applies hillshade — all without any Python per-pixel loop.
    """
    from .hillshade import apply_hillshade, compute_hillshade

    hex_size = grid.hex_size
    sqrt3 = math.sqrt(3.0)

    # ---- 1. Build world-coordinate grids (H, W) ----------------------------
    px = np.arange(tile_size, dtype=np.float64)
    py = np.arange(tile_size, dtype=np.float64)
    # wx[row, col],  wy[row, col]
    wx = world_x0 + px[np.newaxis, :] / scale          # (1, W) → broadcast
    wy = world_y0 + py[:, np.newaxis] / scale           # (H, 1) → broadcast
    wx = np.broadcast_to(wx, (tile_size, tile_size)).copy()
    wy = np.broadcast_to(wy, (tile_size, tile_size)).copy()

    # ---- 2. Find which hex each pixel belongs to ----------------------------
    center_q, center_r = _pixel_to_hex_arrays(wx, wy, hex_size)

    # ---- 3. Collect hex-center data for the *unique* hexes in this tile -----
    unique_pairs = set(zip(center_q.ravel().tolist(), center_r.ravel().tolist()))
    # Also include their immediate neighbors (for blending)
    expanded: set[tuple[int, int]] = set()
    _offsets = [(0, 0), (1, 0), (-1, 0), (1, -1), (0, -1), (0, 1), (-1, 1)]
    for q, r in unique_pairs:
        for dq, dr in _offsets:
            expanded.add((q + dq, r + dr))

    # Build arrays of hex-center info: position, elevation, climate index
    hex_coords: list[tuple[int, int]] = []
    hex_cx_list: list[float] = []
    hex_cy_list: list[float] = []
    hex_elev_list: list[float] = []
    hex_climate_list: list[int] = []

    for q, r in expanded:
        cell = grid.get(q, r)
        if cell is not None:
            cx, cy = hex_to_pixel(q, r, hex_size)
            hex_coords.append((q, r))
            hex_cx_list.append(cx)
            hex_cy_list.append(cy)
            hex_elev_list.append(float(cell.elevation.value))
            hex_climate_list.append(_climate_to_idx(cell.climate))

    if not hex_coords:
        return np.full((tile_size, tile_size, 3), 128, dtype=np.uint8)

    N = len(hex_coords)
    hex_cx = np.array(hex_cx_list, dtype=np.float64)    # (N,)
    hex_cy = np.array(hex_cy_list, dtype=np.float64)
    hex_elev = np.array(hex_elev_list, dtype=np.float64)
    hex_clim = np.array(hex_climate_list, dtype=np.int32)

    # ---- 4. IDW blending of elevation and colour ----------------------------
    # For each pixel compute distance to each hex centre.  To keep memory
    # manageable, process in row-strips of ~32 rows at a time.
    STRIP = 32
    heightmap = np.empty((tile_size, tile_size), dtype=np.float32)
    rgb = np.empty((tile_size, tile_size, 3), dtype=np.float32)

    max_dist = hex_size * 1.2
    max_dist_sq = max_dist * max_dist

    for y0 in range(0, tile_size, STRIP):
        y1 = min(y0 + STRIP, tile_size)
        H = y1 - y0

        # Slice world coords for this strip:  (H, W)
        swx = wx[y0:y1]
        swy = wy[y0:y1]

        # Distance² from every pixel in strip to every hex centre:  (H, W, N)
        dx = swx[:, :, np.newaxis] - hex_cx[np.newaxis, np.newaxis, :]
        dy = swy[:, :, np.newaxis] - hex_cy[np.newaxis, np.newaxis, :]
        dist_sq = dx * dx + dy * dy

        # IDW weights:  w = 1/(d² + ε)   clipped to max_dist
        inv_dsq = np.where(dist_sq < max_dist_sq, 1.0 / (dist_sq + 0.01), 0.0)
        # Sharpen: square the weights so the centre hex dominates
        weights = inv_dsq * inv_dsq                   # (H, W, N)
        w_sum = weights.sum(axis=2, keepdims=True)    # (H, W, 1)
        w_sum = np.maximum(w_sum, 1e-8)
        w_norm = weights / w_sum                      # (H, W, N)

        # Blended elevation per pixel
        strip_hm = (w_norm * hex_elev[np.newaxis, np.newaxis, :]).sum(axis=2)
        heightmap[y0:y1] = strip_hm

        # --- Blended colour ---
        # For each hex, determine its colour from its elevation and climate
        # We compute colours for all N hexes (vectorised), then blend.
        hex_rgb = _hex_colors_array(hex_elev, hex_clim)  # (N, 3) float32

        # Weighted colour per pixel:
        strip_rgb = np.einsum("hwn,nc->hwc", w_norm.astype(np.float32), hex_rgb)
        rgb[y0:y1] = strip_rgb

    # ---- 5. Add Perlin noise to heightmap -----------------------------------
    noise_hm = perlin_noise_2d_array(wx * 0.01, wy * 0.01).astype(np.float32) * 0.3
    heightmap += noise_hm

    # ---- 6. Texture noise on colour ----------------------------------------
    large  = octave_noise_2d_array(wx * 0.005, wy * 0.005, octaves=2)
    medium = octave_noise_2d_array(wx * 0.02,  wy * 0.02,  octaves=2) * 0.5
    fine   = octave_noise_2d_array(wx * 0.08,  wy * 0.08,  octaves=1) * 0.15
    combined_noise = (large + medium + fine).astype(np.float32)

    # Per-pixel noise amplitude: look up from the dominant hex's climate
    dom_clim = hex_clim[
        np.argmax(
            # for each pixel, find the nearest hex (highest weight)
            # Approximate: use center_q/center_r to look up climate
            np.zeros(1),  # placeholder
        )
    ] if False else None  # We'll use a simpler approach:

    # Use a fixed moderate amplitude (faster than per-pixel lookup)
    # Then modulate slightly by whether pixel is water vs land
    is_water = heightmap < 3.8
    noise_amp = np.where(is_water, 3.0, 15.0)
    noise_offset = (combined_noise * noise_amp[:, :])[:, :, np.newaxis]
    rgb = rgb + noise_offset
    rgb = np.clip(rgb, 0, 255)

    # ---- 7. Coastal blending ------------------------------------------------
    # For pixels near the water/land boundary, blend toward shore color
    shore = np.array(SHORE_COLOR, dtype=np.float32)
    # Transition zone: heightmap between 3.0 and 4.5
    coast_mask = (heightmap > 3.0) & (heightmap < 4.5)
    if np.any(coast_mask):
        t = (heightmap - 3.0) / 1.5   # 0 at deep-water side, 1 at land side
        t = np.clip(t, 0, 1)
        # Blend toward shore colour where t ~ 0.5
        shore_strength = 1.0 - 2.0 * np.abs(t - 0.5)  # peaks at t=0.5
        shore_strength = np.clip(shore_strength, 0, 0.5)
        shore_blend = shore_strength[:, :, np.newaxis] * coast_mask[:, :, np.newaxis]
        rgb = rgb * (1.0 - shore_blend) + shore[np.newaxis, np.newaxis, :] * shore_blend

    # ---- 8. Hillshade -------------------------------------------------------
    hillshade = compute_hillshade(heightmap)
    shade_factor = (0.4 + 0.6 * hillshade)[:, :, np.newaxis]
    # Don't shade water
    shade_factor = np.where(
        is_water[:, :, np.newaxis],
        np.float32(1.0),
        shade_factor,
    )
    rgb = rgb * shade_factor
    rgb = np.clip(rgb, 0, 255).astype(np.uint8)

    return rgb


# ---------------------------------------------------------------------------
# Vectorised colour lookup
# ---------------------------------------------------------------------------

def _hex_colors_array(
    elevations: np.ndarray,  # (N,) float
    climates: np.ndarray,    # (N,) int  (climate index)
) -> np.ndarray:
    """Return (N, 3) float32 array of RGB colours for each hex."""
    N = len(elevations)
    rgb = np.empty((N, 3), dtype=np.float32)

    is_water = elevations < 3.8

    # --- Water colours (interpolate between LUT entries) ---
    water_idx = np.clip(elevations, 0, 3).astype(np.int32)
    water_idx_next = np.minimum(water_idx + 1, 3)
    water_t = np.clip(elevations - water_idx, 0, 1)[:, np.newaxis]
    water_col = (
        _WATER_LUT[water_idx] * (1.0 - water_t) + _WATER_LUT[water_idx_next] * water_t
    )

    # --- Land colours (interpolate plains→hills→mountain) ---
    plains = _LAND_PLAINS[climates]    # (N, 3)
    hills  = _LAND_HILLS[climates]
    mount  = _LAND_MOUNTAIN[climates]

    t_ph = np.clip(elevations - 4.0, 0, 1)[:, np.newaxis]   # plains→hills
    t_hm = np.clip(elevations - 5.0, 0, 1)[:, np.newaxis]   # hills→mountain
    land_col = plains * (1.0 - t_ph) + hills * t_ph
    land_col = land_col * (1.0 - t_hm) + mount * t_hm

    rgb[is_water]  = water_col[is_water]
    rgb[~is_water] = land_col[~is_water]
    return rgb


# ---------------------------------------------------------------------------
# Keep the scalar helpers for tools / non-critical paths
# ---------------------------------------------------------------------------

def compute_heightmap_value(
    world_x: float, world_y: float, grid: HexGrid
) -> float:
    """Scalar version — used by tools, not the tile renderer."""
    hex_size = grid.hex_size
    q, r = pixel_to_hex(world_x, world_y, hex_size)

    samples: list[tuple[float, float]] = []
    for dq in range(-1, 2):
        for dr in range(-1, 2):
            nq, nr = q + dq, r + dr
            neighbor = grid.get(nq, nr)
            if neighbor is not None:
                cx, cy = hex_to_pixel(nq, nr, hex_size)
                dist = math.sqrt((world_x - cx) ** 2 + (world_y - cy) ** 2)
                samples.append((dist, float(neighbor.elevation.value)))

    if not samples:
        return 0.0

    weights = [1.0 / (d * d + 0.001) for d, _ in samples]
    total_weight = sum(weights)
    blended = sum(w * e for w, (_, e) in zip(weights, samples)) / total_weight
    return blended
