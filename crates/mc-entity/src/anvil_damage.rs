//! Falling anvil damage mechanics.
//!
//! Anvils that fall onto entities deal crushing damage based on distance fallen,
//! and accumulate wear that progresses through visual damage states.

/// Gravity acceleration for falling anvils, in blocks per second squared.
/// Negative because it acts downward.
pub const ANVIL_GRAVITY: f32 = -32.0;

/// Damage per block fallen above the first block.
pub const DAMAGE_PER_BLOCK: f32 = 2.0;

/// Maximum damage a falling anvil can deal, in health points.
pub const MAX_DAMAGE: f32 = 40.0;

/// Chance (as a fraction) that the anvil degrades per block fallen.
pub const DAMAGE_CHANCE_PER_BLOCK: f32 = 0.05;

/// Horizontal crush radius — anvils fall into a 1x2x1 area.
pub const CRUSH_RADIUS: f32 = 1.0;

/// Visual / structural state of an anvil.
///
/// Progresses Pristine -> Chipped -> Damaged -> Broken as it absorbs falls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnvilState {
    Pristine,
    Chipped,
    Damaged,
    Broken,
}

/// Damage (in HP) dealt by a falling anvil that has fallen `fall_distance` blocks.
///
/// The first block counts as free fall with no damage. Each block beyond the first
/// adds [`DAMAGE_PER_BLOCK`] HP, capped at [`MAX_DAMAGE`]. Negative distances are
/// treated as zero.
pub fn falling_anvil_damage(fall_distance: f32) -> f32 {
    if !fall_distance.is_finite() || fall_distance <= 1.0 {
        return 0.0;
    }
    let raw = (fall_distance - 1.0) * DAMAGE_PER_BLOCK;
    raw.min(MAX_DAMAGE)
}

/// Probability that an anvil degrades one state after falling `fall_distance` blocks.
///
/// Returns a fraction in `[0.0, 1.0]`. Negative or non-finite inputs are clamped to 0.
pub fn anvil_damage_chance(fall_distance: f32) -> f32 {
    if !fall_distance.is_finite() || fall_distance <= 0.0 {
        return 0.0;
    }
    (fall_distance * DAMAGE_CHANCE_PER_BLOCK).min(1.0)
}

/// Advance the anvil to the next damage state. [`AnvilState::Broken`] is terminal.
pub fn damage_anvil(state: AnvilState) -> AnvilState {
    match state {
        AnvilState::Pristine => AnvilState::Chipped,
        AnvilState::Chipped => AnvilState::Damaged,
        AnvilState::Damaged => AnvilState::Broken,
        AnvilState::Broken => AnvilState::Broken,
    }
}

/// Downward velocity (blocks / second) of a falling anvil after `fall_time` seconds.
///
/// Uses constant [`ANVIL_GRAVITY`] acceleration starting from rest. Negative times
/// are treated as zero.
pub fn falling_anvil_velocity(fall_time: f32) -> f32 {
    if !fall_time.is_finite() || fall_time <= 0.0 {
        return 0.0;
    }
    ANVIL_GRAVITY * fall_time
}

/// Horizontal crush radius used to determine which entities the anvil lands on.
pub const fn crush_player_radius() -> f32 {
    CRUSH_RADIUS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_damage_at_or_below_one_block() {
        assert_eq!(falling_anvil_damage(0.0), 0.0);
        assert_eq!(falling_anvil_damage(1.0), 0.0);
        assert_eq!(falling_anvil_damage(-5.0), 0.0);
    }

    #[test]
    fn damage_scales_linearly_above_one_block() {
        assert_eq!(falling_anvil_damage(2.0), 2.0);
        assert_eq!(falling_anvil_damage(5.0), 8.0);
        assert_eq!(falling_anvil_damage(11.0), 20.0);
    }

    #[test]
    fn damage_caps_at_forty() {
        assert_eq!(falling_anvil_damage(21.0), MAX_DAMAGE);
        assert_eq!(falling_anvil_damage(100.0), MAX_DAMAGE);
        assert_eq!(falling_anvil_damage(f32::MAX), MAX_DAMAGE);
    }

    #[test]
    fn damage_chance_is_five_percent_per_block() {
        assert!((anvil_damage_chance(1.0) - 0.05).abs() < 1e-6);
        assert!((anvil_damage_chance(10.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn damage_chance_clamped_to_unit_interval() {
        assert_eq!(anvil_damage_chance(0.0), 0.0);
        assert_eq!(anvil_damage_chance(-1.0), 0.0);
        assert_eq!(anvil_damage_chance(20.0), 1.0);
        assert_eq!(anvil_damage_chance(100.0), 1.0);
    }

    #[test]
    fn anvil_state_progression() {
        assert_eq!(damage_anvil(AnvilState::Pristine), AnvilState::Chipped);
        assert_eq!(damage_anvil(AnvilState::Chipped), AnvilState::Damaged);
        assert_eq!(damage_anvil(AnvilState::Damaged), AnvilState::Broken);
    }

    #[test]
    fn broken_anvil_stays_broken() {
        assert_eq!(damage_anvil(AnvilState::Broken), AnvilState::Broken);
    }

    #[test]
    fn velocity_uses_gravity_acceleration() {
        assert_eq!(falling_anvil_velocity(0.0), 0.0);
        assert_eq!(falling_anvil_velocity(1.0), -32.0);
        assert_eq!(falling_anvil_velocity(2.5), -80.0);
    }

    #[test]
    fn velocity_negative_time_is_zero() {
        assert_eq!(falling_anvil_velocity(-1.0), 0.0);
    }

    #[test]
    fn crush_radius_is_one_block() {
        assert_eq!(crush_player_radius(), 1.0);
    }
}
