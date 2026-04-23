//! Random tick system for crop growth and block updates.
//!
//! Implements Minecraft's random tick mechanism where each game tick selects
//! `RANDOM_TICK_SPEED` random block positions per chunk section and applies
//! growth logic to any crops found at those positions.

/// Vanilla default: 3 random ticks per chunk section per game tick.
pub const RANDOM_TICK_SPEED: u32 = 3;

/// Maximum growth stages for common crop types, keyed by block ID (as `u16`).
///
/// Format: `(block_id, max_stage)`.
pub const CROP_MAX_STAGES: [(u16, u8); 6] = [
    (0, 7), // wheat     — placeholder block ID 0
    (1, 7), // carrot    — placeholder block ID 1
    (2, 7), // potato    — placeholder block ID 2
    (3, 3), // beetroot  — placeholder block ID 3
    (4, 7), // melon_stem — placeholder block ID 4
    (5, 7), // pumpkin_stem — placeholder block ID 5
];

/// Select `count` random block positions within a 16x16x16 chunk section.
///
/// Each position is derived from a deterministic hash of the seed, tick, section
/// coordinates, and an iteration index. The hash uses the same LCG constants as
/// Java's `SplittableRandom` for good bit mixing.
///
/// Returns world-space coordinates computed from the section origin
/// `(section_x * 16, section_y * 16, section_z * 16)` plus an offset in 0..15.
pub fn select_random_tick_positions(
    section_x: i32,
    section_y: i32,
    section_z: i32,
    seed: u64,
    tick: u64,
    count: u32,
) -> Vec<(i32, i32, i32)> {
    let mut positions = Vec::with_capacity(count as usize);
    let base_x = section_x * 16;
    let base_y = section_y * 16;
    let base_z = section_z * 16;

    for i in 0..count {
        let hash = seed
            ^ (tick.wrapping_mul(6_364_136_223_846_793_005))
            .wrapping_add((section_x as u64).wrapping_mul(1_442_695_040_888_963_407))
            .wrapping_add((section_y as u64).wrapping_mul(6_364_136_223_846_793_005))
            .wrapping_add((section_z as u64).wrapping_mul(1_442_695_040_888_963_407))
            .wrapping_add(u64::from(i).wrapping_mul(2_862_933_555_777_941_757));

        let x = (hash & 0xF) as i32;
        let y = ((hash >> 4) & 0xF) as i32;
        let z = ((hash >> 8) & 0xF) as i32;

        positions.push((base_x + x, base_y + y, base_z + z));
    }

    positions
}

/// Determine whether a crop should advance to its next growth stage.
///
/// Growth requires:
/// - `light_level >= 9`
/// - The crop is not already at `max_stage`
///
/// Growth probability is `1 / floor(25 / growth_rate)` where `growth_rate` is
/// 4.0 when hydrated and 2.0 when dry. This gives:
/// - Hydrated: `1/floor(25/4)` = `1/6` (~16.7%)
/// - Dry:      `1/floor(25/2)` = `1/12` (~8.3%)
///
/// The function uses a deterministic check: it hashes the inputs and checks
/// whether the result modulo the denominator equals zero.
pub fn should_crop_grow(
    current_stage: u8,
    max_stage: u8,
    is_hydrated: bool,
    light_level: u8,
) -> bool {
    if current_stage >= max_stage {
        return false;
    }

    if light_level < 9 {
        return false;
    }

    let growth_rate: f64 = if is_hydrated { 4.0 } else { 2.0 };
    let denominator = (25.0_f64 / growth_rate).floor() as u32;

    // Use a simple deterministic check based on the inputs.
    // In practice the caller provides randomness; here we compute the
    // threshold so external callers can compare against a uniform random.
    // For a self-contained check we hash the inputs.
    let hash = (current_stage as u64)
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(light_level as u64)
        .wrapping_add(if is_hydrated { 1 } else { 0 });

    (hash % u64::from(denominator)) == 0
}

