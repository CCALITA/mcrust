//! Water transparency shader with wave animation and blue-tinted output.

use bytemuck::{Pod, Zeroable};

/// WGSL vertex shader for water surfaces with wave displacement.
pub const WATER_VERTEX_WGSL: &str = r#"
struct WaterUniform {
    time: f32,
    water_color: vec3<f32>,
};

@group(1) @binding(0) var<uniform> water: WaterUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = in.position;
    out.world_pos.y += sin(out.world_pos.x * 2.0 + water.time) * 0.05;
    out.clip_position = view_proj * vec4<f32>(out.world_pos, 1.0);
    out.tex_coords = in.tex_coords;
    out.normal = in.normal;
    return out;
}
"#;

/// WGSL fragment shader for water surfaces with blue tint and transparency.
pub const WATER_FRAGMENT_WGSL: &str = r#"
struct WaterUniform {
    time: f32,
    water_color: vec3<f32>,
};

@group(1) @binding(0) var<uniform> water: WaterUniform;

@group(2) @binding(0) var t_diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

struct FragmentInput {
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let brightness = max(dot(normalize(in.normal), normalize(vec3<f32>(0.3, 1.0, 0.2))), 0.2);
    return vec4<f32>(tex_color.rgb * water.water_color * brightness, 0.7);
}
"#;

/// GPU-compatible uniform for water shader parameters.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct WaterUniform {
    pub time: f32,
    pub water_color: [f32; 3],
}

/// Returns the byte size of [`WaterUniform`] (16 bytes, aligned for GPU).
pub fn water_uniform_size() -> usize {
    std::mem::size_of::<WaterUniform>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_uniform_is_16_bytes() {
        assert_eq!(water_uniform_size(), 16);
        assert_eq!(std::mem::size_of::<WaterUniform>(), 16);
    }

    #[test]
    fn vertex_shader_contains_water() {
        assert!(WATER_VERTEX_WGSL.contains("water"));
    }

    #[test]
    fn fragment_shader_contains_water() {
        assert!(WATER_FRAGMENT_WGSL.contains("water"));
    }
}
