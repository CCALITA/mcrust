//! AO-enhanced terrain vertex for ambient occlusion rendering.

use bytemuck::{Pod, Zeroable};

/// Terrain vertex with per-vertex ambient occlusion factor.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct AoVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub ao: f32,
}

/// WGSL struct input for the AO vertex layout.
pub const AO_VERTEX_WGSL_INPUT: &str = "\
struct AoVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) ao: f32,
};";

/// Returns the stride in bytes for `AoVertex` (36 bytes).
pub fn ao_vertex_stride() -> u64 {
    36
}

/// Returns the default ambient occlusion value (fully lit).
pub fn default_ao() -> f32 {
    1.0
}

/// Returns the number of vertex attributes in `AoVertex`.
pub fn ao_attribute_count() -> u32 {
    4
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn ao_vertex_size_is_36_bytes() {
        assert_eq!(mem::size_of::<AoVertex>(), 36);
    }

    #[test]
    fn ao_vertex_stride_is_36() {
        assert_eq!(ao_vertex_stride(), 36);
    }

    #[test]
    fn default_ao_is_one() {
        assert_eq!(default_ao(), 1.0);
    }
}
