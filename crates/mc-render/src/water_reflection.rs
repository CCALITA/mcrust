//! Water surface reflection parameters and helpers.
//!
//! Provides a small value type describing how water reflects the sky and
//! environment, plus pure helper functions for Schlick's Fresnel
//! approximation, above-water checks, and sky/tint color blending.

/// Parameters controlling water surface reflection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterReflection {
    /// Base reflection strength in `[0.0, 1.0]`.
    pub strength: f32,
    /// Fresnel exponent — higher values concentrate reflection at grazing angles.
    pub fresnel_power: f32,
    /// Distance (in world units) over which reflections fade out.
    pub fade_distance: f32,
}

/// Default water reflection tuned for typical daylight scenes.
pub fn default_water_reflection() -> WaterReflection {
    WaterReflection {
        strength: 0.6,
        fresnel_power: 5.0,
        fade_distance: 64.0,
    }
}

/// Schlick's Fresnel approximation for a water surface.
///
/// `view_angle_rad` is the angle between the view vector and the surface
/// normal. Returns a value in `[f0, 1.0]` where `f0 = 0.04` (typical water).
pub fn reflection_strength(view_angle_rad: f32) -> f32 {
    let f0: f32 = 0.04;
    let cos_angle = view_angle_rad.cos().clamp(0.0, 1.0);
    let one_minus = 1.0 - cos_angle;
    f0 + (1.0 - f0) * one_minus.powi(5)
}

/// Returns `true` when the camera is above the water plane and should see
/// a reflection of the sky/scene.
pub fn should_reflect(camera_y: f32, water_y: f32) -> bool {
    camera_y > water_y
}

/// Linear interpolation between the sky color and a water tint, weighted by
/// `strength` (clamped to `[0, 1]`).
pub fn reflection_color(sky: [f32; 3], water_tint: [f32; 3], strength: f32) -> [f32; 3] {
    let s = strength.clamp(0.0, 1.0);
    [
        water_tint[0] * (1.0 - s) + sky[0] * s,
        water_tint[1] * (1.0 - s) + sky[1] * s,
        water_tint[2] * (1.0 - s) + sky[2] * s,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn default_values_match_spec() {
        let r = default_water_reflection();
        assert!(approx_eq(r.strength, 0.6, 1e-6));
        assert!(approx_eq(r.fresnel_power, 5.0, 1e-6));
        assert!(approx_eq(r.fade_distance, 64.0, 1e-6));
    }

    #[test]
    fn fresnel_at_zero_angle_is_f0() {
        // Looking straight down at the surface — cos(0) = 1, so result = f0.
        let s = reflection_strength(0.0);
        assert!(approx_eq(s, 0.04, 1e-6));
    }

    #[test]
    fn fresnel_at_grazing_angle_is_one() {
        // Grazing angle — cos(pi/2) ~= 0, so result = 1.0.
        let s = reflection_strength(std::f32::consts::FRAC_PI_2);
        assert!(approx_eq(s, 1.0, 1e-5));
    }

    #[test]
    fn fresnel_is_monotonic_increasing() {
        let a = reflection_strength(0.1);
        let b = reflection_strength(0.5);
        let c = reflection_strength(1.2);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn fresnel_stays_in_unit_range() {
        for i in 0..=20 {
            let angle = (i as f32) * std::f32::consts::FRAC_PI_2 / 20.0;
            let s = reflection_strength(angle);
            assert!((0.04 - 1e-5..=1.0 + 1e-5).contains(&s));
        }
    }

    #[test]
    fn should_reflect_when_camera_above_water() {
        assert!(should_reflect(64.5, 62.0));
    }

    #[test]
    fn should_not_reflect_when_camera_below_or_at_water() {
        assert!(!should_reflect(60.0, 62.0));
        assert!(!should_reflect(62.0, 62.0));
    }

    #[test]
    fn reflection_color_strength_zero_returns_water_tint() {
        let sky = [1.0, 1.0, 1.0];
        let tint = [0.1, 0.3, 0.7];
        let c = reflection_color(sky, tint, 0.0);
        assert_eq!(c, tint);
    }

    #[test]
    fn reflection_color_strength_one_returns_sky() {
        let sky = [0.4, 0.6, 0.9];
        let tint = [0.1, 0.3, 0.7];
        let c = reflection_color(sky, tint, 1.0);
        assert_eq!(c, sky);
    }

    #[test]
    fn reflection_color_half_is_midpoint() {
        let sky = [1.0, 1.0, 1.0];
        let tint = [0.0, 0.0, 0.0];
        let c = reflection_color(sky, tint, 0.5);
        assert!(approx_eq(c[0], 0.5, 1e-6));
        assert!(approx_eq(c[1], 0.5, 1e-6));
        assert!(approx_eq(c[2], 0.5, 1e-6));
    }

    #[test]
    fn reflection_color_clamps_strength() {
        let sky = [1.0, 1.0, 1.0];
        let tint = [0.0, 0.0, 0.0];
        let over = reflection_color(sky, tint, 2.0);
        let under = reflection_color(sky, tint, -0.5);
        assert_eq!(over, sky);
        assert_eq!(under, tint);
    }
}
