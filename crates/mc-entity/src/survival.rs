use std::collections::HashMap;

use crate::component::Health;

// ---------------------------------------------------------------------------
// Hunger component
// ---------------------------------------------------------------------------

/// Tracks hunger, saturation and exhaustion for a player entity.
///
/// * `food_level` ranges 0..=20 (hunger bar shanks).
/// * `saturation` ranges 0.0..=food_level as f32 (hidden buffer).
/// * `exhaustion` accumulates from actions; when it reaches 4.0 it drains
///   saturation first, then food level.
#[derive(Debug, Clone)]
pub struct HungerComponent {
    pub food_level: u32,
    pub saturation: f32,
    pub exhaustion: f32,
}

impl HungerComponent {
    pub const MAX_FOOD: u32 = 20;
    pub const MAX_SATURATION: f32 = 20.0;
    pub const EXHAUSTION_THRESHOLD: f32 = 4.0;
}

impl Default for HungerComponent {
    fn default() -> Self {
        Self {
            food_level: 20,
            saturation: 5.0,
            exhaustion: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Exhaustion costs
// ---------------------------------------------------------------------------

/// Per-meter exhaustion cost for walking.
pub const EXHAUSTION_WALK_PER_METER: f32 = 0.01;

/// Per-meter exhaustion cost for sprinting.
pub const EXHAUSTION_SPRINT_PER_METER: f32 = 0.1;

/// Per-jump exhaustion cost.
pub const EXHAUSTION_JUMP: f32 = 0.05;

// ---------------------------------------------------------------------------
// Hunger system
// ---------------------------------------------------------------------------

pub struct HungerSystem;

impl HungerSystem {
    // Internal timing accumulators are kept outside this struct (caller
    // responsibility) to keep the system stateless.  The constants below are
    // the intervals at which effects trigger.

    /// Interval between natural-regen heals when food >= 18.
    const REGEN_INTERVAL: f32 = 0.5;
    /// Health restored per regen tick.
    const REGEN_AMOUNT: f32 = 0.5;
    /// Exhaustion added per regen tick.
    const REGEN_EXHAUSTION: f32 = 6.0;

    /// Interval between starvation damage ticks when food == 0.
    const STARVE_INTERVAL: f32 = 4.0;
    /// Damage dealt per starvation tick.
    const STARVE_DAMAGE: f32 = 0.5;

    /// Process one frame of the hunger/health system.
    ///
    /// `regen_timer` and `starve_timer` are caller-owned accumulators that
    /// track elapsed time since the last heal/starve tick.
    pub fn tick(
        hunger: &mut HungerComponent,
        health: &mut Health,
        dt: f32,
        regen_timer: &mut f32,
        starve_timer: &mut f32,
    ) {
        // Drain exhaustion into saturation / food_level.
        Self::drain_exhaustion(hunger);

        // Natural regeneration: food_level >= 18 and not at max health.
        if hunger.food_level >= 18 && health.current < health.max {
            *regen_timer += dt;
            while *regen_timer >= Self::REGEN_INTERVAL {
                *regen_timer -= Self::REGEN_INTERVAL;
                health.heal(Self::REGEN_AMOUNT);
                hunger.exhaustion += Self::REGEN_EXHAUSTION;
                Self::drain_exhaustion(hunger);
            }
        } else {
            *regen_timer = 0.0;
        }

        // Starvation: food_level == 0 and health above minimum.
        if hunger.food_level == 0 && health.current > 0.0 {
            *starve_timer += dt;
            while *starve_timer >= Self::STARVE_INTERVAL {
                *starve_timer -= Self::STARVE_INTERVAL;
                health.damage(Self::STARVE_DAMAGE);
            }
        } else {
            *starve_timer = 0.0;
        }
    }

    /// Consume food, restoring `food_value` food points and `saturation_value`
    /// saturation, clamped to maximums.
    pub fn eat(hunger: &mut HungerComponent, food_value: u32, saturation_value: f32) {
        hunger.food_level = (hunger.food_level + food_value).min(HungerComponent::MAX_FOOD);
        hunger.saturation = (hunger.saturation + saturation_value)
            .min(HungerComponent::MAX_SATURATION)
            .min(hunger.food_level as f32);
    }

    /// Add exhaustion from an action (e.g. walking, sprinting, jumping).
    pub fn add_exhaustion(hunger: &mut HungerComponent, amount: f32) {
        hunger.exhaustion += amount;
    }

    // -- internal -----------------------------------------------------------

    /// When exhaustion >= 4.0, drain saturation first, then food_level.
    fn drain_exhaustion(hunger: &mut HungerComponent) {
        while hunger.exhaustion >= HungerComponent::EXHAUSTION_THRESHOLD {
            hunger.exhaustion -= HungerComponent::EXHAUSTION_THRESHOLD;

            if hunger.saturation > 0.0 {
                hunger.saturation = (hunger.saturation - 1.0).max(0.0);
            } else if hunger.food_level > 0 {
                hunger.food_level -= 1;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Food value table
// ---------------------------------------------------------------------------

/// Returns a mapping of item ID (u16) to (food_points, saturation_modifier).
///
/// The saturation modifier is the *total* saturation restored, not the
/// per-point modifier used in the Minecraft wiki -- we pre-multiply here.
pub fn food_values() -> HashMap<u16, (u32, f32)> {
    let mut map = HashMap::new();
    // Item IDs mirror mc_core::ItemId discriminants.
    // Apple (42)
    map.insert(42, (4, 2.4));
    // GoldenApple (43)
    map.insert(43, (4, 9.6));
    // Bread (44)
    map.insert(44, (5, 6.0));
    // CookedPorkchop (45)
    map.insert(45, (8, 12.8));
    // CookedBeef (46)
    map.insert(46, (8, 12.8));
    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn full_hunger() -> HungerComponent {
        HungerComponent {
            food_level: 20,
            saturation: 5.0,
            exhaustion: 0.0,
        }
    }

    fn starving_hunger() -> HungerComponent {
        HungerComponent {
            food_level: 0,
            saturation: 0.0,
            exhaustion: 0.0,
        }
    }

    fn damaged_health() -> Health {
        Health {
            current: 15.0,
            max: 20.0,
        }
    }

    fn full_health() -> Health {
        Health {
            current: 20.0,
            max: 20.0,
        }
    }

    // -- Natural regeneration -----------------------------------------------

    #[test]
    fn natural_regen_at_full_hunger() {
        let mut hunger = full_hunger();
        let mut health = damaged_health();
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;

        // Tick long enough for at least one regen heal (0.5s interval).
        HungerSystem::tick(&mut hunger, &mut health, 0.6, &mut regen, &mut starve);

        assert!(
            health.current > 15.0,
            "health should increase via regen, got {}",
            health.current
        );
    }

    #[test]
    fn no_regen_when_health_is_full() {
        let mut hunger = full_hunger();
        let mut health = full_health();
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;

        HungerSystem::tick(&mut hunger, &mut health, 1.0, &mut regen, &mut starve);

        assert!(
            (health.current - 20.0).abs() < f32::EPSILON,
            "full health should not change"
        );
    }

    #[test]
    fn no_regen_below_eighteen_food() {
        let mut hunger = HungerComponent {
            food_level: 17,
            saturation: 0.0,
            exhaustion: 0.0,
        };
        let mut health = damaged_health();
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;

        HungerSystem::tick(&mut hunger, &mut health, 1.0, &mut regen, &mut starve);

        assert!(
            (health.current - 15.0).abs() < f32::EPSILON,
            "should not regen below 18 food, got {}",
            health.current
        );
    }

    // -- Starvation ---------------------------------------------------------

    #[test]
    fn starvation_at_zero_food() {
        let mut hunger = starving_hunger();
        let mut health = full_health();
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;

        // Tick for 5 seconds (should get at least one starve tick at 4s interval).
        HungerSystem::tick(&mut hunger, &mut health, 5.0, &mut regen, &mut starve);

        assert!(
            health.current < 20.0,
            "starvation should reduce health, got {}",
            health.current
        );
    }

    #[test]
    fn no_starvation_with_food() {
        let mut hunger = HungerComponent {
            food_level: 1,
            saturation: 0.0,
            exhaustion: 0.0,
        };
        let mut health = full_health();
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;

        HungerSystem::tick(&mut hunger, &mut health, 10.0, &mut regen, &mut starve);

        assert!(
            (health.current - 20.0).abs() < f32::EPSILON,
            "should not starve with food > 0"
        );
    }

    // -- Eating -------------------------------------------------------------

    #[test]
    fn eating_restores_food_and_saturation() {
        let mut hunger = HungerComponent {
            food_level: 10,
            saturation: 0.0,
            exhaustion: 0.0,
        };

        HungerSystem::eat(&mut hunger, 5, 6.0);

        assert_eq!(hunger.food_level, 15);
        assert!((hunger.saturation - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn eating_clamps_food_to_max() {
        let mut hunger = HungerComponent {
            food_level: 18,
            saturation: 0.0,
            exhaustion: 0.0,
        };

        HungerSystem::eat(&mut hunger, 8, 12.8);

        assert_eq!(hunger.food_level, 20);
        // Saturation clamped to min(MAX_SATURATION, food_level)
        assert!(hunger.saturation <= 20.0);
        assert!(hunger.saturation <= hunger.food_level as f32);
    }

    #[test]
    fn eating_from_zero_restores_correctly() {
        let mut hunger = starving_hunger();
        HungerSystem::eat(&mut hunger, 4, 2.4);

        assert_eq!(hunger.food_level, 4);
        assert!((hunger.saturation - 2.4).abs() < f32::EPSILON);
    }

    // -- Exhaustion mechanics -----------------------------------------------

    #[test]
    fn exhaustion_drains_saturation_first() {
        let mut hunger = HungerComponent {
            food_level: 20,
            saturation: 5.0,
            exhaustion: 0.0,
        };

        // Add 4.0 exhaustion (one drain cycle).
        HungerSystem::add_exhaustion(&mut hunger, 4.0);
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;
        let mut health = full_health();

        HungerSystem::tick(&mut hunger, &mut health, 0.0, &mut regen, &mut starve);

        assert_eq!(hunger.food_level, 20, "food should not decrease yet");
        assert!(
            (hunger.saturation - 4.0).abs() < f32::EPSILON,
            "saturation should decrease by 1, got {}",
            hunger.saturation
        );
    }

    #[test]
    fn exhaustion_drains_food_when_no_saturation() {
        let mut hunger = HungerComponent {
            food_level: 20,
            saturation: 0.0,
            exhaustion: 0.0,
        };

        HungerSystem::add_exhaustion(&mut hunger, 4.0);
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;
        let mut health = full_health();

        HungerSystem::tick(&mut hunger, &mut health, 0.0, &mut regen, &mut starve);

        assert_eq!(
            hunger.food_level, 19,
            "food should decrease when saturation is 0"
        );
    }

    #[test]
    fn multiple_exhaustion_drains_cascade() {
        let mut hunger = HungerComponent {
            food_level: 20,
            saturation: 1.0,
            exhaustion: 0.0,
        };

        // 8.0 exhaustion = 2 drain cycles
        // First drains saturation (1.0 -> 0.0), second drains food.
        HungerSystem::add_exhaustion(&mut hunger, 8.0);
        let mut regen = 0.0_f32;
        let mut starve = 0.0_f32;
        let mut health = full_health();

        HungerSystem::tick(&mut hunger, &mut health, 0.0, &mut regen, &mut starve);

        assert!((hunger.saturation).abs() < f32::EPSILON);
        assert_eq!(hunger.food_level, 19);
    }

    // -- Exhaustion constants -----------------------------------------------

    #[test]
    fn exhaustion_constants_are_correct() {
        assert!((EXHAUSTION_WALK_PER_METER - 0.01).abs() < f32::EPSILON);
        assert!((EXHAUSTION_SPRINT_PER_METER - 0.1).abs() < f32::EPSILON);
        assert!((EXHAUSTION_JUMP - 0.05).abs() < f32::EPSILON);
    }

    // -- Food value table ---------------------------------------------------

    #[test]
    fn food_values_contains_expected_entries() {
        let table = food_values();

        let apple = table.get(&42).expect("apple should be in food table");
        assert_eq!(apple.0, 4);
        assert!((apple.1 - 2.4).abs() < f32::EPSILON);

        let bread = table.get(&44).expect("bread should be in food table");
        assert_eq!(bread.0, 5);
        assert!((bread.1 - 6.0).abs() < f32::EPSILON);

        let porkchop = table
            .get(&45)
            .expect("cooked porkchop should be in food table");
        assert_eq!(porkchop.0, 8);
        assert!((porkchop.1 - 12.8).abs() < f32::EPSILON);

        let steak = table.get(&46).expect("cooked beef should be in food table");
        assert_eq!(steak.0, 8);
        assert!((steak.1 - 12.8).abs() < f32::EPSILON);

        let golden = table
            .get(&43)
            .expect("golden apple should be in food table");
        assert_eq!(golden.0, 4);
        assert!((golden.1 - 9.6).abs() < f32::EPSILON);
    }

    // -- HungerComponent defaults -------------------------------------------

    #[test]
    fn hunger_defaults_are_correct() {
        let h = HungerComponent::default();
        assert_eq!(h.food_level, 20);
        assert!((h.saturation - 5.0).abs() < f32::EPSILON);
        assert!((h.exhaustion).abs() < f32::EPSILON);
    }
}
