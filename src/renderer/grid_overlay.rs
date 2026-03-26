use wgpu::util::DeviceExt;

use crate::hex_math::hex_corners;
use crate::model::hex_grid::HexGrid;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 2],
}

impl LineVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct GridOverlay {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl GridOverlay {
    pub fn build(device: &wgpu::Device, grid: &HexGrid, hex_size: f32) -> Self {
        let cells: Vec<(i32, i32)> = grid.cells().collect();

        // Each hex has 6 edges; many are shared, but we emit all for simplicity.
        // 6 edges x 2 vertices = 12 vertices per hex
        let mut vertices = Vec::with_capacity(cells.len() * 12);

        for &(q, r) in &cells {
            let corners = hex_corners(q, r, hex_size);
            for i in 0..6 {
                let (x0, y0) = corners[i];
                let (x1, y1) = corners[(i + 1) % 6];
                vertices.push(LineVertex {
                    position: [x0, y0],
                });
                vertices.push(LineVertex {
                    position: [x1, y1],
                });
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid_line_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertex_buffer,
            num_vertices: vertices.len() as u32,
        }
    }
}
