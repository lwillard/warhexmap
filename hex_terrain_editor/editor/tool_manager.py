"""Tool switching and undo/redo stack management."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Protocol


class Tool(Protocol):
    """Protocol for editor tools."""

    def activate(self) -> None: ...
    def deactivate(self) -> None: ...
    def on_press(self, world_x: float, world_y: float) -> None: ...
    def on_move(self, world_x: float, world_y: float) -> None: ...
    def on_release(self, world_x: float, world_y: float) -> None: ...


@dataclass
class UndoEntry:
    """A single undo/redo entry."""

    name: str
    old_state: dict[str, Any]
    new_state: dict[str, Any]


class ToolManager:
    """Manages active tool and undo/redo stack."""

    def __init__(self) -> None:
        self.current_tool: Any | None = None
        self._undo_stack: list[UndoEntry] = []
        self._redo_stack: list[UndoEntry] = []
        self._on_change_callbacks: list[Any] = []

    def set_tool(self, tool: Any) -> None:
        if self.current_tool is not None and hasattr(self.current_tool, "deactivate"):
            self.current_tool.deactivate()
        self.current_tool = tool
        if hasattr(tool, "activate"):
            tool.activate()

    def push_undo(
        self, name: str, old_state: dict[str, Any], new_state: dict[str, Any]
    ) -> None:
        self._undo_stack.append(UndoEntry(name, old_state, new_state))
        self._redo_stack.clear()

    def undo(self) -> UndoEntry | None:
        if not self._undo_stack:
            return None
        entry = self._undo_stack.pop()
        self._redo_stack.append(entry)
        self._notify_change()
        return entry

    def redo(self) -> UndoEntry | None:
        if not self._redo_stack:
            return None
        entry = self._redo_stack.pop()
        self._undo_stack.append(entry)
        self._notify_change()
        return entry

    @property
    def can_undo(self) -> bool:
        return len(self._undo_stack) > 0

    @property
    def can_redo(self) -> bool:
        return len(self._redo_stack) > 0

    def on_change(self, callback: Any) -> None:
        self._on_change_callbacks.append(callback)

    def _notify_change(self) -> None:
        for cb in self._on_change_callbacks:
            cb()
