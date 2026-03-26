// Decorator shader for instanced rendering of terrain decorations (trees, etc.).
// Each instance is a small quad placed in world space with a color and size.
// Circular decorators discard fragments outside the unit circle.

struct Camera {
    view_proj: mat4x4<f32>,
    hex_size: f32,
    grid_offset_q: i32,
    grid_offset_r: i32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
    // Per-vertex
    @location(0) local_pos: vec2<f32>,
    // Per-instance
    @location(1) world_offset: vec2<f32>,
    @location(2) size: f32,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = in.local_pos * in.size + in.world_offset;
    out.clip_pos = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.local_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Discard fragments outside unit circle for circular decorators
    if length(in.uv) > 1.0 {
        discard;
    }
    return in.color;
}
