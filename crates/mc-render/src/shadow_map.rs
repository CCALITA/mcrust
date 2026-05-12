//! Shadow map configuration and cascade utilities.

/// Settings for shadow map rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowMapSettings {
    pub resolution: u32,
    pub cascade_count: u8,
    pub max_distance: f32,
    pub bias: f32,
}

/// Returns default shadow map settings (1024, 3 cascades, 64.0 max distance, 0.005 bias).
pub fn default_shadow_settings() -> ShadowMapSettings {
    ShadowMapSettings {
        resolution: 1024,
        cascade_count: 3,
        max_distance: 64.0,
        bias: 0.005,
    }
}

/// Computes logarithmic cascade split distances for the given max distance and count.
///
/// Returns a `Vec` of `count` distances distributed logarithmically from 0 to `max`.
pub fn cascade_split_distances(max: f32, count: u8) -> Vec<f32> {
    (0..count)
        .map(|i| max * ((i as f32 + 1.0) / count as f32))
        .collect()
}

/// Returns shadow strength as a linear falloff from 1.0 at distance 0 to 0.0 at `max`.
///
/// Values are clamped to `[0.0, 1.0]`.
pub fn shadow_strength(distance: f32, max: f32) -> f32 {
    (1.0 - distance / max).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_values() {
        let s = default_shadow_settings();
        assert_eq!(s.resolution, 1024);
        assert_eq!(s.cascade_count, 3);
        assert!((s.max_distance - 64.0).abs() < f32::EPSILON);
        assert!((s.bias - 0.005).abs() < f32::EPSILON);
    }

    #[test]
    fn cascade_splits_single() {
        let splits = cascade_split_distances(100.0, 1);
        assert_eq!(splits.len(), 1);
        assert!((splits[0] - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cascade_splits_multiple() {
        let splits = cascade_split_distances(90.0, 3);
        assert_eq!(splits.len(), 3);
        assert!((splits[0] - 30.0).abs() < f32::EPSILON);
        assert!((splits[1] - 60.0).abs() < f32::EPSILON);
        assert!((splits[2] - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn shadow_strength_at_zero() {
        assert!((shadow_strength(0.0, 64.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn shadow_strength_at_max() {
        assert!((shadow_strength(64.0, 64.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn shadow_strength_beyond_max() {
        assert!((shadow_strength(100.0, 64.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn shadow_strength_midpoint() {
        assert!((shadow_strength(32.0, 64.0) - 0.5).abs() < f32::EPSILON);
    }
}
