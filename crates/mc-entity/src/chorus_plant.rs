//! Chorus plant growth mechanics and End-related plant behavior.

/// Maximum height a chorus plant can reach.
pub const MAX_CHORUS_HEIGHT: u8 = 5;

/// Chorus plant state tracking growth progression.
#[derive(Debug, Clone, PartialEq)]
pub struct ChorusPlant {
    pub age: u8,
    pub height: u8,
    pub can_grow: bool,
}

impl ChorusPlant {
    /// Creates a new chorus plant at age 0, height 0, ready to grow.
    pub fn new() -> Self {
        Self {
            age: 0,
            height: 0,
            can_grow: true,
        }
    }
}

/// Ticks chorus plant growth. Returns true if the plant grew this tick.
///
/// Growth depends on the plant not having reached max height and the
/// pseudo-random seed producing a favorable outcome.
pub fn tick_chorus_growth(plant: &mut ChorusPlant, seed: u64) -> bool {
    if !plant.can_grow || plant.height >= MAX_CHORUS_HEIGHT {
        plant.can_grow = false;
        return false;
    }

    // Simple deterministic growth chance based on seed
    let growth_chance = seed % 5;
    if growth_chance == 0 {
        plant.height += 1;
        plant.age += 1;
        if plant.height >= MAX_CHORUS_HEIGHT {
            plant.can_grow = false;
        }
        true
    } else {
        false
    }
}

/// Returns the teleport range for chorus fruit consumption.
pub fn chorus_fruit_teleport_range() -> f32 {
    8.0
}

/// Returns whether breaking a chorus flower drops chorus fruit.
pub fn chorus_flower_break_drops_fruit() -> bool {
    true
}

/// Returns whether end stone is required below for chorus plant placement.
pub fn end_stone_required_below() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_default_plant() {
        let plant = ChorusPlant::new();
        assert_eq!(plant.age, 0);
        assert_eq!(plant.height, 0);
        assert!(plant.can_grow);
    }

    #[test]
    fn growth_increases_height_and_age() {
        let mut plant = ChorusPlant::new();
        // seed % 5 == 0 triggers growth
        let grew = tick_chorus_growth(&mut plant, 10);
        assert!(grew);
        assert_eq!(plant.height, 1);
        assert_eq!(plant.age, 1);
    }

    #[test]
    fn no_growth_when_seed_unfavorable() {
        let mut plant = ChorusPlant::new();
        // seed % 5 != 0
        let grew = tick_chorus_growth(&mut plant, 7);
        assert!(!grew);
        assert_eq!(plant.height, 0);
    }

    #[test]
    fn stops_growing_at_max_height() {
        let mut plant = ChorusPlant {
            age: 4,
            height: MAX_CHORUS_HEIGHT,
            can_grow: true,
        };
        let grew = tick_chorus_growth(&mut plant, 10);
        assert!(!grew);
        assert!(!plant.can_grow);
    }

    #[test]
    fn growth_to_max_disables_can_grow() {
        let mut plant = ChorusPlant {
            age: 3,
            height: MAX_CHORUS_HEIGHT - 1,
            can_grow: true,
        };
        let grew = tick_chorus_growth(&mut plant, 0);
        assert!(grew);
        assert_eq!(plant.height, MAX_CHORUS_HEIGHT);
        assert!(!plant.can_grow);
    }

    #[test]
    fn no_growth_when_can_grow_false() {
        let mut plant = ChorusPlant {
            age: 0,
            height: 0,
            can_grow: false,
        };
        let grew = tick_chorus_growth(&mut plant, 0);
        assert!(!grew);
    }

    #[test]
    fn teleport_range_is_eight() {
        assert!((chorus_fruit_teleport_range() - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn flower_break_drops_fruit() {
        assert!(chorus_flower_break_drops_fruit());
    }

    #[test]
    fn end_stone_required() {
        assert!(end_stone_required_below());
    }
}
