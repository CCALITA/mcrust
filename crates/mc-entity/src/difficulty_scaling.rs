use crate::difficulty::Difficulty;

/// Returns the damage multiplier applied to mob attacks at the given difficulty.
///
/// Peaceful: 0.0, Easy: 0.5, Normal: 1.0, Hard: 1.5
pub fn mob_damage_multiplier(difficulty: Difficulty) -> f32 {
    match difficulty {
        Difficulty::Peaceful => 0.0,
        Difficulty::Easy => 0.5,
        Difficulty::Normal => 1.0,
        Difficulty::Hard => 1.5,
    }
}

/// Returns the mob spawn rate multiplier at the given difficulty.
///
/// Peaceful: 0.0, Easy: 1.0, Normal: 1.0, Hard: 1.5
pub fn mob_spawn_rate_multiplier(difficulty: Difficulty) -> f32 {
    match difficulty {
        Difficulty::Peaceful => 0.0,
        Difficulty::Easy => 1.0,
        Difficulty::Normal => 1.0,
        Difficulty::Hard => 1.5,
    }
}

/// Returns the rate at which hunger drains at the given difficulty.
///
/// Peaceful: 0.0, Easy: 0.5, Normal: 1.0, Hard: 1.5
pub fn hunger_drain_rate(difficulty: Difficulty) -> f32 {
    match difficulty {
        Difficulty::Peaceful => 0.0,
        Difficulty::Easy => 0.5,
        Difficulty::Normal => 1.0,
        Difficulty::Hard => 1.5,
    }
}

/// Returns whether natural health regeneration is enabled at the given difficulty.
///
/// All difficulties: true
pub fn natural_regen_enabled(difficulty: Difficulty) -> bool {
    match difficulty {
        Difficulty::Peaceful => true,
        Difficulty::Easy => true,
        Difficulty::Normal => true,
        Difficulty::Hard => true,
    }
}

/// Returns whether starvation can kill the player at the given difficulty.
///
/// Only Hard: true
pub fn starvation_death(difficulty: Difficulty) -> bool {
    matches!(difficulty, Difficulty::Hard)
}

/// Returns whether friendly fire (PvP damage) is enabled at the given difficulty.
///
/// Peaceful: false, all others: true
pub fn friendly_fire(difficulty: Difficulty) -> bool {
    !matches!(difficulty, Difficulty::Peaceful)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- mob_damage_multiplier ------------------------------------------------

    #[test]
    fn peaceful_mob_damage_is_zero() {
        assert!((mob_damage_multiplier(Difficulty::Peaceful) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_mob_damage_is_half() {
        assert!((mob_damage_multiplier(Difficulty::Easy) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_mob_damage_is_one() {
        assert!((mob_damage_multiplier(Difficulty::Normal) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hard_mob_damage_is_one_point_five() {
        assert!((mob_damage_multiplier(Difficulty::Hard) - 1.5).abs() < f32::EPSILON);
    }

    // -- mob_spawn_rate_multiplier --------------------------------------------

    #[test]
    fn peaceful_spawn_rate_is_zero() {
        assert!((mob_spawn_rate_multiplier(Difficulty::Peaceful) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_spawn_rate_is_one() {
        assert!((mob_spawn_rate_multiplier(Difficulty::Easy) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_spawn_rate_is_one() {
        assert!((mob_spawn_rate_multiplier(Difficulty::Normal) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hard_spawn_rate_is_one_point_five() {
        assert!((mob_spawn_rate_multiplier(Difficulty::Hard) - 1.5).abs() < f32::EPSILON);
    }

    // -- hunger_drain_rate ----------------------------------------------------

    #[test]
    fn peaceful_hunger_drain_is_zero() {
        assert!((hunger_drain_rate(Difficulty::Peaceful) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_hunger_drain_is_half() {
        assert!((hunger_drain_rate(Difficulty::Easy) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_hunger_drain_is_one() {
        assert!((hunger_drain_rate(Difficulty::Normal) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hard_hunger_drain_is_one_point_five() {
        assert!((hunger_drain_rate(Difficulty::Hard) - 1.5).abs() < f32::EPSILON);
    }

    // -- natural_regen_enabled ------------------------------------------------

    #[test]
    fn natural_regen_enabled_for_all_difficulties() {
        assert!(natural_regen_enabled(Difficulty::Peaceful));
        assert!(natural_regen_enabled(Difficulty::Easy));
        assert!(natural_regen_enabled(Difficulty::Normal));
        assert!(natural_regen_enabled(Difficulty::Hard));
    }

    // -- starvation_death -----------------------------------------------------

    #[test]
    fn starvation_death_only_on_hard() {
        assert!(!starvation_death(Difficulty::Peaceful));
        assert!(!starvation_death(Difficulty::Easy));
        assert!(!starvation_death(Difficulty::Normal));
        assert!(starvation_death(Difficulty::Hard));
    }

    // -- friendly_fire --------------------------------------------------------

    #[test]
    fn friendly_fire_disabled_on_peaceful() {
        assert!(!friendly_fire(Difficulty::Peaceful));
    }

    #[test]
    fn friendly_fire_enabled_on_non_peaceful() {
        assert!(friendly_fire(Difficulty::Easy));
        assert!(friendly_fire(Difficulty::Normal));
        assert!(friendly_fire(Difficulty::Hard));
    }
}
