/// Ramer-Douglas-Peucker line simplification algorithm.
///
/// Reduces the number of points in a polyline while preserving shape
/// within the given `epsilon` tolerance (perpendicular distance).
pub fn rdp_simplify(points: &[[f32; 2]], epsilon: f32) -> Vec<[f32; 2]> {
    if points.len() < 3 {
        return points.to_vec();
    }
    rdp_recursive(points, epsilon)
}

fn rdp_recursive(points: &[[f32; 2]], epsilon: f32) -> Vec<[f32; 2]> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }

    let first = points[0];
    let last = points[n - 1];

    let mut max_dist = 0.0f32;
    let mut max_idx = 0;

    for i in 1..n - 1 {
        let d = perpendicular_distance(points[i], first, last);
        if d > max_dist {
            max_dist = d;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        let mut left = rdp_recursive(&points[..=max_idx], epsilon);
        let right = rdp_recursive(&points[max_idx..], epsilon);
        // Remove duplicate pivot point
        left.pop();
        left.extend(right);
        left
    } else {
        vec![first, last]
    }
}

/// Perpendicular distance from point `p` to the line segment from `a` to `b`.
fn perpendicular_distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let len_sq = dx * dx + dy * dy;

    if len_sq < 1e-12 {
        // a and b are effectively the same point
        let ex = p[0] - a[0];
        let ey = p[1] - a[1];
        return (ex * ex + ey * ey).sqrt();
    }

    let area = ((b[0] - a[0]) * (a[1] - p[1]) - (a[0] - p[0]) * (b[1] - a[1])).abs();
    area / len_sq.sqrt()
}