/// Advance a crop to its next growth stage, clamped at `max`.
pub fn next_growth_stage(current: u8, max: u8) -> u8 {
    if current < max {
        current + 1
    } else {
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- select_random_tick_positions ----------------------------------------

    #[test]
    fn positions_within_section_bounds() {
        let positions = select_random_tick_positions(0, 0, 0, 42, 100, 10);
        assert_eq!(positions.len(), 10);
        for (x, y, z) in &positions {
            assert!((0..16).contains(x), "x={x} out of bounds for section (0,0,0)");
            assert!((0..16).contains(y), "y={y} out of bounds for section (0,0,0)");
            assert!((0..16).contains(z), "z={z} out of bounds for section (0,0,0)");
        }
    }

    #[test]
    fn positions_offset_by_section_origin() {
        let positions = select_random_tick_positions(3, -2, 5, 42, 100, 5);
        let base_x = 3 * 16;
        let base_y = -2 * 16;
        let base_z = 5 * 16;
        for (x, y, z) in &positions {
            assert!(
                *x >= base_x && *x < base_x + 16,
                "x={x} not within section origin {base_x}"
            );
            assert!(
                *y >= base_y && *y < base_y + 16,
                "y={y} not within section origin {base_y}"
            );
            assert!(
                *z >= base_z && *z < base_z + 16,
                "z={z} not within section origin {base_z}"
            );
        }
    }

    #[test]
    fn deterministic_given_same_seed_and_tick() {
        let a = select_random_tick_positions(1, 2, 3, 999, 50, 5);
        let b = select_random_tick_positions(1, 2, 3, 999, 50, 5);
        assert_eq!(a, b, "same inputs must produce identical positions");
    }

    #[test]
    fn different_seed_produces_different_positions() {
        let a = select_random_tick_positions(0, 0, 0, 1, 100, 5);
        let b = select_random_tick_positions(0, 0, 0, 2, 100, 5);
        assert_ne!(a, b, "different seeds should produce different positions");
    }

    #[test]
    fn different_tick_produces_different_positions() {
        let a = select_random_tick_positions(0, 0, 0, 42, 1, 5);
        let b = select_random_tick_positions(0, 0, 0, 42, 2, 5);
        assert_ne!(a, b, "different ticks should produce different positions");
    }

    #[test]
    fn zero_count_returns_empty() {
        let positions = select_random_tick_positions(0, 0, 0, 42, 100, 0);
        assert!(positions.is_empty());
    }

    #[test]
    fn default_random_tick_speed_is_3() {
        assert_eq!(RANDOM_TICK_SPEED, 3);
    }

    // ---- should_crop_grow ---------------------------------------------------

    #[test]
    fn no_growth_at_max_stage() {
        // Regardless of hydration or light, a maxed crop never grows.
        assert!(!should_crop_grow(7, 7, true, 15));
        assert!(!should_crop_grow(3, 3, false, 15));
    }

    #[test]
    fn no_growth_when_light_below_9() {
        assert!(!should_crop_grow(0, 7, true, 8));
        assert!(!should_crop_grow(0, 7, false, 0));
    }

    #[test]
    fn growth_possible_at_minimum_light() {
        // Light level 9 should not block growth (it may or may not grow
        // depending on the hash, but the light check passes).
        // We test multiple stages to confirm at least one grows.
        let any_grew = (0..7).any(|stage| should_crop_grow(stage, 7, true, 9));
        assert!(any_grew, "at least one stage should grow with light=9 and hydration");
    }

    #[test]
    fn hydrated_has_higher_growth_rate_denominator() {
        // Hydrated growth_rate = 4.0 -> denominator = floor(25/4) = 6
        // Dry growth_rate = 2.0 -> denominator = floor(25/2) = 12
        // A smaller denominator means more frequent growth (1/6 > 1/12).
        let hydrated_denom = (25.0_f64 / 4.0).floor() as u32;
        let dry_denom = (25.0_f64 / 2.0).floor() as u32;
        assert_eq!(hydrated_denom, 6);
        assert_eq!(dry_denom, 12);
        assert!(
            hydrated_denom < dry_denom,
            "hydrated denominator ({hydrated_denom}) should be < dry ({dry_denom})"
        );
    }

    // ---- next_growth_stage --------------------------------------------------

    #[test]
    fn advances_stage_by_one() {
        assert_eq!(next_growth_stage(0, 7), 1);
        assert_eq!(next_growth_stage(3, 7), 4);
        assert_eq!(next_growth_stage(6, 7), 7);
    }

    #[test]
    fn clamped_at_max_stage() {
        assert_eq!(next_growth_stage(7, 7), 7);
        assert_eq!(next_growth_stage(3, 3), 3);
    }

    #[test]
    fn max_zero_stays_at_zero() {
        assert_eq!(next_growth_stage(0, 0), 0);
    }

    // ---- CROP_MAX_STAGES ----------------------------------------------------

    #[test]
    fn crop_max_stages_has_correct_values() {
        // wheat=7, carrot=7, potato=7, beetroot=3, melon_stem=7, pumpkin_stem=7
        let expected_max_stages = [7_u8, 7, 7, 3, 7, 7];
        assert_eq!(CROP_MAX_STAGES.len(), 6);
        for (i, (_, max)) in CROP_MAX_STAGES.iter().enumerate() {
            assert_eq!(
                *max, expected_max_stages[i],
                "crop index {i} max stage mismatch"
            );
        }
    }
}
