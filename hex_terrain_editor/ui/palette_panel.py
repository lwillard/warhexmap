"""Terrain, climate, decorator, and path type selector palette using tkinter."""

from __future__ import annotations

import tkinter as tk
from typing import Callable

from ..model.terrain_types import Decorator, PathType


class PalettePanel(tk.Frame):
    """Combined palette for selecting paint mode and decorator/path types."""

    def __init__(
        self,
        parent: tk.Widget,
        on_paint_mode_changed: Callable[[str], None] | None = None,
        on_decorator_changed: Callable | None = None,
        on_path_type_changed: Callable | None = None,
    ) -> None:
        super().__init__(parent, bd=1, relief=tk.RIDGE, padx=4, pady=4)
        self.on_paint_mode_changed = on_paint_mode_changed
        self.on_decorator_changed = on_decorator_changed
        self.on_path_type_changed = on_path_type_changed

        # Paint mode
        mode_frame = tk.LabelFrame(self, text="Paint Mode", padx=4, pady=4)
        mode_frame.pack(fill=tk.X, pady=(0, 4))

        self._mode_var = tk.StringVar(value="elevation")
        for mode in ["elevation", "climate", "decorator"]:
            rb = tk.Radiobutton(
                mode_frame,
                text=mode.capitalize(),
                variable=self._mode_var,
                value=mode,
                command=self._on_mode_changed,
            )
            rb.pack(anchor=tk.W)

        # Decorator selection
        dec_frame = tk.LabelFrame(self, text="Decorators", padx=4, pady=4)
        dec_frame.pack(fill=tk.X, pady=(0, 4))

        self._dec_var = tk.StringVar(value=Decorator.WOODS.value)
        for dec in Decorator:
            rb = tk.Radiobutton(
                dec_frame,
                text=dec.value.replace("_", " ").title(),
                variable=self._dec_var,
                value=dec.value,
                command=self._on_decorator_changed,
            )
            rb.pack(anchor=tk.W)

        # Path type selection
        path_frame = tk.LabelFrame(self, text="Path Types", padx=4, pady=4)
        path_frame.pack(fill=tk.X, pady=(0, 4))

        self._path_var = tk.StringVar(value=PathType.ROAD.value)
        for pt in PathType:
            rb = tk.Radiobutton(
                path_frame,
                text=pt.value.replace("_", " ").title(),
                variable=self._path_var,
                value=pt.value,
                command=self._on_path_type_changed,
            )
            rb.pack(anchor=tk.W)

    @property
    def selected_decorator(self) -> Decorator:
        return Decorator(self._dec_var.get())

    @property
    def selected_path_type(self) -> PathType:
        return PathType(self._path_var.get())

    def _on_mode_changed(self) -> None:
        if self.on_paint_mode_changed:
            self.on_paint_mode_changed(self._mode_var.get())

    def _on_decorator_changed(self) -> None:
        if self.on_decorator_changed:
            self.on_decorator_changed(Decorator(self._dec_var.get()))

    def _on_path_type_changed(self) -> None:
        if self.on_path_type_changed:
            self.on_path_type_changed(PathType(self._path_var.get()))
