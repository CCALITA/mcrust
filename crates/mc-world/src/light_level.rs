//! Light level display system.
//!
//! Provides utilities for computing effective light from block and sky
//! components, checking mob-spawn eligibility, converting light to a
//! brightness factor, and formatting a human-readable debug string.

/// Per-block light information combining block-emitted and sky light.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockLight {
    pub block_light: u8,
    pub sky_light: u8,
}

/// Compute sun brightness for a given time of day.
///
/// `time_of_day` ranges from `0.0` (midnight) through `0.5` (noon) to `1.0`
/// (next midnight). Returns a value in `[0.0, 1.0]` following a smooth sine
/// curve: 1.0 at noon, 0.0 at midnight.
pub fn sun_brightness(time_of_day: f32) -> f32 {
    // Map time so that noon (0.5) produces peak brightness.
    // sin(pi * t) peaks at t = 0.5 and is 0 at t = 0.0 and t = 1.0.
    let brightness = (std::f32::consts::PI * time_of_day).sin();
    brightness.clamp(0.0, 1.0)
}

/// Compute the effective (combined) light level from block light, sky light,
/// and the current time of day.
///
/// The effective sky contribution is `sky_light * sun_brightness(time_of_day)`.
/// The result is `max(block_light, effective_sky)`, clamped to `0..=15`.
pub fn combined_light(block_light: u8, sky_light: u8, time_of_day: f32) -> u8 {
    let effective_sky = (sky_light as f32 * sun_brightness(time_of_day)).round() as u8;
    let combined = block_light.max(effective_sky);
    combined.min(15)
}

/// Returns `true` when hostile mobs are allowed to spawn at the given
/// combined light level (light < 7).
pub fn can_mob_spawn(combined_light: u8) -> bool {
    combined_light < 7
}

/// Convert a light level (0..=15) to a nonlinear brightness factor in `[0.0, 1.0]`.
///
/// The curve is:
///   brightness = (1.0 - t) * 0.04 + t^2 * 0.96 + 0.04
/// where `t = level / 15.0`, clamped to `[0.0, 1.0]`.
pub fn light_to_brightness(level: u8) -> f32 {
    let t = level as f32 / 15.0;
    let brightness = (1.0 - t) * 0.04 + t * t * 0.96 + 0.04;
    brightness.clamp(0.0, 1.0)
}

/// Format block and sky light values as a human-readable debug string.
pub fn display_light_info(block_light: u8, sky_light: u8) -> String {
    format!("Block: {block_light}, Sky: {sky_light}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // sun_brightness tests
    // ------------------------------------------------------------------

    #[test]
    fn sun_brightness_at_noon_is_one() {
        let b = sun_brightness(0.5);
        assert!((b - 1.0).abs() < 1e-5, "noon brightness should be 1.0, got {b}");
    }

    #[test]
    fn sun_brightness_at_midnight_is_zero() {
        let b = sun_brightness(0.0);
        assert!(b.abs() < 1e-5, "midnight brightness should be 0.0, got {b}");
    }

    // ------------------------------------------------------------------
    // combined_light tests
    // ------------------------------------------------------------------

    #[test]
    fn noon_full_sky_gives_fifteen() {
        let result = combined_light(0, 15, 0.5);
        assert_eq!(result, 15, "noon with sky 15 should yield 15");
    }

    #[test]
    fn midnight_full_sky_gives_zero_sky() {
        let result = combined_light(0, 15, 0.0);
        assert_eq!(result, 0, "midnight sky contribution should be 0");
    }

    #[test]
    fn block_light_dominates_at_midnight() {
        let result = combined_light(10, 15, 0.0);
        assert_eq!(result, 10, "block light should dominate at midnight");
    }

    #[test]
    fn combined_light_clamped_to_fifteen() {
        // Even with block_light = 15 and full sky, result must not exceed 15.
        let result = combined_light(15, 15, 0.5);
        assert_eq!(result, 15);
    }

    // ------------------------------------------------------------------
    // can_mob_spawn tests
    // ------------------------------------------------------------------

    #[test]
    fn mob_spawns_below_threshold() {
        assert!(can_mob_spawn(0));
        assert!(can_mob_spawn(6));
    }

    #[test]
    fn mob_does_not_spawn_at_or_above_threshold() {
        assert!(!can_mob_spawn(7));
        assert!(!can_mob_spawn(15));
    }

    // ------------------------------------------------------------------
    // light_to_brightness tests
    // ------------------------------------------------------------------

    #[test]
    fn brightness_at_zero_is_low() {
        let b = light_to_brightness(0);
        // t=0: (1.0)*0.04 + 0 + 0.04 = 0.08
        assert!((b - 0.08).abs() < 1e-5, "brightness at 0 should be 0.08, got {b}");
    }

    #[test]
    fn brightness_at_fifteen_is_one() {
        let b = light_to_brightness(15);
        // t=1: 0 + 1.0*0.96 + 0.04 = 1.0
        assert!((b - 1.0).abs() < 1e-5, "brightness at 15 should be 1.0, got {b}");
    }

    #[test]
    fn brightness_is_monotonically_increasing() {
        let mut prev = light_to_brightness(0);
        for level in 1..=15 {
            let current = light_to_brightness(level);
            assert!(
                current >= prev,
                "brightness should be monotonic: level {level} ({current}) < level {} ({prev})",
                level - 1,
            );
            prev = current;
        }
    }

    // ------------------------------------------------------------------
    // display_light_info tests
    // ------------------------------------------------------------------

    #[test]
    fn display_format_is_correct() {
        let info = display_light_info(10, 14);
        assert_eq!(info, "Block: 10, Sky: 14");
    }
}
