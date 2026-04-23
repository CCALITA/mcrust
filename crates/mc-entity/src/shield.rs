//! Shield blocking system with per-damage-type mitigation and durability.
//!
//! Provides [`ShieldState`] for tracking shield activation, cooldown, and
//! durability, plus [`block_damage`] for computing damage reduction based on
//! [`DamageType`].

// ---------------------------------------------------------------------------
// Shield state
// ---------------------------------------------------------------------------

/// Tracks the blocking state, cooldown, and durability of a shield.
#[derive(Debug, Clone, PartialEq)]
pub struct ShieldState {
    pub active: bool,
    pub cooldown: f32,
    pub durability: u16,
    pub max_durability: u16,
}

impl ShieldState {
    /// Create a new shield — not active, zero cooldown, full durability (336).
    pub fn new() -> Self {
        Self {
            active: false,
            cooldown: 0.0,
            durability: 336,
            max_durability: 336,
        }
    }

    /// Begin blocking. Only succeeds when the cooldown has expired.
    pub fn start_blocking(&mut self) {
        if self.cooldown <= 0.0 {
            self.active = true;
        }
    }

    /// Stop blocking.
    pub fn stop_blocking(&mut self) {
        self.active = false;
    }

    /// Advance the cooldown timer by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if self.cooldown > 0.0 {
            self.cooldown = (self.cooldown - dt).max(0.0);
        }
    }
}

impl Default for ShieldState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Damage type (shield-specific, includes Fire and Magic)
// ---------------------------------------------------------------------------

/// Categories of incoming damage relevant to shield blocking rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    Melee,
    Projectile,
    Explosion,
    Fire,
    Magic,
}

// ---------------------------------------------------------------------------
// Block result
// ---------------------------------------------------------------------------

/// Result of a shield block attempt against incoming damage.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockResult {
    pub damage_taken: f32,
    pub durability_cost: u16,
    pub blocked: bool,
}

// ---------------------------------------------------------------------------
// Block damage calculation
// ---------------------------------------------------------------------------

/// Calculate the outcome of incoming damage against a shield.
///
/// Blocking rules (when the shield is active):
/// - **Melee** — 100% blocked, 1 durability cost
/// - **Projectile** — 100% blocked, 1 durability cost
/// - **Explosion** — 50% blocked, durability cost = damage / 2 (rounded down)
/// - **Fire** — 100% blocked, 0 durability cost
/// - **Magic** — not blockable (0% blocked)
///
/// When the shield is not active, all damage passes through unblocked.
pub fn block_damage(state: &ShieldState, damage: f32, damage_type: DamageType) -> BlockResult {
    if !state.active {
        return BlockResult {
            damage_taken: damage,
            durability_cost: 0,
            blocked: false,
        };
    }

    match damage_type {
        DamageType::Melee => BlockResult {
            damage_taken: 0.0,
            durability_cost: 1,
            blocked: true,
        },
        DamageType::Projectile => BlockResult {
            damage_taken: 0.0,
            durability_cost: 1,
            blocked: true,
        },
        DamageType::Explosion => {
            let reduced = damage * 0.5;
            let durability_cost = (damage / 2.0) as u16;
            BlockResult {
                damage_taken: reduced,
                durability_cost,
                blocked: true,
            }
        }
        DamageType::Fire => BlockResult {
            damage_taken: 0.0,
            durability_cost: 0,
            blocked: true,
        },
        DamageType::Magic => BlockResult {
            damage_taken: damage,
            durability_cost: 0,
            blocked: false,
        },
    }
}

// ---------------------------------------------------------------------------
// Axe disable
// ---------------------------------------------------------------------------

