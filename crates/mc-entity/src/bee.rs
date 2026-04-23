use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base sting damage before poison effect.
const STING_BASE_DAMAGE: f32 = 1.0;

/// Poison damage per tick applied after a sting.
const STING_POISON_PER_TICK: f32 = 0.5;

/// Duration of poison effect in ticks (10 seconds at 20 tps).
const STING_POISON_DURATION: u32 = 200;

/// Number of ticks a bee survives after losing its stinger.
const DEATH_TICKS_AFTER_STING: u32 = 60;

/// Maximum honey level a hive can hold before dripping.
const MAX_HONEY_LEVEL: u8 = 5;

/// Maximum number of bees a single hive can house.
const MAX_BEES_PER_HIVE: u8 = 3;

// ---------------------------------------------------------------------------
// Bee state
// ---------------------------------------------------------------------------

/// The internal state of a single bee.
#[derive(Debug, Clone, PartialEq)]
pub struct BeeState {
    /// Whether the bee is currently carrying pollen.
    pub has_pollen: bool,
    /// Remaining anger ticks (> 0 means the bee is hostile).
    pub anger_ticks: u32,
    /// Position of the bee's home hive, if any.
    pub hive_pos: Option<(i32, i32, i32)>,
}

impl BeeState {
    /// Create a new bee with no pollen, no anger, and no hive.
    pub fn new() -> Self {
        Self {
            has_pollen: false,
            anger_ticks: 0,
            hive_pos: None,
        }
    }

    /// Create a new bee associated with the given hive position.
    pub fn with_hive(hive_pos: (i32, i32, i32)) -> Self {
        Self {
            has_pollen: false,
            anger_ticks: 0,
            hive_pos: Some(hive_pos),
        }
    }

    /// Returns `true` if the bee is currently angry.
    pub fn is_angry(&self) -> bool {
        self.anger_ticks > 0
    }
}

// ---------------------------------------------------------------------------
// Bee actions
// ---------------------------------------------------------------------------

/// Action chosen by the bee AI each tick.
#[derive(Debug, Clone, PartialEq)]
pub enum BeeAction {
    /// Pollinate a nearby flower.
    Pollinate,
    /// Return to the home hive (carries pollen if available).
    ReturnToHive,
    /// Sting a target at the given position.
    Sting(Vec3),
    /// Wander randomly.
    Wander,
}

// ---------------------------------------------------------------------------
// Sting mechanics
// ---------------------------------------------------------------------------

/// Damage dealt by a single bee sting.
///
/// Returns `(immediate_damage, poison_per_tick, poison_duration_ticks)`.
pub fn sting_damage() -> (f32, f32, u32) {
    (STING_BASE_DAMAGE, STING_POISON_PER_TICK, STING_POISON_DURATION)
}

