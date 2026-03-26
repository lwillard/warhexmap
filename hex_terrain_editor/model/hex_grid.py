"""HexGrid data model storing the complete map as a dictionary of hex cells."""

from __future__ import annotations

from typing import Iterator

from .hex_cell import HexCell
from .terrain_types import Climate, Elevation
from ..utils.hex_math import hex_neighbors, hex_to_pixel


class HexGrid:
    """Sparse hex grid stored as a dictionary keyed by (q, r) tuples."""

    def __init__(self, width: int = 60, height: int = 30, hex_size: float = 64.0):
        self.width = width
        self.height = height
        self.hex_size = hex_size
        self._cells: dict[tuple[int, int], HexCell] = {}

    def get(self, q: int, r: int) -> HexCell | None:
        """Get the cell at (q, r), or None if it doesn't exist."""
        return self._cells.get((q, r))

    def set(self, q: int, r: int, cell: HexCell) -> None:
        """Set the cell at (q, r)."""
        self._cells[(q, r)] = cell

    def get_or_create(self, q: int, r: int) -> HexCell:
        """Get the cell at (q, r), creating a default one if it doesn't exist."""
        key = (q, r)
        if key not in self._cells:
            self._cells[key] = HexCell(q=q, r=r)
        return self._cells[key]

    def remove(self, q: int, r: int) -> None:
        """Remove the cell at (q, r) if it exists."""
        self._cells.pop((q, r), None)

    def __contains__(self, key: tuple[int, int]) -> bool:
        return key in self._cells

    def __iter__(self) -> Iterator[HexCell]:
        return iter(self._cells.values())

    def __len__(self) -> int:
        return len(self._cells)

    def cells(self) -> Iterator[HexCell]:
        """Iterate over all cells."""
        return iter(self._cells.values())

    def keys(self) -> Iterator[tuple[int, int]]:
        """Iterate over all (q, r) keys."""
        return iter(self._cells.keys())

    def get_neighbors(self, q: int, r: int) -> list[HexCell]:
        """Return existing neighbor cells of (q, r)."""
        result = []
        for nq, nr in hex_neighbors(q, r):
            cell = self.get(nq, nr)
            if cell is not None:
                result.append(cell)
        return result

    def get_neighbor_coords_and_cells(
        self, q: int, r: int
    ) -> list[tuple[int, int, HexCell]]:
        """Return (nq, nr, cell) for existing neighbors."""
        result = []
        for nq, nr in hex_neighbors(q, r):
            cell = self.get(nq, nr)
            if cell is not None:
                result.append((nq, nr, cell))
        return result

    def initialize_rectangular(
        self,
        default_elevation: Elevation = Elevation.PLAINS,
        default_climate: Climate = Climate.Cf,
    ) -> None:
        """Fill the grid with default hex cells in a rectangular layout."""
        for r in range(self.height):
            for q in range(self.width):
                cell = HexCell(
                    q=q,
                    r=r,
                    elevation=default_elevation,
                    climate=default_climate,
                )
                self._cells[(q, r)] = cell

    def bounds(self) -> tuple[int, int, int, int]:
        """Return (min_q, min_r, max_q, max_r) of all cells."""
        if not self._cells:
            return (0, 0, 0, 0)
        qs = [k[0] for k in self._cells]
        rs = [k[1] for k in self._cells]
        return (min(qs), min(rs), max(qs), max(rs))

    def world_bounds(self) -> tuple[float, float, float, float]:
        """Return (min_x, min_y, max_x, max_y) in world pixel coordinates."""
        if not self._cells:
            return (0.0, 0.0, 0.0, 0.0)
        min_x = float("inf")
        min_y = float("inf")
        max_x = float("-inf")
        max_y = float("-inf")
        for q, r in self._cells:
            x, y = hex_to_pixel(q, r, self.hex_size)
            min_x = min(min_x, x)
            min_y = min(min_y, y)
            max_x = max(max_x, x)
            max_y = max(max_y, y)
        # Add margin for hex size
        margin = self.hex_size * 1.5
        return (min_x - margin, min_y - margin, max_x + margin, max_y + margin)

    def to_dict_list(self) -> list[dict]:
        """Serialize all cells to a list of dictionaries."""
        return [cell.to_dict() for cell in self._cells.values()]

    def load_from_dict_list(self, data: list[dict]) -> None:
        """Load cells from a list of dictionaries, clearing existing cells."""
        self._cells.clear()
        for cell_data in data:
            cell = HexCell.from_dict(cell_data)
            self._cells[(cell.q, cell.r)] = cell
