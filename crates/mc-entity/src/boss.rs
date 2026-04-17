use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Experience points awarded for defeating the Ender Dragon.
pub const XP_REWARD: u32 = 12_000;

const CIRCLING_RADIUS: f32 = 50.0;
const CIRCLING_SPEED: f32 = 0.02; // rad/s
const CIRCLING_Y: f32 = 70.0;
const CIRCLING_DURATION: f32 = 30.0;

const STRAFING_SPEED: f32 = 10.0;
const STRAFING_REACH: f32 = 5.0;
const STRAFING_TIMEOUT: f32 = 10.0;

const LANDING_TARGET: Vec3 = Vec3::new(0.0, 64.0, 0.0);
const LANDING_THRESHOLD_Y: f32 = 65.0;

const PERCHING_DURATION: f32 = 10.0;

const DYING_DURATION: f32 = 5.0;

const CRYSTAL_HEAL_PER_SECOND: f32 = 1.0;

// ---------------------------------------------------------------------------
// Dragon phase
// ---------------------------------------------------------------------------

/// Current phase of the Ender Dragon fight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragonPhase {
    Circling,
    Strafing,
    Landing,
    Perching,
    Dying,
    Dead,
}

// ---------------------------------------------------------------------------
// Damage result
// ---------------------------------------------------------------------------

/// Outcome of attempting to damage the dragon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageResult {
    /// The dragon is immune in its current phase.
    Immune,
    /// Damage was applied; contains remaining health.
    Damaged(f32),
    /// The killing blow was dealt.
    Killed,
}

// ---------------------------------------------------------------------------
// Ender Dragon
// ---------------------------------------------------------------------------

/// State for the Ender Dragon boss entity.
#[derive(Debug, Clone)]
pub struct EnderDragon {
    pub health: f32,
    pub max_health: f32,
    pub phase: DragonPhase,
    pub position: Vec3,
    pub target_pos: Vec3,
    pub phase_timer: f32,
    pub circle_angle: f32,
}

impl EnderDragon {
    /// Create a new Ender Dragon with 200 HP, starting in the Circling phase
    /// at position (0, 70, 0).
    pub fn new() -> Self {
        Self {
            health: 200.0,
            max_health: 200.0,
            phase: DragonPhase::Circling,
            position: Vec3::new(0.0, CIRCLING_Y, 0.0),
            target_pos: Vec3::ZERO,
            phase_timer: 0.0,
            circle_angle: 0.0,
        }
    }
}

