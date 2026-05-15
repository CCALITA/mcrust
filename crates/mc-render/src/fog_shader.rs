//! Fog WGSL shader snippet and GPU uniform for linear distance fog.

use bytemuck::{Pod, Zeroable};

/// WGSL function that applies linear fog to a fragment color.
pub const FOG_WGSL_SNIPPET: &str = r#"
fn apply_fog(
    frag_color: vec4<f32>,
    world_pos: vec3<f32>,
    camera_pos: vec3<f32>,
    fog_start: f32,
    fog_end: f32,
    fog_color: vec3<f32>,
) -> vec4<f32> {
    let dist = distance(world_pos, camera_pos);
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    let blended = mix(frag_color.rgb, fog_color, fog_factor);
    return vec4<f32>(blended, frag_color.a);
}
"#;

/// GPU-aligned uniform for fog parameters (32 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct FogUniform {
    pub camera_pos: [f32; 3],
    pub fog_start: f32,
    pub fog_color: [f32; 3],
    pub fog_end: f32,
}

/// Returns the size of [`FogUniform`] in bytes (32).
pub fn fog_uniform_size() -> usize {
    std::mem::size_of::<FogUniform>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_size_is_32_bytes() {
        assert_eq!(fog_uniform_size(), 32);
    }

    #[test]
    fn snippet_contains_apply_fog() {
        assert!(FOG_WGSL_SNIPPET.contains("apply_fog"));
    }

    #[test]
    fn snippet_contains_mix() {
        assert!(FOG_WGSL_SNIPPET.contains("mix"));
    }
}
