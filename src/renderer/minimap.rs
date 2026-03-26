use crate::hex_math::hex_to_pixel;
use crate::model::hex_grid::HexGrid;

const MINIMAP_W: u32 = 180;
const MINIMAP_H: u32 = 120;

pub struct MinimapRenderer {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub egui_texture_id: Option<egui::TextureId>,
}

impl MinimapRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("minimap_texture"),
            size: wgpu::Extent3d {
                width: MINIMAP_W,
                height: MINIMAP_H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            egui_texture_id: None,
        }
    }

    /// Renders a small overview image of the entire map on the CPU and uploads it.
    pub fn render_to_texture(&self, queue: &wgpu::Queue, grid: &HexGrid, hex_size: f32) {
        let bounds = grid.world_bounds(hex_size);
        let (min_x, min_y, max_x, max_y) = bounds;
        let world_w = (max_x - min_x).max(1.0);
        let world_h = (max_y - min_y).max(1.0);

        let mut pixels = vec![30u8; (MINIMAP_W * MINIMAP_H * 4) as usize];
        // Fill with dark background
        for i in 0..(MINIMAP_W * MINIMAP_H) as usize {
            pixels[i * 4] = 30;
            pixels[i * 4 + 1] = 30;
            pixels[i * 4 + 2] = 40;
            pixels[i * 4 + 3] = 255;
        }

        for (q, r) in grid.cells() {
            let cell = match grid.get(q, r) {
                Some(c) => c,
                None => continue,
            };

            let (wx, wy) = hex_to_pixel(q, r, hex_size);
            let px = ((wx - min_x) / world_w * (MINIMAP_W as f32 - 2.0) + 1.0) as i32;
            let py = ((wy - min_y) / world_h * (MINIMAP_H as f32 - 2.0) + 1.0) as i32;

            if px < 0 || px >= MINIMAP_W as i32 || py < 0 || py >= MINIMAP_H as i32 {
                continue;
            }

            let (cr, cg, cb) = minimap_color_for_cell(cell);

            // Draw a small dot (single pixel is fine at this scale)
            let offset = ((py as u32 * MINIMAP_W + px as u32) * 4) as usize;
            if offset + 3 < pixels.len() {
                pixels[offset] = cr;
                pixels[offset + 1] = cg;
                pixels[offset + 2] = cb;
                pixels[offset + 3] = 255;
            }
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(MINIMAP_W * 4),
                rows_per_image: Some(MINIMAP_H),
            },
            wgpu::Extent3d {
                width: MINIMAP_W,
                height: MINIMAP_H,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Registers the minimap texture with egui for display in a UI panel.
    pub fn register_with_egui(
        &mut self,
        renderer: &mut egui_wgpu::Renderer,
        device: &wgpu::Device,
    ) {
        let id = renderer.register_native_texture(
            device,
            &self.view,
            wgpu::FilterMode::Linear,
        );
        self.egui_texture_id = Some(id);
    }
}

/// Map a hex cell to an approximate minimap color based on elevation and climate.
fn minimap_color_for_cell(cell: &crate::model::hex_cell::HexCell) -> (u8, u8, u8) {
    use crate::model::terrain_types::{Climate, Elevation};

    match cell.elevation {
        Elevation::ShallowWater => (140, 165, 185),
        Elevation::DeepWater => (100, 130, 165),
        _ => {
            // Use climate to pick land color
            match cell.climate {
                Climate::BW => (215, 200, 165),
                Climate::BS => (205, 195, 145),
                Climate::Cs => (195, 190, 135),
                Climate::Cw => (160, 180, 110),
                Climate::Cf => (140, 175, 100),
                Climate::Df => (155, 170, 115),
                Climate::Am => (100, 165, 80),
                Climate::Af => (75, 150, 65),
            }
        }
    }
}
