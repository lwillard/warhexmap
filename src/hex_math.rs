/// Hex geometry calculations using axial coordinates with flat-top hexes.

/// Flat-top hex neighbor offsets in axial coordinates (q, r).
pub const NEIGHBOR_OFFSETS: [(i32, i32); 6] = [
    (1, 0),   // East
    (-1, 0),  // West
    (1, -1),  // NE
    (0, -1),  // NW
    (0, 1),   // SE
    (-1, 1),  // SW
];

const SQRT3: f32 = 1.732_050_8;

/// Convert axial hex coordinates to pixel (world) coordinates.
pub fn hex_to_pixel(q: i32, r: i32, size: f32) -> (f32, f32) {
    let x = size * (1.5 * q as f32);
    let y = size * (SQRT3 * 0.5 * q as f32 + SQRT3 * r as f32);
    (x, y)
}

/// Convert pixel (world) coordinates to axial hex coordinates.
pub fn pixel_to_hex(x: f32, y: f32, size: f32) -> (i32, i32) {
    let q_frac = (2.0 / 3.0 * x) / size;
    let r_frac = (-1.0 / 3.0 * x + SQRT3 / 3.0 * y) / size;
    axial_round(q_frac, r_frac)
}

/// Round fractional axial coordinates to the nearest hex.
pub fn axial_round(q_frac: f32, r_frac: f32) -> (i32, i32) {
    let s_frac = -q_frac - r_frac;
    let mut qi = q_frac.round() as i32;
    let mut ri = r_frac.round() as i32;
    let si = s_frac.round() as i32;

    let qd = (qi as f32 - q_frac).abs();
    let rd = (ri as f32 - r_frac).abs();
    let sd = (si as f32 - s_frac).abs();

    if qd > rd && qd > sd {
        qi = -ri - si;
    } else if rd > sd {
        ri = -qi - si;
    }
    (qi, ri)
}

/// Return the 6 neighbor coordinates.
pub fn hex_neighbors(q: i32, r: i32) -> [(i32, i32); 6] {
    let mut result = [(0i32, 0i32); 6];
    for (i, &(dq, dr)) in NEIGHBOR_OFFSETS.iter().enumerate() {
        result[i] = (q + dq, r + dr);
    }
    result
}

/// Hex distance between two axial coordinates.
pub fn hex_distance(q1: i32, r1: i32, q2: i32, r2: i32) -> i32 {
    let s1 = -q1 - r1;
    let s2 = -q2 - r2;
    ((q1 - q2).abs()).max((r1 - r2).abs()).max((s1 - s2).abs())
}

/// Yield all hex coordinates within a given radius.
pub fn hexes_in_radius(q: i32, r: i32, radius: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    for dq in -radius..=radius {
        let r_min = (-radius).max(-dq - radius);
        let r_max = radius.min(-dq + radius);
        for dr in r_min..=r_max {
            result.push((q + dq, r + dr));
        }
    }
    result
}

/// Return the pixel offset of a hex corner (0-5) from hex center.
/// Corner 0 is the rightmost point for flat-top hexes.
pub fn hex_corner_offset(corner: u32, size: f32) -> (f32, f32) {
    let angle_rad = std::f32::consts::PI / 3.0 * corner as f32;
    (size * angle_rad.cos(), size * angle_rad.sin())
}

/// Return the 6 corner pixel coordinates of a hex.
pub fn hex_corners(q: i32, r: i32, size: f32) -> [(f32, f32); 6] {
    let (cx, cy) = hex_to_pixel(q, r, size);
    let mut corners = [(0.0f32, 0.0f32); 6];
    for i in 0..6 {
        let (dx, dy) = hex_corner_offset(i as u32, size);
        corners[i] = (cx + dx, cy + dy);
    }
    corners
}

/// Hex width (flat-top).
pub fn hex_width(size: f32) -> f32 {
    size * 2.0
}

/// Hex height (flat-top).
pub fn hex_height(size: f32) -> f32 {
    size * SQRT3
}
