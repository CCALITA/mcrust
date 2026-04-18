use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// XP dropped when the Wither is killed.
pub const XP_REWARD: u32 = 50;

/// Duration (seconds) the Wither spends in the Spawning phase (invincible).
const SPAWN_DURATION: f32 = 10.0;

/// Duration (seconds) the Dying phase lasts before transitioning to Dead.
const DYING_DURATION: f32 = 5.0;

/// HP threshold at which the Wither enters the HalfHealth phase.
const HALF_HEALTH_THRESHOLD: f32 = 150.0;

/// Default max health for a Wither.
const DEFAULT_MAX_HEALTH: f32 = 300.0;

/// Base skull speed (blocks per second).
const SKULL_SPEED: f32 = 20.0;

/// Attack cooldown in Fighting phase (seconds).
const ATTACK_COOLDOWN_FIGHTING: f32 = 1.0;

/// Attack cooldown in HalfHealth phase (seconds) — faster attacks.
const ATTACK_COOLDOWN_HALF_HEALTH: f32 = 0.5;

// ---------------------------------------------------------------------------
// WitherPhase
// ---------------------------------------------------------------------------

/// Phases of the Wither boss fight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WitherPhase {
    /// Spawning animation — invincible for [`SPAWN_DURATION`] seconds.
    Spawning,
    /// Normal combat at full / above-half health.
    Fighting,
    /// Below half health — gains armor and attacks faster.
    HalfHealth,
    /// Death animation — lasts [`DYING_DURATION`] seconds.
    Dying,
    /// Fully dead — ready for cleanup.
    Dead,
}

// ---------------------------------------------------------------------------
// DamageResult
// ---------------------------------------------------------------------------

/// Outcome of a damage attempt against the Wither.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageResult {
    /// Damage was rejected (e.g. during Spawning).
    Immune,
    /// Damage was applied; inner value is the remaining health.
    Damaged(f32),
    /// The Wither was killed by this hit.
    Killed,
}

// ---------------------------------------------------------------------------
// WitherSkull
// ---------------------------------------------------------------------------

/// A Wither skull projectile.
#[derive(Debug, Clone, PartialEq)]
pub struct WitherSkull {
    pub position: Vec3,
    pub velocity: Vec3,
    pub explosive: bool,
}

// ---------------------------------------------------------------------------
// Wither
// ---------------------------------------------------------------------------

/// State of the Wither boss entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Wither {
    pub health: f32,
    pub max_health: f32,
    pub phase: WitherPhase,
    pub position: Vec3,
    /// Seconds remaining in the current timed phase (Spawning / Dying).
    pub spawn_timer: f32,
    /// Potential target positions (e.g. nearby players/mobs).
    pub targets: Vec<Vec3>,
    /// Internal cooldown tracker for skull attacks.
    attack_cooldown: f32,
}

