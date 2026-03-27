use std::collections::HashMap;

use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct HexCell {
    pub terrain_id: Uuid,
    pub variant_index: u8,
    pub edge_variants: [u8; 6],
    pub decorators: Vec<DecoratorPlacement>,
    pub label: Option<String>,
    pub elevation: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DecoratorPlacement {
    pub decorator_id: Uuid,
    pub variant_index: u8,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HexMap {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub cells: HashMap<(i32, i32), HexCell>,
    pub default_terrain: Uuid,
}

impl HexMap {
    /// Create a new rectangular hex map filled with the default terrain.
    pub fn new(name: &str, width: u32, height: u32, default_terrain: Uuid) -> Self {
        let mut cells = HashMap::new();
        for q in 0..width as i32 {
            for r in 0..height as i32 {
                cells.insert(
                    (q, r),
                    HexCell {
                        terrain_id: default_terrain,
                        variant_index: 0,
                        edge_variants: [0; 6],
                        decorators: Vec::new(),
                        label: None,
                        elevation: 0,
                    },
                );
            }
        }
        Self {
            name: name.to_string(),
            width,
            height,
            cells,
            default_terrain,
        }
    }

    pub fn get(&self, q: i32, r: i32) -> Option<&HexCell> {
        self.cells.get(&(q, r))
    }

    pub fn try_get(&self, q: i32, r: i32) -> Option<&HexCell> {
        self.cells.get(&(q, r))
    }

    /// Set the terrain for a cell.
    ///
    /// If the cell already has the same terrain, cycle the base variant.
    /// If it is a different terrain, assign with a random variant and random edge variants.
    pub fn set_terrain(
        &mut self,
        q: i32,
        r: i32,
        terrain_id: Uuid,
        variant_count: u8,
        edge_variant_counts: &[u8; 6],
    ) {
        let variant_count = variant_count.max(1);

        if let Some(cell) = self.cells.get_mut(&(q, r)) {
            if cell.terrain_id == terrain_id {
                // Same terrain: cycle variant
                cell.variant_index = (cell.variant_index + 1) % variant_count;
            } else {
                // Different terrain: assign with random variants
                let mut rng = rand::thread_rng();
                cell.terrain_id = terrain_id;
                cell.variant_index = rng.gen_range(0..variant_count);
                for i in 0..6 {
                    let edge_count = edge_variant_counts[i].max(1);
                    cell.edge_variants[i] = rng.gen_range(0..edge_count);
                }
            }
        }
    }
}
