"""Select tool for selecting hexes for batch operations."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..utils.hex_math import pixel_to_hex

if TYPE_CHECKING:
    from ..model.hex_grid import HexGrid


class SelectTool:
    """Click to select individual hexes, or drag to select multiple."""

    def __init__(self, grid: "HexGrid", on_selection_changed: Any = None) -> None:
        self.grid = grid
        self.on_selection_changed = on_selection_changed
        self.selected: set[tuple[int, int]] = set()
        self._dragging = False

    def activate(self) -> None:
        pass

    def deactivate(self) -> None:
        self.selected.clear()
        self._dragging = False

    def on_press(self, world_x: float, world_y: float) -> None:
        q, r = pixel_to_hex(world_x, world_y, self.grid.hex_size)
        cell = self.grid.get(q, r)
        if cell is not None:
            self.selected = {(q, r)}
            self._dragging = True
            if self.on_selection_changed:
                self.on_selection_changed(self.selected)

    def on_move(self, world_x: float, world_y: float) -> None:
        if self._dragging:
            q, r = pixel_to_hex(world_x, world_y, self.grid.hex_size)
            cell = self.grid.get(q, r)
            if cell is not None:
                self.selected.add((q, r))
                if self.on_selection_changed:
                    self.on_selection_changed(self.selected)

    def on_release(self, world_x: float, world_y: float) -> None:
        self._dragging = False

    def clear_selection(self) -> None:
        self.selected.clear()
        if self.on_selection_changed:
            self.on_selection_changed(self.selected)
