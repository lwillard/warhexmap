use noise::{NoiseFn, Perlin};

/// Procedurally generated tileable terrain textures stored in a 2D texture array.
pub struct TerrainTextures {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

const TEX_SIZE: u32 = 256;

// Water base colors (layers 0-3)
const WATER_PALETTE: [(u8, u8, u8); 4] = [
    (140, 165, 185),
    (145, 170, 190),
    (155, 178, 198),
    (168, 190, 205),
];

// Land colors per climate, each with [plains, hills, mountain] variants.
// 8 climates x 3 elevations = 24 layers (indices 4..27)
const CLIMATE_PALETTE: [[(u8, u8, u8); 3]; 8] = [
    // BW
    [(215, 200, 165), (200, 180, 145), (180, 155, 120)],
    // BS
    [(205, 195, 145), (190, 175, 130), (170, 150, 115)],
    // Cs
    [(195, 190, 135), (180, 170, 120), (160, 145, 105)],
    // Cw
    [(160, 180, 110), (145, 160, 100), (130, 140, 90)],
    // Cf
    [(140, 175, 100), (125, 155, 90), (110, 135, 80)],
    // Df
    [(155, 170, 115), (140, 155, 100), (125, 135, 90)],
    // Am
    [(100, 165, 80), (85, 145, 70), (70, 125, 60)],
    // Af
    [(75, 150, 65), (60, 130, 55), (50, 110, 50)],
];

impl TerrainTextures {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let layer_count = 28u32;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain_texture_array"),
            size: wgpu::Extent3d {
                width: TEX_SIZE,
                height: TEX_SIZE,
                depth_or_array_layers: layer_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let perlin = Perlin::new(42);

        for layer in 0..layer_count {
            let (base_r, base_g, base_b, amplitude) = if layer < 4 {
                // Water layers
                let (r, g, b) = WATER_PALETTE[layer as usize];
                (r, g, b, 10.0_f64)
            } else {
                // Land layers: (layer - 4) / 3 = climate index, (layer - 4) % 3 = elevation
                let idx = (layer - 4) as usize;
                let climate = idx / 3;
                let elevation = idx % 3;
                let (r, g, b) = CLIMATE_PALETTE[climate][elevation];
                // Higher amplitude for rougher terrain
                let amp = match elevation {
                    0 => 15.0,
                    1 => 20.0,
                    _ => 25.0,
                };
                (r, g, b, amp)
            };

            let mut pixels = vec![0u8; (TEX_SIZE * TEX_SIZE * 4) as usize];
            for y in 0..TEX_SIZE {
                for x in 0..TEX_SIZE {
                    let nx = x as f64 / TEX_SIZE as f64 * 4.0;
                    let ny = y as f64 / TEX_SIZE as f64 * 4.0;

                    // Multi-octave noise for texture variation
                    let n1 = perlin.get([nx, ny, layer as f64 * 7.3]);
                    let n2 = perlin.get([nx * 2.0, ny * 2.0, layer as f64 * 7.3 + 100.0]) * 0.5;
                    let n3 =
                        perlin.get([nx * 4.0, ny * 4.0, layer as f64 * 7.3 + 200.0]) * 0.25;
                    let noise = (n1 + n2 + n3) * amplitude;

                    let r = (base_r as f64 + noise).clamp(0.0, 255.0) as u8;
                    let g = (base_g as f64 + noise * 0.9).clamp(0.0, 255.0) as u8;
                    let b = (base_b as f64 + noise * 0.8).clamp(0.0, 255.0) as u8;

                    let offset = ((y * TEX_SIZE + x) * 4) as usize;
                    pixels[offset] = r;
                    pixels[offset + 1] = g;
                    pixels[offset + 2] = b;
                    pixels[offset + 3] = 255;
                }
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(TEX_SIZE * 4),
                    rows_per_image: Some(TEX_SIZE),
                },
                wgpu::Extent3d {
                    width: TEX_SIZE,
                    height: TEX_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
