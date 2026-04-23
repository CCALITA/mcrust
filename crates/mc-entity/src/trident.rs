/// Trident weapon mechanics including throwing, loyalty return, riptide boost,
/// channeling lightning, and impaling bonus.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GRAVITY: f32 = -9.81;
const BASE_TRIDENT_DAMAGE: f32 = 8.0;
const IMPALING_DAMAGE_PER_LEVEL: f32 = 2.5;
const LOYALTY_SPEED_PER_LEVEL: f32 = 3.0;
const RIPTIDE_SPEED_PER_LEVEL: f32 = 3.0;
const RETURN_ARRIVAL_THRESHOLD: f32 = 1.0;

// ---------------------------------------------------------------------------
// TridentState
// ---------------------------------------------------------------------------

/// Runtime state of a trident entity in the world.
#[derive(Debug, Clone, PartialEq)]
pub struct TridentState {
    pub thrown: bool,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub loyalty_level: u8,
    pub returning: bool,
}

impl TridentState {
    /// Create a new trident in its default (not thrown) state.
    pub fn new() -> Self {
        Self {
            thrown: false,
            position: [0.0; 3],
            velocity: [0.0; 3],
            loyalty_level: 0,
            returning: false,
        }
    }
}

impl Default for TridentState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Throwing
// ---------------------------------------------------------------------------

/// Launch a trident from the given position with the given velocity.
pub fn throw_trident(state: &mut TridentState, pos: [f32; 3], velocity: [f32; 3]) {
    state.thrown = true;
    state.position = pos;
    state.velocity = velocity;
    state.returning = false;
}

// ---------------------------------------------------------------------------
// Tick / physics
// ---------------------------------------------------------------------------

