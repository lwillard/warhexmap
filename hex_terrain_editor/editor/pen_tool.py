"""Pen tool for freehand drawing roads and rivers."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..model.path_feature import PathFeature
from ..model.terrain_types import PathType
from ..utils.color_palettes import ROAD_STYLES, RIVER_STYLES
from ..utils.curve_simplify import rdp_simplify

if TYPE_CHECKING:
    from ..model.project import Project
    from .tool_manager import ToolManager


class PenTool:
    """Freehand draw roads/rivers with curve simplification."""

    def __init__(
        self,
        project: "Project",
        tool_manager: "ToolManager",
        on_path_added: Any = None,
    ) -> None:
        self.project = project
        self.tool_manager = tool_manager
        self.on_path_added = on_path_added

        self.path_type: PathType = PathType.ROAD
        self.raw_points: list[tuple[float, float]] = []
        self._drawing = False

    def activate(self) -> None:
        pass

    def deactivate(self) -> None:
        self._drawing = False
        self.raw_points.clear()

    def on_press(self, world_x: float, world_y: float) -> None:
        self._drawing = True
        self.raw_points = [(world_x, world_y)]

    def on_move(self, world_x: float, world_y: float) -> None:
        if self._drawing:
            self.raw_points.append((world_x, world_y))

    def on_release(self, world_x: float, world_y: float) -> None:
        if not self._drawing:
            return
        self._drawing = False
        self.raw_points.append((world_x, world_y))

        if len(self.raw_points) < 2:
            self.raw_points.clear()
            return

        # Simplify the polyline
        simplified = rdp_simplify(self.raw_points, epsilon=5.0)

        # Get width from style
        ft = self.path_type.value
        if ft in ROAD_STYLES:
            width = ROAD_STYLES[ft]["width"]
        elif ft in RIVER_STYLES:
            width = RIVER_STYLES[ft]["width"]
        else:
            width = 2.0

        path = PathFeature(
            feature_type=self.path_type,
            control_points=simplified,
            width=width,
        )

        self.project.add_path(path)

        self.tool_manager.push_undo(
            f"Draw {self.path_type.value}",
            {"action": "add_path", "path_id": path.id},
            {"action": "add_path", "path": path.to_dict()},
        )

        if self.on_path_added:
            self.on_path_added(path)

        self.raw_points.clear()

    @property
    def preview_points(self) -> list[tuple[float, float]]:
        """Return current raw points for preview rendering."""
        return list(self.raw_points)
