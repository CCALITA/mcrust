//! Directional shadow calculation utilities: orthographic projection,
//! PCF sampling offsets, slope-based bias, and a WGSL shader snippet.

use glam::{Mat4, Vec3};

/// WGSL function that computes a shadow factor via PCF sampling.
pub const SHADOW_WGSL_SNIPPET: &str = r#"
fn shadow_factor(
    shadow_pos: vec4<f32>,
    shadow_map: texture_depth_2d,
    shadow_sampler: sampler_comparison,
    bias: f32,
) -> f32 {
    let proj = shadow_pos.xyz / shadow_pos.w;
    let uv = proj.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);

    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || proj.z < 0.0 || proj.z > 1.0 {
        return 1.0;
    }

    let texel_size = 1.0 / 2048.0;
    let offsets = array<vec2<f32>, 9>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0,  0.0), vec2<f32>(0.0,  0.0), vec2<f32>(1.0,  0.0),
        vec2<f32>(-1.0,  1.0), vec2<f32>(0.0,  1.0), vec2<f32>(1.0,  1.0),
    );

    var total = 0.0;
    for (var i = 0u; i < 9u; i = i + 1u) {
        let sample_uv = uv + offsets[i] * texel_size;
        total += textureSampleCompare(
            shadow_map,
            shadow_sampler,
            sample_uv,
            proj.z - bias,
        );
    }
    return total / 9.0;
}
"#;

/// Builds an orthographic shadow view-projection matrix for a directional light.
///
/// The camera looks from `scene_center + sun_dir * scene_radius` toward `scene_center`,
/// with an ortho volume sized to enclose the scene sphere.
pub fn shadow_view_proj(
    sun_dir: [f32; 3],
    scene_center: [f32; 3],
    scene_radius: f32,
) -> [[f32; 4]; 4] {
    let sun = Vec3::from(sun_dir).normalize();
    let center = Vec3::from(scene_center);
    let eye = center + sun * scene_radius;

    let up = if sun.y.abs() > 0.99 {
        Vec3::Z
    } else {
        Vec3::Y
    };

    let view = Mat4::look_at_rh(eye, center, up);
    let proj = Mat4::orthographic_rh(
        -scene_radius,
        scene_radius,
        -scene_radius,
        scene_radius,
        0.0,
        scene_radius * 2.0,
    );

    (proj * view).to_cols_array_2d()
}

/// Returns 3x3 PCF sample offsets in texel units.
pub fn pcf_sample_offsets() -> [[f32; 2]; 9] {
    [
        [-1.0, -1.0],
        [0.0, -1.0],
        [1.0, -1.0],
        [-1.0, 0.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [-1.0, 1.0],
        [0.0, 1.0],
        [1.0, 1.0],
    ]
}

/// Computes a depth bias that scales with the slope between the surface normal
/// and the light direction, reducing shadow acne on steep surfaces.
///
/// `normal_dot_light` is the dot product of the surface normal and the light
/// direction (both normalised), clamped internally to `[0.001, 1.0]`.
pub fn shadow_bias_for_slope(normal_dot_light: f32) -> f32 {
    let clamped = normal_dot_light.clamp(0.001, 1.0);
    let base_bias = 0.0005;
    let slope_scale = 0.005;
    base_bias + slope_scale * (1.0 - clamped) / clamped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_wgsl_snippet_contains_function() {
        assert!(SHADOW_WGSL_SNIPPET.contains("fn shadow_factor("));
        assert!(SHADOW_WGSL_SNIPPET.contains("textureSampleCompare"));
    }

    #[test]
    fn shadow_view_proj_returns_valid_matrix() {
        let mat = shadow_view_proj([0.0, 1.0, 0.0], [0.0, 0.0, 0.0], 100.0);
        // Should be a 4x4 matrix with finite values
        for row in &mat {
            for val in row {
                assert!(val.is_finite(), "matrix contains non-finite value: {val}");
            }
        }
    }

    #[test]
    fn shadow_view_proj_diagonal_sun() {
        let mat = shadow_view_proj([1.0, 1.0, 0.0], [10.0, 20.0, 30.0], 50.0);
        for row in &mat {
            for val in row {
                assert!(val.is_finite());
            }
        }
    }

    #[test]
    fn pcf_offsets_has_nine_samples() {
        let offsets = pcf_sample_offsets();
        assert_eq!(offsets.len(), 9);
    }

    #[test]
    fn pcf_offsets_center_is_zero() {
        let offsets = pcf_sample_offsets();
        assert_eq!(offsets[4], [0.0, 0.0]);
    }

    #[test]
    fn bias_decreases_with_higher_dot_product() {
        let steep = shadow_bias_for_slope(0.1);
        let flat = shadow_bias_for_slope(0.9);
        assert!(steep > flat, "steep bias {steep} should exceed flat bias {flat}");
    }

    #[test]
    fn bias_is_finite_at_extremes() {
        assert!(shadow_bias_for_slope(0.0).is_finite());
        assert!(shadow_bias_for_slope(1.0).is_finite());
        assert!(shadow_bias_for_slope(-1.0).is_finite());
    }

    #[test]
    fn bias_is_always_positive() {
        for i in 0..=100 {
            let ndl = i as f32 / 100.0;
            assert!(shadow_bias_for_slope(ndl) > 0.0);
        }
    }
}
