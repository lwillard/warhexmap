use std::collections::{HashMap, HashSet};
use crate::model::hex_cell::HexCell;

#[derive(Clone, Debug)]
pub struct UndoEntry {
    pub name: String,
    pub old_cells: HashMap<(i32, i32), HexCell>,
    pub new_cells: HashMap<(i32, i32), HexCell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Brush,
    Pen,
    Select,
    Label,
    Eraser,
    Eyedropper,
}

pub struct ToolManager {
    pub active_tool: ToolKind,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
    pub dirty_hexes: HashSet<(i32, i32)>,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            active_tool: ToolKind::Brush,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty_hexes: HashSet::new(),
        }
    }

    pub fn push_undo(&mut self, entry: UndoEntry) {
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, grid: &mut crate::model::hex_grid::HexGrid) -> bool {
        if let Some(entry) = self.undo_stack.pop() {
            // Restore old cells
            for ((q, r), cell) in &entry.old_cells {
                self.dirty_hexes.insert((*q, *r));
                grid.set(cell.clone());
            }
            self.redo_stack.push(entry);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, grid: &mut crate::model::hex_grid::HexGrid) -> bool {
        if let Some(entry) = self.redo_stack.pop() {
            // Apply new cells
            for ((q, r), cell) in &entry.new_cells {
                self.dirty_hexes.insert((*q, *r));
                grid.set(cell.clone());
            }
            self.undo_stack.push(entry);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn mark_dirty(&mut self, q: i32, r: i32) {
        self.dirty_hexes.insert((q, r));
    }

    /// Return the current set of dirty hexes and clear it.
    pub fn take_dirty(&mut self) -> HashSet<(i32, i32)> {
        std::mem::take(&mut self.dirty_hexes)
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}
