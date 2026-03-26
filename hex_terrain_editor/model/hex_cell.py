"""HexCell data model representing a single hex on the map."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .terrain_types import Climate, Decorator, Elevation


@dataclass
class HexCell:
    """A single hex cell in the map grid."""

    q: int
    r: int
    elevation: Elevation = Elevation.PLAINS
    climate: Climate = Climate.Cf
    decorators: list[Decorator] = field(default_factory=list)
    label: str | None = None
    label_offset: tuple[int, int] = (0, 0)

    def copy(self) -> HexCell:
        """Return a deep copy of this cell."""
        return HexCell(
            q=self.q,
            r=self.r,
            elevation=self.elevation,
            climate=self.climate,
            decorators=list(self.decorators),
            label=self.label,
            label_offset=self.label_offset,
        )

    def to_dict(self) -> dict[str, Any]:
        """Serialize to a dictionary."""
        d: dict[str, Any] = {
            "q": self.q,
            "r": self.r,
            "elevation": self.elevation.name,
            "climate": self.climate.name,
        }
        if self.decorators:
            d["decorators"] = [dec.value for dec in self.decorators]
        if self.label:
            d["label"] = self.label
            d["label_offset"] = list(self.label_offset)
        return d

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> HexCell:
        """Deserialize from a dictionary."""
        decorators = [Decorator(v) for v in data.get("decorators", [])]
        label_offset = tuple(data.get("label_offset", [0, 0]))
        return cls(
            q=data["q"],
            r=data["r"],
            elevation=Elevation[data["elevation"]],
            climate=Climate[data["climate"]],
            decorators=decorators,
            label=data.get("label"),
            label_offset=(label_offset[0], label_offset[1]),
        )

    def add_decorator(self, decorator: Decorator) -> None:
        if decorator not in self.decorators:
            self.decorators.append(decorator)

    def remove_decorator(self, decorator: Decorator) -> None:
        if decorator in self.decorators:
            self.decorators.remove(decorator)

    @property
    def elevation_band(self) -> str:
        """Return the elevation band name for color lookup."""
        if self.elevation.is_water():
            return "water"
        elif self.elevation == Elevation.PLAINS:
            return "plains"
        elif self.elevation == Elevation.HILLS:
            return "hills"
        else:
            return "mountain"
