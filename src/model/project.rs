use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

use super::hex_cell::HexCell;
use super::hex_grid::HexGrid;
use super::path_feature::PathFeature;
use super::terrain_types::{Climate, Elevation};

/// On-disk JSON representation, compatible with the Python editor format.
#[derive(Serialize, Deserialize)]
struct ProjectFile {
    version: String,
    name: String,
    grid: GridInfo,
    hexes: Vec<serde_json::Value>,
    paths: Vec<PathFeature>,
    #[serde(default)]
    labels: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct GridInfo {
    width: u32,
    height: u32,
    hex_size: f32,
}

pub struct Project {
    pub name: String,
    pub grid: HexGrid,
    pub paths: Vec<PathFeature>,
    pub file_path: Option<PathBuf>,
}

impl Project {
    pub fn new() -> Self {
        let mut grid = HexGrid::new(30, 20, 32.0);
        grid.initialize_rectangular(Elevation::Plains, Climate::Cf);
        Self {
            name: String::from("Untitled"),
            grid,
            paths: Vec::new(),
            file_path: None,
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let hexes: Vec<serde_json::Value> = self
            .grid
            .cells
            .values()
            .map(|cell| serde_json::to_value(cell).unwrap_or_default())
            .collect();

        let labels: Vec<serde_json::Value> = self
            .grid
            .cells
            .values()
            .filter_map(|cell| {
                cell.label.as_ref().map(|text| {
                    serde_json::json!({
                        "q": cell.q,
                        "r": cell.r,
                        "text": text,
                        "offset": [cell.label_offset.0, cell.label_offset.1],
                    })
                })
            })
            .collect();

        let project_file = ProjectFile {
            version: "1.0".to_string(),
            name: self.name.clone(),
            grid: GridInfo {
                width: self.grid.width,
                height: self.grid.height,
                hex_size: self.grid.hex_size,
            },
            hexes,
            paths: self.paths.clone(),
            labels,
        };

        let json = serde_json::to_string_pretty(&project_file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn load(&mut self, path: &Path) -> io::Result<()> {
        let data = std::fs::read_to_string(path)?;
        let pf: ProjectFile =
            serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.name = pf.name;
        self.grid = HexGrid::new(pf.grid.width, pf.grid.height, pf.grid.hex_size);

        for hex_val in pf.hexes {
            let cell: HexCell = serde_json::from_value(hex_val)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.grid.set(cell);
        }

        // Merge label data back into cells
        for label_val in pf.labels {
            if let (Some(q), Some(r), Some(text)) = (
                label_val.get("q").and_then(|v| v.as_i64()).map(|v| v as i32),
                label_val.get("r").and_then(|v| v.as_i64()).map(|v| v as i32),
                label_val.get("text").and_then(|v| v.as_str()),
            ) {
                if let Some(cell) = self.grid.get_mut(q, r) {
                    cell.label = Some(text.to_string());
                    if let Some(offset) = label_val.get("offset").and_then(|v| v.as_array()) {
                        if offset.len() == 2 {
                            let ox = offset[0].as_i64().unwrap_or(0) as i32;
                            let oy = offset[1].as_i64().unwrap_or(0) as i32;
                            cell.label_offset = (ox, oy);
                        }
                    }
                }
            }
        }

        self.paths = pf.paths;
        self.file_path = Some(path.to_path_buf());
        Ok(())
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}
