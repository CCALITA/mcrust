use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Ticks without sleep before phantoms start spawning (72 000 = 3 in-game days).
const SPAWN_THRESHOLD_TICKS: u64 = 72_000;

/// Radius (blocks) at which the phantom circles the player.
const CIRCLE_RADIUS: f32 = 16.0;

/// Angular speed while circling (radians per second).
const CIRCLE_SPEED: f32 = 0.8;

/// Height above the player while circling.
const CIRCLE_HEIGHT: f32 = 20.0;

/// Duration of the swoop timer before the phantom starts a new swoop.
const SWOOP_COOLDOWN: f32 = 5.0;

/// Speed multiplier when swooping toward the player.
const SWOOP_SPEED: f32 = 24.0;

/// Distance threshold to consider a swoop as "arrived".
const SWOOP_ARRIVE_DIST: f32 = 1.5;

/// Base damage per hit at size 0.
const BASE_DAMAGE: f32 = 2.0;

/// Extra damage per size increment.
const DAMAGE_PER_SIZE: f32 = 1.0;

// ---------------------------------------------------------------------------
// PhantomAction
// ---------------------------------------------------------------------------

/// Actions produced by [`phantom_tick`] each frame.
#[derive(Debug, Clone, PartialEq)]
pub enum PhantomAction {
    /// Circling around the player at altitude.
    Circle,
    /// Swooping toward the given world position to attack.
    Swoop(Vec3),
    /// Idle — no meaningful state change this tick.
    Idle,
}

// ---------------------------------------------------------------------------
// PhantomState
// ---------------------------------------------------------------------------

/// Runtime state for a single Phantom entity.
#[derive(Debug, Clone, PartialEq)]
pub struct PhantomState {
    /// Size category (0 = small, larger = bigger). Affects damage and hitbox.
    pub size: u8,
    /// Current target position the phantom is heading toward, if any.
    pub target: Option<Vec3>,
    /// Timer that counts down to the next swoop attempt (seconds).
    pub swoop_timer: f32,
    /// Internal angle tracking for the circling motion (radians).
    circle_angle: f32,
    /// Current world position of the phantom.
    pub position: Vec3,
}

