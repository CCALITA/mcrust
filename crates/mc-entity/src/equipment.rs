use glam::Vec3;

// ---------------------------------------------------------------------------
// Shield state
// ---------------------------------------------------------------------------

/// Tracks the blocking state and cooldown of a shield.
///
/// When blocking, incoming damage is fully absorbed. After the shield is
/// disabled (e.g. by an axe attack), a cooldown timer prevents re-blocking
/// until it expires.
#[derive(Debug, Clone, PartialEq)]
pub struct ShieldState {
    pub blocking: bool,
    pub cooldown: f32,
}

impl ShieldState {
    /// Create a new shield in the idle (non-blocking, no cooldown) state.
    pub fn new() -> Self {
        Self {
            blocking: false,
            cooldown: 0.0,
        }
    }

    /// Begin blocking. Only succeeds if the shield is not on cooldown.
    pub fn start_block(&mut self) {
        if self.cooldown <= 0.0 {
            self.blocking = true;
        }
    }

    /// Stop blocking.
    pub fn stop_block(&mut self) {
        self.blocking = false;
    }

    /// Returns `true` when the shield is actively blocking.
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    /// Advance the cooldown timer by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if self.cooldown > 0.0 {
            self.cooldown = (self.cooldown - dt).max(0.0);
        }
    }

    /// Calculate post-shield damage. Returns 0 when the shield is blocking,
    /// otherwise returns `incoming` unchanged.
    pub fn block_damage(&self, incoming: f32) -> f32 {
        if self.blocking {
            0.0
        } else {
            incoming
        }
    }

    /// Disable the shield for `duration` seconds. This stops any active block
    /// and sets a cooldown that prevents re-blocking.
    pub fn disable(&mut self, duration: f32) {
        self.blocking = false;
        self.cooldown = duration;
    }
}

impl Default for ShieldState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Elytra state
// ---------------------------------------------------------------------------

/// Tracks the gliding state and durability of an elytra.
///
/// Gliding can only begin while the player is falling. Each tick of gliding
/// consumes one point of durability; the elytra stops working at zero.
#[derive(Debug, Clone, PartialEq)]
pub struct ElytraState {
    pub gliding: bool,
    pub durability: u32,
    pub max_durability: u32,
}

impl ElytraState {
    /// Create a new elytra at full durability in the non-gliding state.
    pub fn new(max_durability: u32) -> Self {
        Self {
            gliding: false,
            durability: max_durability,
            max_durability,
        }
    }

    /// Attempt to start gliding. The player must be falling (`is_falling`)
    /// and the elytra must have remaining durability.
    ///
    /// Returns `true` if gliding began successfully.
    pub fn start_glide(&mut self, is_falling: bool) -> bool {
        if is_falling && self.durability > 0 {
            self.gliding = true;
            true
        } else {
            false
        }
    }

    /// Stop gliding.
    pub fn stop_glide(&mut self) {
        self.gliding = false;
    }

