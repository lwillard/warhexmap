use crate::hex::geometry::hex_ring;

#[derive(PartialEq, Clone, Copy)]
pub enum MapTool {
    PaintTerrain,
    PaintDecorator,
    Eraser,
    Eyedropper,
    Label,
}

/// Returns all hex coords affected by a brush of given size centered at (q, r).
///
/// brush_size 1 = single hex
/// brush_size 2 = hex + immediate ring
/// etc.
pub fn brush_coords(q: i32, r: i32, brush_size: u8) -> Vec<(i32, i32)> {
    let mut coords = vec![(q, r)];
    for radius in 1..brush_size as i32 {
        coords.extend(hex_ring(q, r, radius));
    }
    coords
}
