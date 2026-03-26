use crate::model::path_feature::PathFeature;
use crate::model::terrain_types::PathType;
use super::curve_simplify::rdp_simplify;

pub struct PenTool {
    pub path_type: PathType,
    raw_points: Vec<[f32; 2]>,
    drawing: bool,
}

impl PenTool {
    pub fn new() -> Self {
        Self {
            path_type: PathType::Road,
            raw_points: Vec::new(),
            drawing: false,
        }
    }

    pub fn on_press(&mut self, world_x: f32, world_y: f32) {
        self.drawing = true;
        self.raw_points.clear();
        self.raw_points.push([world_x, world_y]);
    }

    pub fn on_move(&mut self, world_x: f32, world_y: f32) {
        if self.drawing {
            self.raw_points.push([world_x, world_y]);
        }
    }

    /// Finish drawing and return the simplified path feature, if any.
    pub fn on_release(&mut self) -> Option<PathFeature> {
        if !self.drawing {
            return None;
        }
        self.drawing = false;

        if self.raw_points.len() < 2 {
            self.raw_points.clear();
            return None;
        }

        let epsilon = 2.0;
        let simplified = rdp_simplify(&self.raw_points, epsilon);
        self.raw_points.clear();

        if simplified.len() < 2 {
            return None;
        }

        let mut feature = PathFeature::new(self.path_type);
        feature.control_points = simplified;
        Some(feature)
    }

    pub fn is_drawing(&self) -> bool {
        self.drawing
    }

    /// Return the raw points collected so far (for live preview).
    pub fn preview_points(&self) -> &[[f32; 2]] {
        &self.raw_points
    }
}

impl Default for PenTool {
    fn default() -> Self {
        Self::new()
    }
}
