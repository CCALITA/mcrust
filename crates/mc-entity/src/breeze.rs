// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default health for a Breeze mob.
const DEFAULT_HEALTH: f32 = 30.0;

/// Maximum range (blocks) at which the breeze can launch a wind charge.
const ATTACK_RANGE: f32 = 16.0;

/// Damage dealt by a single wind charge impact.
const WIND_CHARGE_DAMAGE: f32 = 1.0;

/// Height (blocks) the breeze reaches when jumping.
const JUMP_HEIGHT: f32 = 5.0;

/// Knockback power applied by the breeze's wind burst.
const KNOCKBACK_POWER: f32 = 3.0;

/// Cooldown (seconds) between consecutive wind charge attacks.
const ATTACK_COOLDOWN_SECS: f32 = 3.0;

// ---------------------------------------------------------------------------
// BreezeState
// ---------------------------------------------------------------------------

/// Runtime state for a single Breeze entity.
#[derive(Debug, Clone, PartialEq)]
pub struct BreezeState {
    /// Current world position.
    pub pos: [f32; 3],
    /// Remaining hit-points.
    pub health: f32,
    /// Seconds remaining before the next attack is allowed.
    pub attack_cooldown: f32,
    /// Whether the breeze is currently in a jump.
    pub jumping: bool,
}

