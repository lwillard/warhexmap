"""Hex inspector / property editor panel using tkinter."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Callable

from ..model.hex_cell import HexCell
from ..model.terrain_types import Climate, Elevation


class PropertiesPanel(tk.Frame):
    """Shows and edits properties of the currently selected hex."""

    def __init__(
        self,
        parent: tk.Widget,
        on_elevation_changed: Callable | None = None,
        on_climate_changed: Callable | None = None,
    ) -> None:
        super().__init__(parent, bd=1, relief=tk.RIDGE, padx=4, pady=4)
        self.on_elevation_changed = on_elevation_changed
        self.on_climate_changed = on_climate_changed

        # Hex info section
        info_frame = tk.LabelFrame(self, text="Hex Info", padx=4, pady=4)
        info_frame.pack(fill=tk.X, pady=(0, 4))

        self._coord_var = tk.StringVar(value="--")
        self._elev_var = tk.StringVar(value="--")
        self._climate_var = tk.StringVar(value="--")
        self._dec_var = tk.StringVar(value="--")
        self._label_var = tk.StringVar(value="--")

        for label, var in [
            ("Coord:", self._coord_var),
            ("Elevation:", self._elev_var),
            ("Climate:", self._climate_var),
            ("Decorators:", self._dec_var),
            ("Label:", self._label_var),
        ]:
            row = tk.Frame(info_frame)
            row.pack(fill=tk.X, pady=1)
            tk.Label(row, text=label, width=10, anchor=tk.W).pack(side=tk.LEFT)
            tk.Label(row, textvariable=var, anchor=tk.W).pack(side=tk.LEFT, fill=tk.X)

        # Brush settings section
        brush_frame = tk.LabelFrame(self, text="Brush Settings", padx=4, pady=4)
        brush_frame.pack(fill=tk.X, pady=(0, 4))

        # Elevation selector
        tk.Label(brush_frame, text="Elevation:").pack(anchor=tk.W)
        self._elevation_var = tk.StringVar(value=Elevation.PLAINS.name)
        elevation_combo = ttk.Combobox(
            brush_frame,
            textvariable=self._elevation_var,
            values=[e.name for e in Elevation],
            state="readonly",
        )
        elevation_combo.pack(fill=tk.X, pady=(0, 4))
        elevation_combo.bind("<<ComboboxSelected>>", self._on_elevation_selected)

        # Climate selector
        tk.Label(brush_frame, text="Climate:").pack(anchor=tk.W)
        self._climate_var_combo = tk.StringVar(value=Climate.Cf.name)
        climate_combo = ttk.Combobox(
            brush_frame,
            textvariable=self._climate_var_combo,
            values=[f"{c.name} ({c.value})" for c in Climate],
            state="readonly",
        )
        climate_combo.pack(fill=tk.X, pady=(0, 4))
        climate_combo.bind("<<ComboboxSelected>>", self._on_climate_selected)

        # Brush radius
        tk.Label(brush_frame, text="Brush Radius:").pack(anchor=tk.W)
        self._radius_var = tk.IntVar(value=0)
        radius_spin = tk.Spinbox(
            brush_frame, from_=0, to=5, textvariable=self._radius_var, width=5
        )
        radius_spin.pack(anchor=tk.W)

    def update_hex_info(self, cell: HexCell | None) -> None:
        if cell is None:
            self._coord_var.set("--")
            self._elev_var.set("--")
            self._climate_var.set("--")
            self._dec_var.set("--")
            self._label_var.set("--")
            return

        self._coord_var.set(f"({cell.q}, {cell.r})")
        self._elev_var.set(cell.elevation.name)
        self._climate_var.set(cell.climate.name)
        decs = ", ".join(d.value for d in cell.decorators) if cell.decorators else "none"
        self._dec_var.set(decs)
        self._label_var.set(cell.label or "none")

    @property
    def selected_elevation(self) -> Elevation:
        return Elevation[self._elevation_var.get()]

    @property
    def selected_climate(self) -> Climate:
        name = self._climate_var_combo.get().split(" ")[0]
        return Climate[name]

    @property
    def brush_radius(self) -> int:
        return self._radius_var.get()

    def _on_elevation_selected(self, event: object) -> None:
        if self.on_elevation_changed:
            self.on_elevation_changed(self.selected_elevation)

    def _on_climate_selected(self, event: object) -> None:
        if self.on_climate_changed:
            self.on_climate_changed(self.selected_climate)
