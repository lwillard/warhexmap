"""Scrollable, zoomable map canvas widget using tkinter."""

from __future__ import annotations

import tkinter as tk
from typing import Any, Callable

from PIL import Image, ImageTk

from ..model.hex_grid import HexGrid
from ..model.path_feature import PathFeature
from ..renderer.compositor import MapCompositor
from ..utils.hex_math import pixel_to_hex


class MapViewport(tk.Canvas):
    """Main map display canvas with pan and zoom."""

    def __init__(
        self,
        parent: tk.Widget,
        grid: HexGrid,
        paths: list[PathFeature],
        **kwargs: Any,
    ) -> None:
        super().__init__(parent, bg="#c8c8c8", highlightthickness=0, **kwargs)
        self.grid = grid
        self.paths = paths
        self.compositor = MapCompositor(grid, paths)

        # View state
        self.view_x: float = 0.0
        self.view_y: float = 0.0
        self.zoom: int = 2
        self.min_zoom: int = 0
        self.max_zoom: int = 4

        # Callbacks
        self.on_hex_clicked: Callable[[int, int], None] | None = None
        self.on_hex_hovered: Callable[[int, int], None] | None = None
        self.on_tool_press: Callable[[float, float], None] | None = None
        self.on_tool_move: Callable[[float, float], None] | None = None
        self.on_tool_release: Callable[[float, float], None] | None = None

        # Pan state
        self._panning = False
        self._pan_start_x = 0
        self._pan_start_y = 0
        self._pan_view_start = (0.0, 0.0)

        # Cached image reference (prevent GC)
        self._photo_image: ImageTk.PhotoImage | None = None
        self._needs_render = True

        # Bind events
        self.bind("<ButtonPress-1>", self._on_left_press)
        self.bind("<B1-Motion>", self._on_left_drag)
        self.bind("<ButtonRelease-1>", self._on_left_release)
        self.bind("<ButtonPress-2>", self._on_middle_press)
        self.bind("<B2-Motion>", self._on_middle_drag)
        self.bind("<ButtonRelease-2>", self._on_middle_release)
        self.bind("<ButtonPress-3>", self._on_middle_press)  # right-click for pan too
        self.bind("<B3-Motion>", self._on_middle_drag)
        self.bind("<ButtonRelease-3>", self._on_middle_release)
        self.bind("<MouseWheel>", self._on_scroll)
        self.bind("<Motion>", self._on_motion)
        self.bind("<Configure>", self._on_configure)

    def invalidate(self) -> None:
        self._needs_render = True
        self._render()

    def invalidate_all_tiles(self) -> None:
        self.compositor.invalidate_all()
        self.invalidate()

    def invalidate_hex(self, q: int, r: int) -> None:
        self.compositor.invalidate_hex(q, r)
        self.invalidate()

    def center_on_hex(self, q: int, r: int) -> None:
        from ..utils.hex_math import hex_to_pixel
        cx, cy = hex_to_pixel(q, r, self.grid.hex_size)
        scale = 2.0 ** self.zoom
        self.view_x = cx - self.winfo_width() / (2 * scale)
        self.view_y = cy - self.winfo_height() / (2 * scale)
        self.invalidate()

    def screen_to_world(self, sx: float, sy: float) -> tuple[float, float]:
        scale = 2.0 ** self.zoom
        return (self.view_x + sx / scale, self.view_y + sy / scale)

    def _render(self) -> None:
        w = self.winfo_width()
        h = self.winfo_height()
        if w <= 1 or h <= 1:
            return

        img = self.compositor.render_viewport(
            self.view_x, self.view_y, w, h, self.zoom
        )

        self._photo_image = ImageTk.PhotoImage(img)
        self.delete("all")
        self.create_image(0, 0, anchor=tk.NW, image=self._photo_image)
        self._needs_render = False

    # --- Event Handlers ---

    def _on_left_press(self, event: tk.Event) -> None:
        wx, wy = self.screen_to_world(event.x, event.y)
        if self.on_tool_press:
            self.on_tool_press(wx, wy)
        q, r = pixel_to_hex(wx, wy, self.grid.hex_size)
        if self.on_hex_clicked:
            self.on_hex_clicked(q, r)

    def _on_left_drag(self, event: tk.Event) -> None:
        wx, wy = self.screen_to_world(event.x, event.y)
        if self.on_tool_move:
            self.on_tool_move(wx, wy)

    def _on_left_release(self, event: tk.Event) -> None:
        wx, wy = self.screen_to_world(event.x, event.y)
        if self.on_tool_release:
            self.on_tool_release(wx, wy)

    def _on_middle_press(self, event: tk.Event) -> None:
        self._panning = True
        self._pan_start_x = event.x
        self._pan_start_y = event.y
        self._pan_view_start = (self.view_x, self.view_y)
        self.config(cursor="fleur")

    def _on_middle_drag(self, event: tk.Event) -> None:
        if not self._panning:
            return
        scale = 2.0 ** self.zoom
        dx = (event.x - self._pan_start_x) / scale
        dy = (event.y - self._pan_start_y) / scale
        self.view_x = self._pan_view_start[0] - dx
        self.view_y = self._pan_view_start[1] - dy
        self.invalidate()

    def _on_middle_release(self, event: tk.Event) -> None:
        self._panning = False
        self.config(cursor="")

    def _on_scroll(self, event: tk.Event) -> None:
        old_zoom = self.zoom
        wx, wy = self.screen_to_world(event.x, event.y)

        if event.delta > 0:
            self.zoom = min(self.zoom + 1, self.max_zoom)
        else:
            self.zoom = max(self.zoom - 1, self.min_zoom)

        if self.zoom != old_zoom:
            new_scale = 2.0 ** self.zoom
            self.view_x = wx - event.x / new_scale
            self.view_y = wy - event.y / new_scale
            self.compositor.invalidate_all()
            self.invalidate()

    def _on_motion(self, event: tk.Event) -> None:
        if self._panning:
            return
        wx, wy = self.screen_to_world(event.x, event.y)
        q, r = pixel_to_hex(wx, wy, self.grid.hex_size)
        if self.on_hex_hovered:
            self.on_hex_hovered(q, r)

    def _on_configure(self, event: tk.Event) -> None:
        self.after(50, self.invalidate)
