"""Eyedropper tool for sampling terrain from an existing hex."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..utils.hex_math import pixel_to_hex

if TYPE_CHECKING:
    from ..model.hex_grid import HexGrid


class EyedropperTool:
    """Click to sample elevation and climate from a hex."""

    def __init__(self, grid: "HexGrid", on_sample: Any = None) -> None:
        self.grid = grid
        self.on_sample = on_sample

    def activate(self) -> None:
        pass

    def deactivate(self) -> None:
        pass

    def on_press(self, world_x: float, world_y: float) -> None:
        q, r = pixel_to_hex(world_x, world_y, self.grid.hex_size)
        cell = self.grid.get(q, r)
        if cell is not None and self.on_sample:
            self.on_sample(cell.elevation, cell.climate, list(cell.decorators))

    def on_move(self, world_x: float, world_y: float) -> None:
        pass

    def on_release(self, world_x: float, world_y: float) -> None:
        pass
