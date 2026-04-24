//! Powder snow frost overlay — vignette, FOV narrowing, and freeze damage thresholds.

/// Frost overlay state tracking how frozen the player is while standing in
/// powder snow. Intensity ramps up over 7 seconds and decays over 5 seconds.
pub struct FrostOverlay {
    /// Current freeze progress in `[0.0, 1.0]`.
    pub intensity: f32,
    /// Border opacity derived from intensity, used by the vignette shader.
    pub border_opacity: f32,
}

impl FrostOverlay {
    /// Create a new frost overlay with zero intensity.
    pub fn new() -> Self {
        Self {
            intensity: 0.0,
            border_opacity: 0.0,
        }
    }
}

impl Default for FrostOverlay {
    fn default() -> Self {
        Self::new()
    }
}

/// Advance the frost overlay by `dt` seconds.
///
/// When `in_powder_snow` is true, intensity increases at `dt / 7.0` per second
/// (reaching 1.0 after 7 seconds). When false, it decays at `dt / 5.0`.
/// `border_opacity` tracks the current intensity.
pub fn tick_frost(overlay: &mut FrostOverlay, in_powder_snow: bool, dt: f32) {
    if in_powder_snow {
        overlay.intensity = (overlay.intensity + dt / 7.0).min(1.0);
    } else {
        overlay.intensity = (overlay.intensity - dt / 5.0).max(0.0);
    }
    overlay.border_opacity = overlay.intensity;
}

/// Compute the vignette alpha for the frost border at the given `intensity`.
///
/// Returns `intensity * 0.8`, so the overlay is never fully opaque.
pub fn frost_vignette_alpha(intensity: f32) -> f32 {
    intensity * 0.8
}

/// Compute the FOV multiplier at the given freeze `intensity`.
///
/// The field of view narrows as the player freezes:
/// `1.0 - intensity * 0.3`, so at full freeze the FOV is 70% of normal.
pub fn frost_fov_multiplier(intensity: f32) -> f32 {
    1.0 - intensity * 0.3
}

/// The intensity threshold at which the player starts taking freeze damage.
pub fn freeze_damage_threshold() -> f32 {
    1.0
}

/// Damage dealt per tick once the freeze threshold is reached.
pub fn freeze_damage_per_tick() -> f32 {
    1.0
}

/// Whether wearing any piece of leather armor prevents freezing.
pub fn leather_armor_prevents_freeze() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_overlay_starts_at_zero() {
        let overlay = FrostOverlay::new();
        assert!((overlay.intensity - 0.0).abs() < f32::EPSILON);
        assert!((overlay.border_opacity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_matches_new() {
        let a = FrostOverlay::new();
        let b = FrostOverlay::default();
        assert!((a.intensity - b.intensity).abs() < f32::EPSILON);
        assert!((a.border_opacity - b.border_opacity).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_increases_intensity_in_powder_snow() {
        let mut overlay = FrostOverlay::new();
        tick_frost(&mut overlay, true, 1.0);
        let expected = 1.0 / 7.0;
        assert!((overlay.intensity - expected).abs() < 1e-6);
        assert!((overlay.border_opacity - expected).abs() < 1e-6);
    }

    #[test]
    fn tick_clamps_intensity_at_one() {
        let mut overlay = FrostOverlay::new();
        // 10 seconds in powder snow — well past the 7-second ramp
        tick_frost(&mut overlay, true, 10.0);
        assert!((overlay.intensity - 1.0).abs() < f32::EPSILON);
        assert!((overlay.border_opacity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_decreases_intensity_outside_powder_snow() {
        let mut overlay = FrostOverlay::new();
        overlay.intensity = 0.5;
        tick_frost(&mut overlay, false, 1.0);
        let expected = 0.5 - 1.0 / 5.0;
        assert!((overlay.intensity - expected).abs() < 1e-6);
        assert!((overlay.border_opacity - expected).abs() < 1e-6);
    }

    #[test]
    fn tick_clamps_intensity_at_zero() {
        let mut overlay = FrostOverlay::new();
        overlay.intensity = 0.1;
        // Large dt should not go below zero
        tick_frost(&mut overlay, false, 10.0);
        assert!((overlay.intensity - 0.0).abs() < f32::EPSILON);
        assert!((overlay.border_opacity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn full_freeze_cycle() {
        let mut overlay = FrostOverlay::new();

        // Ramp up for exactly 7 seconds
        for _ in 0..70 {
            tick_frost(&mut overlay, true, 0.1);
        }
        assert!((overlay.intensity - 1.0).abs() < 1e-5);

        // Decay for exactly 5 seconds
        for _ in 0..50 {
            tick_frost(&mut overlay, false, 0.1);
        }
        assert!(overlay.intensity < 1e-5);
    }

    #[test]
    fn border_opacity_tracks_intensity() {
        let mut overlay = FrostOverlay::new();
        tick_frost(&mut overlay, true, 3.5);
        assert!((overlay.border_opacity - overlay.intensity).abs() < f32::EPSILON);

        tick_frost(&mut overlay, false, 1.0);
        assert!((overlay.border_opacity - overlay.intensity).abs() < f32::EPSILON);
    }

    #[test]
    fn vignette_alpha_scales_by_point_eight() {
        assert!((frost_vignette_alpha(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((frost_vignette_alpha(0.5) - 0.4).abs() < f32::EPSILON);
        assert!((frost_vignette_alpha(1.0) - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn fov_multiplier_narrows_with_intensity() {
        assert!((frost_fov_multiplier(0.0) - 1.0).abs() < f32::EPSILON);
        assert!((frost_fov_multiplier(0.5) - 0.85).abs() < f32::EPSILON);
        assert!((frost_fov_multiplier(1.0) - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn freeze_damage_threshold_is_one() {
        assert!((freeze_damage_threshold() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn freeze_damage_per_tick_is_one() {
        assert!((freeze_damage_per_tick() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn leather_armor_prevents_freeze_returns_true() {
        assert!(leather_armor_prevents_freeze());
    }
}
