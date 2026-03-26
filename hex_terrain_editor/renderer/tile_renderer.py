"""Raster tile generation per hex — renders individual map tiles."""

from __future__ import annotations

import numpy as np
from PIL import Image, ImageDraw

from ..model.hex_grid import HexGrid
from ..model.path_feature import PathFeature
from ..utils.hex_math import hex_corners, hex_to_pixel, pixel_to_hex
from ..utils.color_palettes import HEX_GRID_STYLE, ROAD_STYLES, RIVER_STYLES
from ..utils.curve_simplify import bezier_to_polyline
from .blend_engine import render_terrain_tile
from .decorator_renderer import render_decorators_on_hex
from .label_renderer import render_label

TILE_SIZE = 256


def get_scale(zoom: int) -> float:
    """Pixels per world unit at a given zoom level."""
    return 2.0 ** zoom


def tile_to_world(tx: int, ty: int, zoom: int) -> tuple[float, float]:
    """Convert tile coordinates to world coordinates of the tile's top-left corner."""
    scale = get_scale(zoom)
    return (tx * TILE_SIZE / scale, ty * TILE_SIZE / scale)


def world_to_tile(world_x: float, world_y: float, zoom: int) -> tuple[int, int]:
    """Convert world coordinates to the tile that contains them."""
    scale = get_scale(zoom)
    return (int(world_x * scale / TILE_SIZE), int(world_y * scale / TILE_SIZE))


def render_tile(
    tx: int,
    ty: int,
    zoom: int,
    grid: HexGrid,
    paths: list[PathFeature],
    show_grid: bool = True,
) -> Image.Image:
    """Render a single map tile at the given tile coordinates and zoom level.

    Returns an RGBA PIL Image of size (TILE_SIZE, TILE_SIZE).
    """
    scale = get_scale(zoom)
    world_x0, world_y0 = tile_to_world(tx, ty, zoom)

    # Layer 0-3: Terrain with hillshading
    terrain_rgb = render_terrain_tile(world_x0, world_y0, TILE_SIZE, scale, grid)
    img = Image.fromarray(terrain_rgb, "RGB").convert("RGBA")
    draw = ImageDraw.Draw(img)

    # Layer 4: Decorators
    _render_decorators(draw, grid, world_x0, world_y0, scale)

    # Layer 5: Path features
    _render_paths(draw, paths, world_x0, world_y0, scale)

    # Layer 6: Hex grid overlay
    if show_grid:
        min_zoom, max_zoom = HEX_GRID_STYLE["show_at_zoom"]
        if min_zoom <= zoom <= max_zoom:
            _render_hex_grid(draw, grid, world_x0, world_y0, scale)

    # Layer 7: Labels
    _render_labels(draw, grid, world_x0, world_y0, scale)

    return img


def _render_decorators(
    draw: ImageDraw.ImageDraw,
    grid: HexGrid,
    world_x0: float,
    world_y0: float,
    scale: float,
) -> None:
    """Render area decorators for hexes visible in this tile."""
    margin = grid.hex_size * 2
    x_max = world_x0 + TILE_SIZE / scale
    y_max = world_y0 + TILE_SIZE / scale

    for cell in grid.cells():
        if not cell.decorators:
            continue
        cx, cy = hex_to_pixel(cell.q, cell.r, grid.hex_size)
        if (cx + margin < world_x0 or cx - margin > x_max
                or cy + margin < world_y0 or cy - margin > y_max):
            continue
        render_decorators_on_hex(draw, cell, grid.hex_size, world_x0, world_y0, scale)


def _render_paths(
    draw: ImageDraw.ImageDraw,
    paths: list[PathFeature],
    world_x0: float,
    world_y0: float,
    scale: float,
) -> None:
    """Render road and river path features."""
    x_max = world_x0 + TILE_SIZE / scale
    y_max = world_y0 + TILE_SIZE / scale

    for path in paths:
        bbox = path.bounding_box()
        if bbox[2] < world_x0 or bbox[0] > x_max or bbox[3] < world_y0 or bbox[1] > y_max:
            continue

        # Get style
        ft = path.feature_type.value
        if ft in ROAD_STYLES:
            style = ROAD_STYLES[ft]
        elif ft in RIVER_STYLES:
            style = RIVER_STYLES[ft]
        else:
            continue

        color = style["color"]
        width = max(1, int(style["width"] * scale))

        # Convert control points to screen coords
        screen_pts = [
            ((p[0] - world_x0) * scale, (p[1] - world_y0) * scale)
            for p in path.control_points
        ]

        if len(screen_pts) >= 2:
            # Draw outline if present
            outline = style.get("outline")
            if outline:
                draw.line(screen_pts, fill=outline, width=width + 2, joint="curve")
            draw.line(screen_pts, fill=color, width=width, joint="curve")


def _render_hex_grid(
    draw: ImageDraw.ImageDraw,
    grid: HexGrid,
    world_x0: float,
    world_y0: float,
    scale: float,
) -> None:
    """Render the hex grid overlay."""
    color = HEX_GRID_STYLE["color"]
    margin = grid.hex_size * 2
    x_max = world_x0 + TILE_SIZE / scale
    y_max = world_y0 + TILE_SIZE / scale

    for cell in grid.cells():
        cx, cy = hex_to_pixel(cell.q, cell.r, grid.hex_size)
        if (cx + margin < world_x0 or cx - margin > x_max
                or cy + margin < world_y0 or cy - margin > y_max):
            continue

        corners = hex_corners(cell.q, cell.r, grid.hex_size)
        screen_corners = [
            ((c[0] - world_x0) * scale, (c[1] - world_y0) * scale)
            for c in corners
        ]
        screen_corners.append(screen_corners[0])  # close the polygon
        draw.line(screen_corners, fill=color, width=1)


def _render_labels(
    draw: ImageDraw.ImageDraw,
    grid: HexGrid,
    world_x0: float,
    world_y0: float,
    scale: float,
) -> None:
    """Render hex labels."""
    margin = grid.hex_size * 2
    x_max = world_x0 + TILE_SIZE / scale
    y_max = world_y0 + TILE_SIZE / scale

    for cell in grid.cells():
        if not cell.label:
            continue
        cx, cy = hex_to_pixel(cell.q, cell.r, grid.hex_size)
        if (cx + margin < world_x0 or cx - margin > x_max
                or cy + margin < world_y0 or cy - margin > y_max):
            continue

        sx = (cx + cell.label_offset[0] - world_x0) * scale
        sy = (cy + cell.label_offset[1] - world_y0) * scale
        render_label(draw, cell.label, sx, sy, font_size=max(8, int(10 * scale)))
