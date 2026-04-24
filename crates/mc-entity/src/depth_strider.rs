//! Depth Strider enchantment — underwater movement speed bonuses.
//!
//! Depth Strider increases a player's underwater movement speed.
//! Each level reduces the speed penalty of water by one-third,
//! and level III lets the player move at near-normal land speed.

/// Base multiplier applied to movement speed when submerged without
/// any Depth Strider enchantment. Vanilla Minecraft slows water
/// movement to roughly 20% of land speed.
pub const WATER_SPEED_BASE_MULTIPLIER: f32 = 0.2;

/// Maximum enchantment level for Depth Strider.
pub const MAX_DEPTH_STRIDER_LEVEL: u8 = 3;

/// Speed boost per Depth Strider level (each level adds ~33.3%).
const SPEED_PER_LEVEL: f32 = 0.333;

/// Returns the multiplicative speed factor for a given Depth Strider level.
///
/// - Level 0 returns `1.0` (no bonus).
/// - Level 1 returns `1.333`.
/// - Level 2 returns `1.666`.
/// - Level 3 returns `1.999`.
///
/// The level is clamped to [`MAX_DEPTH_STRIDER_LEVEL`].
pub fn depth_strider_speed_multiplier(level: u8) -> f32 {
    let clamped = level.min(MAX_DEPTH_STRIDER_LEVEL);
    1.0 + clamped as f32 * SPEED_PER_LEVEL
}

/// Applies the water speed modifier to a base movement speed.
///
/// When `in_water` is `true`, the base speed is first reduced by
/// [`WATER_SPEED_BASE_MULTIPLIER`], then scaled by the Depth Strider
/// bonus for the given enchantment level.
///
/// When `in_water` is `false`, the base speed is returned unchanged.
pub fn apply_water_speed(base: f32, level: u8, in_water: bool) -> f32 {
    if !in_water {
        return base;
    }
    base * WATER_SPEED_BASE_MULTIPLIER * depth_strider_speed_multiplier(level)
}

/// Returns the swim speed with an optional Dolphin's Grace effect.
///
/// When `has_dolphins_grace` is `true`, the base speed is multiplied by 4.
/// Otherwise the base speed is returned unchanged.
pub fn swim_speed_with_dolphins_grace(base: f32, has_dolphins_grace: bool) -> f32 {
    if has_dolphins_grace {
        base * 4.0
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_expected_values() {
        assert!((WATER_SPEED_BASE_MULTIPLIER - 0.2).abs() < f32::EPSILON);
        assert_eq!(MAX_DEPTH_STRIDER_LEVEL, 3);
    }

    #[test]
    fn multiplier_at_level_zero_is_one() {
        let m = depth_strider_speed_multiplier(0);
        assert!((m - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_scales_linearly_with_level() {
        let m1 = depth_strider_speed_multiplier(1);
        assert!((m1 - 1.333).abs() < 0.001);

        let m2 = depth_strider_speed_multiplier(2);
        assert!((m2 - 1.666).abs() < 0.001);

        let m3 = depth_strider_speed_multiplier(3);
        assert!((m3 - 1.999).abs() < 0.001);
    }

    #[test]
    fn multiplier_clamps_above_max_level() {
        let m4 = depth_strider_speed_multiplier(4);
        let m3 = depth_strider_speed_multiplier(3);
        assert!((m4 - m3).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_water_speed_returns_base_when_not_in_water() {
        let result = apply_water_speed(10.0, 3, false);
        assert!((result - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_water_speed_reduces_speed_in_water_without_enchantment() {
        let result = apply_water_speed(10.0, 0, true);
        let expected = 10.0 * WATER_SPEED_BASE_MULTIPLIER;
        assert!((result - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_water_speed_with_depth_strider_level_3() {
        let result = apply_water_speed(10.0, 3, true);
        let expected = 10.0 * WATER_SPEED_BASE_MULTIPLIER * depth_strider_speed_multiplier(3);
        assert!((result - expected).abs() < 0.01);
    }

    #[test]
    fn apply_water_speed_with_zero_base() {
        let result = apply_water_speed(0.0, 3, true);
        assert!((result).abs() < f32::EPSILON);
    }

    #[test]
    fn dolphins_grace_multiplies_by_four() {
        let result = swim_speed_with_dolphins_grace(5.0, true);
        assert!((result - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn no_dolphins_grace_returns_base() {
        let result = swim_speed_with_dolphins_grace(5.0, false);
        assert!((result - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn dolphins_grace_with_zero_base() {
        let result = swim_speed_with_dolphins_grace(0.0, true);
        assert!((result).abs() < f32::EPSILON);
    }

    #[test]
    fn full_water_speed_pipeline() {
        // Simulate a player in water with Depth Strider III and Dolphin's Grace.
        let base = 4.317; // vanilla walk speed
        let water_speed = apply_water_speed(base, 3, true);
        let final_speed = swim_speed_with_dolphins_grace(water_speed, true);

        // Should be significantly faster than base water speed without enchantments.
        let plain_water = apply_water_speed(base, 0, true);
        assert!(final_speed > plain_water * 4.0);
    }
}
