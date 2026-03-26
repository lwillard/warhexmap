use std::collections::HashMap;
use crate::hex_math;
use crate::model::hex_cell::HexCell;
use crate::model::hex_grid::HexGrid;
use super::tool_manager::{ToolManager, UndoEntry};

pub struct EraserTool {
    pub radius: i32,
    erasing: bool,
    old_states: HashMap<(i32, i32), HexCell>,
}

impl EraserTool {
    pub fn new() -> Self {
        Self {
            radius: 0,
            erasing: false,
            old_states: HashMap::new(),
        }
    }

    pub fn on_press(
        &mut self,
        world_x: f32,
        world_y: f32,
        grid: &mut HexGrid,
        tool_manager: &mut ToolManager,
    ) {
        self.erasing = true;
        self.old_states.clear();
        self.erase_at(world_x, world_y, grid, tool_manager);
    }

    pub fn on_move(
        &mut self,
        world_x: f32,
        world_y: f32,
        grid: &mut HexGrid,
        tool_manager: &mut ToolManager,
    ) {
        if self.erasing {
            self.erase_at(world_x, world_y, grid, tool_manager);
        }
    }

    pub fn on_release(&mut self, tool_manager: &mut ToolManager, grid: &HexGrid) {
        if !self.erasing {
            return;
        }
        self.erasing = false;

        if self.old_states.is_empty() {
            return;
        }

        let mut new_states = HashMap::new();
        for &(q, r) in self.old_states.keys() {
            if let Some(cell) = grid.get(q, r) {
                new_states.insert((q, r), cell.clone());
            }
        }

        tool_manager.push_undo(UndoEntry {
            name: "Erase Decorators".to_string(),
            old_cells: std::mem::take(&mut self.old_states),
            new_cells: new_states,
        });
    }

    fn erase_at(
        &mut self,
        world_x: f32,
        world_y: f32,
        grid: &mut HexGrid,
        tool_manager: &mut ToolManager,
    ) {
        let (cq, cr) = hex_math::pixel_to_hex(world_x, world_y, grid.hex_size);
        let affected = hex_math::hexes_in_radius(cq, cr, self.radius);

        for (q, r) in affected {
            if let Some(cell) = grid.get_mut(q, r) {
                if cell.decorators.is_empty() {
                    continue;
                }
                if !self.old_states.contains_key(&(q, r)) {
                    self.old_states.insert((q, r), cell.clone());
                }
                cell.decorators.clear();
                tool_manager.mark_dirty(q, r);
            }
        }
    }
}

impl Default for EraserTool {
    fn default() -> Self {
        Self::new()
    }
}
