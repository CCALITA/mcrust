//! Conduit power effects.
//!
//! A fully-activated conduit (surrounded by at least 16 prismarine blocks in the
//! correct frame, with the player inside its range while in water) grants the
//! player water breathing, night vision and haste. It also attacks nearby
//! hostile mobs.

/// Status effects granted to a player within a conduit's active range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConduitPowerEffect {
    pub water_breathing: bool,
    pub night_vision: bool,
    pub haste: bool,
}

/// Minimum number of prismarine blocks needed to activate a conduit.
pub const fn conduit_minimum_prismarine() -> u8 {
    16
}

/// Maximum attack/effect range possible with a fully built conduit frame.
pub const fn conduit_maximum_range() -> u8 {
    96
}

/// Damage dealt per second to hostile mobs inside the attack range.
pub const fn conduit_attack_damage_per_second() -> f32 {
    4.0
}

/// Apply conduit power effects to a player.
///
/// All three effects are granted when the player is both in water and within
/// the conduit's active range. Otherwise no effect is applied.
pub fn apply_conduit_power(in_water: bool, in_range: bool) -> ConduitPowerEffect {
    if in_water && in_range {
        ConduitPowerEffect {
            water_breathing: true,
            night_vision: true,
            haste: true,
        }
    } else {
        ConduitPowerEffect::default()
    }
}

/// Attack range for hostile mobs in blocks.
///
/// Base range is 8 blocks once the conduit is active (16 prismarine). Each
/// additional prismarine block above the minimum extends the range by 0.5
/// blocks, capped at 16 blocks. Returns 0.0 when the conduit is not active.
pub fn conduit_attack_range(prismarine_count: u8) -> f32 {
    let minimum = conduit_minimum_prismarine();
    if prismarine_count < minimum {
        return 0.0;
    }
    let extra = (prismarine_count - minimum) as f32;
    (8.0 + 0.5 * extra).min(16.0)
}

/// Priority score for choosing which mob the conduit attacks first.
///
/// Higher scores win. Closer mobs are preferred (priority decreases with
/// distance) and hostile mobs are strongly prioritized over non-hostile ones.
pub fn conduit_target_priority(distance: f32, is_hostile: bool) -> f32 {
    let distance = distance.max(0.0);
    let base = 100.0 - distance;
    if is_hostile {
        base + 1000.0
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_vanilla() {
        assert_eq!(conduit_minimum_prismarine(), 16);
        assert_eq!(conduit_maximum_range(), 96);
        assert!((conduit_attack_damage_per_second() - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_grants_all_three_effects_in_water_and_range() {
        let effect = apply_conduit_power(true, true);
        assert!(effect.water_breathing);
        assert!(effect.night_vision);
        assert!(effect.haste);
    }

    #[test]
    fn apply_grants_nothing_when_out_of_water() {
        let effect = apply_conduit_power(false, true);
        assert_eq!(effect, ConduitPowerEffect::default());
    }

    #[test]
    fn apply_grants_nothing_when_out_of_range() {
        let effect = apply_conduit_power(true, false);
        assert_eq!(effect, ConduitPowerEffect::default());
    }

    #[test]
    fn apply_grants_nothing_when_neither_condition_met() {
        let effect = apply_conduit_power(false, false);
        assert_eq!(effect, ConduitPowerEffect::default());
    }

    #[test]
    fn attack_range_zero_below_minimum_prismarine() {
        assert_eq!(conduit_attack_range(0), 0.0);
        assert_eq!(conduit_attack_range(15), 0.0);
    }

    #[test]
    fn attack_range_is_base_at_minimum() {
        assert!((conduit_attack_range(16) - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn attack_range_grows_half_per_extra_prismarine() {
        assert!((conduit_attack_range(17) - 8.5).abs() < f32::EPSILON);
        assert!((conduit_attack_range(18) - 9.0).abs() < f32::EPSILON);
        assert!((conduit_attack_range(20) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn attack_range_caps_at_sixteen() {
        assert!((conduit_attack_range(48) - 16.0).abs() < f32::EPSILON);
        assert!((conduit_attack_range(96) - 16.0).abs() < f32::EPSILON);
        assert!((conduit_attack_range(255) - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hostile_mobs_outrank_non_hostile_at_same_distance() {
        let hostile = conduit_target_priority(5.0, true);
        let passive = conduit_target_priority(5.0, false);
        assert!(hostile > passive);
    }

    #[test]
    fn closer_mobs_have_higher_priority() {
        let near = conduit_target_priority(2.0, true);
        let far = conduit_target_priority(10.0, true);
        assert!(near > far);
    }

    #[test]
    fn close_passive_still_loses_to_distant_hostile() {
        let near_passive = conduit_target_priority(1.0, false);
        let far_hostile = conduit_target_priority(15.0, true);
        assert!(far_hostile > near_passive);
    }

    #[test]
    fn negative_distance_is_clamped() {
        let a = conduit_target_priority(-5.0, true);
        let b = conduit_target_priority(0.0, true);
        assert!((a - b).abs() < f32::EPSILON);
    }
}
