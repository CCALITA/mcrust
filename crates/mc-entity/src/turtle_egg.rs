//! Turtle egg hatching mechanics.

/// Maximum number of eggs in a single block.
pub const MAX_EGGS: u8 = 4;

/// Number of hatch stages before a baby turtle spawns.
pub const HATCH_STAGES: u8 = 3;

/// A turtle egg block that can contain multiple eggs.
#[derive(Debug, Clone, PartialEq)]
pub struct TurtleEgg {
    pub count: u8,
    pub hatch_progress: u8,
    pub on_sand: bool,
}

impl TurtleEgg {
    /// Create a new turtle egg block with the given number of eggs.
    pub fn new(count: u8) -> Self {
        let count = count.min(MAX_EGGS).max(1);
        Self {
            count,
            hatch_progress: 0,
            on_sand: false,
        }
    }
}

/// Tick a turtle egg. Returns `true` if the egg hatches (completes all stages).
/// Hatch chance is 0.01 per tick if it is night and the egg is on sand.
pub fn tick_egg(egg: &mut TurtleEgg, is_night: bool, seed: u64) -> bool {
    if !is_night || !egg.on_sand {
        return false;
    }

    // Simple deterministic pseudo-random from seed
    let hash = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let chance = (hash % 10000) as f64 / 10000.0;

    if chance < 0.01 {
        egg.hatch_progress += 1;
        if egg.hatch_progress >= HATCH_STAGES {
            return true;
        }
    }

    false
}

/// Calculate the chance of an entity trampling a turtle egg based on its weight.
pub fn egg_trample_chance(entity_weight: f32) -> f32 {
    (entity_weight / 100.0).clamp(0.0, 1.0)
}

/// Speed of a baby turtle.
pub fn baby_turtle_speed() -> f32 {
    0.12
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_egg() {
        let egg = TurtleEgg::new(2);
        assert_eq!(egg.count, 2);
        assert_eq!(egg.hatch_progress, 0);
        assert!(!egg.on_sand);
    }

    #[test]
    fn test_new_egg_clamped() {
        let egg = TurtleEgg::new(10);
        assert_eq!(egg.count, MAX_EGGS);
        let egg = TurtleEgg::new(0);
        assert_eq!(egg.count, 1);
    }

    #[test]
    fn test_tick_no_hatch_when_day() {
        let mut egg = TurtleEgg::new(1);
        egg.on_sand = true;
        assert!(!tick_egg(&mut egg, false, 42));
        assert_eq!(egg.hatch_progress, 0);
    }

    #[test]
    fn test_tick_no_hatch_when_not_on_sand() {
        let mut egg = TurtleEgg::new(1);
        assert!(!tick_egg(&mut egg, true, 42));
        assert_eq!(egg.hatch_progress, 0);
    }

    #[test]
    fn test_tick_hatch_progress() {
        let mut egg = TurtleEgg::new(1);
        egg.on_sand = true;
        // Find a seed that triggers hatch
        let mut hatched = false;
        for seed in 0..10000u64 {
            if tick_egg(&mut egg, true, seed) {
                hatched = true;
                break;
            }
            if egg.hatch_progress > 0 {
                // Progress was made
                break;
            }
        }
        assert!(egg.hatch_progress > 0 || hatched);
    }

    #[test]
    fn test_full_hatch() {
        let mut egg = TurtleEgg::new(1);
        egg.on_sand = true;
        egg.hatch_progress = HATCH_STAGES - 1;
        // Find seed that triggers
        for seed in 0..100000u64 {
            if tick_egg(&mut egg, true, seed) {
                assert_eq!(egg.hatch_progress, HATCH_STAGES);
                return;
            }
            // Reset if it didn't trigger
            egg.hatch_progress = HATCH_STAGES - 1;
        }
        panic!("No seed found that triggers hatch");
    }

    #[test]
    fn test_egg_trample_chance() {
        assert_eq!(egg_trample_chance(0.0), 0.0);
        assert_eq!(egg_trample_chance(50.0), 0.5);
        assert_eq!(egg_trample_chance(100.0), 1.0);
        assert_eq!(egg_trample_chance(200.0), 1.0);
    }

    #[test]
    fn test_baby_turtle_speed() {
        assert_eq!(baby_turtle_speed(), 0.12);
    }
}
