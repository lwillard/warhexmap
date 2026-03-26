use serde::{Deserialize, Serialize};
use super::terrain_types::{Climate, Decorator, Elevation};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HexCell {
    pub q: i32,
    pub r: i32,
    pub elevation: Elevation,
    pub climate: Climate,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<Decorator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub label_offset: (i32, i32),
}

impl HexCell {
    pub fn new(q: i32, r: i32) -> Self {
        Self {
            q,
            r,
            elevation: Elevation::Plains,
            climate: Climate::Cf,
            decorators: Vec::new(),
            label: None,
            label_offset: (0, 0),
        }
    }

    pub fn elevation_band(&self) -> &'static str {
        if self.elevation.is_water() {
            "water"
        } else {
            match self.elevation {
                Elevation::Hills => "hills",
                Elevation::Mountains => "mountain",
                _ => "plains",
            }
        }
    }

    pub fn add_decorator(&mut self, d: Decorator) {
        if !self.decorators.contains(&d) {
            self.decorators.push(d);
        }
    }

    pub fn remove_decorator(&mut self, d: Decorator) {
        self.decorators.retain(|existing| *existing != d);
    }

    pub fn texture_index(&self) -> u32 {
        self.elevation.texture_index(self.climate)
    }
}
