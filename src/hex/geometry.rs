use std::f32::consts::PI;

pub const SPRITE_SIZE: u32 = 170;
pub const HEX_RADIUS: f32 = 85.0;
pub const HEX_WIDTH: f32 = 170.0;
pub const HEX_HEIGHT: f32 = 147.224;
pub const HEX_HORIZ: f32 = 127.5;
pub const HEX_VERT: f32 = 147.224;
pub const HEX_HALF_H: f32 = 73.612;

/// Axial neighbor offsets for flat-top hexes.
/// Index 0 = N, 1 = NE, 2 = SE, 3 = S, 4 = SW, 5 = NW
pub const AXIAL_NEIGHBORS: [(i32, i32); 6] = [
    (0, -1),  // 0: N
    (1, -1),  // 1: NE
    (1, 0),   // 2: SE
    (0, 1),   // 3: S
    (-1, 1),  // 4: SW
    (-1, 0),  // 5: NW
];

/// Convert axial hex coordinates to pixel coordinates (flat-top layout).
pub fn hex_to_pixel(q: i32, r: i32) -> (f32, f32) {
    let x = HEX_RADIUS * (3.0 / 2.0 * q as f32);
    let y = HEX_RADIUS * (3.0_f32.sqrt() * (r as f32 + q as f32 / 2.0));
    (x, y)
}

/// Convert pixel coordinates to axial hex coordinates using cube-coordinate rounding.
#[allow(unused_assignments)]
pub fn pixel_to_hex(px: f32, py: f32) -> (i32, i32) {
    let sqrt3 = 3.0_f32.sqrt();

    // Inverse of hex_to_pixel for flat-top
    let q_frac = (2.0 / 3.0) * px / HEX_RADIUS;
    let r_frac = (-1.0 / 3.0 * px + sqrt3 / 3.0 * py) / HEX_RADIUS;

    // Convert axial fractional to cube fractional
    let x = q_frac;
    let z = r_frac;
    let y = -x - z;

    // Round cube coordinates
    let mut rx = x.round();
    let mut ry = y.round();
    let mut rz = z.round();

    let x_diff = (rx - x).abs();
    let y_diff = (ry - y).abs();
    let z_diff = (rz - z).abs();

    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff > z_diff {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }

    (rx as i32, rz as i32)
}

/// Return the opposite edge index: (edge + 3) % 6
pub fn opposite_edge(edge: usize) -> usize {
    (edge + 3) % 6
}

/// Return all hex coordinates in a ring of the given radius around (cq, cr).
/// Returns an empty vec for radius <= 0.
pub fn hex_ring(cq: i32, cr: i32, radius: i32) -> Vec<(i32, i32)> {
    if radius <= 0 {
        return vec![];
    }

    let mut results = Vec::new();

    // Start at the hex reached by moving `radius` steps in direction 4 (SW)
    let (dq, dr) = AXIAL_NEIGHBORS[4];
    let mut q = cq + dq * radius;
    let mut r = cr + dr * radius;

    // Walk along each of the 6 edges
    for direction in 0..6 {
        let (nq, nr) = AXIAL_NEIGHBORS[direction];
        for _ in 0..radius {
            results.push((q, r));
            q += nq;
            r += nr;
        }
    }

    results
}

/// Return the 6 corner points of a flat-top hex centered at (cx, cy) with the given radius.
pub fn hex_polygon_points(cx: f32, cy: f32, radius: f32) -> Vec<egui::Pos2> {
    (0..6)
        .map(|i| {
            let angle = PI / 180.0 * (60.0 * i as f32);
            egui::Pos2::new(cx + radius * angle.cos(), cy + radius * angle.sin())
        })
        .collect()
}
