"""Overview minimap widget using tkinter."""

from __future__ import annotations

import tkinter as tk
from typing import Any, Callable

from PIL import Image, ImageDraw, ImageTk

from ..model.hex_grid import HexGrid
from ..utils.color_palettes import CLIMATE_PALETTE, WATER_PALETTE
from ..utils.hex_math import hex_to_pixel


class Minimap(tk.Canvas):
    """Small overview of the entire map with a view rectangle."""

    def __init__(
        self,
        parent: tk.Widget,
        grid: HexGrid,
        on_view_changed: Callable[[float, float], None] | None = None,
        width: int = 180,
        height: int = 120,
    ) -> None:
        super().__init__(parent, width=width, height=height, bg="#b4b4b4", highlightthickness=1)
        self.grid = grid
        self.on_view_changed = on_view_changed
        self._width = width
        self._height = height

        self.view_rect: tuple[float, float, float, float] = (0, 0, 100, 100)
        self._photo: ImageTk.PhotoImage | None = None
        self._world_bounds: tuple[float, float, float, float] | None = None

        self.bind("<ButtonPress-1>", self._on_click)
        self.bind("<B1-Motion>", self._on_click)

    def set_view_rect(self, x: float, y: float, w: float, h: float) -> None:
        self.view_rect = (x, y, w, h)
        self.render()

    def render(self) -> None:
        bounds = self.grid.world_bounds()
        if bounds[2] <= bounds[0] or bounds[3] <= bounds[1]:
            return

        self._world_bounds = bounds
        bw = bounds[2] - bounds[0]
        bh = bounds[3] - bounds[1]
        sx = self._width / bw
        sy = self._height / bh
        scale = min(sx, sy) * 0.9
        ox = (self._width - bw * scale) / 2
        oy = (self._height - bh * scale) / 2

        img = Image.new("RGB", (self._width, self._height), (180, 180, 180))
        draw = ImageDraw.Draw(img)

        for cell in self.grid.cells():
            cx, cy = hex_to_pixel(cell.q, cell.r, self.grid.hex_size)
            px = ox + (cx - bounds[0]) * scale
            py = oy + (cy - bounds[1]) * scale

            if cell.elevation.is_water():
                pal = WATER_PALETTE.get(cell.elevation.value, WATER_PALETTE[2])
                color = pal["base"]
            else:
                cpal = CLIMATE_PALETTE.get(cell.climate.name, CLIMATE_PALETTE["Cf"])
                band = cell.elevation_band
                color = cpal.get(band, cpal["plains"])

            r = max(2, int(self.grid.hex_size * scale * 0.4))
            draw.ellipse([px - r, py - r, px + r, py + r], fill=color)

        # Draw viewport rectangle
        vx, vy, vw, vh = self.view_rect
        rx = ox + (vx - bounds[0]) * scale
        ry = oy + (vy - bounds[1]) * scale
        rw = vw * scale
        rh = vh * scale
        draw.rectangle([rx, ry, rx + rw, ry + rh], outline=(255, 0, 0), width=2)

        self._photo = ImageTk.PhotoImage(img)
        self.delete("all")
        self.create_image(0, 0, anchor=tk.NW, image=self._photo)

    def _on_click(self, event: tk.Event) -> None:
        bounds = self._world_bounds or self.grid.world_bounds()
        if bounds[2] <= bounds[0] or bounds[3] <= bounds[1]:
            return

        bw = bounds[2] - bounds[0]
        bh = bounds[3] - bounds[1]
        sx = self._width / bw
        sy = self._height / bh
        scale = min(sx, sy) * 0.9
        ox = (self._width - bw * scale) / 2
        oy = (self._height - bh * scale) / 2

        world_x = bounds[0] + (event.x - ox) / scale
        world_y = bounds[1] + (event.y - oy) / scale

        if self.on_view_changed:
            self.on_view_changed(world_x, world_y)
