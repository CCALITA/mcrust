//! Fall damage calculation with enchantment and potion reductions.
//!
//! Provides functions for computing fall damage accounting for Feather Falling
//! enchantment, armor damage reduction, and Jump Boost potion effects.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Blocks of free-fall before damage begins.
const FREE_FALL_BLOCKS: f32 = 3.0;

/// Damage reduction per Feather Falling enchantment level (12% each).
const FEATHER_FALLING_REDUCTION_PER_LEVEL: f32 = 0.12;

/// Damage reduction per armor point (4% each).
const ARMOR_REDUCTION_PER_POINT: f32 = 0.04;

/// Maximum Feather Falling enchantment level.
const MAX_FEATHER_FALLING_LEVEL: u8 = 4;

// ---------------------------------------------------------------------------
// Fall damage
// ---------------------------------------------------------------------------

/// Calculate fall damage with Feather Falling enchantment.
///
/// Base damage is `max(0, floor(distance - 3.0))`. Feather Falling reduces
/// damage by 12% per level (capped at level 4, i.e. 48% maximum reduction).
/// Returns 0 if the result would be negative.
pub fn calculate_fall_damage(fall_distance: f32, feather_falling_level: u8) -> u32 {
    let base = (fall_distance - FREE_FALL_BLOCKS).floor();
    if base <= 0.0 {
        return 0;
    }

    let level = feather_falling_level.min(MAX_FEATHER_FALLING_LEVEL);
    let reduction = FEATHER_FALLING_REDUCTION_PER_LEVEL * level as f32;
    let reduced = base * (1.0 - reduction);

    if reduced <= 0.0 { 0 } else { reduced as u32 }
}

/// Check whether a fall from `distance` blocks would be lethal.
///
/// Armor reduces damage by `armor_points * 4%` (applied after Feather Falling).
/// Returns `true` when the final damage exceeds `health`.
pub fn is_lethal_fall(
    distance: f32,
    health: f32,
    armor_points: f32,
    feather_level: u8,
) -> bool {
    let raw_damage = calculate_fall_damage(distance, feather_level) as f32;
    let armor_reduction = (armor_points * ARMOR_REDUCTION_PER_POINT).min(1.0);
    let final_damage = raw_damage * (1.0 - armor_reduction);
    final_damage > health
}

/// Maximum fall distance that results in zero damage.
///
/// Without Feather Falling this is 3.0 blocks. Each enchantment level adds
/// extra safe distance proportional to the 12% reduction.
pub fn safe_fall_distance(feather_level: u8) -> f32 {
    let level = feather_level.min(MAX_FEATHER_FALLING_LEVEL);
    if level == 0 {
        return FREE_FALL_BLOCKS;
    }
    // damage = floor(d - 3) * (1 - 0.12 * level)
    // damage becomes 0 when floor(d - 3) == 0, i.e. d < 4.0
    // With feather falling the damage is reduced but the threshold where
    // floor(d - 3) first becomes 1 is always at d = 4.0. However the 1 HP
    // of base damage is reduced below 1 and then cast to u32 (truncated to 0)
    // when reduction >= 1.0 / 1.0 = 100%. At FF IV (48%) 1 * 0.52 = 0.52
    // which truncates to 0.
    //
    // We need the largest distance d such that calculate_fall_damage(d, level) == 0.
    // floor(d - 3) * (1 - 0.12 * level) < 1  =>  floor(d - 3) < 1 / (1 - 0.12 * level)
    // Let threshold = floor(1 / (1 - 0.12 * level))
    // Then d < threshold + 3 + 1  =>  safe = threshold + 3 + 1 - epsilon
    // But we want the max d that is still safe, so safe_d = threshold + 3.0
    // because floor(d - 3) ranges 0..threshold which all truncate to 0 damage.
    let multiplier = 1.0 - FEATHER_FALLING_REDUCTION_PER_LEVEL * level as f32;
    // max base damage (integer) that still rounds to 0 after reduction
    let max_base = (1.0 / multiplier).ceil() as u32 - 1;
    // floor(d - 3) == max_base  =>  d can be up to max_base + 3 + 0.999..
    // The largest whole-number safe distance:
    FREE_FALL_BLOCKS + max_base as f32
}

/// Maximum fall distance that is survivable given health, armor, and
/// Feather Falling level.
///
/// Returns the largest distance `d` such that `is_lethal_fall(d, ...) == false`.
pub fn max_survivable_fall(health: f32, armor: f32, feather: u8) -> f32 {
    let level = feather.min(MAX_FEATHER_FALLING_LEVEL);
    let ff_mult = 1.0 - FEATHER_FALLING_REDUCTION_PER_LEVEL * level as f32;
    let armor_mult = 1.0 - (armor * ARMOR_REDUCTION_PER_POINT).min(1.0);
    let effective_mult = ff_mult * armor_mult;

    if effective_mult <= 0.0 {
        // Full reduction -- any fall is survivable
        return f32::MAX;
    }

    // final_damage = floor(d - 3) * effective_mult
    // lethal when final_damage > health
    // max safe: floor(d - 3) * effective_mult <= health
    //           floor(d - 3) <= health / effective_mult
    let max_base = (health / effective_mult).floor();
    // floor(d - 3) == max_base  =>  d can be up to max_base + 3 + 0.999..
    FREE_FALL_BLOCKS + max_base
}

