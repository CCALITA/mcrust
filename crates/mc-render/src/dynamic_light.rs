//! Dynamic block light sources and per-fragment light accumulation.

/// A point light source emitted by a light-emitting block.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicLightSource {
    /// World-space position of the light source.
    pub pos: [f32; 3],
    /// Light level intensity (0..15, matching Minecraft light levels).
    pub intensity: f32,
    /// RGB color of the emitted light (each channel 0.0..1.0).
    pub color: [f32; 3],
    /// Maximum reach in blocks before the light falls off to zero.
    pub range: u8,
}

/// Create a light source for a torch (light level 14, warm yellow).
pub fn light_for_torch(pos: [f32; 3]) -> DynamicLightSource {
    DynamicLightSource {
        pos,
        intensity: 14.0,
        color: [1.0, 0.9, 0.7],
        range: 14,
    }
}

/// Create a light source for glowstone (light level 15, warm gold).
pub fn light_for_glowstone(pos: [f32; 3]) -> DynamicLightSource {
    DynamicLightSource {
        pos,
        intensity: 15.0,
        color: [1.0, 0.9, 0.5],
        range: 15,
    }
}

/// Create a light source for a lantern (light level 15, warm amber).
pub fn light_for_lantern(pos: [f32; 3]) -> DynamicLightSource {
    DynamicLightSource {
        pos,
        intensity: 15.0,
        color: [1.0, 0.85, 0.5],
        range: 15,
    }
}

/// Create a light source for a redstone torch (light level 7, dim red).
pub fn light_for_redstone_torch(pos: [f32; 3]) -> DynamicLightSource {
    DynamicLightSource {
        pos,
        intensity: 7.0,
        color: [1.0, 0.2, 0.2],
        range: 7,
    }
}

/// Create a light source for a soul lantern (light level 10, cool blue).
pub fn light_for_soul_lantern(pos: [f32; 3]) -> DynamicLightSource {
    DynamicLightSource {
        pos,
        intensity: 10.0,
        color: [0.4, 0.7, 1.0],
        range: 10,
    }
}

