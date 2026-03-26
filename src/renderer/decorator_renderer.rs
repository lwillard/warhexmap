use wgpu::util::DeviceExt;

use crate::hex_math::hex_to_pixel;
use crate::model::hex_grid::HexGrid;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DecoratorInstance {
    pub world_offset: [f32; 2],
    pub size: f32,
    pub color: [f32; 4],
    pub _pad: f32,
}

impl DecoratorInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DecoratorInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

/// Unit quad vertices for instanced rendering.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
}

pub struct DecoratorMesh {
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub num_instances: u32,
}

/// Deterministic hash for reproducible decorator placement.
fn det_hash(q: i32, r: i32, idx: u32) -> u32 {
    let a = (q as u32).wrapping_mul(73856093);
    let b = (r as u32).wrapping_mul(19349663);
    let c = idx.wrapping_mul(83492791);
    a ^ b ^ c
}

/// Extract a pseudo-random f32 in [0, 1) from a hash value at a given bit offset.
fn hash_f32(h: u32, shift: u32) -> f32 {
    let bits = (h.wrapping_shr(shift)) & 0xFFFF;
    bits as f32 / 65536.0
}

impl DecoratorMesh {
    pub fn build(device: &wgpu::Device, grid: &HexGrid, hex_size: f32) -> Self {
        // Unit quad: centered at origin, size 1x1
        let quad_verts = [
            QuadVertex {
                position: [-0.5, -0.5],
            },
            QuadVertex {
                position: [0.5, -0.5],
            },
            QuadVertex {
                position: [0.5, 0.5],
            },
            QuadVertex {
                position: [-0.5, 0.5],
            },
        ];
        let quad_indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("decorator_quad_vb"),
            contents: bytemuck::cast_slice(&quad_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("decorator_quad_ib"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Gather instances from all hexes with decorators
        let mut instances = Vec::new();
        let radius = hex_size * 0.8; // scatter within 80% of hex radius

        for (q, r) in grid.cells() {
            let cell = match grid.get(q, r) {
                Some(c) => c,
                None => continue,
            };

            for (dec_idx, decorator) in cell.decorators.iter().enumerate() {
                let (density, size, color) = decorator_params(decorator, hex_size);
                let count = (density * 12.0) as u32; // up to ~12 items at density 1.0

                let (cx, cy) = hex_to_pixel(q, r, hex_size);

                for i in 0..count {
                    let h = det_hash(q, r, dec_idx as u32 * 100 + i);
                    // Random offset within hex
                    let fx = (hash_f32(h, 0) - 0.5) * 2.0 * radius;
                    let fy = (hash_f32(h, 8) - 0.5) * 2.0 * radius;

                    // Only place inside hex approximate circle
                    if fx * fx + fy * fy > radius * radius {
                        continue;
                    }

                    instances.push(DecoratorInstance {
                        world_offset: [cx + fx, cy + fy],
                        size,
                        color,
                        _pad: 0.0,
                    });
                }
            }
        }

        // Ensure at least one element so buffer creation doesn't fail on zero size
        if instances.is_empty() {
            instances.push(DecoratorInstance {
                world_offset: [0.0; 2],
                size: 0.0,
                color: [0.0; 4],
                _pad: 0.0,
            });
        }

        let num_instances = if grid.cells().next().is_some() {
            instances.len() as u32
        } else {
            0
        };

        // Recount properly: if we only have the dummy, treat as 0
        let num_instances = if instances.len() == 1 && instances[0].size == 0.0 {
            0
        } else {
            num_instances
        };

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("decorator_instance_buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            quad_vertex_buffer,
            quad_index_buffer,
            instance_buffer,
            num_instances,
        }
    }
}

/// Returns (density, size, rgba_color) for a decorator.
fn decorator_params(decorator: &crate::model::terrain_types::Decorator, hex_size: f32) -> (f32, f32, [f32; 4]) {
    use crate::model::terrain_types::Decorator;
    let s = hex_size * 0.12;
    match decorator {
        Decorator::Woods => (0.3, s, [0.2, 0.55, 0.15, 1.0]),
        Decorator::DenseForest => (0.8, s * 1.1, [0.1, 0.40, 0.08, 1.0]),
        Decorator::Farms => (0.25, s * 0.9, [0.6, 0.7, 0.2, 1.0]),
        Decorator::Buildings => (0.15, s * 0.7, [0.5, 0.42, 0.35, 1.0]),
        Decorator::Grassland => (0.35, s * 0.6, [0.3, 0.5, 0.35, 1.0]),
        Decorator::DenseBuildings => (0.1, s * 0.8, [0.55, 0.5, 0.45, 1.0]),
    }
}
