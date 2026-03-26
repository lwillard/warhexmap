use std::collections::HashSet;
use crate::hex_math;
use crate::model::hex_grid::HexGrid;

pub struct SelectTool {
    pub selected: HashSet<(i32, i32)>,
    dragging: bool,
    start_hex: Option<(i32, i32)>,
}

impl SelectTool {
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            dragging: false,
            start_hex: None,
        }
    }

    pub fn on_press(&mut self, world_x: f32, world_y: f32, grid: &HexGrid, additive: bool) {
        let (q, r) = hex_math::pixel_to_hex(world_x, world_y, grid.hex_size);
        self.dragging = true;
        self.start_hex = Some((q, r));

        if !additive {
            self.selected.clear();
        }

        if grid.get(q, r).is_some() {
            self.selected.insert((q, r));
        }
    }

    pub fn on_move(&mut self, world_x: f32, world_y: f32, grid: &HexGrid) {
        if !self.dragging {
            return;
        }
        let (q, r) = hex_math::pixel_to_hex(world_x, world_y, grid.hex_size);
        if grid.get(q, r).is_some() {
            self.selected.insert((q, r));
        }
    }

    pub fn on_release(&mut self) {
        self.dragging = false;
        self.start_hex = None;
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn is_selected(&self, q: i32, r: i32) -> bool {
        self.selected.contains(&(q, r))
    }
}

impl Default for SelectTool {
    fn default() -> Self {
        Self::new()
    }
}
