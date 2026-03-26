"""Decorator rendering: forests, farms, buildings overlaid on terrain."""

from __future__ import annotations

import math
import random

from PIL import Image, ImageDraw

from ..model.hex_cell import HexCell
from ..model.terrain_types import Decorator
from ..utils.color_palettes import DECORATOR_COLORS
from ..utils.hex_math import hex_corners, hex_to_pixel


def render_decorators_on_hex(
    draw: ImageDraw.ImageDraw,
    cell: HexCell,
    hex_size: float,
    offset_x: float,
    offset_y: float,
    scale: float,
    seed: int = 0,
) -> None:
    """Render all decorators for a hex cell onto a PIL ImageDraw."""
    cx, cy = hex_to_pixel(cell.q, cell.r, hex_size)
    sx = (cx - offset_x) * scale
    sy = (cy - offset_y) * scale
    scaled_size = hex_size * scale

    for decorator in cell.decorators:
        rng = random.Random(seed + hash((cell.q, cell.r, decorator.value)))
        if decorator == Decorator.WOODS:
            _render_woods(draw, sx, sy, scaled_size, rng, dense=False)
        elif decorator == Decorator.DENSE_FOREST:
            _render_woods(draw, sx, sy, scaled_size, rng, dense=True)
        elif decorator == Decorator.FARMS:
            _render_farms(draw, sx, sy, scaled_size, rng)
        elif decorator == Decorator.GRASSLAND:
            _render_grassland(draw, sx, sy, scaled_size, rng)
        elif decorator == Decorator.BUILDINGS:
            _render_buildings(draw, sx, sy, scaled_size, rng, dense=False)
        elif decorator == Decorator.DENSE_BUILDINGS:
            _render_buildings(draw, sx, sy, scaled_size, rng, dense=True)


def _render_woods(
    draw: ImageDraw.ImageDraw,
    cx: float, cy: float,
    size: float,
    rng: random.Random,
    dense: bool,
) -> None:
    """Render forest as clusters of dark green splotches."""
    colors = DECORATOR_COLORS["dense_forest" if dense else "woods"]
    tree_color = colors["tree_color"]
    shadow_color = colors["shadow_color"]
    count = int(size * (1.2 if dense else 0.5))
    radius = size * 0.75

    for _ in range(count):
        angle = rng.uniform(0, 2 * math.pi)
        dist = rng.uniform(0, radius) * math.sqrt(rng.random())
        tx = cx + dist * math.cos(angle)
        ty = cy + dist * math.sin(angle)
        r = rng.uniform(size * 0.03, size * 0.07)

        # Vary color slightly
        cr = max(0, min(255, tree_color[0] + rng.randint(-15, 15)))
        cg = max(0, min(255, tree_color[1] + rng.randint(-15, 15)))
        cb = max(0, min(255, tree_color[2] + rng.randint(-10, 10)))

        # Shadow
        draw.ellipse(
            [tx - r + 1, ty - r + 1, tx + r + 1, ty + r + 1],
            fill=shadow_color,
        )
        # Canopy
        draw.ellipse([tx - r, ty - r, tx + r, ty + r], fill=(cr, cg, cb))


def _render_farms(
    draw: ImageDraw.ImageDraw,
    cx: float, cy: float,
    size: float,
    rng: random.Random,
) -> None:
    """Render farm fields as irregular patchwork rectangles."""
    colors_list = DECORATOR_COLORS["farms"]["colors"]
    radius = size * 0.65
    plot_size = size * 0.15

    for _ in range(12):
        px = cx + rng.uniform(-radius, radius)
        py = cy + rng.uniform(-radius, radius)
        w = rng.uniform(plot_size * 0.6, plot_size * 1.4)
        h = rng.uniform(plot_size * 0.6, plot_size * 1.4)
        color = rng.choice(colors_list)

        draw.rectangle([px - w / 2, py - h / 2, px + w / 2, py + h / 2], fill=color)
        border = tuple(max(0, c - 20) for c in color)
        draw.rectangle(
            [px - w / 2, py - h / 2, px + w / 2, py + h / 2],
            outline=border,
            width=1,
        )


def _render_grassland(
    draw: ImageDraw.ImageDraw,
    cx: float, cy: float,
    size: float,
    rng: random.Random,
) -> None:
    """Render grassland as light green stipple dots."""
    base = DECORATOR_COLORS["grassland"]["base"]
    var = DECORATOR_COLORS["grassland"]["variation"]
    radius = size * 0.7

    for _ in range(int(size * 0.4)):
        angle = rng.uniform(0, 2 * math.pi)
        dist = rng.uniform(0, radius) * math.sqrt(rng.random())
        gx = cx + dist * math.cos(angle)
        gy = cy + dist * math.sin(angle)
        r = rng.uniform(1, 2.5)
        cr = max(0, min(255, base[0] + rng.randint(-var, var)))
        cg = max(0, min(255, base[1] + rng.randint(-var, var)))
        cb = max(0, min(255, base[2] + rng.randint(-var, var)))
        draw.ellipse([gx - r, gy - r, gx + r, gy + r], fill=(cr, cg, cb))


def _render_buildings(
    draw: ImageDraw.ImageDraw,
    cx: float, cy: float,
    size: float,
    rng: random.Random,
    dense: bool,
) -> None:
    """Render buildings as scattered small rectangles."""
    key = "dense_buildings" if dense else "buildings"
    colors = DECORATOR_COLORS[key]
    wall = colors["wall_color"]
    roof = colors["roof_color"]
    count = int(size * (0.8 if dense else 0.2))
    radius = size * (0.55 if dense else 0.65)

    for _ in range(count):
        angle = rng.uniform(0, 2 * math.pi)
        dist = rng.uniform(0, radius) * (1.0 if dense else math.sqrt(rng.random()))
        bx = cx + dist * math.cos(angle)
        by = cy + dist * math.sin(angle)
        w = rng.uniform(size * 0.03, size * 0.06)
        h = rng.uniform(size * 0.025, size * 0.05)

        draw.rectangle([bx - w, by - h, bx + w, by + h], fill=wall)
        # Roof as a slightly smaller darker rect
        draw.rectangle(
            [bx - w * 0.8, by - h * 0.8, bx + w * 0.8, by + h * 0.3],
            fill=roof,
        )
