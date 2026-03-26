"""Main application window for the Hex Terrain Map Editor (tkinter)."""

from __future__ import annotations

import tkinter as tk
from tkinter import filedialog, messagebox, simpledialog
from typing import Any

from .editor.brush_tool import BrushTool
from .editor.eraser_tool import EraserTool
from .editor.eyedropper_tool import EyedropperTool
from .editor.label_tool import LabelTool
from .editor.pen_tool import PenTool
from .editor.select_tool import SelectTool
from .editor.tool_manager import ToolManager
from .model.hex_grid import HexGrid
from .model.project import Project
from .model.terrain_types import Climate, Decorator, Elevation, PathType
from .renderer.compositor import MapCompositor
from .ui.map_viewport import MapViewport
from .ui.minimap import Minimap
from .ui.palette_panel import PalettePanel
from .ui.properties_panel import PropertiesPanel
from .ui.toolbar import Toolbar


class HexTerrainEditorApp:
    """Main application."""

    def __init__(self) -> None:
        self.root = tk.Tk()
        self.root.title("Hex Terrain Map Editor")
        self.root.geometry("1400x900")
        self.root.minsize(1024, 700)

        # Project and data
        self.project = Project("New Map")
        self.project.grid = HexGrid(width=30, height=20, hex_size=64.0)
        self.project.grid.initialize_rectangular(
            default_elevation=Elevation.PLAINS,
            default_climate=Climate.Cf,
        )

        # Tool manager
        self.tool_manager = ToolManager()

        # Build UI
        self._build_menu_bar()
        self._build_layout()
        self._build_tools()
        self._connect_signals()

        # Keyboard shortcuts
        self.root.bind("<Key>", self._on_key)
        self.root.bind("<Control-z>", lambda e: self._undo())
        self.root.bind("<Control-y>", lambda e: self._redo())
        self.root.bind("<Control-s>", lambda e: self._save_project())
        self.root.bind("<Control-o>", lambda e: self._open_project())
        self.root.bind("<Control-n>", lambda e: self._new_project())

        # Initial render after window is drawn
        self.root.after(100, self._initial_render)

    def _build_menu_bar(self) -> None:
        menubar = tk.Menu(self.root)
        self.root.config(menu=menubar)

        # File menu
        file_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="File", menu=file_menu)
        file_menu.add_command(label="New", accelerator="Ctrl+N", command=self._new_project)
        file_menu.add_command(label="Open...", accelerator="Ctrl+O", command=self._open_project)
        file_menu.add_command(label="Save", accelerator="Ctrl+S", command=self._save_project)
        file_menu.add_command(label="Save As...", command=self._save_project_as)
        file_menu.add_separator()
        file_menu.add_command(label="Export PNG...", command=self._export_png)
        file_menu.add_separator()
        file_menu.add_command(label="Quit", accelerator="Ctrl+Q", command=self.root.quit)

        # Edit menu
        edit_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="Edit", menu=edit_menu)
        edit_menu.add_command(label="Undo", accelerator="Ctrl+Z", command=self._undo)
        edit_menu.add_command(label="Redo", accelerator="Ctrl+Y", command=self._redo)

        # View menu
        view_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="View", menu=view_menu)
        view_menu.add_command(label="Toggle Grid", accelerator="G", command=self._toggle_grid)

    def _build_layout(self) -> None:
        # Main horizontal layout
        main_frame = tk.Frame(self.root)
        main_frame.pack(fill=tk.BOTH, expand=True)

        # Left panel
        left_panel = tk.Frame(main_frame, width=200)
        left_panel.pack(side=tk.LEFT, fill=tk.Y)
        left_panel.pack_propagate(False)

        self.toolbar = Toolbar(left_panel, on_tool_selected=self._on_tool_selected)
        self.toolbar.pack(fill=tk.X)

        self.palette_panel = PalettePanel(
            left_panel,
            on_paint_mode_changed=self._on_paint_mode_changed,
            on_decorator_changed=self._on_decorator_changed,
            on_path_type_changed=self._on_path_type_changed,
        )
        self.palette_panel.pack(fill=tk.X)

        self.minimap = Minimap(
            left_panel,
            self.project.grid,
            on_view_changed=self._on_minimap_click,
        )
        self.minimap.pack(pady=4)

        # Right panel
        right_panel = tk.Frame(main_frame, width=220)
        right_panel.pack(side=tk.RIGHT, fill=tk.Y)
        right_panel.pack_propagate(False)

        self.properties_panel = PropertiesPanel(
            right_panel,
            on_elevation_changed=self._on_elevation_changed,
            on_climate_changed=self._on_climate_changed,
        )
        self.properties_panel.pack(fill=tk.BOTH, expand=True)

        # Center: map viewport
        self.viewport = MapViewport(
            main_frame,
            self.project.grid,
            self.project.paths,
        )
        self.viewport.pack(fill=tk.BOTH, expand=True)

        # Status bar
        self._status_var = tk.StringVar(value="Ready")
        status_bar = tk.Label(
            self.root,
            textvariable=self._status_var,
            bd=1,
            relief=tk.SUNKEN,
            anchor=tk.W,
            padx=8,
        )
        status_bar.pack(side=tk.BOTTOM, fill=tk.X)

    def _build_tools(self) -> None:
        grid = self.project.grid

        self.brush_tool = BrushTool(
            grid, self.tool_manager, on_hex_changed=self._on_hex_changed
        )
        self.pen_tool = PenTool(
            self.project, self.tool_manager, on_path_added=self._on_path_added
        )
        self.select_tool = SelectTool(grid, on_selection_changed=self._on_selection_changed)
        self.label_tool = LabelTool(
            grid, self.tool_manager,
            on_label_request=self._on_label_request,
            on_hex_changed=self._on_hex_changed,
        )
        self.eraser_tool = EraserTool(
            grid, self.tool_manager, on_hex_changed=self._on_hex_changed
        )
        self.eyedropper_tool = EyedropperTool(grid, on_sample=self._on_eyedropper_sample)

        self._tools = {
            "brush": self.brush_tool,
            "pen": self.pen_tool,
            "select": self.select_tool,
            "label": self.label_tool,
            "eraser": self.eraser_tool,
            "eyedropper": self.eyedropper_tool,
        }

        self.tool_manager.set_tool(self.brush_tool)

    def _connect_signals(self) -> None:
        self.viewport.on_tool_press = self._on_tool_press
        self.viewport.on_tool_move = self._on_tool_move
        self.viewport.on_tool_release = self._on_tool_release
        self.viewport.on_hex_hovered = self._on_hex_hovered
        self.viewport.on_hex_clicked = self._on_hex_clicked

    def _initial_render(self) -> None:
        self.viewport.center_on_hex(15, 10)
        self.minimap.render()

    # --- Tool Events ---

    def _on_tool_press(self, wx: float, wy: float) -> None:
        tool = self.tool_manager.current_tool
        if tool and hasattr(tool, "on_press"):
            tool.on_press(wx, wy)

    def _on_tool_move(self, wx: float, wy: float) -> None:
        tool = self.tool_manager.current_tool
        if tool and hasattr(tool, "on_move"):
            tool.on_move(wx, wy)

    def _on_tool_release(self, wx: float, wy: float) -> None:
        tool = self.tool_manager.current_tool
        if tool and hasattr(tool, "on_release"):
            tool.on_release(wx, wy)
        self.viewport.invalidate()

    def _on_tool_selected(self, tool_id: str) -> None:
        tool = self._tools.get(tool_id)
        if tool:
            self.tool_manager.set_tool(tool)
            self._status_var.set(f"Tool: {tool_id.capitalize()}")

    # --- Hex Events ---

    def _on_hex_changed(self, q: int, r: int) -> None:
        self.viewport.invalidate_hex(q, r)

    def _on_hex_hovered(self, q: int, r: int) -> None:
        cell = self.project.grid.get(q, r)
        if cell:
            self._status_var.set(
                f"({q}, {r})  {cell.elevation.name}  {cell.climate.name}"
            )

    def _on_hex_clicked(self, q: int, r: int) -> None:
        cell = self.project.grid.get(q, r)
        self.properties_panel.update_hex_info(cell)

    # --- Path / Selection / Label ---

    def _on_path_added(self, path: object) -> None:
        self.viewport.invalidate_all_tiles()

    def _on_selection_changed(self, selected: set) -> None:
        count = len(selected)
        self._status_var.set(f"{count} hex{'es' if count != 1 else ''} selected")

    def _on_label_request(self, q: int, r: int, current_text: str) -> None:
        text = simpledialog.askstring(
            "Set Label", f"Label for ({q}, {r}):", initialvalue=current_text,
            parent=self.root,
        )
        if text is not None:
            self.label_tool.set_label(q, r, text)

    def _on_eyedropper_sample(
        self, elevation: Elevation, climate: Climate, decorators: list
    ) -> None:
        self.brush_tool.elevation_value = elevation
        self.brush_tool.climate_value = climate
        self._status_var.set(f"Sampled: {elevation.name}, {climate.name}")

    # --- Settings Changes ---

    def _on_elevation_changed(self, elevation: Elevation) -> None:
        self.brush_tool.elevation_value = elevation

    def _on_climate_changed(self, climate: Climate) -> None:
        self.brush_tool.climate_value = climate

    def _on_paint_mode_changed(self, mode: str) -> None:
        self.brush_tool.paint_mode = mode

    def _on_decorator_changed(self, decorator: Decorator) -> None:
        self.brush_tool.decorator_value = decorator

    def _on_path_type_changed(self, path_type: PathType) -> None:
        self.pen_tool.path_type = path_type

    # --- Minimap ---

    def _on_minimap_click(self, world_x: float, world_y: float) -> None:
        scale = 2.0 ** self.viewport.zoom
        self.viewport.view_x = world_x - self.viewport.winfo_width() / (2 * scale)
        self.viewport.view_y = world_y - self.viewport.winfo_height() / (2 * scale)
        self.viewport.invalidate()

    # --- File Operations ---

    def _new_project(self) -> None:
        self.project = Project("New Map")
        self.project.grid = HexGrid(width=30, height=20, hex_size=64.0)
        self.project.grid.initialize_rectangular()
        self._rebuild_after_load()

    def _open_project(self) -> None:
        path = filedialog.askopenfilename(
            title="Open Project",
            filetypes=[("Hex Map Files", "*.json"), ("All Files", "*.*")],
        )
        if path:
            try:
                self.project.load(path)
                self._rebuild_after_load()
                self._status_var.set(f"Opened: {path}")
            except Exception as e:
                messagebox.showerror("Error", f"Failed to open: {e}")

    def _save_project(self) -> None:
        if self.project.file_path:
            self.project.save(self.project.file_path)
            self._status_var.set(f"Saved: {self.project.file_path}")
        else:
            self._save_project_as()

    def _save_project_as(self) -> None:
        path = filedialog.asksaveasfilename(
            title="Save Project",
            defaultextension=".json",
            filetypes=[("Hex Map Files", "*.json"), ("All Files", "*.*")],
        )
        if path:
            self.project.save(path)
            self._status_var.set(f"Saved: {path}")

    def _export_png(self) -> None:
        path = filedialog.asksaveasfilename(
            title="Export PNG",
            defaultextension=".png",
            filetypes=[("PNG Files", "*.png"), ("All Files", "*.*")],
        )
        if path:
            try:
                bounds = self.project.grid.world_bounds()
                img = self.viewport.compositor.render_viewport(
                    bounds[0], bounds[1], 1024, 768, self.viewport.zoom
                )
                img.save(path, "PNG")
                self._status_var.set(f"Exported: {path}")
            except Exception as e:
                messagebox.showerror("Error", f"Export failed: {e}")

    # --- Edit Operations ---

    def _undo(self) -> None:
        entry = self.tool_manager.undo()
        if entry:
            self._apply_undo_state(entry.old_state)
            self._status_var.set(f"Undo: {entry.name}")

    def _redo(self) -> None:
        entry = self.tool_manager.redo()
        if entry:
            self._apply_undo_state(entry.new_state)
            self._status_var.set(f"Redo: {entry.name}")

    def _apply_undo_state(self, state: dict) -> None:
        if "cells" in state:
            from .model.hex_cell import HexCell
            for key, cell_data in state["cells"].items():
                cell = HexCell.from_dict(cell_data)
                self.project.grid.set(cell.q, cell.r, cell)
                self.viewport.invalidate_hex(cell.q, cell.r)
        self.viewport.invalidate()
        self.minimap.render()

    def _toggle_grid(self) -> None:
        self.viewport.compositor.show_grid = not self.viewport.compositor.show_grid
        self.viewport.invalidate_all_tiles()

    def _rebuild_after_load(self) -> None:
        self.viewport.grid = self.project.grid
        self.viewport.paths = self.project.paths
        self.viewport.compositor = MapCompositor(self.project.grid, self.project.paths)
        self._build_tools()
        self._connect_signals()
        self.viewport.center_on_hex(
            self.project.grid.width // 2, self.project.grid.height // 2
        )
        self.minimap.grid = self.project.grid
        self.viewport.invalidate_all_tiles()
        self.minimap.render()

    def _on_key(self, event: tk.Event) -> None:
        shortcuts = {
            "b": "brush",
            "p": "pen",
            "s": "select",
            "l": "label",
            "e": "eraser",
            "i": "eyedropper",
        }
        tool_id = shortcuts.get(event.char)
        if tool_id:
            self._on_tool_selected(tool_id)
            self.toolbar.set_active_tool(tool_id)
        elif event.char == "g":
            self._toggle_grid()

    def run(self) -> None:
        """Start the application main loop."""
        self.root.mainloop()
