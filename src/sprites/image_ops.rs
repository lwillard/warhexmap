use std::f32::consts::PI;

use crate::types::SpriteImage;

/// Alpha-composite `src` onto `dest` at position (dx, dy).
/// `dest` is a flat RGBA buffer with row width `dest_w` pixels.
pub fn blit(dest: &mut [u8], dest_w: u32, src: &SpriteImage, dx: u32, dy: u32) {
    let dest_h = dest.len() as u32 / (dest_w * 4);
    for sy in 0..src.height {
        let out_y = dy + sy;
        if out_y >= dest_h {
            break;
        }
        for sx in 0..src.width {
            let out_x = dx + sx;
            if out_x >= dest_w {
                break;
            }
            let si = ((sy * src.width + sx) * 4) as usize;
            let di = ((out_y * dest_w + out_x) * 4) as usize;

            let sa = src.pixels[si + 3] as u32;
            if sa == 0 {
                continue;
            }
            if sa == 255 {
                dest[di] = src.pixels[si];
                dest[di + 1] = src.pixels[si + 1];
                dest[di + 2] = src.pixels[si + 2];
                dest[di + 3] = 255;
                continue;
            }

            let da = dest[di + 3] as u32;
            let inv_sa = 255 - sa;

            // Standard "over" alpha composite
            let out_a = sa + da * inv_sa / 255;
            if out_a == 0 {
                continue;
            }
            for c in 0..3 {
                let sc = src.pixels[si + c] as u32;
                let dc = dest[di + c] as u32;
                dest[di + c] = ((sc * sa + dc * da * inv_sa / 255) / out_a) as u8;
            }
            dest[di + 3] = out_a as u8;
        }
    }
}

