"""Tool palette toolbar using tkinter."""

from __future__ import annotations

import tkinter as tk
from typing import Callable


class Toolbar(tk.Frame):
    """Vertical tool selection bar."""

    TOOLS = [
        ("brush", "Brush (B)"),
        ("pen", "Pen (P)"),
        ("select", "Select (S)"),
        ("label", "Label (L)"),
        ("eraser", "Eraser (E)"),
        ("eyedropper", "Eyedropper (I)"),
    ]

    def __init__(
        self,
        parent: tk.Widget,
        on_tool_selected: Callable[[str], None] | None = None,
    ) -> None:
        super().__init__(parent, bd=1, relief=tk.RIDGE, padx=4, pady=4)
        self.on_tool_selected = on_tool_selected
        self._buttons: dict[str, tk.Button] = {}
        self._active_tool = "brush"

        tk.Label(self, text="Tools", font=("Arial", 10, "bold")).pack(pady=(0, 4))

        for tool_id, tooltip in self.TOOLS:
            btn = tk.Button(
                self,
                text=tool_id.capitalize(),
                width=10,
                command=lambda tid=tool_id: self._select(tid),
            )
            btn.pack(pady=1)
            self._buttons[tool_id] = btn

        self._highlight_active()

    def _select(self, tool_id: str) -> None:
        self._active_tool = tool_id
        self._highlight_active()
        if self.on_tool_selected:
            self.on_tool_selected(tool_id)

    def set_active_tool(self, tool_id: str) -> None:
        self._active_tool = tool_id
        self._highlight_active()

    def _highlight_active(self) -> None:
        for tid, btn in self._buttons.items():
            if tid == self._active_tool:
                btn.config(relief=tk.SUNKEN, bg="#b0c4de")
            else:
                btn.config(relief=tk.RAISED, bg="SystemButtonFace")
