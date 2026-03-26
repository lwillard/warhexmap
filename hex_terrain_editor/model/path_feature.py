"""Path features: roads, rivers, streams as polyline/Bézier features."""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from typing import Any

from .terrain_types import PathType


@dataclass
class PathFeature:
    """A polyline feature (road, river, etc.) that crosses hex boundaries."""

    id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])
    feature_type: PathType = PathType.ROAD
    control_points: list[tuple[float, float]] = field(default_factory=list)
    width: float = 2.0
    properties: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "type": self.feature_type.value,
            "control_points": [list(p) for p in self.control_points],
            "width": self.width,
            "properties": self.properties,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> PathFeature:
        return cls(
            id=data["id"],
            feature_type=PathType(data["type"]),
            control_points=[tuple(p) for p in data["control_points"]],
            width=data.get("width", 2.0),
            properties=data.get("properties", {}),
        )

    def bounding_box(self) -> tuple[float, float, float, float]:
        """Return (min_x, min_y, max_x, max_y) of the control points."""
        if not self.control_points:
            return (0, 0, 0, 0)
        xs = [p[0] for p in self.control_points]
        ys = [p[1] for p in self.control_points]
        margin = self.width * 2
        return (min(xs) - margin, min(ys) - margin, max(xs) + margin, max(ys) + margin)
