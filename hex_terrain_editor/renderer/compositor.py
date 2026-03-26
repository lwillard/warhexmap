"""Final compositing of all layers into the visible viewport."""

from __future__ import annotations

from PIL import Image

from ..model.hex_grid import HexGrid
from ..model.path_feature import PathFeature
from .tile_cache import TileCache
from .tile_renderer import TILE_SIZE, render_tile, get_scale


class MapCompositor:
    """Manages tile-based rendering and compositing for the map viewport."""

    def __init__(self, grid: HexGrid, paths: list[PathFeature]) -> None:
        self.grid = grid
        self.paths = paths
        self.tile_cache = TileCache()
        self.show_grid = True

    def render_viewport(
        self,
        view_x: float,
        view_y: float,
        view_width: int,
        view_height: int,
        zoom: int,
    ) -> Image.Image:
        """Render the visible portion of the map.

        Args:
            view_x, view_y: World coordinates of the viewport's top-left corner.
            view_width, view_height: Viewport dimensions in pixels.
            zoom: Current zoom level.

        Returns:
            RGBA PIL Image of the viewport.
        """
        scale = get_scale(zoom)
        img = Image.new("RGBA", (view_width, view_height), (200, 200, 200, 255))

        # Determine which tiles are visible
        min_tx = int(view_x * scale / TILE_SIZE)
        max_tx = int((view_x + view_width / scale) * scale / TILE_SIZE) + 1
        min_ty = int(view_y * scale / TILE_SIZE)
        max_ty = int((view_y + view_height / scale) * scale / TILE_SIZE) + 1

        for ty in range(min_ty, max_ty + 1):
            for tx in range(min_tx, max_tx + 1):
                tile = self.tile_cache.get(tx, ty, zoom)
                if tile is None:
                    tile = render_tile(
                        tx, ty, zoom, self.grid, self.paths, self.show_grid
                    )
                    self.tile_cache.put(tx, ty, zoom, tile)

                # Compute where this tile goes in the viewport
                tile_world_x = tx * TILE_SIZE / scale
                tile_world_y = ty * TILE_SIZE / scale
                px = int((tile_world_x - view_x) * scale)
                py = int((tile_world_y - view_y) * scale)

                img.paste(tile, (px, py), tile)

        return img

    def invalidate_all(self) -> None:
        self.tile_cache.invalidate_all()

    def invalidate_hex(self, q: int, r: int) -> None:
        """Invalidate tiles affected by changes to a hex and its neighbors."""
        from ..utils.hex_math import hex_to_pixel, hex_neighbors

        hex_size = self.grid.hex_size
        coords = [(q, r)] + hex_neighbors(q, r)

        for cq, cr in coords:
            cx, cy = hex_to_pixel(cq, cr, hex_size)
            self.tile_cache.invalidate_region(
                cx, cy, hex_size * 2, TILE_SIZE, list(range(5))
            )