/// An axe strike disables the shield: sets cooldown to 5.0 s and deactivates.
pub fn axe_disable_shield(state: &mut ShieldState) {
    state.cooldown = 5.0;
    state.active = false;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction -------------------------------------------------------

    #[test]
    fn new_shield_defaults() {
        let s = ShieldState::new();
        assert!(!s.active);
        assert!((s.cooldown).abs() < f32::EPSILON);
        assert_eq!(s.durability, 336);
        assert_eq!(s.max_durability, 336);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(ShieldState::default(), ShieldState::new());
    }

    // -- Blocking activation ------------------------------------------------

    #[test]
    fn start_blocking_activates_shield() {
        let mut s = ShieldState::new();
        s.start_blocking();
        assert!(s.active);
    }

    #[test]
    fn stop_blocking_deactivates_shield() {
        let mut s = ShieldState::new();
        s.start_blocking();
        s.stop_blocking();
        assert!(!s.active);
    }

    // -- Cooldown prevents blocking -----------------------------------------

    #[test]
    fn cooldown_prevents_blocking() {
        let mut s = ShieldState::new();
        s.cooldown = 3.0;
        s.start_blocking();
        assert!(!s.active, "should not block while on cooldown");
    }

    #[test]
    fn cooldown_expires_then_blocking_works() {
        let mut s = ShieldState::new();
        s.cooldown = 1.0;
        s.tick(1.5);
        assert!((s.cooldown).abs() < f32::EPSILON);

        s.start_blocking();
        assert!(s.active);
    }

    // -- Tick ---------------------------------------------------------------

    #[test]
    fn tick_decrements_cooldown() {
        let mut s = ShieldState::new();
        s.cooldown = 3.0;
        s.tick(1.0);
        assert!((s.cooldown - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_clamps_cooldown_at_zero() {
        let mut s = ShieldState::new();
        s.cooldown = 0.5;
        s.tick(10.0);
        assert!(s.cooldown >= 0.0);
        assert!((s.cooldown).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_noop_when_no_cooldown() {
        let mut s = ShieldState::new();
        s.tick(1.0);
        assert!((s.cooldown).abs() < f32::EPSILON);
    }

    // -- Melee blocking (100%, 1 durability) --------------------------------

    #[test]
    fn melee_fully_blocked() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let result = block_damage(&s, 8.0, DamageType::Melee);
        assert!(result.blocked);
        assert!((result.damage_taken).abs() < f32::EPSILON);
        assert_eq!(result.durability_cost, 1);
    }

    // -- Projectile blocking (100%, 1 durability) ---------------------------

    #[test]
    fn projectile_fully_blocked() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let result = block_damage(&s, 5.0, DamageType::Projectile);
        assert!(result.blocked);
        assert!((result.damage_taken).abs() < f32::EPSILON);
        assert_eq!(result.durability_cost, 1);
    }

    // -- Explosion blocking (50%, damage/2 durability) ----------------------

    #[test]
    fn explosion_half_blocked() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let result = block_damage(&s, 10.0, DamageType::Explosion);
        assert!(result.blocked);
        assert!((result.damage_taken - 5.0).abs() < f32::EPSILON);
        assert_eq!(result.durability_cost, 5);
    }

    #[test]
    fn explosion_odd_damage_rounds_down_durability() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let result = block_damage(&s, 7.0, DamageType::Explosion);
        assert!(result.blocked);
        assert!((result.damage_taken - 3.5).abs() < f32::EPSILON);
        assert_eq!(result.durability_cost, 3); // 7 / 2 = 3.5 → 3 as u16
    }

    // -- Fire blocking (100%, 0 durability) ---------------------------------

    #[test]
    fn fire_fully_blocked_no_durability() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let result = block_damage(&s, 4.0, DamageType::Fire);
        assert!(result.blocked);
        assert!((result.damage_taken).abs() < f32::EPSILON);
        assert_eq!(result.durability_cost, 0);
    }

    // -- Magic not blockable ------------------------------------------------

    #[test]
    fn magic_not_blockable() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let result = block_damage(&s, 6.0, DamageType::Magic);
        assert!(!result.blocked);
        assert!((result.damage_taken - 6.0).abs() < f32::EPSILON);
        assert_eq!(result.durability_cost, 0);
    }

    // -- Not blocking passes full damage ------------------------------------

    #[test]
    fn inactive_shield_passes_all_damage() {
        let s = ShieldState::new();
        for dt in [
            DamageType::Melee,
            DamageType::Projectile,
            DamageType::Explosion,
            DamageType::Fire,
            DamageType::Magic,
        ] {
            let result = block_damage(&s, 10.0, dt);
            assert!(!result.blocked, "inactive shield should not block {dt:?}");
            assert!(
                (result.damage_taken - 10.0).abs() < f32::EPSILON,
                "inactive shield should pass full damage for {dt:?}"
            );
            assert_eq!(result.durability_cost, 0);
        }
    }

    // -- Axe disable --------------------------------------------------------

    #[test]
    fn axe_disable_sets_cooldown_and_deactivates() {
        let mut s = ShieldState::new();
        s.start_blocking();
        assert!(s.active);

        axe_disable_shield(&mut s);
        assert!(!s.active);
        assert!((s.cooldown - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn axe_disable_prevents_immediate_reblock() {
        let mut s = ShieldState::new();
        axe_disable_shield(&mut s);
        s.start_blocking();
        assert!(!s.active, "should not re-block while on axe cooldown");
    }

    // -- Durability tracking ------------------------------------------------

    #[test]
    fn durability_cost_accumulates_across_hits() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let initial = s.durability;

        let r1 = block_damage(&s, 10.0, DamageType::Melee);
        s.durability = s.durability.saturating_sub(r1.durability_cost);

        let r2 = block_damage(&s, 10.0, DamageType::Projectile);
        s.durability = s.durability.saturating_sub(r2.durability_cost);

        let r3 = block_damage(&s, 20.0, DamageType::Explosion);
        s.durability = s.durability.saturating_sub(r3.durability_cost);

        // 1 + 1 + 10 = 12 total durability cost
        assert_eq!(s.durability, initial - 12);
    }

    #[test]
    fn fire_does_not_consume_durability() {
        let mut s = ShieldState::new();
        s.start_blocking();
        let initial = s.durability;

        let result = block_damage(&s, 100.0, DamageType::Fire);
        s.durability = s.durability.saturating_sub(result.durability_cost);

        assert_eq!(s.durability, initial);
    }
}