/// Reduce fall damage by Jump Boost potion effect.
///
/// Each level of Jump Boost reduces fall damage by 1 (to a minimum of 0).
pub fn apply_jump_boost_reduction(damage: u32, jump_boost_level: u8) -> u32 {
    damage.saturating_sub(jump_boost_level as u32)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- calculate_fall_damage -----------------------------------------------

    #[test]
    fn three_blocks_is_zero_damage() {
        assert_eq!(calculate_fall_damage(3.0, 0), 0);
    }

    #[test]
    fn four_blocks_is_one_damage() {
        assert_eq!(calculate_fall_damage(4.0, 0), 1);
    }

    #[test]
    fn twenty_three_point_five_blocks_is_twenty_damage() {
        assert_eq!(calculate_fall_damage(23.5, 0), 20);
    }

    #[test]
    fn negative_distance_is_zero() {
        assert_eq!(calculate_fall_damage(-5.0, 0), 0);
    }

    #[test]
    fn zero_distance_is_zero() {
        assert_eq!(calculate_fall_damage(0.0, 0), 0);
    }

    #[test]
    fn feather_falling_iv_reduces_by_48_percent() {
        // 10 blocks -> base = floor(10 - 3) = 7
        // FF IV: 7 * (1 - 0.48) = 7 * 0.52 = 3.64 -> truncated to 3
        assert_eq!(calculate_fall_damage(10.0, 4), 3);
    }

    #[test]
    fn feather_falling_i_reduces_by_12_percent() {
        // 10 blocks -> base = 7, 7 * 0.88 = 6.16 -> 6
        assert_eq!(calculate_fall_damage(10.0, 1), 6);
    }

    #[test]
    fn feather_falling_capped_at_level_4() {
        // Level 5+ should behave like level 4
        assert_eq!(
            calculate_fall_damage(10.0, 5),
            calculate_fall_damage(10.0, 4)
        );
    }

    // -- is_lethal_fall ------------------------------------------------------

    #[test]
    fn lethal_fall_at_twenty_hp_no_armor() {
        // 23.5 blocks = 20 damage, health = 20 -> not lethal (20 <= 20)
        assert!(!is_lethal_fall(23.5, 20.0, 0.0, 0));
        // 24 blocks = 21 damage > 20 -> lethal
        assert!(is_lethal_fall(24.0, 20.0, 0.0, 0));
    }

    #[test]
    fn armor_reduces_lethal_threshold() {
        // 24 blocks = 21 base damage, 10 armor points = 40% reduction
        // 21 * 0.6 = 12.6 which is <= 20 -> not lethal
        assert!(!is_lethal_fall(24.0, 20.0, 10.0, 0));
    }

    #[test]
    fn feather_falling_prevents_lethal() {
        // 24 blocks no FF = 21 base, lethal at 20 hp
        assert!(is_lethal_fall(24.0, 20.0, 0.0, 0));
        // With FF IV: 21 * 0.52 = 10.92 -> 10 as u32, 10.0 <= 20.0 -> not lethal
        assert!(!is_lethal_fall(24.0, 20.0, 0.0, 4));
    }

    // -- safe_fall_distance --------------------------------------------------

    #[test]
    fn safe_distance_no_enchant() {
        assert!((safe_fall_distance(0) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn safe_distance_feather_falling_iv() {
        // FF IV: multiplier = 0.52, 1/0.52 = 1.923 -> ceil = 2, minus 1 = 1
        // safe = 3.0 + 1.0 = 4.0
        let safe = safe_fall_distance(4);
        assert!(
            (safe - 4.0).abs() < f32::EPSILON,
            "expected 4.0, got {safe}"
        );
    }

    // -- max_survivable_fall -------------------------------------------------

    #[test]
    fn max_survivable_twenty_hp_no_armor() {
        // max floor(d-3) * 1.0 <= 20  =>  floor(d-3) <= 20  =>  d <= 23
        let max = max_survivable_fall(20.0, 0.0, 0);
        assert!(
            (max - 23.0).abs() < f32::EPSILON,
            "expected 23.0, got {max}"
        );
    }

    #[test]
    fn max_survivable_with_armor() {
        // armor 10 -> mult = 0.6, max floor(d-3) <= 20/0.6 = 33.33 -> 33
        // d <= 33 + 3 = 36
        let max = max_survivable_fall(20.0, 10.0, 0);
        assert!(
            (max - 36.0).abs() < f32::EPSILON,
            "expected 36.0, got {max}"
        );
    }

    #[test]
    fn max_survivable_with_feather_falling() {
        // FF IV mult = 0.52, armor 0 -> effective = 0.52
        // max floor(d-3) <= 20/0.52 = 38.46 -> 38
        // d <= 38 + 3 = 41
        let max = max_survivable_fall(20.0, 0.0, 4);
        assert!(
            (max - 41.0).abs() < f32::EPSILON,
            "expected 41.0, got {max}"
        );
    }

    // -- apply_jump_boost_reduction ------------------------------------------

    #[test]
    fn jump_boost_reduces_damage() {
        assert_eq!(apply_jump_boost_reduction(5, 2), 3);
    }

    #[test]
    fn jump_boost_does_not_go_negative() {
        assert_eq!(apply_jump_boost_reduction(1, 5), 0);
    }

    #[test]
    fn jump_boost_zero_is_noop() {
        assert_eq!(apply_jump_boost_reduction(7, 0), 7);
    }
}
