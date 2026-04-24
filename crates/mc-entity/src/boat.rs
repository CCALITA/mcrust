//! Boat physics: buoyancy, paddle-driven thrust and turning, drag, and
//! passenger mounting for the nine wood-type boat variants.

// ---------------------------------------------------------------------------
// Boat type
// ---------------------------------------------------------------------------

/// The wood species a boat is crafted from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoatType {
    Oak,
    Birch,
    Spruce,
    Jungle,
    DarkOak,
    Acacia,
    Bamboo,
    Cherry,
    Mangrove,
}

// ---------------------------------------------------------------------------
// Boat state
// ---------------------------------------------------------------------------

/// Runtime state for a single boat entity.
#[derive(Debug, Clone)]
pub struct BoatState {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub yaw: f32,
    pub in_water: bool,
    pub passenger: Option<u64>,
    pub boat_type: BoatType,
}

impl BoatState {
    /// Create a new boat at `pos` with zero velocity, no passenger, and the
    /// given wood type.
    pub fn new(pos: [f32; 3], boat_type: BoatType) -> Self {
        Self {
            position: pos,
            velocity: [0.0, 0.0, 0.0],
            yaw: 0.0,
            in_water: false,
            passenger: None,
            boat_type,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Buoyancy impulse added per tick when the boat is in water (m/tick^2).
const BUOYANCY: f32 = 0.04;

/// Gravitational pull per tick when the boat is not in water (m/tick^2).
const GRAVITY: f32 = 0.04;

/// Forward thrust applied when both paddles are active (m/tick).
const PADDLE_THRUST: f32 = 0.04;

/// Yaw change in degrees when only one paddle is active.
const SINGLE_PADDLE_TURN_DEG: f32 = 4.0;

/// Per-tick drag multiplier applied to velocity while on water.
const WATER_DRAG: f32 = 0.9;

/// Maximum horizontal speed in m/tick.
const MAX_SPEED: f32 = 0.4;

// ---------------------------------------------------------------------------
// Tick
// ---------------------------------------------------------------------------

/// Advance a boat by one tick.
///
/// * **Buoyancy** — when `state.in_water` the vertical velocity receives a
///   positive impulse (`+0.04`).
/// * **Gravity** — when *not* in water the vertical velocity is reduced by
///   `0.04 * dt`.
/// * **Paddling** — both paddles active → forward thrust of `0.04` along the
///   yaw direction; only one paddle → the yaw rotates by `4°` per tick (left
///   paddle turns right, right paddle turns left).
/// * **Drag** — on water, velocity is scaled by `0.9` each tick.
/// * **Speed cap** — horizontal speed is clamped to `0.4 m/tick`.
pub fn tick_boat(state: &mut BoatState, dt: f32, paddle_left: bool, paddle_right: bool) {
    // -- Buoyancy / gravity ------------------------------------------------
    if state.in_water {
        state.velocity[1] += BUOYANCY;
    } else {
        state.velocity[1] -= GRAVITY * dt;
    }

    // -- Paddle turning ----------------------------------------------------
    let turn_rad = SINGLE_PADDLE_TURN_DEG.to_radians();
    match (paddle_left, paddle_right) {
        (true, true) => {
            // Both paddles: thrust forward along yaw.
            let (sin_yaw, cos_yaw) = state.yaw.sin_cos();
            state.velocity[0] += -sin_yaw * PADDLE_THRUST;
            state.velocity[2] += cos_yaw * PADDLE_THRUST;
        }
        (true, false) => {
            // Left paddle only → turn right (positive yaw).
            state.yaw += turn_rad;
        }
        (false, true) => {
            // Right paddle only → turn left (negative yaw).
            state.yaw -= turn_rad;
        }
        (false, false) => {}
    }

    // -- Drag on water -----------------------------------------------------
    if state.in_water {
        state.velocity[0] *= WATER_DRAG;
        state.velocity[1] *= WATER_DRAG;
        state.velocity[2] *= WATER_DRAG;
    }

    // -- Max speed cap (horizontal only) -----------------------------------
    let hx = state.velocity[0];
    let hz = state.velocity[2];
    let h_speed = (hx * hx + hz * hz).sqrt();
    if h_speed > MAX_SPEED {
        let scale = MAX_SPEED / h_speed;
        state.velocity[0] *= scale;
        state.velocity[2] *= scale;
    }

    // -- Integrate position ------------------------------------------------
    state.position[0] += state.velocity[0] * dt;
    state.position[1] += state.velocity[1] * dt;
    state.position[2] += state.velocity[2] * dt;
}

// ---------------------------------------------------------------------------
// Collision box
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box for a boat: `[min_x, min_y, min_z, max_x, max_y, max_z]`.
///
/// Width = 1.375, height = 0.5625, depth = 1.375 (centred on the entity position).
pub fn boat_collision_box() -> [f32; 6] {
    let half_w: f32 = 1.375 / 2.0;
    let half_d: f32 = 1.375 / 2.0;
    let h: f32 = 0.5625;
    [-half_w, 0.0, -half_d, half_w, h, half_d]
}

// ---------------------------------------------------------------------------
// Mounting
// ---------------------------------------------------------------------------

/// Attempt to seat `player_id` in the boat.  Returns `false` if the boat
/// already has a passenger.
pub fn mount_passenger(state: &mut BoatState, player_id: u64) -> bool {
    if state.passenger.is_some() {
        return false;
    }
    state.passenger = Some(player_id);
    true
}

/// Remove the current passenger and return their id, or `None` if the boat
/// was already empty.
pub fn dismount(state: &mut BoatState) -> Option<u64> {
    state.passenger.take()
}

// ---------------------------------------------------------------------------
// Speed query
// ---------------------------------------------------------------------------

/// Maximum horizontal speed a boat can travel (m/tick).
pub fn boat_max_speed() -> f32 {
    MAX_SPEED
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Gravity vs buoyancy -----------------------------------------------

    #[test]
    fn gravity_pulls_boat_down_when_not_in_water() {
        let mut state = BoatState::new([0.0, 10.0, 0.0], BoatType::Oak);
        state.in_water = false;

        tick_boat(&mut state, 1.0, false, false);

        assert!(
            state.velocity[1] < 0.0,
            "expected negative vy from gravity, got {}",
            state.velocity[1],
        );
        assert!(
            state.position[1] < 10.0,
            "boat should have fallen, y = {}",
            state.position[1],
        );
    }

    #[test]
    fn buoyancy_pushes_boat_up_when_in_water() {
        let mut state = BoatState::new([0.0, 5.0, 0.0], BoatType::Birch);
        state.in_water = true;

        tick_boat(&mut state, 1.0, false, false);

        // Buoyancy adds +0.04 before drag (*0.9) → net vy = 0.036 > 0
        assert!(
            state.velocity[1] > 0.0,
            "expected positive vy from buoyancy, got {}",
            state.velocity[1],
        );
    }

    // -- Paddle thrust -----------------------------------------------------

    #[test]
    fn both_paddles_produce_forward_thrust() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Spruce);
        state.in_water = true;
        state.yaw = 0.0; // facing +Z

        tick_boat(&mut state, 1.0, true, true);

        assert!(
            state.velocity[2] > 0.0,
            "expected forward (+Z) thrust, got vz = {}",
            state.velocity[2],
        );
    }

    #[test]
    fn single_paddle_does_not_add_thrust() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Jungle);
        state.in_water = true;

