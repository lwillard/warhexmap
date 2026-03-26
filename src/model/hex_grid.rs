use std::collections::HashMap;
use super::hex_cell::HexCell;
use super::terrain_types::{Climate, Elevation};
use crate::hex_math;

pub struct HexGrid {
    pub width: u32,
    pub height: u32,
    pub hex_size: f32,
    pub cells: HashMap<(i32, i32), HexCell>,
    /// Offset applied when mapping axial coords to the data texture.
    /// Set by `build_hex_data_texture`.
    pub grid_offset: (i32, i32),
}

impl HexGrid {
    pub fn new(width: u32, height: u32, hex_size: f32) -> Self {
        Self {
            width,
            height,
            hex_size,
            cells: HashMap::new(),
            grid_offset: (0, 0),
        }
    }

    pub fn get(&self, q: i32, r: i32) -> Option<&HexCell> {
        self.cells.get(&(q, r))
    }

    pub fn get_mut(&mut self, q: i32, r: i32) -> Option<&mut HexCell> {
        self.cells.get_mut(&(q, r))
    }

    pub fn set(&mut self, cell: HexCell) {
        self.cells.insert((cell.q, cell.r), cell);
    }

    pub fn remove(&mut self, q: i32, r: i32) -> Option<HexCell> {
        self.cells.remove(&(q, r))
    }

    /// Fill a rectangular region of the grid with default cells.
    pub fn initialize_rectangular(&mut self, default_elevation: Elevation, default_climate: Climate) {
        self.cells.clear();
        for r in 0..self.height as i32 {
            for q in 0..self.width as i32 {
                let mut cell = HexCell::new(q, r);
                cell.elevation = default_elevation;
                cell.climate = default_climate;
                self.cells.insert((q, r), cell);
            }
        }
    }

    /// Iterate over all cell coordinates.
    pub fn cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.cells.keys().copied()
    }

    /// Iterate over all cells.
    pub fn cells_iter(&self) -> impl Iterator<Item = &HexCell> {
        self.cells.values()
    }

    /// Compute the bounding box in axial coordinates.
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        if self.cells.is_empty() {
            return (0, 0, 0, 0);
        }
        let mut min_q = i32::MAX;
        let mut min_r = i32::MAX;
        let mut max_q = i32::MIN;
        let mut max_r = i32::MIN;
        for &(q, r) in self.cells.keys() {
            min_q = min_q.min(q);
            min_r = min_r.min(r);
            max_q = max_q.max(q);
            max_r = max_r.max(r);
        }
        (min_q, min_r, max_q, max_r)
    }

    /// Compute the bounding box in world (pixel) coordinates, including hex
    /// extents at the edges.
    pub fn world_bounds(&self, hex_size: f32) -> (f32, f32, f32, f32) {
        if self.cells.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let hw = hex_math::hex_width(hex_size) * 0.5;
        let hh = hex_math::hex_height(hex_size) * 0.5;
        for &(q, r) in self.cells.keys() {
            let (px, py) = hex_math::hex_to_pixel(q, r, hex_size);
            min_x = min_x.min(px - hw);
            min_y = min_y.min(py - hh);
            max_x = max_x.max(px + hw);
            max_y = max_y.max(py + hh);
        }
        (min_x, min_y, max_x, max_y)
    }

    /// Return references to all existing neighbor cells for a given hex.
    pub fn get_neighbors(&self, q: i32, r: i32) -> Vec<&HexCell> {
        let neighbors = hex_math::hex_neighbors(q, r);
        neighbors
            .iter()
            .filter_map(|&(nq, nr)| self.cells.get(&(nq, nr)))
            .collect()
    }

    /// Build a data texture for GPU upload.
    ///
    /// Each pixel at position `(q - min_q, r - min_r)` stores:
    /// - R: texture_index
    /// - G: elevation value (discriminant)
    /// - B: 0
    /// - A: 255
    ///
    /// Returns `(pixel_data, texture_width, texture_height)`.
    /// Also updates `self.grid_offset` with `(min_q, min_r)`.
    pub fn build_hex_data_texture(&mut self) -> (Vec<u8>, u32, u32) {
        let (min_q, min_r, max_q, max_r) = self.bounds();
        self.grid_offset = (min_q, min_r);

        let tex_w = (max_q - min_q + 1).max(1) as u32;
        let tex_h = (max_r - min_r + 1).max(1) as u32;
        let mut data = vec![0u8; (tex_w * tex_h * 4) as usize];

        for cell in self.cells.values() {
            let col = (cell.q - min_q) as u32;
            let row = (cell.r - min_r) as u32;
            let idx = ((row * tex_w + col) * 4) as usize;
            if idx + 3 < data.len() {
                data[idx] = cell.texture_index() as u8;
                data[idx + 1] = cell.elevation as u8;
                data[idx + 2] = 0;
                data[idx + 3] = 255;
            }
        }

        (data, tex_w, tex_h)
    }
}