impl Default for EnderDragon {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Ender Crystal
// ---------------------------------------------------------------------------

/// An End Crystal that heals the dragon while alive.
#[derive(Debug, Clone)]
pub struct EnderCrystal {
    pub position: Vec3,
    pub alive: bool,
}

// ---------------------------------------------------------------------------
// Tick functions
// ---------------------------------------------------------------------------

/// Advance the dragon state by `dt` seconds.
///
/// Transitions between phases according to the Ender Dragon fight rules.
pub fn dragon_tick(dragon: &mut EnderDragon, player_pos: Vec3, dt: f32) {
    match dragon.phase {
        DragonPhase::Circling => tick_circling(dragon, dt),
        DragonPhase::Strafing => tick_strafing(dragon, player_pos, dt),
        DragonPhase::Landing => tick_landing(dragon, dt),
        DragonPhase::Perching => tick_perching(dragon, dt),
        DragonPhase::Dying => tick_dying(dragon, dt),
        DragonPhase::Dead => {} // no-op
    }
}

fn tick_circling(dragon: &mut EnderDragon, dt: f32) {
    dragon.phase_timer += dt;
    dragon.circle_angle += CIRCLING_SPEED * dt;

    dragon.position = Vec3::new(
        CIRCLING_RADIUS * dragon.circle_angle.cos(),
        CIRCLING_Y,
        CIRCLING_RADIUS * dragon.circle_angle.sin(),
    );

    if dragon.phase_timer >= CIRCLING_DURATION {
        dragon.phase = DragonPhase::Strafing;
        dragon.phase_timer = 0.0;
    }
}

fn tick_strafing(dragon: &mut EnderDragon, player_pos: Vec3, dt: f32) {
    dragon.phase_timer += dt;
    dragon.target_pos = player_pos;

    let direction = player_pos - dragon.position;
    let distance = direction.length();

    if distance > f32::EPSILON {
        let step = direction.normalize() * STRAFING_SPEED * dt;
        // Don't overshoot the target.
        if step.length() >= distance {
            dragon.position = player_pos;
        } else {
            dragon.position += step;
        }
    }

    let reached = (dragon.position - player_pos).length() < STRAFING_REACH;
    if reached || dragon.phase_timer >= STRAFING_TIMEOUT {
        dragon.phase = DragonPhase::Circling;
        dragon.phase_timer = 0.0;
    }
}

fn tick_landing(dragon: &mut EnderDragon, dt: f32) {
    let direction = LANDING_TARGET - dragon.position;
    let distance = direction.length();

    if distance > f32::EPSILON {
        let step = direction.normalize() * STRAFING_SPEED * dt;
        if step.length() >= distance {
            dragon.position = LANDING_TARGET;
        } else {
            dragon.position += step;
        }
    }

    if dragon.position.y < LANDING_THRESHOLD_Y {
        dragon.phase = DragonPhase::Perching;
        dragon.phase_timer = 0.0;
    }
}

fn tick_perching(dragon: &mut EnderDragon, dt: f32) {
    dragon.phase_timer += dt;

    if dragon.phase_timer >= PERCHING_DURATION {
        dragon.phase = DragonPhase::Circling;
        dragon.phase_timer = 0.0;
    }
}

fn tick_dying(dragon: &mut EnderDragon, dt: f32) {
    dragon.phase_timer += dt;

    if dragon.phase_timer >= DYING_DURATION {
        dragon.phase = DragonPhase::Dead;
        dragon.phase_timer = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------

/// Attempt to deal `amount` damage to the dragon.
///
/// The dragon only takes damage during the [`DragonPhase::Perching`] phase.
/// Returns a [`DamageResult`] indicating what happened.
pub fn dragon_damage(dragon: &mut EnderDragon, amount: f32) -> DamageResult {
    if dragon.phase != DragonPhase::Perching {
        return DamageResult::Immune;
    }

    dragon.health = (dragon.health - amount).max(0.0);

    if dragon.health <= 0.0 {
        dragon.phase = DragonPhase::Dying;
        dragon.phase_timer = 0.0;
        DamageResult::Killed
    } else {
        DamageResult::Damaged(dragon.health)
    }
}

// ---------------------------------------------------------------------------
// Crystal healing
// ---------------------------------------------------------------------------

/// Heal the dragon by 1 HP/s for each alive crystal.
pub fn crystal_heal_tick(dragon: &mut EnderDragon, crystals: &[EnderCrystal], dt: f32) {
    let alive_count = crystals.iter().filter(|c| c.alive).count() as f32;
    let heal_amount = alive_count * CRYSTAL_HEAL_PER_SECOND * dt;

    if heal_amount > 0.0 {
        dragon.health = (dragon.health + heal_amount).min(dragon.max_health);
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
    fn new_dragon_has_correct_defaults() {
        let dragon = EnderDragon::new();
        assert!((dragon.health - 200.0).abs() < f32::EPSILON);
        assert!((dragon.max_health - 200.0).abs() < f32::EPSILON);
        assert_eq!(dragon.phase, DragonPhase::Circling);
        assert!((dragon.position.x).abs() < f32::EPSILON);
        assert!((dragon.position.y - 70.0).abs() < f32::EPSILON);
        assert!((dragon.position.z).abs() < f32::EPSILON);
    }

    // -- Circling -----------------------------------------------------------

    #[test]
    fn circling_moves_position() {
        let mut dragon = EnderDragon::new();
        let initial_pos = dragon.position;

        dragon_tick(&mut dragon, Vec3::ZERO, 1.0);

        assert_ne!(
            dragon.position, initial_pos,
            "dragon position should change while circling"
        );
        assert_eq!(dragon.phase, DragonPhase::Circling);
    }

    #[test]
    fn circling_transitions_to_strafing_after_30s() {
        let mut dragon = EnderDragon::new();

        // Tick for 31 seconds total
        for _ in 0..31 {
            dragon_tick(&mut dragon, Vec3::ZERO, 1.0);
        }

        assert_eq!(
            dragon.phase,
            DragonPhase::Strafing,
            "should transition to Strafing after 30s"
        );
    }

    #[test]
    fn circling_stays_at_correct_altitude() {
        let mut dragon = EnderDragon::new();

        dragon_tick(&mut dragon, Vec3::ZERO, 5.0);

        assert!(
            (dragon.position.y - CIRCLING_Y).abs() < f32::EPSILON,
            "circling dragon should stay at y=70"
        );
    }

    // -- Strafing -----------------------------------------------------------

    #[test]
    fn strafing_approaches_player() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Strafing;
        dragon.phase_timer = 0.0;
        dragon.position = Vec3::new(100.0, 70.0, 0.0);

        let player_pos = Vec3::new(0.0, 70.0, 0.0);
        let initial_distance = (dragon.position - player_pos).length();

        dragon_tick(&mut dragon, player_pos, 1.0);

        let new_distance = (dragon.position - player_pos).length();
        assert!(
            new_distance < initial_distance,
            "strafing dragon should get closer to player (was {initial_distance}, now {new_distance})"
        );
    }

    #[test]
    fn strafing_transitions_to_circling_on_reach() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Strafing;
        dragon.phase_timer = 0.0;
        // Place dragon very close to the player
        dragon.position = Vec3::new(2.0, 70.0, 0.0);

        let player_pos = Vec3::new(0.0, 70.0, 0.0);
        dragon_tick(&mut dragon, player_pos, 1.0);

        assert_eq!(
            dragon.phase,
            DragonPhase::Circling,
            "should return to Circling after reaching within 5 blocks"
        );
    }

    #[test]
    fn strafing_transitions_to_circling_on_timeout() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Strafing;
        dragon.phase_timer = 0.0;
        dragon.position = Vec3::new(1000.0, 70.0, 0.0);

        let player_pos = Vec3::new(0.0, 70.0, 0.0);
        // Tick past the 10s timeout
        for _ in 0..11 {
            dragon_tick(&mut dragon, player_pos, 1.0);
            if dragon.phase != DragonPhase::Strafing {
                break;
            }
        }

        assert_eq!(
            dragon.phase,
            DragonPhase::Circling,
            "should return to Circling after 10s strafing timeout"
        );
    }

    // -- Landing ------------------------------------------------------------

    #[test]
    fn landing_descends_to_perching() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Landing;
        dragon.position = Vec3::new(0.0, 80.0, 0.0);

        // Tick enough for the dragon to descend to y < 65
        for _ in 0..20 {
            dragon_tick(&mut dragon, Vec3::ZERO, 1.0);
            if dragon.phase == DragonPhase::Perching {
                break;
            }
        }

        assert_eq!(
            dragon.phase,
            DragonPhase::Perching,
            "should transition to Perching after landing"
        );
    }