/// Determines the tick at which a bee dies after losing its stinger.
///
/// A bee that has stung lives for exactly 60 more ticks, then dies.
/// Returns `None` if the bee has not stung (no death scheduled).
pub fn lose_stinger(has_stung: bool) -> Option<u32> {
    if has_stung {
        Some(DEATH_TICKS_AFTER_STING)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Hive state
// ---------------------------------------------------------------------------

/// The state of a bee hive / nest block.
#[derive(Debug, Clone, PartialEq)]
pub struct HiveState {
    /// Number of bees currently inside the hive.
    pub bees: u8,
    /// Current honey level (0..=5).
    pub honey_level: u8,
}

impl HiveState {
    /// Create an empty hive.
    pub fn new() -> Self {
        Self {
            bees: 0,
            honey_level: 0,
        }
    }

    /// Returns `true` if the hive is full of honey (level 5).
    pub fn is_full(&self) -> bool {
        self.honey_level >= MAX_HONEY_LEVEL
    }

    /// Returns `true` if the hive can accept another bee.
    pub fn can_accept_bee(&self) -> bool {
        self.bees < MAX_BEES_PER_HIVE
    }

    /// A bee enters the hive carrying pollen, increasing honey level.
    /// Returns the updated hive state, or `None` if the hive cannot accept
    /// more bees.
    pub fn bee_enters_with_pollen(&self) -> Option<HiveState> {
        if !self.can_accept_bee() {
            return None;
        }
        Some(HiveState {
            bees: self.bees + 1,
            honey_level: (self.honey_level + 1).min(MAX_HONEY_LEVEL),
        })
    }

    /// A bee exits the hive (e.g. to forage). Returns the updated hive state,
    /// or `None` if no bees are inside.
    pub fn bee_exits(&self) -> Option<HiveState> {
        if self.bees == 0 {
            return None;
        }
        Some(HiveState {
            bees: self.bees - 1,
            honey_level: self.honey_level,
        })
    }
}

// ---------------------------------------------------------------------------
// Bee tick
// ---------------------------------------------------------------------------

/// Choose the bee's action for this tick.
///
/// Priority order:
/// 1. If angry and a target exists, sting the nearest target.
/// 2. If carrying pollen and has a hive, return to hive.
/// 3. If flowers are nearby and not carrying pollen, pollinate.
/// 4. Wander.
///
/// `dt` is the delta-time multiplier (typically 1.0 for a single tick).
/// It is reserved for future speed adjustments but does not currently
/// alter behavior.
pub fn bee_tick(
    state: &BeeState,
    flowers_nearby: &[Vec3],
    target: Option<Vec3>,
    _dt: f32,
) -> BeeAction {
    // Priority 1 — sting when angry.
    if state.is_angry() {
        if let Some(target_pos) = target {
            return BeeAction::Sting(target_pos);
        }
    }

    // Priority 2 — deliver pollen to hive.
    if state.has_pollen && state.hive_pos.is_some() {
        return BeeAction::ReturnToHive;
    }

    // Priority 3 — pollinate a flower if not carrying pollen.
    if !state.has_pollen && !flowers_nearby.is_empty() {
        return BeeAction::Pollinate;
    }

    // Priority 4 — nothing else to do.
    BeeAction::Wander
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- BeeState construction -----------------------------------------------

    #[test]
    fn new_bee_has_no_pollen_no_anger_no_hive() {
        let bee = BeeState::new();
        assert!(!bee.has_pollen);
        assert_eq!(bee.anger_ticks, 0);
        assert_eq!(bee.hive_pos, None);
        assert!(!bee.is_angry());
    }

    #[test]
    fn bee_with_hive_stores_position() {
        let bee = BeeState::with_hive((10, 64, -20));
        assert_eq!(bee.hive_pos, Some((10, 64, -20)));
        assert!(!bee.has_pollen);
        assert_eq!(bee.anger_ticks, 0);
    }

    // -- Sting damage --------------------------------------------------------

    #[test]
    fn sting_damage_returns_correct_values() {
        let (base, poison, duration) = sting_damage();
        assert!((base - 1.0).abs() < f32::EPSILON);
        assert!((poison - 0.5).abs() < f32::EPSILON);
        assert_eq!(duration, 200);
    }

    // -- Lose stinger --------------------------------------------------------

    #[test]
    fn lose_stinger_schedules_death_after_sting() {
        let result = lose_stinger(true);
        assert_eq!(result, Some(60));
    }

    #[test]
    fn lose_stinger_returns_none_when_not_stung() {
        let result = lose_stinger(false);
        assert_eq!(result, None);
    }

    // -- HiveState -----------------------------------------------------------

    #[test]
    fn new_hive_is_empty() {
        let hive = HiveState::new();
        assert_eq!(hive.bees, 0);
        assert_eq!(hive.honey_level, 0);
        assert!(!hive.is_full());
        assert!(hive.can_accept_bee());
    }

    #[test]
    fn bee_enters_hive_with_pollen_increases_honey() {
        let hive = HiveState::new();
        let updated = hive.bee_enters_with_pollen().unwrap();
        assert_eq!(updated.bees, 1);
        assert_eq!(updated.honey_level, 1);
    }

    #[test]
    fn hive_honey_level_caps_at_five() {
        let hive = HiveState {
            bees: 0,
            honey_level: 5,
        };
        assert!(hive.is_full());
        let updated = hive.bee_enters_with_pollen().unwrap();
        assert_eq!(updated.honey_level, 5); // capped
    }

    #[test]
    fn hive_rejects_bee_when_full_of_bees() {
        let hive = HiveState {
            bees: 3,
            honey_level: 0,
        };
        assert!(!hive.can_accept_bee());
        assert!(hive.bee_enters_with_pollen().is_none());
    }

    #[test]
    fn bee_exits_hive_decreases_count() {
        let hive = HiveState {
            bees: 2,
            honey_level: 3,
        };
        let updated = hive.bee_exits().unwrap();
        assert_eq!(updated.bees, 1);
        assert_eq!(updated.honey_level, 3); // unchanged
    }

    #[test]
    fn bee_cannot_exit_empty_hive() {
        let hive = HiveState::new();
        assert!(hive.bee_exits().is_none());
    }

    // -- bee_tick behavior ----------------------------------------------------

    #[test]
    fn angry_bee_stings_target() {
        let bee = BeeState {
            has_pollen: false,
            anger_ticks: 100,
            hive_pos: None,
        };
        let target = Vec3::new(5.0, 1.0, 3.0);
        let action = bee_tick(&bee, &[], Some(target), 1.0);
        assert_eq!(action, BeeAction::Sting(target));
    }

    #[test]
    fn angry_bee_wanders_when_no_target() {
        let bee = BeeState {
            has_pollen: false,
            anger_ticks: 50,
            hive_pos: None,
        };
        let action = bee_tick(&bee, &[], None, 1.0);
        assert_eq!(action, BeeAction::Wander);
    }

    #[test]
    fn bee_with_pollen_returns_to_hive() {
        let bee = BeeState {
            has_pollen: true,
            anger_ticks: 0,
            hive_pos: Some((10, 64, 10)),
        };
        let flowers = vec![Vec3::new(3.0, 1.0, 3.0)];
        let action = bee_tick(&bee, &flowers, None, 1.0);
        assert_eq!(action, BeeAction::ReturnToHive);
    }

    #[test]
    fn bee_without_pollen_pollinates_when_flowers_nearby() {
        let bee = BeeState {
            has_pollen: false,
            anger_ticks: 0,
            hive_pos: Some((10, 64, 10)),
        };
        let flowers = vec![Vec3::new(2.0, 1.0, 2.0)];
        let action = bee_tick(&bee, &flowers, None, 1.0);
        assert_eq!(action, BeeAction::Pollinate);
    }

    #[test]
    fn bee_wanders_when_nothing_to_do() {
        let bee = BeeState::new();
        let action = bee_tick(&bee, &[], None, 1.0);
        assert_eq!(action, BeeAction::Wander);
    }

    #[test]
    fn angry_bee_prioritizes_sting_over_return_to_hive() {
        let bee = BeeState {
            has_pollen: true,
            anger_ticks: 30,
            hive_pos: Some((10, 64, 10)),
        };
        let target = Vec3::new(4.0, 1.0, 4.0);
        let action = bee_tick(&bee, &[], Some(target), 1.0);
        assert_eq!(action, BeeAction::Sting(target));
    }

    #[test]
    fn bee_without_hive_wanders_even_with_pollen() {
        let bee = BeeState {
            has_pollen: true,
            anger_ticks: 0,
            hive_pos: None,
        };
        let action = bee_tick(&bee, &[], None, 1.0);
        assert_eq!(action, BeeAction::Wander);
    }
}