impl Wither {
    /// Create a new Wither at `pos` with 300 HP in the Spawning phase.
    pub fn new(pos: Vec3) -> Self {
        Self {
            health: DEFAULT_MAX_HEALTH,
            max_health: DEFAULT_MAX_HEALTH,
            phase: WitherPhase::Spawning,
            position: pos,
            spawn_timer: SPAWN_DURATION,
            targets: Vec::new(),
            attack_cooldown: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Tick
// ---------------------------------------------------------------------------

/// Advance the Wither by `dt` seconds, returning any skulls fired this tick.
///
/// Phase behaviour:
/// - **Spawning** — countdown; invincible. Transitions to Fighting at 0.
/// - **Fighting** — shoots skulls at nearest target on cooldown.
/// - **HalfHealth** — same as Fighting but with faster attack rate.
/// - **Dying** — countdown; transitions to Dead at 0.
/// - **Dead** — no-op.
pub fn wither_tick(wither: &mut Wither, player_pos: Vec3, dt: f32) -> Vec<WitherSkull> {
    match wither.phase {
        WitherPhase::Spawning => {
            wither.spawn_timer -= dt;
            if wither.spawn_timer <= 0.0 {
                wither.spawn_timer = 0.0;
                wither.phase = WitherPhase::Fighting;
            }
            Vec::new()
        }
        WitherPhase::Fighting | WitherPhase::HalfHealth => {
            wither.attack_cooldown -= dt;
            if wither.attack_cooldown > 0.0 {
                return Vec::new();
            }

            // Pick the closest target (or fall back to player_pos).
            let target = closest_target(&wither.targets, wither.position)
                .unwrap_or(player_pos);

            let skull = fire_skull(wither.position, target);

            let cooldown = match wither.phase {
                WitherPhase::HalfHealth => ATTACK_COOLDOWN_HALF_HEALTH,
                _ => ATTACK_COOLDOWN_FIGHTING,
            };
            wither.attack_cooldown = cooldown;

            vec![skull]
        }
        WitherPhase::Dying => {
            wither.spawn_timer -= dt;
            if wither.spawn_timer <= 0.0 {
                wither.spawn_timer = 0.0;
                wither.phase = WitherPhase::Dead;
            }
            Vec::new()
        }
        WitherPhase::Dead => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------

/// Apply `amount` damage to the Wither, respecting phase-based immunities.
///
/// - **Spawning** → `Immune`
/// - **Fighting** → full damage
/// - **HalfHealth** → half damage (armored)
/// - **Dying / Dead** → `Immune`
pub fn wither_damage(wither: &mut Wither, amount: f32) -> DamageResult {
    match wither.phase {
        WitherPhase::Spawning | WitherPhase::Dying | WitherPhase::Dead => DamageResult::Immune,
        WitherPhase::Fighting => apply_wither_damage(wither, amount),
        WitherPhase::HalfHealth => apply_wither_damage(wither, amount * 0.5),
    }
}

// ---------------------------------------------------------------------------
// Helpers (private)
// ---------------------------------------------------------------------------

/// Internal: subtract `effective` damage, transition phases, and return result.
fn apply_wither_damage(wither: &mut Wither, effective: f32) -> DamageResult {
    wither.health = (wither.health - effective).max(0.0);

    if wither.health <= 0.0 {
        wither.phase = WitherPhase::Dying;
        wither.spawn_timer = DYING_DURATION;
        return DamageResult::Killed;
    }

    // Transition Fighting → HalfHealth when crossing the threshold.
    if wither.phase == WitherPhase::Fighting && wither.health <= HALF_HEALTH_THRESHOLD {
        wither.phase = WitherPhase::HalfHealth;
    }

    DamageResult::Damaged(wither.health)
}

/// Find the closest position in `targets` to `origin`.
fn closest_target(targets: &[Vec3], origin: Vec3) -> Option<Vec3> {
    targets
        .iter()
        .min_by(|a, b| {
            let da = a.distance_squared(origin);
            let db = b.distance_squared(origin);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

/// Create a [`WitherSkull`] aimed from `origin` towards `target`.
fn fire_skull(origin: Vec3, target: Vec3) -> WitherSkull {
    let direction = (target - origin).normalize_or_zero();
    WitherSkull {
        position: origin,
        velocity: direction * SKULL_SPEED,
        explosive: false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction -------------------------------------------------------

    #[test]
    fn new_wither_has_300_hp_and_spawning_phase() {
        let w = Wither::new(Vec3::ZERO);
        assert!((w.health - 300.0).abs() < f32::EPSILON);
        assert!((w.max_health - 300.0).abs() < f32::EPSILON);
        assert_eq!(w.phase, WitherPhase::Spawning);
        assert!((w.spawn_timer - 10.0).abs() < f32::EPSILON);
    }

    // -- Spawn timer --------------------------------------------------------

    #[test]
    fn spawn_timer_counts_down() {
        let mut w = Wither::new(Vec3::ZERO);
        wither_tick(&mut w, Vec3::ZERO, 3.0);
        assert!((w.spawn_timer - 7.0).abs() < f32::EPSILON);
        assert_eq!(w.phase, WitherPhase::Spawning);
    }

    #[test]
    fn spawn_phase_transitions_to_fighting_after_10s() {
        let mut w = Wither::new(Vec3::ZERO);
        wither_tick(&mut w, Vec3::ZERO, 10.0);
        assert_eq!(w.phase, WitherPhase::Fighting);
        assert!((w.spawn_timer).abs() < f32::EPSILON);
    }

    #[test]
    fn spawn_phase_transitions_even_with_overshoot() {
        let mut w = Wither::new(Vec3::ZERO);
        wither_tick(&mut w, Vec3::ZERO, 15.0);
        assert_eq!(w.phase, WitherPhase::Fighting);
    }

    // -- Immunity during spawn ----------------------------------------------

    #[test]
    fn immune_during_spawning() {
        let mut w = Wither::new(Vec3::ZERO);
        let result = wither_damage(&mut w, 100.0);
        assert_eq!(result, DamageResult::Immune);
        assert!((w.health - 300.0).abs() < f32::EPSILON);
    }

    // -- Phase transitions at HP thresholds ---------------------------------

    #[test]
    fn fighting_transitions_to_half_health_at_150() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Fighting;

        let result = wither_damage(&mut w, 150.0);
        assert_eq!(w.phase, WitherPhase::HalfHealth);
        assert_eq!(result, DamageResult::Damaged(150.0));
    }

    #[test]
    fn fighting_transitions_to_half_health_below_150() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Fighting;

        let result = wither_damage(&mut w, 160.0);
        assert_eq!(w.phase, WitherPhase::HalfHealth);
        assert_eq!(result, DamageResult::Damaged(140.0));
    }

    #[test]
    fn half_health_takes_half_damage() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::HalfHealth;
        w.health = 150.0;

        let result = wither_damage(&mut w, 40.0);
        // Should take 20.0 effective damage → 130.0 HP remaining
        assert_eq!(result, DamageResult::Damaged(130.0));
    }

    #[test]
    fn wither_killed_enters_dying_phase() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Fighting;
        w.health = 10.0;

        let result = wither_damage(&mut w, 10.0);
        assert_eq!(result, DamageResult::Killed);
        assert_eq!(w.phase, WitherPhase::Dying);
    }

    #[test]
    fn dying_phase_transitions_to_dead() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Dying;
        w.spawn_timer = 5.0;

        wither_tick(&mut w, Vec3::ZERO, 5.0);
        assert_eq!(w.phase, WitherPhase::Dead);
    }

    #[test]
    fn immune_during_dying() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Dying;
        w.spawn_timer = 5.0;

        let result = wither_damage(&mut w, 100.0);
        assert_eq!(result, DamageResult::Immune);
    }

    #[test]
    fn dead_phase_is_noop() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Dead;

        let skulls = wither_tick(&mut w, Vec3::ZERO, 1.0);
        assert!(skulls.is_empty());
        assert_eq!(w.phase, WitherPhase::Dead);
    }

    // -- Skull targeting ----------------------------------------------------

    #[test]
    fn fighting_fires_skull_at_player() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Fighting;
        w.attack_cooldown = 0.0;

        let player = Vec3::new(10.0, 0.0, 0.0);
        let skulls = wither_tick(&mut w, player, 0.1);

        assert_eq!(skulls.len(), 1);
        // Skull should move toward the player (+X direction).
        assert!(skulls[0].velocity.x > 0.0);
    }

    #[test]
    fn skull_targets_nearest_from_target_list() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Fighting;
        w.attack_cooldown = 0.0;

        let far = Vec3::new(100.0, 0.0, 0.0);
        let close = Vec3::new(5.0, 0.0, 0.0);
        w.targets = vec![far, close];

        let player_pos = Vec3::new(50.0, 0.0, 0.0);
        let skulls = wither_tick(&mut w, player_pos, 0.1);

        assert_eq!(skulls.len(), 1);
        // Should aim at `close` (5,0,0) not `far` or `player_pos`.
        let dir = skulls[0].velocity.normalize();
        assert!(dir.x > 0.9, "skull should aim toward closest target (+X)");
    }

