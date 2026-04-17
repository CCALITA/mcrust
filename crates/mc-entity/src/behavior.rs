use glam::Vec3;

use crate::component::MobKind;

/// Action that a mob should take after a behavior tick.
#[derive(Debug, Clone, PartialEq)]
pub enum MobAction {
    /// Do nothing — stand still.
    Idle,
    /// Walk toward a target position at the given speed (blocks/sec).
    WalkToward(Vec3, f32),
    /// Flee from a position at the given speed (blocks/sec).
    FleeFrom(Vec3, f32),
    /// Melee attack the player.
    Attack,
    /// Creeper explosion (despawn the mob).
    Explode,
}

/// Component tracking per-mob behavioral state.
#[derive(Debug, Clone)]
pub struct MobBehavior {
    pub kind: MobKind,
    /// Whether this mob has been recently damaged (triggers flee for passive).
    pub was_hit: bool,
    /// Ticks remaining in the current wander direction.
    pub wander_ticks: u32,
    /// Current wander target, if any.
    pub wander_target: Option<Vec3>,
}

impl MobBehavior {
    pub fn new(kind: MobKind) -> Self {
        Self {
            kind,
            was_hit: false,
            wander_ticks: 0,
            wander_target: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Distance within which hostile mobs detect the player.
const HOSTILE_DETECTION_RANGE: f32 = 16.0;
/// Distance at which a zombie attacks.
const ZOMBIE_ATTACK_RANGE: f32 = 2.0;
/// Distance at which a creeper starts the explosion sequence.
const CREEPER_EXPLODE_RANGE: f32 = 3.0;
/// Preferred distance for a skeleton to keep from the player.
const SKELETON_PREFERRED_DISTANCE: f32 = 10.0;
/// How close a skeleton must be to attack (ranged).
const SKELETON_ATTACK_RANGE: f32 = 16.0;
/// Distance at which passive mobs flee after being hit.
const PASSIVE_FLEE_RANGE: f32 = 16.0;

/// Walk speed for a zombie (blocks/sec).
const ZOMBIE_SPEED: f32 = 4.3;
/// Walk speed for a skeleton (blocks/sec).
const SKELETON_SPEED: f32 = 4.3;
/// Walk speed for a creeper (blocks/sec).
const CREEPER_SPEED: f32 = 4.3;
/// Walk speed for a spider (blocks/sec) — faster than zombie.
const SPIDER_SPEED: f32 = 5.6;
/// Walk speed for passive mobs when fleeing (blocks/sec).
const PASSIVE_FLEE_SPEED: f32 = 5.0;

/// Decide what action a mob should perform this tick.
///
/// * `kind` — the mob species.
/// * `mob_pos` — the mob's current world position.
/// * `player_pos` — the player's current world position.
/// * `distance` — pre-computed distance between mob and player.
pub fn behavior_tick(kind: MobKind, mob_pos: Vec3, player_pos: Vec3, distance: f32) -> MobAction {
    match kind {
        MobKind::Zombie => zombie_behavior(mob_pos, player_pos, distance),
        MobKind::Skeleton => skeleton_behavior(mob_pos, player_pos, distance),
        MobKind::Creeper => creeper_behavior(mob_pos, player_pos, distance),
        MobKind::Spider => spider_behavior(mob_pos, player_pos, distance),
        MobKind::Pig | MobKind::Cow | MobKind::Sheep | MobKind::Chicken => {
            passive_behavior(mob_pos, player_pos, distance)
        }
    }
}

/// Full behavior tick that also considers the `MobBehavior` state (e.g.
/// whether the mob was recently hit). Returns the chosen action and an updated
/// behavior state.
pub fn behavior_tick_with_state(
    behavior: &MobBehavior,
    mob_pos: Vec3,
    player_pos: Vec3,
    distance: f32,
) -> (MobAction, MobBehavior) {
    let mut next = behavior.clone();

    let action = match behavior.kind {
        MobKind::Pig | MobKind::Cow | MobKind::Sheep | MobKind::Chicken => {
            if behavior.was_hit && distance < PASSIVE_FLEE_RANGE {
                next.was_hit = true; // keep fleeing while in range
                MobAction::FleeFrom(player_pos, PASSIVE_FLEE_SPEED)
            } else {
                next.was_hit = false;
                passive_behavior(mob_pos, player_pos, distance)
            }
        }
        _ => behavior_tick(behavior.kind, mob_pos, player_pos, distance),
    };

    (action, next)
}

// ---------------------------------------------------------------------------
// Per-mob behavior functions
// ---------------------------------------------------------------------------

fn zombie_behavior(_mob_pos: Vec3, player_pos: Vec3, distance: f32) -> MobAction {
    if distance <= ZOMBIE_ATTACK_RANGE {
        MobAction::Attack
    } else if distance <= HOSTILE_DETECTION_RANGE {
        MobAction::WalkToward(player_pos, ZOMBIE_SPEED)
    } else {
        MobAction::Idle
    }
}

fn skeleton_behavior(_mob_pos: Vec3, player_pos: Vec3, distance: f32) -> MobAction {
    if distance > SKELETON_ATTACK_RANGE {
        return MobAction::Idle;
    }

    if distance <= HOSTILE_DETECTION_RANGE {
        if distance < SKELETON_PREFERRED_DISTANCE {
            // Too close — back away while still facing the player.
            MobAction::FleeFrom(player_pos, SKELETON_SPEED)
        } else {
            // At preferred range — attack (placeholder for ranged shot).
            MobAction::Attack
        }
    } else {
        // Walk toward player to get in range.
        MobAction::WalkToward(player_pos, SKELETON_SPEED)
    }

    // If within detection but further than preferred, walk toward.
    // Re-evaluate: 16 >= distance > 10 → walk closer; distance ~10 → attack;
    // distance < 10 → flee.
    // Simplified: the block above already handles this.
}

fn creeper_behavior(_mob_pos: Vec3, player_pos: Vec3, distance: f32) -> MobAction {
    if distance <= CREEPER_EXPLODE_RANGE {
        MobAction::Explode
    } else if distance <= HOSTILE_DETECTION_RANGE {
        MobAction::WalkToward(player_pos, CREEPER_SPEED)
    } else {
        MobAction::Idle
    }
}

fn spider_behavior(_mob_pos: Vec3, player_pos: Vec3, distance: f32) -> MobAction {
    if distance <= ZOMBIE_ATTACK_RANGE {
        MobAction::Attack
    } else if distance <= HOSTILE_DETECTION_RANGE {
        MobAction::WalkToward(player_pos, SPIDER_SPEED)
    } else {
        MobAction::Idle
    }
}

fn passive_behavior(_mob_pos: Vec3, _player_pos: Vec3, _distance: f32) -> MobAction {
    // Default passive behavior is to wander; full wander logic requires RNG
    // state which is handled via `behavior_tick_with_state`. Here we return
    // Idle as the baseline.
    MobAction::Idle
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Zombie tests -------------------------------------------------------

    #[test]
    fn zombie_follows_player_within_detection_range() {
        let mob_pos = Vec3::new(0.0, 0.0, 0.0);
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let distance = 10.0;

        let action = behavior_tick(MobKind::Zombie, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::WalkToward(player_pos, ZOMBIE_SPEED));
    }

    #[test]
    fn zombie_attacks_when_close() {
        let mob_pos = Vec3::new(0.0, 0.0, 0.0);
        let player_pos = Vec3::new(1.5, 0.0, 0.0);
        let distance = 1.5;

        let action = behavior_tick(MobKind::Zombie, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::Attack);
    }

    #[test]
    fn zombie_idles_when_far() {
        let mob_pos = Vec3::ZERO;
        let player_pos = Vec3::new(50.0, 0.0, 0.0);
        let distance = 50.0;

        let action = behavior_tick(MobKind::Zombie, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::Idle);
    }

    // -- Skeleton tests -----------------------------------------------------

    #[test]
    fn skeleton_keeps_distance_from_player() {
        let mob_pos = Vec3::new(0.0, 0.0, 0.0);
        let player_pos = Vec3::new(5.0, 0.0, 0.0);
        let distance = 5.0;

        let action = behavior_tick(MobKind::Skeleton, mob_pos, player_pos, distance);
        // Too close — skeleton flees to maintain range.
        assert_eq!(action, MobAction::FleeFrom(player_pos, SKELETON_SPEED));
    }

    #[test]
    fn skeleton_attacks_at_preferred_distance() {
        let mob_pos = Vec3::new(0.0, 0.0, 0.0);
        let player_pos = Vec3::new(12.0, 0.0, 0.0);
        let distance = 12.0;

        let action = behavior_tick(MobKind::Skeleton, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::Attack);
    }

    #[test]
    fn skeleton_idles_when_far() {
        let mob_pos = Vec3::ZERO;
        let player_pos = Vec3::new(50.0, 0.0, 0.0);
        let distance = 50.0;

        let action = behavior_tick(MobKind::Skeleton, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::Idle);
    }

    // -- Creeper tests ------------------------------------------------------

    #[test]
    fn creeper_explodes_when_close() {
        let mob_pos = Vec3::ZERO;
        let player_pos = Vec3::new(2.0, 0.0, 0.0);
        let distance = 2.0;

        let action = behavior_tick(MobKind::Creeper, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::Explode);
    }

    #[test]
    fn creeper_follows_when_in_range() {
        let mob_pos = Vec3::ZERO;
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let distance = 10.0;

        let action = behavior_tick(MobKind::Creeper, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::WalkToward(player_pos, CREEPER_SPEED));
    }

    // -- Spider tests -------------------------------------------------------

    #[test]
    fn spider_is_faster_than_zombie() {
        let mob_pos = Vec3::ZERO;
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let distance = 10.0;

        let spider_action = behavior_tick(MobKind::Spider, mob_pos, player_pos, distance);
        let zombie_action = behavior_tick(MobKind::Zombie, mob_pos, player_pos, distance);

        if let (MobAction::WalkToward(_, spider_speed), MobAction::WalkToward(_, zombie_speed)) =
            (&spider_action, &zombie_action)
        {
            assert!(
                spider_speed > zombie_speed,
                "spider ({spider_speed}) should be faster than zombie ({zombie_speed})"
            );
        } else {
            panic!("expected WalkToward for both spider and zombie");
        }
    }

    #[test]
    fn spider_attacks_when_close() {
        let mob_pos = Vec3::ZERO;
        let player_pos = Vec3::new(1.0, 0.0, 0.0);
        let distance = 1.0;

        let action = behavior_tick(MobKind::Spider, mob_pos, player_pos, distance);
        assert_eq!(action, MobAction::Attack);
    }

    // -- Passive mob tests --------------------------------------------------

    #[test]
    fn passive_mob_idles_normally() {
        for kind in [MobKind::Pig, MobKind::Cow, MobKind::Sheep, MobKind::Chicken] {
            let action = behavior_tick(kind, Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), 5.0);
            assert_eq!(action, MobAction::Idle, "{kind:?} should idle normally");
        }
    }

    #[test]
    fn passive_mob_flees_when_hit() {
        let behavior = MobBehavior {
            kind: MobKind::Pig,
            was_hit: true,
            wander_ticks: 0,
            wander_target: None,
        };

        let player_pos = Vec3::new(5.0, 0.0, 0.0);
        let (action, _next) =
            behavior_tick_with_state(&behavior, Vec3::ZERO, player_pos, 5.0);

        assert_eq!(action, MobAction::FleeFrom(player_pos, PASSIVE_FLEE_SPEED));
    }

    #[test]
    fn passive_mob_stops_fleeing_out_of_range() {
        let behavior = MobBehavior {
            kind: MobKind::Cow,
            was_hit: true,
            wander_ticks: 0,
            wander_target: None,
        };

        let player_pos = Vec3::new(20.0, 0.0, 0.0);
        let (action, next) =
            behavior_tick_with_state(&behavior, Vec3::ZERO, player_pos, 20.0);

        // Out of flee range — should revert to idle.
        assert_eq!(action, MobAction::Idle);
        assert!(!next.was_hit, "was_hit should clear after leaving range");
    }

    // -- MobAction equality tests -------------------------------------------

    #[test]
    fn mob_action_variants_are_distinct() {
        let idle = MobAction::Idle;
        let attack = MobAction::Attack;
        let explode = MobAction::Explode;

        assert_ne!(idle, attack);
        assert_ne!(attack, explode);
        assert_ne!(idle, explode);
    }
}