impl BreezeState {
    /// Create a new breeze at the given position with default stats.
    pub fn new(pos: [f32; 3]) -> Self {
        Self {
            pos,
            health: DEFAULT_HEALTH,
            attack_cooldown: 0.0,
            jumping: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Attribute helpers
// ---------------------------------------------------------------------------

/// Maximum range at which the breeze can attack (blocks).
pub fn breeze_attack_range() -> f32 {
    ATTACK_RANGE
}

/// Damage dealt by a single wind charge.
pub fn breeze_wind_charge_damage() -> f32 {
    WIND_CHARGE_DAMAGE
}

/// Height the breeze reaches when jumping (blocks).
pub fn breeze_jump_height() -> f32 {
    JUMP_HEIGHT
}

/// Knockback power applied by the breeze's wind burst.
pub fn breeze_knockback_power() -> f32 {
    KNOCKBACK_POWER
}

/// Breezes deflect arrows and are immune to most projectiles.
pub fn breeze_is_immune_to_projectiles() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Tick
// ---------------------------------------------------------------------------

/// Advance the breeze by `dt` seconds.
///
/// If the target is within [`breeze_attack_range`] and the attack cooldown has
/// expired, a wind charge is launched toward `target_pos` and the cooldown
/// resets. The function returns `Some(direction)` when a wind charge is fired,
/// or `None` otherwise.
///
/// The cooldown is always decremented by `dt` each tick.
pub fn tick_breeze(
    state: &mut BreezeState,
    target_pos: [f32; 3],
    dt: f32,
) -> Option<[f32; 3]> {
    // Decrement cooldown.
    state.attack_cooldown = (state.attack_cooldown - dt).max(0.0);

    // Compute vector toward target.
    let dx = target_pos[0] - state.pos[0];
    let dy = target_pos[1] - state.pos[1];
    let dz = target_pos[2] - state.pos[2];
    let dist_sq = dx * dx + dy * dy + dz * dz;
    let range = breeze_attack_range();

    if dist_sq <= range * range && state.attack_cooldown <= 0.0 {
        // Normalise direction.
        let dist = dist_sq.sqrt();
        let direction = if dist > f32::EPSILON {
            [dx / dist, dy / dist, dz / dist]
        } else {
            [0.0, 0.0, 0.0]
        };

        state.attack_cooldown = ATTACK_COOLDOWN_SECS;
        return Some(direction);
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction -------------------------------------------------------

    #[test]
    fn new_breeze_has_correct_defaults() {
        let b = BreezeState::new([1.0, 2.0, 3.0]);

        assert_eq!(b.pos, [1.0, 2.0, 3.0]);
        assert!((b.health - 30.0).abs() < f32::EPSILON);
        assert!((b.attack_cooldown - 0.0).abs() < f32::EPSILON);
        assert!(!b.jumping);
    }

    // -- Attribute helpers --------------------------------------------------

    #[test]
    fn attack_range_is_16() {
        assert!((breeze_attack_range() - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wind_charge_damage_is_1() {
        assert!((breeze_wind_charge_damage() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn jump_height_is_5() {
        assert!((breeze_jump_height() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn knockback_power_is_3() {
        assert!((breeze_knockback_power() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn immune_to_projectiles() {
        assert!(breeze_is_immune_to_projectiles());
    }

    // -- tick_breeze: fires when in range and off cooldown -------------------

    #[test]
    fn fires_wind_charge_when_in_range_and_ready() {
        let mut b = BreezeState::new([0.0, 0.0, 0.0]);
        let target = [10.0, 0.0, 0.0];

        let result = tick_breeze(&mut b, target, 0.05);

        assert!(result.is_some());
        let dir = result.unwrap();
        // Direction should point toward +X.
        assert!(dir[0] > 0.9);
        // Cooldown should have been reset.
        assert!((b.attack_cooldown - 3.0).abs() < f32::EPSILON);
    }

    // -- tick_breeze: does not fire while on cooldown -----------------------

    #[test]
    fn does_not_fire_while_on_cooldown() {
        let mut b = BreezeState::new([0.0, 0.0, 0.0]);
        b.attack_cooldown = 2.0;
        let target = [5.0, 0.0, 0.0];

        let result = tick_breeze(&mut b, target, 0.05);

        assert!(result.is_none());
        // Cooldown should have decreased.
        assert!(b.attack_cooldown < 2.0);
    }

    // -- tick_breeze: does not fire when out of range -----------------------

    #[test]
    fn does_not_fire_when_out_of_range() {
        let mut b = BreezeState::new([0.0, 0.0, 0.0]);
        let target = [100.0, 0.0, 0.0]; // well outside 16-block range

        let result = tick_breeze(&mut b, target, 0.05);

        assert!(result.is_none());
    }

    // -- tick_breeze: cooldown decrements each tick -------------------------

    #[test]
    fn cooldown_decrements_each_tick() {
        let mut b = BreezeState::new([0.0, 0.0, 0.0]);
        b.attack_cooldown = 1.0;
        let target = [100.0, 0.0, 0.0]; // out of range so no attack

        tick_breeze(&mut b, target, 0.25);

        assert!((b.attack_cooldown - 0.75).abs() < f32::EPSILON);
    }

    // -- tick_breeze: cooldown does not go below zero -----------------------

    #[test]
    fn cooldown_does_not_go_negative() {
        let mut b = BreezeState::new([0.0, 0.0, 0.0]);
        b.attack_cooldown = 0.1;
        let target = [100.0, 0.0, 0.0];

        tick_breeze(&mut b, target, 1.0);

        assert!((b.attack_cooldown - 0.0).abs() < f32::EPSILON);
    }

    // -- tick_breeze: direction is normalised --------------------------------

    #[test]
    fn wind_charge_direction_is_normalised() {
        let mut b = BreezeState::new([0.0, 0.0, 0.0]);
        let target = [3.0, 4.0, 0.0]; // distance = 5

        let dir = tick_breeze(&mut b, target, 0.05).unwrap();
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();

        assert!((len - 1.0).abs() < 1e-5);
    }

    // -- tick_breeze: zero-distance target ----------------------------------

    #[test]
    fn zero_distance_target_returns_zero_direction() {
        let mut b = BreezeState::new([5.0, 5.0, 5.0]);
        let target = [5.0, 5.0, 5.0];

        let dir = tick_breeze(&mut b, target, 0.05).unwrap();

        assert!((dir[0]).abs() < f32::EPSILON);
        assert!((dir[1]).abs() < f32::EPSILON);
        assert!((dir[2]).abs() < f32::EPSILON);
    }
}
