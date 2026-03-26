"""Procedural terrain texture generation and climate-based color mapping."""

from __future__ import annotations

import math

from ..model.terrain_types import Climate, Elevation
from ..utils.color_palettes import CLIMATE_PALETTE, WATER_PALETTE
from ..utils.hex_math import clamp
from ..utils.noise import octave_noise_2d


def get_water_color(elevation_value: float) -> tuple[int, int, int]:
    """Map a water elevation value to an RGB color with smooth interpolation."""
    if elevation_value <= 0.5:
        c = WATER_PALETTE[0]["base"]
    elif elevation_value <= 1.5:
        t = elevation_value - 0.5
        c = _lerp_rgb(WATER_PALETTE[0]["base"], WATER_PALETTE[1]["base"], t)
    elif elevation_value <= 2.5:
        t = elevation_value - 1.5
        c = _lerp_rgb(WATER_PALETTE[1]["base"], WATER_PALETTE[2]["base"], t)
    elif elevation_value <= 3.5:
        t = elevation_value - 2.5
        c = _lerp_rgb(WATER_PALETTE[2]["base"], WATER_PALETTE[3]["base"], t)
    else:
        c = WATER_PALETTE[3]["base"]
    return c


def get_land_color(
    elevation_value: float, climate: Climate
) -> tuple[int, int, int]:
    """Map land elevation + climate to a base RGB color."""
    palette = CLIMATE_PALETTE.get(climate.name, CLIMATE_PALETTE["Cf"])
    plains = palette["plains"]
    hills = palette["hills"]
    mountain = palette["mountain"]

    if elevation_value <= 4.5:
        return plains
    elif elevation_value <= 5.5:
        t = elevation_value - 4.5
        return _lerp_rgb(plains, hills, t)
    elif elevation_value <= 6.0:
        t = (elevation_value - 5.5) / 0.5
        return _lerp_rgb(hills, mountain, t)
    else:
        return mountain


def get_terrain_color(
    elevation_value: float,
    climate: Climate,
    world_x: float,
    world_y: float,
) -> tuple[int, int, int]:
    """Get terrain color with procedural noise texture applied."""
    if elevation_value < 3.8:
        base = get_water_color(elevation_value)
        # Minimal noise for water
        noise_amp = 3
    else:
        base = get_land_color(elevation_value, climate)
        palette = CLIMATE_PALETTE.get(climate.name, CLIMATE_PALETTE["Cf"])
        noise_amp = palette.get("texture_noise_amplitude", 15)

    # Multi-scale noise for texture
    large = octave_noise_2d(world_x * 0.005, world_y * 0.005, octaves=2)
    medium = octave_noise_2d(world_x * 0.02, world_y * 0.02, octaves=2) * 0.5
    fine = octave_noise_2d(world_x * 0.08, world_y * 0.08, octaves=1) * 0.15
    combined = (large + medium + fine) * noise_amp

    return (
        int(clamp(base[0] + combined, 0, 255)),
        int(clamp(base[1] + combined, 0, 255)),
        int(clamp(base[2] + combined, 0, 255)),
    )


def _lerp_rgb(
    c1: tuple[int, int, int], c2: tuple[int, int, int], t: float
) -> tuple[int, int, int]:
    t = max(0.0, min(1.0, t))
    return (
        int(c1[0] + (c2[0] - c1[0]) * t),
        int(c1[1] + (c2[1] - c1[1]) * t),
        int(c1[2] + (c2[2] - c1[2]) * t),
    )
