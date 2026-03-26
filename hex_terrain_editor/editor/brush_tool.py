"""Brush tool for painting elevation/climate onto hexes."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..model.hex_cell import HexCell
from ..model.terrain_types import Climate, Decorator, Elevation
from ..utils.hex_math import hexes_in_radius, pixel_to_hex

if TYPE_CHECKING:
    from ..model.hex_grid import HexGrid
    from .tool_manager import ToolManager


class BrushTool:
    """Click or drag to paint elevation, climate, or decorators onto hexes."""

    def __init__(
        self, grid: "HexGrid", tool_manager: "ToolManager", on_hex_changed: Any = None
    ) -> None:
        self.grid = grid
        self.tool_manager = tool_manager
        self.on_hex_changed = on_hex_changed

        self.paint_mode: str = "elevation"  # "elevation", "climate", "decorator"
        self.elevation_value: Elevation = Elevation.HILLS
        self.climate_value: Climate = Climate.Cf
        self.decorator_value: Decorator = Decorator.WOODS
        self.brush_radius: int = 0  # 0 = single hex

        self._painting = False
        self._old_states: dict[tuple[int, int], dict] = {}

    def activate(self) -> None:
        pass

    def deactivate(self) -> None:
        self._painting = False

    def on_press(self, world_x: float, world_y: float) -> None:
        self._painting = True
        self._old_states = {}
        self._paint_at(world_x, world_y)

    def on_move(self, world_x: float, world_y: float) -> None:
        if self._painting:
            self._paint_at(world_x, world_y)

    def on_release(self, world_x: float, world_y: float) -> None:
        if self._painting and self._old_states:
            # Build new states for undo
            new_states = {}
            for key in self._old_states:
                cell = self.grid.get(key[0], key[1])
                if cell:
                    new_states[key] = cell.to_dict()

            self.tool_manager.push_undo(
                f"Paint {self.paint_mode}",
                {"cells": self._old_states},
                {"cells": new_states},
            )
        self._painting = False
        self._old_states = {}

    def _paint_at(self, world_x: float, world_y: float) -> None:
        q, r = pixel_to_hex(world_x, world_y, self.grid.hex_size)

        for hq, hr in hexes_in_radius(q, r, self.brush_radius):
            cell = self.grid.get(hq, hr)
            if cell is None:
                continue

            key = (hq, hr)
            if key not in self._old_states:
                self._old_states[key] = cell.to_dict()

            if self.paint_mode == "elevation":
                cell.elevation = self.elevation_value
            elif self.paint_mode == "climate":
                cell.climate = self.climate_value
            elif self.paint_mode == "decorator":
                cell.add_decorator(self.decorator_value)

            if self.on_hex_changed:
                self.on_hex_changed(hq, hr)
