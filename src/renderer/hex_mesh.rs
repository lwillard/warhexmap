use wgpu::util::DeviceExt;

use crate::hex_math::{hex_corners, hex_to_pixel};
use crate::model::hex_grid::HexGrid;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HexVertex {
    pub position: [f32; 2],
    pub hex_center: [f32; 2],
    pub hex_coord: [i32; 2],
}

impl HexVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<HexVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Sint32x2,
                },
            ],
        }
    }
}

pub struct HexMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl HexMesh {
    pub fn build(device: &wgpu::Device, grid: &HexGrid, hex_size: f32) -> Self {
        let cells: Vec<(i32, i32)> = grid.cells().collect();
        let hex_count = cells.len();

        // Each hex: 7 vertices (center + 6 corners), 18 indices (6 triangles x 3)
        let mut vertices = Vec::with_capacity(hex_count * 7);
        let mut indices = Vec::with_capacity(hex_count * 18);

        for &(q, r) in &cells {
            let (cx, cy) = hex_to_pixel(q, r, hex_size);
            let corners = hex_corners(q, r, hex_size);
            let base = vertices.len() as u32;

            // Center vertex
            vertices.push(HexVertex {
                position: [cx, cy],
                hex_center: [cx, cy],
                hex_coord: [q, r],
            });

            // 6 corner vertices
            for &(px, py) in &corners {
                vertices.push(HexVertex {
                    position: [px, py],
                    hex_center: [cx, cy],
                    hex_coord: [q, r],
                });
            }

            // 6 triangles (fan from center)
            for i in 0..6u32 {
                indices.push(base); // center
                indices.push(base + 1 + i); // corner i
                indices.push(base + 1 + (i + 1) % 6); // corner (i+1) % 6
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("hex_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("hex_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}