    #[test]
    fn respects_attack_cooldown() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::Fighting;
        w.attack_cooldown = 0.0;

        let player = Vec3::new(10.0, 0.0, 0.0);

        // First tick fires.
        let skulls = wither_tick(&mut w, player, 0.1);
        assert_eq!(skulls.len(), 1);

        // Immediately after, cooldown should prevent firing.
        let skulls2 = wither_tick(&mut w, player, 0.1);
        assert!(skulls2.is_empty());
    }

    #[test]
    fn half_health_phase_has_faster_attacks() {
        let mut w = Wither::new(Vec3::ZERO);
        w.phase = WitherPhase::HalfHealth;
        w.health = 100.0;
        w.attack_cooldown = 0.0;

        let player = Vec3::new(10.0, 0.0, 0.0);

        // Fire once to set cooldown.
        let _ = wither_tick(&mut w, player, 0.0);
        let cooldown_after_fire = w.attack_cooldown;

        // HalfHealth cooldown should be 0.5s (faster than Fighting's 1.0s).
        assert!(
            (cooldown_after_fire - ATTACK_COOLDOWN_HALF_HEALTH).abs() < f32::EPSILON,
            "expected {ATTACK_COOLDOWN_HALF_HEALTH}, got {cooldown_after_fire}"
        );
    }

    // -- XP reward ----------------------------------------------------------

    #[test]
    fn xp_reward_is_50() {
        assert_eq!(XP_REWARD, 50);
    }
}
