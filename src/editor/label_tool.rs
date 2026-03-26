use crate::hex_math;
use crate::model::hex_grid::HexGrid;

pub struct LabelTool {
    /// When set, the UI should prompt the user for label text at this hex.
    pub pending_label: Option<(i32, i32)>,
}

impl LabelTool {
    pub fn new() -> Self {
        Self {
            pending_label: None,
        }
    }

    pub fn on_press(&mut self, world_x: f32, world_y: f32, grid: &HexGrid) {
        let (q, r) = hex_math::pixel_to_hex(world_x, world_y, grid.hex_size);
        if grid.get(q, r).is_some() {
            self.pending_label = Some((q, r));
        }
    }

    /// Apply the label text to the hex cell. Call this after the UI collects
    /// the label string from the user.
    pub fn apply_label(&mut self, grid: &mut HexGrid, text: Option<String>) {
        if let Some((q, r)) = self.pending_label.take() {
            if let Some(cell) = grid.get_mut(q, r) {
                cell.label = text.filter(|s| !s.is_empty());
            }
        }
    }
}

impl Default for LabelTool {
    fn default() -> Self {
        Self::new()
    }
}