        tick_boat(&mut state, 1.0, true, false);

        let hx = state.velocity[0];
        let hz = state.velocity[2];
        let h_speed = (hx * hx + hz * hz).sqrt();
        assert!(
            h_speed < f32::EPSILON,
            "single paddle should not add horizontal thrust, speed = {}",
            h_speed,
        );
    }

    // -- Turning -----------------------------------------------------------

    #[test]
    fn left_paddle_turns_yaw_right() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::DarkOak);
        let initial_yaw = state.yaw;

        tick_boat(&mut state, 1.0, true, false);

        assert!(
            state.yaw > initial_yaw,
            "left paddle should increase yaw (turn right), yaw = {}",
            state.yaw,
        );
    }

    #[test]
    fn right_paddle_turns_yaw_left() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Acacia);
        let initial_yaw = state.yaw;

        tick_boat(&mut state, 1.0, false, true);

        assert!(
            state.yaw < initial_yaw,
            "right paddle should decrease yaw (turn left), yaw = {}",
            state.yaw,
        );
    }

    #[test]
    fn turn_amount_is_four_degrees() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Bamboo);
        let initial_yaw = state.yaw;

        tick_boat(&mut state, 1.0, true, false);

        let expected = SINGLE_PADDLE_TURN_DEG.to_radians();
        let actual = (state.yaw - initial_yaw).abs();
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "turn should be {} rad (~4°), got {}",
            expected,
            actual,
        );
    }

    // -- Mount / dismount --------------------------------------------------

    #[test]
    fn mount_succeeds_when_empty() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Cherry);
        assert!(mount_passenger(&mut state, 1));
        assert_eq!(state.passenger, Some(1));
    }

    #[test]
    fn mount_fails_when_occupied() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Mangrove);
        assert!(mount_passenger(&mut state, 1));
        assert!(!mount_passenger(&mut state, 2));
        assert_eq!(state.passenger, Some(1));
    }

    #[test]
    fn dismount_returns_passenger_and_clears() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Oak);
        mount_passenger(&mut state, 42);
        assert_eq!(dismount(&mut state), Some(42));
        assert_eq!(state.passenger, None);
    }

    #[test]
    fn dismount_returns_none_when_empty() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Oak);
        assert_eq!(dismount(&mut state), None);
    }

    #[test]
    fn mount_dismount_cycle() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Birch);

        // Mount player 10
        assert!(mount_passenger(&mut state, 10));
        assert_eq!(state.passenger, Some(10));

        // Cannot mount player 20 while occupied
        assert!(!mount_passenger(&mut state, 20));

        // Dismount player 10
        assert_eq!(dismount(&mut state), Some(10));

        // Now mount player 20
        assert!(mount_passenger(&mut state, 20));
        assert_eq!(state.passenger, Some(20));
    }

    // -- Max speed cap -----------------------------------------------------

    #[test]
    fn speed_capped_at_max() {
        let mut state = BoatState::new([0.0, 0.0, 0.0], BoatType::Spruce);
        state.in_water = true;
        // Give it an absurdly high velocity
        state.velocity = [10.0, 0.0, 10.0];

        tick_boat(&mut state, 1.0, true, true);

        let hx = state.velocity[0];
        let hz = state.velocity[2];
        let h_speed = (hx * hx + hz * hz).sqrt();
        assert!(
            h_speed <= MAX_SPEED + f32::EPSILON,
            "horizontal speed {} should not exceed max {}",
            h_speed,
            MAX_SPEED,
        );
    }

    #[test]
    fn boat_max_speed_returns_constant() {
        assert!((boat_max_speed() - 0.4).abs() < f32::EPSILON);
    }

    // -- Collision box -----------------------------------------------------

    #[test]
    fn collision_box_dimensions() {
        let bb = boat_collision_box();
        let width = bb[3] - bb[0];
        let height = bb[4] - bb[1];
        let depth = bb[5] - bb[2];

        assert!((width - 1.375).abs() < f32::EPSILON, "width = {}", width);
        assert!((height - 0.5625).abs() < f32::EPSILON, "height = {}", height);
        assert!((depth - 1.375).abs() < f32::EPSILON, "depth = {}", depth);
    }

    // -- Construction ------------------------------------------------------

    #[test]
    fn new_boat_has_zero_velocity_and_no_passenger() {
        let state = BoatState::new([1.0, 2.0, 3.0], BoatType::Oak);
        assert_eq!(state.position, [1.0, 2.0, 3.0]);
        assert_eq!(state.velocity, [0.0, 0.0, 0.0]);
        assert_eq!(state.yaw, 0.0);
        assert!(!state.in_water);
        assert_eq!(state.passenger, None);
        assert_eq!(state.boat_type, BoatType::Oak);
    }

    #[test]
    fn all_boat_types_constructible() {
        for bt in [
            BoatType::Oak,
            BoatType::Birch,
            BoatType::Spruce,
            BoatType::Jungle,
            BoatType::DarkOak,
            BoatType::Acacia,
            BoatType::Bamboo,
            BoatType::Cherry,
            BoatType::Mangrove,
        ] {
            let state = BoatState::new([0.0, 0.0, 0.0], bt);
            assert_eq!(state.boat_type, bt);
        }
    }
}
