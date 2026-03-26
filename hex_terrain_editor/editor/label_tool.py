"""Label tool for placing/editing text labels on hexes."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..utils.hex_math import pixel_to_hex

if TYPE_CHECKING:
    from ..model.hex_grid import HexGrid
    from .tool_manager import ToolManager


class LabelTool:
    """Click a hex to add or edit its label."""

    def __init__(
        self,
        grid: "HexGrid",
        tool_manager: "ToolManager",
        on_label_request: Any = None,
        on_hex_changed: Any = None,
    ) -> None:
        self.grid = grid
        self.tool_manager = tool_manager
        self.on_label_request = on_label_request
        self.on_hex_changed = on_hex_changed

    def activate(self) -> None:
        pass

    def deactivate(self) -> None:
        pass

    def on_press(self, world_x: float, world_y: float) -> None:
        q, r = pixel_to_hex(world_x, world_y, self.grid.hex_size)
        cell = self.grid.get(q, r)
        if cell is None:
            return

        if self.on_label_request:
            self.on_label_request(q, r, cell.label or "")

    def on_move(self, world_x: float, world_y: float) -> None:
        pass

    def on_release(self, world_x: float, world_y: float) -> None:
        pass

    def set_label(self, q: int, r: int, text: str) -> None:
        """Set the label on a hex (called after user input)."""
        cell = self.grid.get(q, r)
        if cell is None:
            return

        old_label = cell.label
        cell.label = text if text else None

        self.tool_manager.push_undo(
            "Set label",
            {"q": q, "r": r, "label": old_label},
            {"q": q, "r": r, "label": cell.label},
        )

        if self.on_hex_changed:
            self.on_hex_changed(q, r)
