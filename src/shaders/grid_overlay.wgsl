// Grid overlay shader for hex map.
// Renders hex grid lines as semi-transparent gray.

struct Camera {
    view_proj: mat4x4<f32>,
    hex_size: f32,
    grid_offset_q: i32,
    grid_offset_r: i32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.314, 0.314, 0.314, 0.314);
}
