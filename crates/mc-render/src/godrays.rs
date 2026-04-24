//! Godrays / volumetric light scattering settings and helpers.
//!
//! Provides simple, deterministic math for computing godray parameters
//! based on time of day, weather, and camera orientation. The actual
//! shader-side volumetric scattering uses these values as inputs.

/// Parameters controlling the volumetric light (godray) post-process.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GodraySettings {
    /// World-space direction of the light source (normalized, pointing toward the sun).
    pub sun_dir: [f32; 3],
    /// Overall intensity multiplier in [0, 1].
    pub intensity: f32,
    /// Per-sample decay factor in (0, 1]; higher = longer rays.
    pub decay: f32,
    /// Number of radial samples along each ray.
    pub samples: u8,
}

/// Default godray settings — tuned for a clear day.
pub fn default_godrays() -> GodraySettings {
    GodraySettings {
        sun_dir: [0.0, 1.0, 0.0],
        intensity: 0.6,
        decay: 0.95,
        samples: 64,
    }
}

/// Godray tint color for the given time of day in [0, 1].
///
/// - 0.0  → dawn (orange)
/// - 0.25 → noon (yellow)
/// - 0.5  → dusk (orange)
/// - 0.75 → midnight (dark)
pub fn godray_color(time_of_day: f32) -> [f32; 3] {
    let t = time_of_day.rem_euclid(1.0);
    // Sun elevation factor: 1.0 at noon (t=0.25), 0.0 at dawn/dusk, -1.0 at midnight.
    let elevation = (t * std::f32::consts::TAU).sin();
    // Clamp to [0,1] for daytime; night stays dark.
    let day = elevation.max(0.0);
    // How close to horizon (dawn/dusk) vs overhead.
    // `horizon_weight` is 1 at dawn/dusk, 0 at noon.
    let horizon_weight = (1.0 - day).clamp(0.0, 1.0);

    // Base colors
    let yellow = [1.0_f32, 0.95, 0.55];
    let orange = [1.0_f32, 0.55, 0.20];
    let night = [0.02_f32, 0.02, 0.05];

    if day <= 0.0 {
        return night;
    }

    // Mix yellow (noon) with orange (horizon), then scale down by elevation so
    // dusk/dawn are dim orange and noon is bright yellow.
    let mixed = [
        yellow[0] * (1.0 - horizon_weight) + orange[0] * horizon_weight,
        yellow[1] * (1.0 - horizon_weight) + orange[1] * horizon_weight,
        yellow[2] * (1.0 - horizon_weight) + orange[2] * horizon_weight,
    ];
    // Slight dimming at horizon.
    let brightness = 0.35 + 0.65 * day;
    [
        mixed[0] * brightness,
        mixed[1] * brightness,
        mixed[2] * brightness,
    ]
}

/// Whether the sun is within the camera's forward hemisphere.
///
/// `camera_yaw` and `sun_yaw` are in radians. Returns true if the absolute
/// angular difference is strictly less than 90 degrees.
pub fn godray_visible(camera_yaw: f32, sun_yaw: f32) -> bool {
    let mut diff = (camera_yaw - sun_yaw).rem_euclid(std::f32::consts::TAU);
    if diff > std::f32::consts::PI {
        diff = std::f32::consts::TAU - diff;
    }
    diff < std::f32::consts::FRAC_PI_2
}

/// Effective intensity multiplier given weather and atmospheric fog density.
///
/// - `weather`: 0 = clear, 1 = rain, 2 = thunder.
/// - `fog_density`: in [0, 1]; above 0.5 suppresses godrays fully.
pub fn godray_intensity(weather: u8, fog_density: f32) -> f32 {
    if weather != 0 {
        return 0.0;
    }
    if fog_density >= 0.5 {
        return 0.0;
    }
    // Clear weather: linearly fade from 1.0 at fog=0 to 0.0 at fog=0.5.
    (1.0 - fog_density * 2.0).clamp(0.0, 1.0)
}

