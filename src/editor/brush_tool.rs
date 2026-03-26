use std::collections::HashMap;
use crate::hex_math;
use crate::model::hex_cell::HexCell;
use crate::model::hex_grid::HexGrid;
use crate::model::terrain_types::{Climate, Decorator, Elevation};
use super::tool_manager::{ToolManager, UndoEntry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintMode {
    Elevation,
    Climate,
    Decorator,
}

pub struct BrushTool {
    pub paint_mode: PaintMode,
    pub elevation_value: Elevation,
    pub climate_value: Climate,
    pub decorator_value: Decorator,
    pub radius: i32,
    painting: bool,
    old_states: HashMap<(i32, i32), HexCell>,
}

impl BrushTool {
    pub fn new() -> Self {
        Self {
            paint_mode: PaintMode::Elevation,
            elevation_value: Elevation::Plains,
            climate_value: Climate::Cf,
            decorator_value: Decorator::Grassland,
            radius: 0,
            painting: false,
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
        self.painting = true;
        self.old_states.clear();
        self.paint_at(world_x, world_y, grid, tool_manager);
    }

    pub fn on_move(
        &mut self,
        world_x: f32,
        world_y: f32,
        grid: &mut HexGrid,
        tool_manager: &mut ToolManager,
    ) {
        if self.painting {
            self.paint_at(world_x, world_y, grid, tool_manager);
        }
    }

    pub fn on_release(
        &mut self,
        _world_x: f32,
        _world_y: f32,
        grid: &mut HexGrid,
        tool_manager: &mut ToolManager,
    ) {
        if !self.painting {
            return;
        }
        self.painting = false;

        if self.old_states.is_empty() {
            return;
        }

        // Collect new states for undo
        let mut new_states = HashMap::new();
        for &(q, r) in self.old_states.keys() {
            if let Some(cell) = grid.get(q, r) {
                new_states.insert((q, r), cell.clone());
            }
        }

        let name = match self.paint_mode {
            PaintMode::Elevation => format!("Paint Elevation {:?}", self.elevation_value),
            PaintMode::Climate => format!("Paint Climate {:?}", self.climate_value),
            PaintMode::Decorator => format!("Paint Decorator {:?}", self.decorator_value),
        };

        tool_manager.push_undo(UndoEntry {
            name,
            old_cells: std::mem::take(&mut self.old_states),
            new_cells: new_states,
        });
    }

    fn paint_at(
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
                // Save original state if not already recorded
                if !self.old_states.contains_key(&(q, r)) {
                    self.old_states.insert((q, r), cell.clone());
                }

                match self.paint_mode {
                    PaintMode::Elevation => cell.elevation = self.elevation_value,
                    PaintMode::Climate => cell.climate = self.climate_value,
                    PaintMode::Decorator => cell.add_decorator(self.decorator_value),
                }

                tool_manager.mark_dirty(q, r);
            }
        }
    }
}

impl Default for BrushTool {
    fn default() -> Self {
        Self::new()
    }
}