/// Bilinear resize of a SpriteImage.
pub fn resize_sprite(src: &SpriteImage, new_w: u32, new_h: u32) -> SpriteImage {
    let mut pixels = vec![0u8; (new_w * new_h * 4) as usize];

    let x_ratio = if new_w > 1 {
        (src.width as f32 - 1.0) / (new_w as f32 - 1.0)
    } else {
        0.0
    };
    let y_ratio = if new_h > 1 {
        (src.height as f32 - 1.0) / (new_h as f32 - 1.0)
    } else {
        0.0
    };

    for dy in 0..new_h {
        for dx in 0..new_w {
            let src_x = dx as f32 * x_ratio;
            let src_y = dy as f32 * y_ratio;

            let x0 = src_x.floor() as u32;
            let y0 = src_y.floor() as u32;
            let x1 = (x0 + 1).min(src.width - 1);
            let y1 = (y0 + 1).min(src.height - 1);

            let fx = src_x - x0 as f32;
            let fy = src_y - y0 as f32;

            let idx = |x: u32, y: u32| (y * src.width + x) as usize * 4;
            let di = (dy * new_w + dx) as usize * 4;

            for c in 0..4 {
                let c00 = src.pixels[idx(x0, y0) + c] as f32;
                let c10 = src.pixels[idx(x1, y0) + c] as f32;
                let c01 = src.pixels[idx(x0, y1) + c] as f32;
                let c11 = src.pixels[idx(x1, y1) + c] as f32;

                let top = c00 + (c10 - c00) * fx;
                let bot = c01 + (c11 - c01) * fx;
                let val = top + (bot - top) * fy;
                pixels[di + c] = val.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    SpriteImage {
        width: new_w,
        height: new_h,
        pixels,
    }
}

/// Generate an alpha mask for a flat-top hex polygon, centered in a `size x size` image.
/// Returns a single-channel (alpha) buffer of length `size * size`.
pub fn generate_hex_mask(size: u32, radius: f32) -> Vec<u8> {
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;

    // Flat-top hex vertices
    let vertices: Vec<(f32, f32)> = (0..6)
        .map(|i| {
            let angle = PI / 180.0 * (60.0 * i as f32);
            (cx + radius * angle.cos(), cy + radius * angle.sin())
        })
        .collect();

    let mut mask = vec![0u8; (size * size) as usize];
    for y in 0..size {
        for x in 0..size {
            if point_in_polygon(x as f32 + 0.5, y as f32 + 0.5, &vertices) {
                mask[(y * size + x) as usize] = 255;
            }
        }
    }
    mask
}

/// Point-in-polygon test using ray casting.
pub fn point_in_polygon(px: f32, py: f32, vertices: &[(f32, f32)]) -> bool {
    let n = vertices.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = vertices[i];
        let (xj, yj) = vertices[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Generate a flat-color hex sprite for testing.
/// The hex shape is filled with `color`; pixels outside the hex are transparent.
pub fn generate_test_base_sprite(size: u32, color: [u8; 4]) -> SpriteImage {
    let radius = size as f32 / 2.0;
    let mask = generate_hex_mask(size, radius);

    let mut pixels = vec![0u8; (size * size * 4) as usize];
    for i in 0..(size * size) as usize {
        if mask[i] > 0 {
            let pi = i * 4;
            pixels[pi] = color[0];
            pixels[pi + 1] = color[1];
            pixels[pi + 2] = color[2];
            pixels[pi + 3] = color[3];
        }
    }

    SpriteImage {
        width: size,
        height: size,
        pixels,
    }
}

/// Generate a test edge overlay with an alpha gradient along a specific edge.
/// `edge` is 0..5 for the six edges of a flat-top hex.
pub fn generate_test_edge_overlay(size: u32, color: [u8; 4], edge: usize) -> SpriteImage {
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let radius = size as f32 / 2.0;

    // Flat-top hex vertices
    let vertices: Vec<(f32, f32)> = (0..6)
        .map(|i| {
            let angle = PI / 180.0 * (60.0 * i as f32);
            (cx + radius * angle.cos(), cy + radius * angle.sin())
        })
        .collect();

    // Edge midpoint: between vertex[edge] and vertex[(edge+1)%6]
    let (x0, y0) = vertices[edge];
    let (x1, y1) = vertices[(edge + 1) % 6];
    let mid_x = (x0 + x1) / 2.0;
    let mid_y = (y0 + y1) / 2.0;

    // Direction from center to edge midpoint
    let dx = mid_x - cx;
    let dy = mid_y - cy;
    let dist_to_edge = (dx * dx + dy * dy).sqrt();
    let nx = dx / dist_to_edge;
    let ny = dy / dist_to_edge;

    // Gradient: full alpha at the edge, fading to 0 toward center
    let gradient_depth = radius * 0.35; // how far inward the gradient extends

    let mask = generate_hex_mask(size, radius);
    let mut pixels = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) as usize;
            if mask[idx] == 0 {
                continue;
            }

            // Project pixel onto the edge normal direction
            let px = x as f32 + 0.5 - cx;
            let py = y as f32 + 0.5 - cy;
            let proj = px * nx + py * ny;

            // Alpha falls off from the edge inward
            let edge_dist = dist_to_edge - proj;
            if edge_dist < gradient_depth {
                let t = 1.0 - (edge_dist / gradient_depth).clamp(0.0, 1.0);
                let alpha = (t * color[3] as f32).round().clamp(0.0, 255.0) as u8;
                if alpha > 0 {
                    let pi = idx * 4;
                    pixels[pi] = color[0];
                    pixels[pi + 1] = color[1];
                    pixels[pi + 2] = color[2];
                    pixels[pi + 3] = alpha;
                }
            }
        }
    }

    SpriteImage {
        width: size,
        height: size,
        pixels,
    }
}

/// Apply the hex mask to a sprite, setting alpha to 0 for pixels outside the hex polygon.
/// The sprite must be `size x size`. The hex is centered with the given radius.
pub fn clip_to_hex(sprite: &mut SpriteImage, radius: f32) {
    let mask = generate_hex_mask(sprite.width, radius);
    for i in 0..(sprite.width * sprite.height) as usize {
        if mask[i] == 0 {
            sprite.pixels[i * 4 + 3] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_polygon_triangle() {
        let tri = [(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)];
        assert!(point_in_polygon(5.0, 5.0, &tri));
        assert!(!point_in_polygon(20.0, 20.0, &tri));
    }

    #[test]
    fn test_generate_hex_mask_has_filled_center() {
        let mask = generate_hex_mask(170, 85.0);
        // Center pixel should be inside
        assert_eq!(mask[85 * 170 + 85], 255);
    }

    #[test]
    fn test_blit_opaque() {
        let src = SpriteImage {
            width: 2,
            height: 2,
            pixels: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
        };
        let mut dest = vec![0u8; 4 * 4 * 4]; // 4x4
        blit(&mut dest, 4, &src, 1, 1);
        // Check pixel at (1,1) in dest
        let idx = (1 * 4 + 1) * 4;
        assert_eq!(dest[idx], 255);
        assert_eq!(dest[idx + 1], 0);
        assert_eq!(dest[idx + 2], 0);
        assert_eq!(dest[idx + 3], 255);
    }

    #[test]
    fn test_resize_identity() {
        let src = SpriteImage {
            width: 2,
            height: 2,
            pixels: vec![100, 150, 200, 255, 100, 150, 200, 255, 100, 150, 200, 255, 100, 150, 200, 255],
        };
        let resized = resize_sprite(&src, 2, 2);
        assert_eq!(resized.pixels, src.pixels);
    }
}