/// World-space sun position on a unit hemisphere given the time of day in [0, 1].
///
/// Convention: +X is east, +Y is up, +Z is south.
/// - 0.0  → due east, on horizon (dawn)
/// - 0.25 → overhead (noon)
/// - 0.5  → due west, on horizon (dusk)
/// - 0.75 → below horizon (midnight)
pub fn sun_position(time_of_day: f32) -> [f32; 3] {
    let t = time_of_day.rem_euclid(1.0);
    let angle = t * std::f32::consts::TAU; // 0 at dawn, pi/2 at noon
    // At t=0: angle=0 → (cos=1, sin=0) → east, horizon.
    // At t=0.25: angle=pi/2 → (cos=0, sin=1) → up.
    // At t=0.5: angle=pi → (cos=-1, sin=0) → west, horizon.
    let x = angle.cos();
    let y = angle.sin();
    [x, y, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn defaults_have_expected_values() {
        let g = default_godrays();
        assert!(approx(g.intensity, 0.6, 1e-6));
        assert!(approx(g.decay, 0.95, 1e-6));
        assert_eq!(g.samples, 64);
    }

    #[test]
    fn color_yellow_at_noon() {
        let c = godray_color(0.25);
        // R > G > B, R should be bright, B should be low.
        assert!(c[0] > 0.8);
        assert!(c[1] > 0.7);
        assert!(c[2] < c[1]);
        assert!(c[0] >= c[1]);
    }

    #[test]
    fn color_orange_at_dawn_and_dusk() {
        // Dawn: just after t=0.0; dusk: just before t=0.5.
        for &t in &[0.01_f32, 0.49_f32] {
            let c = godray_color(t);
            assert!(c[0] > c[1], "R>G expected at time {}", t);
            assert!(c[1] > c[2], "G>B expected at time {}", t);
        }
    }

    #[test]
    fn color_dark_at_night() {
        let c = godray_color(0.75);
        assert!(c[0] < 0.1);
        assert!(c[1] < 0.1);
        assert!(c[2] < 0.1);
    }

    #[test]
    fn visible_when_facing_sun() {
        assert!(godray_visible(0.0, 0.0));
        assert!(godray_visible(0.5, 0.6));
    }

    #[test]
    fn invisible_when_facing_away() {
        assert!(!godray_visible(0.0, std::f32::consts::PI));
        assert!(!godray_visible(
            0.0,
            std::f32::consts::FRAC_PI_2 + 0.01
        ));
    }

    #[test]
    fn visible_wraps_around_tau() {
        // Camera yaw just past tau, sun near 0: should still be visible.
        assert!(godray_visible(
            std::f32::consts::TAU + 0.1,
            0.0
        ));
    }

    #[test]
    fn intensity_zero_in_rain() {
        assert_eq!(godray_intensity(1, 0.0), 0.0);
        assert_eq!(godray_intensity(2, 0.0), 0.0);
    }

    #[test]
    fn intensity_zero_in_heavy_fog() {
        assert_eq!(godray_intensity(0, 0.5), 0.0);
        assert_eq!(godray_intensity(0, 0.9), 0.0);
    }

    #[test]
    fn intensity_full_in_clear_no_fog() {
        assert!(approx(godray_intensity(0, 0.0), 1.0, 1e-6));
    }

    #[test]
    fn intensity_decreases_with_fog() {
        let a = godray_intensity(0, 0.0);
        let b = godray_intensity(0, 0.2);
        let c = godray_intensity(0, 0.4);
        assert!(a > b && b > c);
    }

    #[test]
    fn sun_east_at_dawn() {
        let p = sun_position(0.0);
        assert!(approx(p[0], 1.0, 1e-5));
        assert!(approx(p[1], 0.0, 1e-5));
    }

    #[test]
    fn sun_overhead_at_noon() {
        let p = sun_position(0.25);
        assert!(approx(p[0], 0.0, 1e-5));
        assert!(approx(p[1], 1.0, 1e-5));
    }

    #[test]
    fn sun_west_at_dusk() {
        let p = sun_position(0.5);
        assert!(approx(p[0], -1.0, 1e-5));
        assert!(approx(p[1], 0.0, 1e-5));
    }

    #[test]
    fn sun_below_horizon_at_midnight() {
        let p = sun_position(0.75);
        assert!(p[1] < 0.0);
    }
}
