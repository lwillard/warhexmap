"""Enum definitions for elevation, climate, and decorator types."""

from __future__ import annotations

from enum import Enum, IntEnum


class Elevation(IntEnum):
    """Elevation levels ordered by height."""
    VERY_DEEP_WATER = 0
    DEEP_WATER = 1
    WATER = 2
    SHALLOW_WATER = 3
    PLAINS = 4
    HILLS = 5
    MOUNTAINS = 6

    def is_water(self) -> bool:
        return self.value <= Elevation.SHALLOW_WATER.value

    def is_land(self) -> bool:
        return not self.is_water()


class Climate(Enum):
    """Köppen climate classification groups."""
    BW = "Arid Desert"
    BS = "Arid Steppe"
    Cs = "Mediterranean"
    Cw = "Subtropical Highland"
    Cf = "Oceanic/Humid"
    Df = "Continental"
    Am = "Tropical Monsoon"
    Af = "Tropical Rainforest"


class Decorator(Enum):
    """Area-fill decorators for hexes."""
    GRASSLAND = "grassland"
    FARMS = "farms"
    WOODS = "woods"
    DENSE_FOREST = "dense_forest"
    BUILDINGS = "buildings"
    DENSE_BUILDINGS = "dense_buildings"


class PathType(Enum):
    """Types of path features (roads, rivers, etc.)."""
    DIRT_ROAD = "dirt_road"
    ROAD = "road"
    MAJOR_ROAD = "major_road"
    RAIL = "rail"
    STREAM = "stream"
    RIVER = "river"
    MAJOR_RIVER = "major_river"

    def is_road(self) -> bool:
        return self in (PathType.DIRT_ROAD, PathType.ROAD, PathType.MAJOR_ROAD, PathType.RAIL)

    def is_water_feature(self) -> bool:
        return self in (PathType.STREAM, PathType.RIVER, PathType.MAJOR_RIVER)
