"""Multi-resolution tile cache manager."""

from __future__ import annotations

from typing import Any

from PIL import Image


class TileCache:
    """Caches rendered tile images keyed by (tx, ty, zoom)."""

    def __init__(self) -> None:
        self._tiles: dict[tuple[int, int, int], Image.Image] = {}
        self._dirty: set[tuple[int, int, int]] = set()

    def get(self, tx: int, ty: int, zoom: int) -> Image.Image | None:
        key = (tx, ty, zoom)
        if key in self._dirty:
            return None
        return self._tiles.get(key)

    def put(self, tx: int, ty: int, zoom: int, tile: Image.Image) -> None:
        key = (tx, ty, zoom)
        self._tiles[key] = tile
        self._dirty.discard(key)

    def invalidate(self, tx: int, ty: int, zoom: int) -> None:
        self._dirty.add((tx, ty, zoom))

    def invalidate_all(self) -> None:
        """Mark all tiles as dirty."""
        self._dirty.update(self._tiles.keys())

    def is_dirty(self, tx: int, ty: int, zoom: int) -> bool:
        return (tx, ty, zoom) in self._dirty

    def clear(self) -> None:
        self._tiles.clear()
        self._dirty.clear()

    def invalidate_region(
        self, world_x: float, world_y: float, radius: float, tile_size: int, zoom_levels: list[int]
    ) -> None:
        """Invalidate all tiles overlapping a world-space circle at all zoom levels."""
        for zoom in zoom_levels:
            scale = 2.0 ** zoom
            # Convert world coords to tile coords
            min_tx = int((world_x - radius) * scale / tile_size)
            max_tx = int((world_x + radius) * scale / tile_size) + 1
            min_ty = int((world_y - radius) * scale / tile_size)
            max_ty = int((world_y + radius) * scale / tile_size) + 1
            for tx in range(min_tx, max_tx + 1):
                for ty in range(min_ty, max_ty + 1):
                    self._dirty.add((tx, ty, zoom))