impl PhantomState {
    /// Create a new phantom with the given `size` spawning above `player_pos`.
    pub fn new(size: u8, player_pos: Vec3) -> Self {
        Self {
            size,
            target: None,
            swoop_timer: SWOOP_COOLDOWN,
            circle_angle: 0.0,
            position: player_pos + Vec3::new(0.0, CIRCLE_HEIGHT, 0.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Spawn check
// ---------------------------------------------------------------------------

/// Returns `true` when enough ticks have passed without the player sleeping
/// to warrant phantom spawning (72 000 ticks / 3 in-game days).
pub fn should_spawn(ticks_since_sleep: u64) -> bool {
    ticks_since_sleep >= SPAWN_THRESHOLD_TICKS
}

// ---------------------------------------------------------------------------
// Tick
// ---------------------------------------------------------------------------

/// Advance the phantom by `dt` seconds relative to `player_pos`.
///
/// Behaviour:
/// - While `swoop_timer > 0` the phantom **circles** overhead.
/// - When the timer expires, it picks a **swoop** target at the player and
///   dives.
/// - Once it arrives at the target (or has no target), it resets to circling.
pub fn phantom_tick(state: &mut PhantomState, player_pos: Vec3, dt: f32) -> PhantomAction {
    // --- Swooping -----------------------------------------------------------
    if let Some(swoop_target) = state.target {
        let to_target = swoop_target - state.position;
        let dist = to_target.length();

        if dist < SWOOP_ARRIVE_DIST {
            // Arrived — reset to circling.
            state.target = None;
            state.swoop_timer = SWOOP_COOLDOWN;
            return PhantomAction::Idle;
        }

        let direction = to_target.normalize_or_zero();
        state.position += direction * SWOOP_SPEED * dt;
        return PhantomAction::Swoop(swoop_target);
    }

    // --- Circling -----------------------------------------------------------
    state.swoop_timer -= dt;

    if state.swoop_timer <= 0.0 {
        // Begin a new swoop toward the player.
        state.target = Some(player_pos);
        state.swoop_timer = 0.0;
        return PhantomAction::Swoop(player_pos);
    }

    // Update circling position.
    state.circle_angle += CIRCLE_SPEED * dt;
    if state.circle_angle > std::f32::consts::TAU {
        state.circle_angle -= std::f32::consts::TAU;
    }

    let offset = Vec3::new(
        CIRCLE_RADIUS * state.circle_angle.cos(),
        CIRCLE_HEIGHT,
        CIRCLE_RADIUS * state.circle_angle.sin(),
    );
    state.position = player_pos + offset;

    PhantomAction::Circle
}

// ---------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------

/// Calculate the damage a phantom of the given `size` deals on a swoop hit.
///
/// Formula: `BASE_DAMAGE + size * DAMAGE_PER_SIZE` (i.e. 2 + size).
pub fn swoop_damage(size: u8) -> f32 {
    BASE_DAMAGE + f32::from(size) * DAMAGE_PER_SIZE
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- should_spawn -------------------------------------------------------

    #[test]
    fn does_not_spawn_before_threshold() {
        assert!(!should_spawn(0));
        assert!(!should_spawn(71_999));
    }

    #[test]
    fn spawns_at_exactly_threshold() {
        assert!(should_spawn(72_000));
    }

    #[test]
    fn spawns_after_threshold() {
        assert!(should_spawn(100_000));
    }

    // -- swoop_damage -------------------------------------------------------

    #[test]
    fn swoop_damage_scales_with_size() {
        assert!((swoop_damage(0) - 2.0).abs() < f32::EPSILON);
        assert!((swoop_damage(1) - 3.0).abs() < f32::EPSILON);
        assert!((swoop_damage(4) - 6.0).abs() < f32::EPSILON);
    }

    // -- PhantomState construction ------------------------------------------

    #[test]
    fn new_phantom_starts_above_player() {
        let player = Vec3::new(10.0, 64.0, 10.0);
        let p = PhantomState::new(1, player);

        assert_eq!(p.size, 1);
        assert!(p.target.is_none());
        assert!((p.swoop_timer - SWOOP_COOLDOWN).abs() < f32::EPSILON);
        // Should be CIRCLE_HEIGHT above the player.
        assert!((p.position.y - (player.y + CIRCLE_HEIGHT)).abs() < f32::EPSILON);
    }

    // -- phantom_tick: circling ---------------------------------------------

    #[test]
    fn circles_while_swoop_timer_positive() {
        let player = Vec3::ZERO;
        let mut p = PhantomState::new(0, player);

        let action = phantom_tick(&mut p, player, 1.0);
        assert_eq!(action, PhantomAction::Circle);
        assert!(p.target.is_none());
        // Timer should have decreased.
        assert!(p.swoop_timer < SWOOP_COOLDOWN);
    }

    // -- phantom_tick: swoop initiation ------------------------------------

    #[test]
    fn initiates_swoop_when_timer_expires() {
        let player = Vec3::new(0.0, 64.0, 0.0);
        let mut p = PhantomState::new(1, player);
        // Expire the timer in one big step.
        p.swoop_timer = 0.5;

        let action = phantom_tick(&mut p, player, 1.0);
        assert!(matches!(action, PhantomAction::Swoop(_)));
        assert!(p.target.is_some());
    }

    // -- phantom_tick: swoop arrival ---------------------------------------

    #[test]
    fn swoop_arrives_and_resets_to_idle() {
        let player = Vec3::new(0.0, 64.0, 0.0);
        let mut p = PhantomState::new(0, player);
        // Place phantom almost at the target.
        p.target = Some(player);
        p.position = player + Vec3::new(0.5, 0.0, 0.0);

        let action = phantom_tick(&mut p, player, 0.1);
        assert_eq!(action, PhantomAction::Idle);
        assert!(p.target.is_none());
        assert!((p.swoop_timer - SWOOP_COOLDOWN).abs() < f32::EPSILON);
    }

    // -- phantom_tick: swoop movement --------------------------------------

    #[test]
    fn swoop_moves_toward_target() {
        let player = Vec3::new(100.0, 64.0, 0.0);
        let mut p = PhantomState::new(0, Vec3::ZERO);
        p.target = Some(player);
        p.position = Vec3::ZERO;

        let before = p.position;
        let action = phantom_tick(&mut p, player, 0.5);

        assert!(matches!(action, PhantomAction::Swoop(_)));
        // Position should have moved closer to the player (+X direction).
        assert!(p.position.x > before.x);
    }
}
