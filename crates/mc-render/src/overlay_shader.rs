//! 2D overlay shader for rendering flat UI elements (crosshairs, health bars, etc.).

use bytemuck::{Pod, Zeroable};

/// WGSL vertex shader for 2D overlays.
pub const OVERLAY_VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    return VertexOutput(vec4(in.position, 0.0, 1.0), in.color);
}
"#;

/// WGSL fragment shader for 2D overlays.
pub const OVERLAY_FRAGMENT_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

/// GPU vertex for 2D overlay rendering (position + color).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct OverlayVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn overlay_vertex_size_is_24_bytes() {
        assert_eq!(mem::size_of::<OverlayVertex>(), 24);
    }

    #[test]
    fn vertex_shader_contains_vs_main() {
        assert!(OVERLAY_VERTEX_SHADER.contains("vs_main"));
    }

    #[test]
    fn fragment_shader_contains_fs_main() {
        assert!(OVERLAY_FRAGMENT_SHADER.contains("fs_main"));
    }
}