    // -- Perching -----------------------------------------------------------

    #[test]
    fn perching_transitions_to_circling_after_10s() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Perching;
        dragon.phase_timer = 0.0;

        for _ in 0..11 {
            dragon_tick(&mut dragon, Vec3::ZERO, 1.0);
            if dragon.phase != DragonPhase::Perching {
                break;
            }
        }

        assert_eq!(
            dragon.phase,
            DragonPhase::Circling,
            "should return to Circling after 10s perching"
        );
    }

    // -- Damage -------------------------------------------------------------

    #[test]
    fn damage_only_during_perching() {
        let mut dragon = EnderDragon::new();

        // Circling phase -- immune
        let result = dragon_damage(&mut dragon, 50.0);
        assert_eq!(result, DamageResult::Immune);
        assert!(
            (dragon.health - 200.0).abs() < f32::EPSILON,
            "health should not change"
        );

        // Strafing phase -- immune
        dragon.phase = DragonPhase::Strafing;
        let result = dragon_damage(&mut dragon, 50.0);
        assert_eq!(result, DamageResult::Immune);

        // Perching phase -- vulnerable
        dragon.phase = DragonPhase::Perching;
        let result = dragon_damage(&mut dragon, 50.0);
        assert_eq!(result, DamageResult::Damaged(150.0));
        assert!((dragon.health - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_immune_during_landing() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Landing;
        let result = dragon_damage(&mut dragon, 100.0);
        assert_eq!(result, DamageResult::Immune);
    }

    #[test]
    fn damage_immune_during_dying() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Dying;
        let result = dragon_damage(&mut dragon, 100.0);
        assert_eq!(result, DamageResult::Immune);
    }

    #[test]
    fn death_at_zero_hp() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Perching;

        let result = dragon_damage(&mut dragon, 200.0);
        assert_eq!(result, DamageResult::Killed);
        assert!(dragon.health <= 0.0);
        assert_eq!(dragon.phase, DragonPhase::Dying);
    }

    #[test]
    fn death_at_overkill_damage() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Perching;

        let result = dragon_damage(&mut dragon, 999.0);
        assert_eq!(result, DamageResult::Killed);
        assert!(
            (dragon.health).abs() < f32::EPSILON,
            "health should clamp at 0"
        );
    }

    #[test]
    fn dying_transitions_to_dead() {
        let mut dragon = EnderDragon::new();
        dragon.phase = DragonPhase::Dying;
        dragon.phase_timer = 0.0;

        for _ in 0..6 {
            dragon_tick(&mut dragon, Vec3::ZERO, 1.0);
            if dragon.phase == DragonPhase::Dead {
                break;
            }
        }

        assert_eq!(
            dragon.phase,
            DragonPhase::Dead,
            "should transition to Dead after 5s dying animation"
        );
    }

    // -- Crystal healing ----------------------------------------------------

    #[test]
    fn crystal_healing_increases_health() {
        let mut dragon = EnderDragon::new();
        dragon.health = 100.0;

        let crystals = vec![
            EnderCrystal {
                position: Vec3::new(10.0, 70.0, 0.0),
                alive: true,
            },
            EnderCrystal {
                position: Vec3::new(-10.0, 70.0, 0.0),
                alive: true,
            },
        ];

        crystal_heal_tick(&mut dragon, &crystals, 1.0);

        // 2 alive crystals * 1 HP/s * 1s = 2 HP healed
        assert!(
            (dragon.health - 102.0).abs() < f32::EPSILON,
            "health should be 102, got {}",
            dragon.health
        );
    }

    #[test]
    fn crystal_healing_does_not_exceed_max() {
        let mut dragon = EnderDragon::new();
        dragon.health = 199.0;

        let crystals = vec![EnderCrystal {
            position: Vec3::ZERO,
            alive: true,
        }];

        crystal_heal_tick(&mut dragon, &crystals, 5.0);

        assert!(
            (dragon.health - 200.0).abs() < f32::EPSILON,
            "health should cap at max_health"
        );
    }

    #[test]
    fn dead_crystals_do_not_heal() {
        let mut dragon = EnderDragon::new();
        dragon.health = 100.0;

        let crystals = vec![
            EnderCrystal {
                position: Vec3::ZERO,
                alive: false,
            },
            EnderCrystal {
                position: Vec3::ZERO,
                alive: false,
            },
        ];

        crystal_heal_tick(&mut dragon, &crystals, 1.0);

        assert!(
            (dragon.health - 100.0).abs() < f32::EPSILON,
            "dead crystals should not heal"
        );
    }

    #[test]
    fn mixed_crystals_only_alive_heal() {
        let mut dragon = EnderDragon::new();
        dragon.health = 150.0;

        let crystals = vec![
            EnderCrystal {
                position: Vec3::ZERO,
                alive: true,
            },
            EnderCrystal {
                position: Vec3::ZERO,
                alive: false,
            },
            EnderCrystal {
                position: Vec3::ZERO,
                alive: true,
            },
        ];

        crystal_heal_tick(&mut dragon, &crystals, 1.0);

        // 2 alive * 1 HP/s * 1s = 2 HP
        assert!(
            (dragon.health - 152.0).abs() < f32::EPSILON,
            "only alive crystals should heal"
        );
    }

    // -- XP reward ----------------------------------------------------------

    #[test]
    fn xp_reward_is_12000() {
        assert_eq!(XP_REWARD, 12_000);
    }

    // -- Phase transitions summary ------------------------------------------

    #[test]
    fn full_phase_cycle_circling_to_strafing_to_circling() {
        let mut dragon = EnderDragon::new();
        assert_eq!(dragon.phase, DragonPhase::Circling);

        // Run circling for 31 seconds -> Strafing
        for _ in 0..31 {
            dragon_tick(&mut dragon, Vec3::ZERO, 1.0);
        }
        assert_eq!(dragon.phase, DragonPhase::Strafing);

        // Place dragon at origin, player at origin -> within reach immediately
        dragon.position = Vec3::ZERO;
        dragon_tick(&mut dragon, Vec3::ZERO, 0.1);
        assert_eq!(dragon.phase, DragonPhase::Circling);
    }
}