/// Advance the trident simulation by `dt` seconds.
///
/// Applies gravity and movement while in flight. If the trident has a loyalty
/// enchantment and its velocity has reached zero (stuck), it starts returning
/// toward the owner.
pub fn tick_trident(state: &mut TridentState, owner_pos: [f32; 3], dt: f32) {
    if !state.thrown {
        return;
    }

    if state.returning {
        let ret_vel = return_velocity(state.position, owner_pos, state.loyalty_level);
        state.velocity = ret_vel;

        state.position[0] += state.velocity[0] * dt;
        state.position[1] += state.velocity[1] * dt;
        state.position[2] += state.velocity[2] * dt;

        // If close enough to owner, consider it arrived.
        let dx = owner_pos[0] - state.position[0];
        let dy = owner_pos[1] - state.position[1];
        let dz = owner_pos[2] - state.position[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq < RETURN_ARRIVAL_THRESHOLD * RETURN_ARRIVAL_THRESHOLD {
            state.thrown = false;
            state.returning = false;
        }
        return;
    }

    // Apply gravity to the Y component.
    state.velocity[1] += GRAVITY * dt;

    // Move.
    state.position[0] += state.velocity[0] * dt;
    state.position[1] += state.velocity[1] * dt;
    state.position[2] += state.velocity[2] * dt;

    // Check if the trident is stuck (velocity near zero after hitting something).
    let speed_sq = state.velocity[0] * state.velocity[0]
        + state.velocity[1] * state.velocity[1]
        + state.velocity[2] * state.velocity[2];

    if speed_sq < 0.01 && state.loyalty_level > 0 {
        state.returning = true;
    }
}

// ---------------------------------------------------------------------------
// Loyalty return
// ---------------------------------------------------------------------------

/// Compute the velocity vector that sends the trident back toward its owner.
///
/// Speed is `loyalty_level * 3`. Returns a zero vector if the trident is
/// already at the owner's position.
pub fn return_velocity(
    trident_pos: [f32; 3],
    owner_pos: [f32; 3],
    loyalty_level: u8,
) -> [f32; 3] {
    let dx = owner_pos[0] - trident_pos[0];
    let dy = owner_pos[1] - trident_pos[1];
    let dz = owner_pos[2] - trident_pos[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();

    if dist < f32::EPSILON {
        return [0.0; 3];
    }

    let speed = loyalty_level as f32 * LOYALTY_SPEED_PER_LEVEL;
    [dx / dist * speed, dy / dist * speed, dz / dist * speed]
}

// ---------------------------------------------------------------------------
// Riptide
// ---------------------------------------------------------------------------

/// Returns the riptide boost speed if conditions are met.
///
/// Riptide only works when the player is in water or it is raining.
/// Speed is `3 * riptide_level`.
pub fn riptide_boost(riptide_level: u8, in_water: bool, is_raining: bool) -> Option<f32> {
    if !in_water && !is_raining {
        return None;
    }
    Some(RIPTIDE_SPEED_PER_LEVEL * riptide_level as f32)
}

// ---------------------------------------------------------------------------
// Channeling
// ---------------------------------------------------------------------------

/// Returns `true` if a channeling strike should summon lightning.
///
/// Both conditions must be met: the world is thundering **and** the trident
/// hit a mob.
pub fn channeling_strikes(is_thundering: bool, hit_mob: bool) -> bool {
    is_thundering && hit_mob
}

// ---------------------------------------------------------------------------
// Impaling
// ---------------------------------------------------------------------------

/// Bonus damage from the Impaling enchantment.
///
/// Deals `2.5 * level` extra damage to targets that are in water (or are
/// aquatic mobs). Returns 0 if the target is not in water.
pub fn impaling_bonus(level: u8, target_in_water: bool) -> f32 {
    if target_in_water {
        IMPALING_DAMAGE_PER_LEVEL * level as f32
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Base damage
// ---------------------------------------------------------------------------

/// Base trident damage: 8.0 for both melee and thrown.
pub fn trident_damage() -> f32 {
    BASE_TRIDENT_DAMAGE
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- TridentState::new ----------------------------------------------------

    #[test]
    fn new_trident_is_not_thrown() {
        let state = TridentState::new();
        assert!(!state.thrown);
        assert!(!state.returning);
        assert_eq!(state.loyalty_level, 0);
    }

    // -- throw / return cycle -------------------------------------------------

    #[test]
    fn throw_sets_state_correctly() {
        let mut state = TridentState::new();
        throw_trident(&mut state, [1.0, 2.0, 3.0], [10.0, 5.0, 0.0]);

        assert!(state.thrown);
        assert_eq!(state.position, [1.0, 2.0, 3.0]);
        assert_eq!(state.velocity, [10.0, 5.0, 0.0]);
        assert!(!state.returning);
    }

    #[test]
    fn tick_applies_gravity_and_movement() {
        let mut state = TridentState::new();
        throw_trident(&mut state, [0.0, 100.0, 0.0], [10.0, 0.0, 0.0]);

        let owner = [0.0, 64.0, 0.0];
        tick_trident(&mut state, owner, 1.0);

        // X should advance by velocity * dt.
        assert!((state.position[0] - 10.0).abs() < 1e-3);
        // Y should drop due to gravity.
        assert!(state.position[1] < 100.0);
    }

    #[test]
    fn loyalty_trident_returns_when_stuck() {
        let mut state = TridentState::new();
        state.loyalty_level = 2;
        throw_trident(&mut state, [50.0, 64.0, 0.0], [20.0, 0.0, 0.0]);

        let owner = [0.0, 64.0, 0.0];

        // Simulate the trident hitting a block: zero out velocity.
        state.velocity = [0.0, 0.0, 0.0];
        tick_trident(&mut state, owner, 0.001);
        assert!(state.returning, "trident should start returning when stuck with loyalty");

        // Continue ticking until it arrives back.
        for _ in 0..500 {
            tick_trident(&mut state, owner, 0.1);
            if !state.thrown {
                break;
            }
        }
        assert!(!state.thrown, "trident should have returned to owner");
    }

    // -- return_velocity ------------------------------------------------------

    #[test]
    fn return_velocity_points_toward_owner() {
        let vel = return_velocity([10.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1);
        assert!(vel[0] < 0.0);
        let speed = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
        assert!((speed - 3.0).abs() < 1e-3);
    }

    #[test]
    fn return_velocity_scales_with_loyalty() {
        let vel2 = return_velocity([10.0, 0.0, 0.0], [0.0, 0.0, 0.0], 2);
        let vel3 = return_velocity([10.0, 0.0, 0.0], [0.0, 0.0, 0.0], 3);

        let speed2 = (vel2[0] * vel2[0] + vel2[1] * vel2[1] + vel2[2] * vel2[2]).sqrt();
        let speed3 = (vel3[0] * vel3[0] + vel3[1] * vel3[1] + vel3[2] * vel3[2]).sqrt();

        assert!((speed2 - 6.0).abs() < 1e-3);
        assert!((speed3 - 9.0).abs() < 1e-3);
    }

    #[test]
    fn return_velocity_zero_distance_returns_zero() {
        let vel = return_velocity([5.0, 5.0, 5.0], [5.0, 5.0, 5.0], 3);
        assert_eq!(vel, [0.0, 0.0, 0.0]);
    }

    // -- riptide_boost --------------------------------------------------------

    #[test]
    fn riptide_boost_in_water() {
        let result = riptide_boost(2, true, false);
        assert_eq!(result, Some(6.0));
    }

    #[test]
    fn riptide_boost_in_rain() {
        let result = riptide_boost(3, false, true);
        assert_eq!(result, Some(9.0));
    }

    #[test]
    fn riptide_boost_on_dry_land_returns_none() {
        let result = riptide_boost(3, false, false);
        assert_eq!(result, None);
    }

    #[test]
    fn riptide_boost_in_water_and_rain() {
        let result = riptide_boost(1, true, true);
        assert_eq!(result, Some(3.0));
    }

    // -- channeling_strikes ---------------------------------------------------

    #[test]
    fn channeling_strikes_during_thunder_and_hit() {
        assert!(channeling_strikes(true, true));
    }

    #[test]
    fn channeling_no_strike_without_thunder() {
        assert!(!channeling_strikes(false, true));
    }

    #[test]
    fn channeling_no_strike_without_hit() {
        assert!(!channeling_strikes(true, false));
    }

    #[test]
    fn channeling_no_strike_neither_condition() {
        assert!(!channeling_strikes(false, false));
    }

    // -- impaling_bonus -------------------------------------------------------

    #[test]
    fn impaling_bonus_in_water() {
        assert!((impaling_bonus(3, true) - 7.5).abs() < f32::EPSILON);
    }

    #[test]
    fn impaling_bonus_not_in_water() {
        assert!((impaling_bonus(5, false)).abs() < f32::EPSILON);
    }

    #[test]
    fn impaling_bonus_level_zero() {
        assert!((impaling_bonus(0, true)).abs() < f32::EPSILON);
    }

    // -- trident_damage -------------------------------------------------------

    #[test]
    fn base_damage_is_eight() {
        assert!((trident_damage() - 8.0).abs() < f32::EPSILON);
    }
}
