// Terrain splatting shader for hex map rendering.
// Uses IDW blending of terrain textures across hex boundaries
// with hillshade lighting and coastal blending.

const SQRT3: f32 = 1.7320508;
const NEIGHBOR_OFFSETS: array<vec2<i32>, 6> = array(
    vec2(1, 0), vec2(-1, 0), vec2(1, -1), vec2(0, -1), vec2(0, 1), vec2(-1, 1)
);

struct Camera {
    view_proj: mat4x4<f32>,
    hex_size: f32,
    grid_offset_q: i32,
    grid_offset_r: i32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var hex_data_texture: texture_2d<u32>;
@group(1) @binding(1) var terrain_textures: texture_2d_array<f32>;
@group(1) @binding(2) var terrain_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) hex_center_pos: vec2<f32>,
    @location(2) hex_coord: vec2<i32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) @interpolate(flat) hex_coord: vec2<i32>,
    @location(2) @interpolate(flat) hex_center_pos: vec2<f32>,
};

fn hex_center(q: i32, r: i32, size: f32) -> vec2<f32> {
    return vec2(size * 1.5 * f32(q), size * (SQRT3 * 0.5 * f32(q) + SQRT3 * f32(r)));
}

fn lookup_hex_data(q: i32, r: i32) -> vec2<u32> {
    let tx = q - camera.grid_offset_q;
    let ty = r - camera.grid_offset_r;
    let dims = textureDimensions(hex_data_texture);
    if tx < 0 || ty < 0 || u32(tx) >= dims.x || u32(ty) >= dims.y {
        return vec2<u32>(0u, 0u);
    }
    let data = textureLoad(hex_data_texture, vec2<i32>(tx, ty), 0);
    return vec2<u32>(data.r, data.g);
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.world_pos = in.position;
    out.hex_coord = in.hex_coord;
    out.hex_center_pos = in.hex_center_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_pos;
    let cq = in.hex_coord.x;
    let cr = in.hex_coord.y;
    let center = in.hex_center_pos;

    // Look up center hex data
    let center_data = lookup_hex_data(cq, cr);
    let center_tex_idx = center_data.x;
    let center_elev = f32(center_data.y) / 36.0;

    // Compute distance from fragment to center hex
    let d_center = distance(world_pos, center);
    var w_center = 1.0 / (d_center * d_center + 0.01);
    w_center = w_center * w_center; // square for sharpness

    var total_weight = w_center;

    // Terrain tiling UV
    let uv = world_pos * 0.01;

    // Sample center hex terrain
    var blended_color = textureSample(terrain_textures, terrain_sampler, uv, center_tex_idx) * w_center;
    var blended_elev = center_elev * w_center;

    // Process 6 neighbors
    for (var i = 0u; i < 6u; i = i + 1u) {
        let offset = NEIGHBOR_OFFSETS[i];
        let nq = cq + offset.x;
        let nr = cr + offset.y;

        let neighbor_center = hex_center(nq, nr, camera.hex_size);
        let d_neighbor = distance(world_pos, neighbor_center);
        var w = 1.0 / (d_neighbor * d_neighbor + 0.01);
        w = w * w;

        let neighbor_data = lookup_hex_data(nq, nr);
        let n_tex_idx = neighbor_data.x;
        let n_elev = f32(neighbor_data.y) / 36.0;

        let n_color = textureSample(terrain_textures, terrain_sampler, uv, n_tex_idx) * w;
        blended_color = blended_color + n_color;
        blended_elev = blended_elev + n_elev * w;
        total_weight = total_weight + w;
    }

    // Normalize
    var color = blended_color / total_weight;
    let elevation = blended_elev / total_weight;

    // Coastal blending (elevation 3.0 to 4.5)
    let shore_color = vec4<f32>(0.784, 0.745, 0.627, 1.0);
    if elevation >= 3.0 && elevation <= 4.5 {
        let t = 1.0 - (elevation - 3.0) / 1.5;
        color = mix(color, shore_color, t);
    }

    // Hillshade using screen-space derivatives of blended elevation
    let dzdx = dpdx(elevation);
    let dzdy = dpdy(elevation);

    // Azimuth 315 degrees, altitude 45 degrees
    let azimuth_rad = radians(315.0);
    let altitude_rad = radians(45.0);

    let slope = atan(sqrt(dzdx * dzdx + dzdy * dzdy));
    let aspect = atan2(-dzdy, -dzdx);

    let shade = cos(altitude_rad) * cos(slope) +
                sin(altitude_rad) * sin(slope) * cos(azimuth_rad - aspect);
    let shade_clamped = clamp(shade, 0.0, 1.0);

    // Apply hillshade only to land pixels (elevation > 3.0 means above water)
    if elevation > 3.0 {
        color = vec4<f32>(color.rgb * (0.4 + 0.6 * shade_clamped), 1.0);
    } else {
        color = vec4<f32>(color.rgb, 1.0);
    }

    return color;
}
