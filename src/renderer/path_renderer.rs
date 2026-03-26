use wgpu::util::DeviceExt;

use crate::model::path_feature::PathFeature;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PathVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl PathVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PathVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct PathMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl PathMesh {
    pub fn build(device: &wgpu::Device, paths: &[PathFeature]) -> Self {
        let mut vertices = Vec::new();

        for path in paths {
            let points = &path.control_points;
            if points.len() < 2 {
                continue;
            }

            let color = path.feature_type.color();
            let half_width = path.width * 0.5;

            // Build triangle strip vertices for thick line
            let mut strip = Vec::with_capacity(points.len() * 2);

            for i in 0..points.len() {
                // Compute tangent direction
                let (dx, dy) = if i == 0 {
                    (points[1][0] - points[0][0], points[1][1] - points[0][1])
                } else if i == points.len() - 1 {
                    let last = points.len() - 1;
                    (
                        points[last][0] - points[last - 1][0],
                        points[last][1] - points[last - 1][1],
                    )
                } else {
                    (
                        points[i + 1][0] - points[i - 1][0],
                        points[i + 1][1] - points[i - 1][1],
                    )
                };

                let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                // Perpendicular normal (rotate tangent 90 degrees)
                let nx = -dy / len * half_width;
                let ny = dx / len * half_width;

                let (px, py) = (points[i][0], points[i][1]);
                strip.push(PathVertex {
                    position: [px + nx, py + ny],
                    color,
                });
                strip.push(PathVertex {
                    position: [px - nx, py - ny],
                    color,
                });
            }

            // Convert triangle strip to triangle list for consistent topology
            if strip.len() >= 4 {
                for i in 0..(strip.len() - 2) {
                    if i % 2 == 0 {
                        vertices.push(strip[i]);
                        vertices.push(strip[i + 1]);
                        vertices.push(strip[i + 2]);
                    } else {
                        vertices.push(strip[i + 1]);
                        vertices.push(strip[i]);
                        vertices.push(strip[i + 2]);
                    }
                }
            }
        }

        // Ensure we have at least a degenerate buffer so wgpu doesn't complain
        if vertices.is_empty() {
            vertices.push(PathVertex {
                position: [0.0; 2],
                color: [0.0; 4],
            });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("path_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let num_vertices = if paths.is_empty() {
            0
        } else {
            vertices.len() as u32
        };

        Self {
            vertex_buffer,
            num_vertices,
        }
    }
}