    /// Advance the elytra simulation by one tick. While gliding, durability
    /// decreases by 1 per tick. When durability reaches zero gliding stops
    /// automatically.
    pub fn tick(&mut self, _dt: f32) {
        if self.gliding {
            self.durability = self.durability.saturating_sub(1);
            if self.durability == 0 {
                self.gliding = false;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Elytra physics
// ---------------------------------------------------------------------------

/// Compute the updated velocity for an elytra glider.
///
/// The `pitch` angle (in radians) controls vertical behaviour:
/// - **Negative pitch** (looking down): accelerates downward, increasing speed.
/// - **Near-zero pitch** (level): gentle glide with slight downward drift.
/// - **Positive pitch** (looking up): decelerates, trading speed for altitude.
///
/// `dt` is the time-step in seconds.
pub fn elytra_physics(velocity: Vec3, pitch: f32, dt: f32) -> Vec3 {
    let gravity = 0.08 * dt;
    let drag = 0.99_f32;

    let vertical_adjust = if pitch < -0.1 {
        // Looking down -- accelerate
        -0.5 * pitch.abs() * dt
    } else if pitch > 0.1 {
        // Looking up -- decelerate / climb
        0.3 * pitch * dt
    } else {
        // Level glide -- gentle sink
        -0.01 * dt
    };

    let new_y = velocity.y - gravity + vertical_adjust;

    // Drag on horizontal axes
    let new_x = velocity.x * drag;
    let new_z = velocity.z * drag;

    Vec3::new(new_x, new_y, new_z)
}

// ---------------------------------------------------------------------------
// Firework boost
// ---------------------------------------------------------------------------

/// Apply a firework rocket boost to the current velocity.
///
/// Adds `1.5 * look_dir` to the velocity, simulating the Minecraft firework
/// elytra boost mechanic.
pub fn firework_boost(velocity: Vec3, look_dir: Vec3) -> Vec3 {
    velocity + 1.5 * look_dir
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ShieldState: blocking reduces damage --------------------------------

    #[test]
    fn blocking_absorbs_all_damage() {
        let mut shield = ShieldState::new();
        shield.start_block();
        assert!(shield.is_blocking());
        let taken = shield.block_damage(10.0);
        assert!((taken).abs() < f32::EPSILON, "expected 0 damage while blocking");
    }

    #[test]
    fn not_blocking_passes_full_damage() {
        let shield = ShieldState::new();
        assert!(!shield.is_blocking());
        let taken = shield.block_damage(10.0);
        assert!((taken - 10.0).abs() < f32::EPSILON);
    }

    // -- ShieldState: cooldown prevents blocking -----------------------------

    #[test]
    fn cooldown_prevents_blocking() {
        let mut shield = ShieldState::new();
        shield.disable(5.0);
        shield.start_block();
        assert!(!shield.is_blocking(), "should not block while on cooldown");
    }

    #[test]
    fn cooldown_expires_after_ticking() {
        let mut shield = ShieldState::new();
        shield.disable(1.0);

        // Tick past the cooldown
        shield.tick(1.5);
        assert!(
            (shield.cooldown).abs() < f32::EPSILON,
            "cooldown should have expired"
        );

        // Now blocking should succeed
        shield.start_block();
        assert!(shield.is_blocking());
    }

    #[test]
    fn disable_stops_active_block() {
        let mut shield = ShieldState::new();
        shield.start_block();
        assert!(shield.is_blocking());

        shield.disable(3.0);
        assert!(!shield.is_blocking());
        assert!((shield.cooldown - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_decrements_cooldown() {
        let mut shield = ShieldState::new();
        shield.disable(2.0);
        shield.tick(0.5);
        assert!((shield.cooldown - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn cooldown_does_not_go_negative() {
        let mut shield = ShieldState::new();
        shield.disable(0.5);
        shield.tick(10.0);
        assert!(shield.cooldown >= 0.0);
    }

    #[test]
    fn stop_block_clears_blocking() {
        let mut shield = ShieldState::new();
        shield.start_block();
        shield.stop_block();
        assert!(!shield.is_blocking());
    }

    #[test]
    fn default_shield_is_idle() {
        let shield = ShieldState::default();
        assert!(!shield.blocking);
        assert!((shield.cooldown).abs() < f32::EPSILON);
    }

    // -- ElytraState: requires falling to start glide ------------------------

    #[test]
    fn elytra_requires_falling_to_glide() {
        let mut elytra = ElytraState::new(432);
        let started = elytra.start_glide(false);
        assert!(!started);
        assert!(!elytra.gliding);
    }

    #[test]
    fn elytra_starts_gliding_when_falling() {
        let mut elytra = ElytraState::new(432);
        let started = elytra.start_glide(true);
        assert!(started);
        assert!(elytra.gliding);
    }

    #[test]
    fn elytra_cannot_glide_at_zero_durability() {
        let mut elytra = ElytraState::new(432);
        elytra.durability = 0;
        let started = elytra.start_glide(true);
        assert!(!started);
    }

    #[test]
    fn elytra_tick_decrements_durability_while_gliding() {
        let mut elytra = ElytraState::new(10);
        elytra.start_glide(true);
        elytra.tick(0.05);
        assert_eq!(elytra.durability, 9);
    }

    #[test]
    fn elytra_stops_gliding_at_zero_durability() {
        let mut elytra = ElytraState::new(1);
        elytra.start_glide(true);
        elytra.tick(0.05);
        assert_eq!(elytra.durability, 0);
        assert!(!elytra.gliding, "should stop gliding when durability runs out");
    }

    #[test]
    fn elytra_tick_does_nothing_when_not_gliding() {
        let mut elytra = ElytraState::new(432);
        elytra.tick(0.05);
        assert_eq!(elytra.durability, 432);
    }

    #[test]
    fn elytra_stop_glide_works() {
        let mut elytra = ElytraState::new(432);
        elytra.start_glide(true);
        elytra.stop_glide();
        assert!(!elytra.gliding);
    }

    // -- Elytra physics: pitch directions ------------------------------------

    #[test]
    fn elytra_physics_looking_down_accelerates() {
        let vel = Vec3::new(0.0, 0.0, 10.0);
        let result = elytra_physics(vel, -0.5, 0.05);
        // Looking down should make y more negative (falling faster)
        assert!(
            result.y < vel.y,
            "looking down should decrease y: {} vs {}",
            result.y,
            vel.y,
        );
    }

    #[test]
    fn elytra_physics_looking_up_decelerates() {
        let vel = Vec3::new(0.0, -2.0, 10.0);
        let result = elytra_physics(vel, 0.5, 0.05);
        // Looking up should make y less negative (climbing / slowing descent)
        assert!(
            result.y > vel.y - 0.08 * 0.05,
            "looking up should add upward component"
        );
    }

    #[test]
    fn elytra_physics_level_glide_gentle_descent() {
        let vel = Vec3::new(0.0, 0.0, 10.0);
        let result = elytra_physics(vel, 0.0, 0.05);
        // Should drift downward gently
        assert!(result.y < 0.0, "level glide should drift down");
    }

    #[test]
    fn elytra_physics_applies_drag_to_horizontal() {
        let vel = Vec3::new(10.0, 0.0, 10.0);
        let result = elytra_physics(vel, 0.0, 0.05);
        assert!(result.x < vel.x, "drag should reduce x velocity");
        assert!(result.z < vel.z, "drag should reduce z velocity");
    }

    // -- Firework boost ------------------------------------------------------

    #[test]
    fn firework_boost_adds_velocity_in_look_direction() {
        let vel = Vec3::new(0.0, 0.0, 5.0);
        let look = Vec3::new(0.0, 0.5, 1.0);
        let result = firework_boost(vel, look);

        let expected = vel + 1.5 * look;
        assert!(
            (result - expected).length() < f32::EPSILON,
            "firework should add 1.5 * look_dir"
        );
    }

    #[test]
    fn firework_boost_stacks_with_existing_velocity() {
        let vel = Vec3::new(3.0, 1.0, 5.0);
        let look = Vec3::new(1.0, 0.0, 0.0);
        let result = firework_boost(vel, look);

        assert!((result.x - 4.5).abs() < f32::EPSILON);
        assert!((result.y - 1.0).abs() < f32::EPSILON);
        assert!((result.z - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn firework_boost_zero_look_dir_no_change() {
        let vel = Vec3::new(1.0, 2.0, 3.0);
        let result = firework_boost(vel, Vec3::ZERO);
        assert!((result - vel).length() < f32::EPSILON);
    }
}
