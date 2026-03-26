use crate::hex_math;
use crate::model::hex_grid::HexGrid;
use crate::model::terrain_types::{Climate, Elevation};

pub struct EyedropperTool {
    /// The most recently sampled elevation and climate, if any.
    pub sampled: Option<(Elevation, Climate)>,
}

impl EyedropperTool {
    pub fn new() -> Self {
        Self { sampled: None }
    }

    pub fn on_press(&mut self, world_x: f32, world_y: f32, grid: &HexGrid) {
        let (q, r) = hex_math::pixel_to_hex(world_x, world_y, grid.hex_size);
        if let Some(cell) = grid.get(q, r) {
            self.sampled = Some((cell.elevation, cell.climate));
        }
    }
}

impl Default for EyedropperTool {
    fn default() -> Self {
        Self::new()
    }
}
