use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::terrain_types::PathType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathFeature {
    pub id: String,
    pub feature_type: PathType,
    pub control_points: Vec<[f32; 2]>,
    pub width: f32,
    #[serde(default)]
    pub properties: serde_json::Value,
}

impl PathFeature {
    pub fn new(feature_type: PathType) -> Self {
        let id = Uuid::new_v4().to_string();
        let short_id = id[..8].to_string();
        Self {
            id: short_id,
            feature_type,
            control_points: Vec::new(),
            width: feature_type.width(),
            properties: serde_json::Value::Null,
        }
    }

    /// Compute axis-aligned bounding box of the path, padded by `width * 2`.
    /// Returns `(min_x, min_y, max_x, max_y)`.
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        if self.control_points.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for pt in &self.control_points {
            min_x = min_x.min(pt[0]);
            min_y = min_y.min(pt[1]);
            max_x = max_x.max(pt[0]);
            max_y = max_y.max(pt[1]);
        }
        let pad = self.width * 2.0;
        (min_x - pad, min_y - pad, max_x + pad, max_y + pad)
    }
}
