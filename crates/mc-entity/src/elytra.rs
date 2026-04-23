//! Elytra flight mechanics — state tracking, tick simulation, damage, and boosts.

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Full elytra flight state including velocity, fall distance, and durability.
///
/// Unlike the simpler `equipment::ElytraState` (which tracks only gliding and
/// durability), this struct owns the velocity vector and fall-distance counter
/// so the flight simulation is fully self-contained.
#[derive(Debug, Clone, PartialEq)]
pub struct ElytraState {
    /// Whether the player is currently flying with the elytra.
    pub flying: bool,
    /// Current velocity in m/s as `[x, y, z]`.
    pub velocity: [f32; 3],
    /// Accumulated fall distance in blocks (used for landing-damage calculation).
    pub fall_distance: f32,
    /// Remaining durability points.
    pub durability: u16,
    /// Maximum durability (vanilla default: 432).
    pub max_durability: u16,
}

impl ElytraState {
    /// Create a new elytra at full durability (432) in the non-flying state.
    pub fn new() -> Self {
        Self {
            flying: false,
            velocity: [0.0; 3],
            fall_distance: 0.0,
            durability: 432,
            max_durability: 432,
        }
    }

    /// Attempt to start flying. The elytra must not already be in flight.
    ///
    /// Returns `true` if flight began successfully.
    pub fn start_flight(&mut self) -> bool {
        if self.flying {
            return false;
        }
        self.flying = true;
        true
    }

    /// Stop flying and reset velocity to zero.
    pub fn stop_flight(&mut self) {
        self.flying = false;
        self.velocity = [0.0; 3];
    }
}

impl Default for ElytraState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Flight tick
// ---------------------------------------------------------------------------

/// Advance the elytra flight simulation by one tick.
///
/// While flying, applies:
/// 1. **Gravity** — `−0.05` added to the Y velocity each tick.
/// 2. **Drag** — all velocity components multiplied by `0.99`.
/// 3. **Directional thrust** — derived from the player's look direction:
///    - Pitching down (negative pitch) accelerates forward and downward.
///    - Pitching up (positive pitch) decelerates but gains height.
///
/// `pitch` and `yaw` are in radians. `dt` is the time-step in seconds.
pub fn tick_flight(state: &mut ElytraState, pitch: f32, yaw: f32, dt: f32) {
    if !state.flying {
        return;
    }

    let gravity: f32 = -0.05;
    let drag: f32 = 0.99;

    // Apply gravity.
    state.velocity[1] += gravity * dt;

    // Apply drag to all axes.
    state.velocity[0] *= drag;
    state.velocity[1] *= drag;
    state.velocity[2] *= drag;

    // Directional thrust from look direction.
    // cos(pitch) gives the horizontal magnitude; sin(pitch) gives the vertical.
    let cos_pitch = pitch.cos();
    let sin_pitch = pitch.sin();
    let cos_yaw = yaw.cos();
    let sin_yaw = yaw.sin();

    // Thrust magnitude scales with pitch angle.
    let thrust = if pitch < 0.0 {
        // Looking down — speed up (stronger thrust).
        0.04 * pitch.abs() * dt
    } else {
        // Looking up — trade speed for height (weaker thrust, upward).
        0.02 * pitch * dt
    };

    // Horizontal thrust along the look direction.
    state.velocity[0] += thrust * cos_pitch * (-sin_yaw) * dt;
    state.velocity[2] += thrust * cos_pitch * cos_yaw * dt;

    // Vertical component: pitching down pushes velocity downward,
    // pitching up pushes velocity upward.
    if pitch < 0.0 {
        state.velocity[1] -= thrust * sin_pitch.abs() * dt;
    } else {
        state.velocity[1] += thrust * sin_pitch * dt;
    }

    // Accumulate fall distance when descending.
    if state.velocity[1] < 0.0 {
        state.fall_distance += (-state.velocity[1]) * dt;
    }
}

// ---------------------------------------------------------------------------
// Landing damage
// ---------------------------------------------------------------------------

