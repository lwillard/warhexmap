"""Save/load project data as JSON."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .hex_grid import HexGrid
from .path_feature import PathFeature


class Project:
    """Represents a complete map project."""

    def __init__(self, name: str = "Untitled Map"):
        self.name = name
        self.grid = HexGrid()
        self.paths: list[PathFeature] = []
        self.labels: list[dict[str, Any]] = []
        self.file_path: Path | None = None

    def save(self, path: Path | str) -> None:
        """Save project to a JSON file."""
        path = Path(path)
        data = {
            "version": "1.0",
            "name": self.name,
            "grid": {
                "width": self.grid.width,
                "height": self.grid.height,
                "hex_size": self.grid.hex_size,
                "orientation": "flat-top",
            },
            "hexes": self.grid.to_dict_list(),
            "paths": [p.to_dict() for p in self.paths],
            "labels": self.labels,
        }
        path.write_text(json.dumps(data, indent=2), encoding="utf-8")
        self.file_path = path

    def load(self, path: Path | str) -> None:
        """Load project from a JSON file."""
        path = Path(path)
        data = json.loads(path.read_text(encoding="utf-8"))

        self.name = data.get("name", "Untitled Map")
        grid_info = data.get("grid", {})
        self.grid = HexGrid(
            width=grid_info.get("width", 60),
            height=grid_info.get("height", 30),
            hex_size=grid_info.get("hex_size", 64.0),
        )
        self.grid.load_from_dict_list(data.get("hexes", []))
        self.paths = [PathFeature.from_dict(p) for p in data.get("paths", [])]
        self.labels = data.get("labels", [])
        self.file_path = path

    def add_path(self, path: PathFeature) -> None:
        self.paths.append(path)

    def remove_path(self, path_id: str) -> None:
        self.paths = [p for p in self.paths if p.id != path_id]
