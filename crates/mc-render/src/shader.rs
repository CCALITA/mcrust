//! WGSL shaders for voxel rendering and sky dome.

/// Terrain shader with dynamic sky-based lighting.
///
/// Bind groups:
/// - group(0): CameraUniform (view_proj)
/// - group(1): texture atlas (t_diffuse, s_diffuse)
/// - group(2): SkyUniform (sky_color, sun_dir, ambient)
pub const SHADER_SOURCE: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct SkyUniform {
    sky_color: vec3<f32>,
    _pad0: f32,
    sun_dir: vec3<f32>,
    ambient: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> sky: SkyUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    out.normal = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Dynamic directional lighting from the day/night cycle
    let light_dir = normalize(sky.sun_dir);
    let diffuse = max(dot(in.normal, light_dir), 0.0) * 0.6;
    let brightness = sky.ambient + diffuse;

    return vec4<f32>(tex_color.rgb * brightness, tex_color.a);
}
"#;

/// Full-screen sky shader.
///
/// Renders a single full-screen triangle. The fragment shader produces a
/// vertical gradient from the horizon color toward the zenith color, both
/// derived from the `SkyUniform`.
///
/// Bind groups:
/// - group(0): SkyUniform
pub const SKY_SHADER_SOURCE: &str = r#"
struct SkyUniform {
    sky_color: vec3<f32>,
    _pad0: f32,
    sun_dir: vec3<f32>,
    ambient: f32,
};

@group(0) @binding(0)
var<uniform> sky: SkyUniform;

struct SkyVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Full-screen triangle: 3 vertices that cover the entire screen.
// Vertex IDs 0, 1, 2 map to a triangle that fills clip space.
@vertex
fn vs_sky(@builtin(vertex_index) vertex_index: u32) -> SkyVertexOutput {
    var out: SkyVertexOutput;

    // Generate clip-space positions for a full-screen triangle
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);

    out.clip_position = vec4<f32>(x, y, 0.9999, 1.0);
    // Map to [0,1] UV: u = (x+1)/2, v = (y+1)/2
    out.uv = vec2<f32>((x + 1.0) * 0.5, (y + 1.0) * 0.5);
    return out;
}

@fragment
fn fs_sky(in: SkyVertexOutput) -> @location(0) vec4<f32> {
    // v=0 is bottom of screen, v=1 is top.
    // Horizon color is slightly brighter/warmer, zenith is the base sky color.
    let horizon_factor = 1.0 - in.uv.y; // 1 at bottom, 0 at top
    let horizon_boost = vec3<f32>(0.15, 0.10, 0.05) * horizon_factor * horizon_factor;
    let color = sky.sky_color + horizon_boost;

    return vec4<f32>(color, 1.0);
}
"#;