/// Calculate landing damage from the horizontal component of velocity.
///
/// Damage is only dealt when horizontal speed exceeds `0.5` m/s. The damage
/// scales linearly with the excess speed: `(speed − 0.5) * 10.0`.
pub fn landing_damage(velocity: [f32; 3]) -> f32 {
    let horizontal_speed = (velocity[0] * velocity[0] + velocity[2] * velocity[2]).sqrt();
    if horizontal_speed > 0.5 {
        (horizontal_speed - 0.5) * 10.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Firework boost
// ---------------------------------------------------------------------------

/// Apply a firework rocket boost, adding `1.5` m/s in the look direction.
///
/// The look direction is derived from the current velocity heading. If the
/// velocity is near-zero a small forward impulse along +Z is used instead.
pub fn firework_boost(state: &mut ElytraState) {
    let speed = elytra_speed(state);
    if speed > 0.001 {
        let inv = 1.5 / speed;
        state.velocity[0] += state.velocity[0] * inv;
        state.velocity[1] += state.velocity[1] * inv;
        state.velocity[2] += state.velocity[2] * inv;
    } else {
        // No meaningful velocity — apply boost along +Z.
        state.velocity[2] += 1.5;
    }
}

// ---------------------------------------------------------------------------
// Speed
// ---------------------------------------------------------------------------

/// Return the magnitude of the elytra's velocity vector.
pub fn elytra_speed(state: &ElytraState) -> f32 {
    let [x, y, z] = state.velocity;
    (x * x + y * y + z * z).sqrt()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_elytra_has_correct_defaults() {
        let e = ElytraState::new();
        assert!(!e.flying);
        assert_eq!(e.velocity, [0.0; 3]);
        assert_eq!(e.fall_distance, 0.0);
        assert_eq!(e.durability, 432);
        assert_eq!(e.max_durability, 432);
    }

    #[test]
    fn start_flight_succeeds_when_not_flying() {
        let mut e = ElytraState::new();
        assert!(e.start_flight());
        assert!(e.flying);
    }

    #[test]
    fn start_flight_fails_when_already_flying() {
        let mut e = ElytraState::new();
        e.start_flight();
        assert!(!e.start_flight());
    }

    #[test]
    fn stop_flight_resets_state() {
        let mut e = ElytraState::new();
        e.start_flight();
        e.velocity = [1.0, 2.0, 3.0];
        e.stop_flight();
        assert!(!e.flying);
        assert_eq!(e.velocity, [0.0; 3]);
    }

    #[test]
    fn gravity_applies_during_flight() {
        let mut e = ElytraState::new();
        e.start_flight();
        e.velocity = [0.0, 0.0, 0.0];

        tick_flight(&mut e, 0.0, 0.0, 1.0);

        // Y velocity should be negative after gravity.
        assert!(e.velocity[1] < 0.0, "gravity should pull Y downward");
    }

    #[test]
    fn tick_flight_noop_when_not_flying() {
        let mut e = ElytraState::new();
        e.velocity = [1.0, 2.0, 3.0];
        tick_flight(&mut e, 0.0, 0.0, 1.0);
        // Velocity unchanged since not flying.
        assert_eq!(e.velocity, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn firework_boost_increases_speed() {
        let mut e = ElytraState::new();
        e.start_flight();
        e.velocity = [0.0, 0.0, 1.0];

        let speed_before = elytra_speed(&e);
        firework_boost(&mut e);
        let speed_after = elytra_speed(&e);

        assert!(
            speed_after > speed_before,
            "firework boost should increase speed: {speed_before} -> {speed_after}"
        );
    }

    #[test]
    fn firework_boost_from_zero_velocity() {
        let mut e = ElytraState::new();
        e.start_flight();
        // velocity is [0,0,0]
        firework_boost(&mut e);
        assert!(
            elytra_speed(&e) > 1.0,
            "boost from standstill should produce meaningful speed"
        );
    }

    #[test]
    fn landing_damage_zero_when_slow() {
        let dmg = landing_damage([0.3, -5.0, 0.2]);
        assert_eq!(dmg, 0.0, "slow horizontal speed should deal no damage");
    }

    #[test]
    fn landing_damage_nonzero_when_fast() {
        let dmg = landing_damage([1.0, -2.0, 0.0]);
        assert!(dmg > 0.0, "fast horizontal speed should deal damage");
        // horizontal speed = 1.0, damage = (1.0 - 0.5) * 10.0 = 5.0
        let expected = (1.0_f32 - 0.5) * 10.0;
        assert!((dmg - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_calculation() {
        let e = ElytraState {
            flying: true,
            velocity: [3.0, 4.0, 0.0],
            fall_distance: 0.0,
            durability: 432,
            max_durability: 432,
        };
        let s = elytra_speed(&e);
        assert!((s - 5.0).abs() < f32::EPSILON, "3-4-5 triangle: speed should be 5.0");
    }

    #[test]
    fn speed_zero_when_stationary() {
        let e = ElytraState::new();
        assert_eq!(elytra_speed(&e), 0.0);
    }

    #[test]
    fn fall_distance_accumulates_when_descending() {
        let mut e = ElytraState::new();
        e.start_flight();
        e.velocity = [0.0, -1.0, 0.0];

        tick_flight(&mut e, 0.0, 0.0, 1.0);

        assert!(
            e.fall_distance > 0.0,
            "fall distance should accumulate when velocity.y < 0"
        );
    }
}
