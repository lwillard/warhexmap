"""Eraser tool for removing decorators from hexes."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..utils.hex_math import pixel_to_hex

if TYPE_CHECKING:
    from ..model.hex_grid import HexGrid
    from .tool_manager import ToolManager


class EraserTool:
    """Click or drag to remove all decorators from hexes."""

    def __init__(
        self,
        grid: "HexGrid",
        tool_manager: "ToolManager",
        on_hex_changed: Any = None,
    ) -> None:
        self.grid = grid
        self.tool_manager = tool_manager
        self.on_hex_changed = on_hex_changed
        self._erasing = False
        self._old_states: dict[tuple[int, int], dict] = {}

    def activate(self) -> None:
        pass

    def deactivate(self) -> None:
        self._erasing = False

    def on_press(self, world_x: float, world_y: float) -> None:
        self._erasing = True
        self._old_states = {}
        self._erase_at(world_x, world_y)

    def on_move(self, world_x: float, world_y: float) -> None:
        if self._erasing:
            self._erase_at(world_x, world_y)

    def on_release(self, world_x: float, world_y: float) -> None:
        if self._erasing and self._old_states:
            new_states = {}
            for key in self._old_states:
                cell = self.grid.get(key[0], key[1])
                if cell:
                    new_states[key] = cell.to_dict()

            self.tool_manager.push_undo(
                "Erase decorators",
                {"cells": self._old_states},
                {"cells": new_states},
            )
        self._erasing = False
        self._old_states = {}

    def _erase_at(self, world_x: float, world_y: float) -> None:
        q, r = pixel_to_hex(world_x, world_y, self.grid.hex_size)
        cell = self.grid.get(q, r)
        if cell is None or not cell.decorators:
            return

        key = (q, r)
        if key not in self._old_states:
            self._old_states[key] = cell.to_dict()

        cell.decorators.clear()
        if self.on_hex_changed:
            self.on_hex_changed(q, r)
