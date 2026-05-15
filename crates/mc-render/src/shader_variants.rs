//! Terrain shader variant composer — builds WGSL fragment shaders from feature flags.

/// Individual shader features that can be composed into a variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderFeature {
    Fog,
    AmbientOcclusion,
    VertexColor,
    NormalMapping,
}

impl ShaderFeature {
    /// Returns the bitmask bit position for this feature.
    fn bit(self) -> u32 {
        match self {
            ShaderFeature::Fog => 0,
            ShaderFeature::AmbientOcclusion => 1,
            ShaderFeature::VertexColor => 2,
            ShaderFeature::NormalMapping => 3,
        }
    }
}

/// A composed set of shader features that determines which WGSL sections are included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderVariant {
    pub features: Vec<ShaderFeature>,
}

/// Returns a variant with no features enabled.
pub fn default_variant() -> ShaderVariant {
    ShaderVariant {
        features: Vec::new(),
    }
}

/// Returns a variant with Fog and AmbientOcclusion enabled.
pub fn full_variant() -> ShaderVariant {
    ShaderVariant {
        features: vec![ShaderFeature::Fog, ShaderFeature::AmbientOcclusion],
    }
}

/// Computes a unique bitmask key for a shader variant.
pub fn variant_key(variant: &ShaderVariant) -> u32 {
    variant
        .features
        .iter()
        .fold(0u32, |acc, f| acc | (1 << f.bit()))
}

/// Composes a WGSL fragment shader string from the given variant.
///
/// Starts with a base diffuse sampling section, then conditionally appends
/// fog and ambient occlusion sections based on the variant's features.
pub fn compose_fragment_shader(variant: &ShaderVariant) -> String {
    let mut shader = String::from(
        "\
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
",
    );

    if variant.features.contains(&ShaderFeature::AmbientOcclusion) {
        shader.push_str(
            "\
    // Ambient Occlusion
    let ao = in.ao_value;
    color = vec4<f32>(color.rgb * ao, color.a);
",
        );
    }

    if variant.features.contains(&ShaderFeature::Fog) {
        shader.push_str(
            "\
    // Fog
    let fog_factor = clamp((fog_end - in.view_distance) / (fog_end - fog_start), 0.0, 1.0);
    color = mix(fog_color, color, fog_factor);
",
        );
    }

    shader.push_str(
        "\
    return color;
}
",
    );

    shader
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_variant_has_no_fog() {
        let variant = default_variant();
        let shader = compose_fragment_shader(&variant);
        assert!(!shader.contains("fog_factor"), "default variant should not contain fog");
        assert!(!variant.features.contains(&ShaderFeature::Fog));
    }

    #[test]
    fn full_variant_has_fog_and_ao() {
        let variant = full_variant();
        let shader = compose_fragment_shader(&variant);
        assert!(shader.contains("fog_factor"), "full variant should contain fog");
        assert!(shader.contains("ao_value"), "full variant should contain AO");
        assert!(variant.features.contains(&ShaderFeature::Fog));
        assert!(variant.features.contains(&ShaderFeature::AmbientOcclusion));
    }

    #[test]
    fn variant_key_uniqueness() {
        let empty = default_variant();
        let fog_only = ShaderVariant {
            features: vec![ShaderFeature::Fog],
        };
        let ao_only = ShaderVariant {
            features: vec![ShaderFeature::AmbientOcclusion],
        };
        let full = full_variant();

        let keys = [
            variant_key(&empty),
            variant_key(&fog_only),
            variant_key(&ao_only),
            variant_key(&full),
        ];

        // All keys must be distinct
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j], "keys at index {i} and {j} must differ");
            }
        }
    }

    #[test]
    fn base_shader_always_has_diffuse_sample() {
        let variant = default_variant();
        let shader = compose_fragment_shader(&variant);
        assert!(shader.contains("textureSample(t_diffuse"));
    }
}