/// Compute the accumulated light color at `pos` from all `sources`.
///
/// Each source contributes its color scaled by `(intensity / 15) * falloff`,
/// where `falloff = max(0, 1 - distance / range)`.  The result is clamped
/// component-wise to `[0.0, 1.0]`.
pub fn compute_light_at(pos: [f32; 3], sources: &[DynamicLightSource]) -> [f32; 3] {
    let mut accumulated = [0.0_f32; 3];

    for src in sources {
        let dx = pos[0] - src.pos[0];
        let dy = pos[1] - src.pos[1];
        let dz = pos[2] - src.pos[2];
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        let range_f = src.range as f32;
        if range_f <= 0.0 || distance >= range_f {
            continue;
        }

        let falloff = 1.0 - distance / range_f;
        let brightness = (src.intensity / 15.0) * falloff;

        accumulated[0] += src.color[0] * brightness;
        accumulated[1] += src.color[1] * brightness;
        accumulated[2] += src.color[2] * brightness;
    }

    // Clamp each channel to [0.0, 1.0].
    accumulated[0] = accumulated[0].clamp(0.0, 1.0);
    accumulated[1] = accumulated[1].clamp(0.0, 1.0);
    accumulated[2] = accumulated[2].clamp(0.0, 1.0);

    accumulated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torch_parameters() {
        let src = light_for_torch([0.0, 0.0, 0.0]);
        assert_eq!(src.intensity, 14.0);
        assert_eq!(src.color, [1.0, 0.9, 0.7]);
        assert_eq!(src.range, 14);
    }

    #[test]
    fn glowstone_parameters() {
        let src = light_for_glowstone([1.0, 2.0, 3.0]);
        assert_eq!(src.intensity, 15.0);
        assert_eq!(src.color, [1.0, 0.9, 0.5]);
        assert_eq!(src.range, 15);
    }

    #[test]
    fn lantern_parameters() {
        let src = light_for_lantern([0.0, 0.0, 0.0]);
        assert_eq!(src.intensity, 15.0);
        assert_eq!(src.color, [1.0, 0.85, 0.5]);
        assert_eq!(src.range, 15);
    }

    #[test]
    fn redstone_torch_parameters() {
        let src = light_for_redstone_torch([0.0, 0.0, 0.0]);
        assert_eq!(src.intensity, 7.0);
        assert_eq!(src.color, [1.0, 0.2, 0.2]);
        assert_eq!(src.range, 7);
    }

    #[test]
    fn soul_lantern_parameters() {
        let src = light_for_soul_lantern([0.0, 0.0, 0.0]);
        assert_eq!(src.intensity, 10.0);
        assert_eq!(src.color, [0.4, 0.7, 1.0]);
        assert_eq!(src.range, 10);
    }

    #[test]
    fn no_sources_gives_darkness() {
        let result = compute_light_at([5.0, 5.0, 5.0], &[]);
        assert_eq!(result, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn at_source_position_gives_full_brightness() {
        let src = light_for_glowstone([0.0, 0.0, 0.0]);
        let result = compute_light_at([0.0, 0.0, 0.0], &[src]);
        // intensity 15 / 15 * falloff 1.0 * color
        assert_eq!(result, [1.0, 0.9, 0.5]);
    }

    #[test]
    fn beyond_range_gives_darkness() {
        let src = light_for_torch([0.0, 0.0, 0.0]);
        // 14 blocks away = at the range boundary
        let result = compute_light_at([14.0, 0.0, 0.0], &[src]);
        assert_eq!(result, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn beyond_range_far_gives_darkness() {
        let src = light_for_torch([0.0, 0.0, 0.0]);
        let result = compute_light_at([100.0, 0.0, 0.0], &[src]);
        assert_eq!(result, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn linear_falloff_at_half_range() {
        let src = light_for_glowstone([0.0, 0.0, 0.0]);
        // Half range = 7.5 blocks, falloff = 0.5, brightness = 1.0 * 0.5 = 0.5
        let result = compute_light_at([7.5, 0.0, 0.0], &[src]);
        assert!((result[0] - 0.5).abs() < 1e-5);
        assert!((result[1] - 0.45).abs() < 1e-5);
        assert!((result[2] - 0.25).abs() < 1e-5);
    }

    #[test]
    fn accumulation_of_two_sources() {
        let src_a = DynamicLightSource {
            pos: [0.0, 0.0, 0.0],
            intensity: 15.0,
            color: [1.0, 0.0, 0.0],
            range: 10,
        };
        let src_b = DynamicLightSource {
            pos: [0.0, 0.0, 0.0],
            intensity: 15.0,
            color: [0.0, 1.0, 0.0],
            range: 10,
        };
        let result = compute_light_at([0.0, 0.0, 0.0], &[src_a, src_b]);
        assert_eq!(result, [1.0, 1.0, 0.0]);
    }

    #[test]
    fn accumulation_clamps_to_one() {
        let src_a = DynamicLightSource {
            pos: [0.0, 0.0, 0.0],
            intensity: 15.0,
            color: [1.0, 1.0, 1.0],
            range: 10,
        };
        let src_b = DynamicLightSource {
            pos: [0.0, 0.0, 0.0],
            intensity: 15.0,
            color: [1.0, 1.0, 1.0],
            range: 10,
        };
        let result = compute_light_at([0.0, 0.0, 0.0], &[src_a, src_b]);
        assert_eq!(result, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn diagonal_distance_falloff() {
        let src = DynamicLightSource {
            pos: [0.0, 0.0, 0.0],
            intensity: 15.0,
            color: [1.0, 1.0, 1.0],
            range: 10,
        };
        // Distance = sqrt(3^2 + 4^2) = 5.0, falloff = 1 - 5/10 = 0.5
        let result = compute_light_at([3.0, 4.0, 0.0], &[src]);
        assert!((result[0] - 0.5).abs() < 1e-5);
        assert!((result[1] - 0.5).abs() < 1e-5);
        assert!((result[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn partial_intensity_source() {
        // Redstone torch: intensity 7 / 15 ~ 0.4667
        let src = light_for_redstone_torch([0.0, 0.0, 0.0]);
        let result = compute_light_at([0.0, 0.0, 0.0], &[src]);
        let expected_brightness = 7.0 / 15.0;
        assert!((result[0] - 1.0 * expected_brightness).abs() < 1e-5);
        assert!((result[1] - 0.2 * expected_brightness).abs() < 1e-5);
        assert!((result[2] - 0.2 * expected_brightness).abs() < 1e-5);
    }
}
