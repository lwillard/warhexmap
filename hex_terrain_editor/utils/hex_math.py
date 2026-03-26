"""Hex geometry calculations using axial coordinates with flat-top hexes."""

from __future__ import annotations

import math
from typing import Iterator

# Flat-top hex neighbor offsets in axial coordinates (q, r)
NEIGHBOR_OFFSETS = [
    (+1, 0),   # East
    (-1, 0),   # West
    (+1, -1),  # NE
    (0, -1),   # NW
    (0, +1),   # SE
    (-1, +1),  # SW
]


def hex_to_pixel(q: int, r: int, size: float) -> tuple[float, float]:
    """Convert axial hex coordinates to pixel (world) coordinates.

    Args:
        q: Axial column coordinate.
        r: Axial row coordinate.
        size: Hex size (center to corner distance).

    Returns:
        (x, y) pixel coordinates of the hex center.
    """
    x = size * (3.0 / 2.0 * q)
    y = size * (math.sqrt(3.0) / 2.0 * q + math.sqrt(3.0) * r)
    return (x, y)


def pixel_to_hex(x: float, y: float, size: float) -> tuple[int, int]:
    """Convert pixel (world) coordinates to axial hex coordinates.

    Returns the hex that contains the given pixel point.
    """
    q_frac = (2.0 / 3.0 * x) / size
    r_frac = (-1.0 / 3.0 * x + math.sqrt(3.0) / 3.0 * y) / size
    return axial_round(q_frac, r_frac)


def axial_round(q_frac: float, r_frac: float) -> tuple[int, int]:
    """Round fractional axial coordinates to the nearest hex."""
    s_frac = -q_frac - r_frac

    q_int = round(q_frac)
    r_int = round(r_frac)
    s_int = round(s_frac)

    q_diff = abs(q_int - q_frac)
    r_diff = abs(r_int - r_frac)
    s_diff = abs(s_int - s_frac)

    if q_diff > r_diff and q_diff > s_diff:
        q_int = -r_int - s_int
    elif r_diff > s_diff:
        r_int = -q_int - s_int

    return (q_int, r_int)


def hex_neighbors(q: int, r: int) -> list[tuple[int, int]]:
    """Return the 6 neighbor coordinates of a hex."""
    return [(q + dq, r + dr) for dq, dr in NEIGHBOR_OFFSETS]


def hex_distance(q1: int, r1: int, q2: int, r2: int) -> int:
    """Compute the hex distance between two axial coordinates."""
    s1 = -q1 - r1
    s2 = -q2 - r2
    return max(abs(q1 - q2), abs(r1 - r2), abs(s1 - s2))


def hexes_in_radius(q: int, r: int, radius: int) -> Iterator[tuple[int, int]]:
    """Yield all hex coordinates within a given radius of (q, r)."""
    for dq in range(-radius, radius + 1):
        for dr in range(max(-radius, -dq - radius), min(radius, -dq + radius) + 1):
            yield (q + dq, r + dr)


def hex_corner_offset(corner: int, size: float) -> tuple[float, float]:
    """Return the pixel offset of a hex corner (0-5) from the hex center.

    Corner 0 is the rightmost point for flat-top hexes, going counter-clockwise.
    """
    angle_deg = 60.0 * corner
    angle_rad = math.radians(angle_deg)
    return (size * math.cos(angle_rad), size * math.sin(angle_rad))


def hex_corners(q: int, r: int, size: float) -> list[tuple[float, float]]:
    """Return the 6 corner pixel coordinates of a hex."""
    cx, cy = hex_to_pixel(q, r, size)
    corners = []
    for i in range(6):
        dx, dy = hex_corner_offset(i, size)
        corners.append((cx + dx, cy + dy))
    return corners


def hex_width(size: float) -> float:
    """Return the width of a flat-top hex."""
    return size * 2.0


def hex_height(size: float) -> float:
    """Return the height of a flat-top hex."""
    return size * math.sqrt(3.0)


def hex_horiz_spacing(size: float) -> float:
    """Return the horizontal distance between hex centers."""
    return size * 3.0 / 2.0


def hex_vert_spacing(size: float) -> float:
    """Return the vertical distance between hex centers."""
    return size * math.sqrt(3.0)


def point_in_hex(px: float, py: float, q: int, r: int, size: float) -> bool:
    """Test if a pixel point is inside a specific hex."""
    hq, hr = pixel_to_hex(px, py, size)
    return hq == q and hr == r


def lerp(a: float, b: float, t: float) -> float:
    """Linear interpolation."""
    return a + (b - a) * t


def lerp_color(
    c1: tuple[int, int, int], c2: tuple[int, int, int], t: float
) -> tuple[int, int, int]:
    """Linearly interpolate between two RGB colors."""
    return (
        int(lerp(c1[0], c2[0], t)),
        int(lerp(c1[1], c2[1], t)),
        int(lerp(c1[2], c2[2], t)),
    )


def clamp(value: float, min_val: float, max_val: float) -> float:
    """Clamp a value to the given range."""
    return max(min_val, min(max_val, value))
