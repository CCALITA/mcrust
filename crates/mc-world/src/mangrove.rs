//! Mangrove propagule growth, tree generation, and root placement.

/// Maximum age before a hanging propagule drops.
pub const MAX_PROPAGULE_AGE: u8 = 4;

/// Block IDs for valid propagule planting soil.
const MUD_BLOCK_ID: u16 = 200;
const DIRT_BLOCK_ID: u16 = 3;
const CLAY_BLOCK_ID: u16 = 82;

/// State of a mangrove propagule block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropaguleState {
    pub hanging: bool,
    pub age: u8,
    pub waterlogged: bool,
}

impl PropaguleState {
    pub fn new() -> Self {
        Self {
            hanging: false,
            age: 0,
            waterlogged: false,
        }
    }
}

/// Tick a propagule. Returns `true` if the propagule should drop (detach).
/// A propagule grows only when hanging below a mangrove log. At max age it drops.
pub fn tick_propagule(state: &mut PropaguleState, above_is_mangrove: bool, seed: u64) -> bool {
    if !state.hanging {
        return false;
    }
    if !above_is_mangrove {
        return false;
    }
    // Pseudo-random growth chance (~25%)
    if seed % 4 == 0 {
        if state.age >= MAX_PROPAGULE_AGE {
            return true;
        }
        state.age += 1;
    }
    false
}

/// Whether the given soil block ID supports mangrove tree growth.
pub fn propagule_can_grow_tree(soil: u16) -> bool {
    matches!(soil, MUD_BLOCK_ID | DIRT_BLOCK_ID | CLAY_BLOCK_ID)
}

/// Generate root positions around a mangrove trunk.
pub fn mangrove_root_positions(trunk: (i32, i32, i32), seed: u64) -> Vec<(i32, i32, i32)> {
    let (tx, ty, tz) = trunk;
    let mut roots = Vec::new();

    // Deterministic root placement based on seed
    let offsets: [(i32, i32); 8] = [
        (-1, 0), (1, 0), (0, -1), (0, 1),
        (-1, -1), (-1, 1), (1, -1), (1, 1),
    ];

    for (i, (dx, dz)) in offsets.iter().enumerate() {
        // Use seed to decide which roots to place (~50% chance each)
        if (seed.wrapping_mul(31).wrapping_add(i as u64)) % 2 == 0 {
            roots.push((tx + dx, ty - 1, tz + dz));
        }
    }

    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_propagule_defaults() {
        let state = PropaguleState::new();
        assert!(!state.hanging);
        assert_eq!(state.age, 0);
        assert!(!state.waterlogged);
    }

    #[test]
    fn tick_does_nothing_when_not_hanging() {
        let mut state = PropaguleState::new();
        let dropped = tick_propagule(&mut state, true, 0);
        assert!(!dropped);
        assert_eq!(state.age, 0);
    }

    #[test]
    fn tick_does_nothing_without_mangrove_above() {
        let mut state = PropaguleState { hanging: true, age: 0, waterlogged: false };
        let dropped = tick_propagule(&mut state, false, 0);
        assert!(!dropped);
        assert_eq!(state.age, 0);
    }

    #[test]
    fn tick_grows_when_hanging_and_seed_aligned() {
        let mut state = PropaguleState { hanging: true, age: 0, waterlogged: false };
        // seed % 4 == 0 triggers growth
        let dropped = tick_propagule(&mut state, true, 4);
        assert!(!dropped);
        assert_eq!(state.age, 1);
    }

    #[test]
    fn tick_drops_at_max_age() {
        let mut state = PropaguleState { hanging: true, age: MAX_PROPAGULE_AGE, waterlogged: false };
        let dropped = tick_propagule(&mut state, true, 0);
        assert!(dropped);
    }

    #[test]
    fn can_grow_on_valid_soils() {
        assert!(propagule_can_grow_tree(MUD_BLOCK_ID));
        assert!(propagule_can_grow_tree(DIRT_BLOCK_ID));
        assert!(propagule_can_grow_tree(CLAY_BLOCK_ID));
        assert!(!propagule_can_grow_tree(1)); // stone
    }

    #[test]
    fn root_positions_are_deterministic() {
        let roots1 = mangrove_root_positions((0, 64, 0), 42);
        let roots2 = mangrove_root_positions((0, 64, 0), 42);
        assert_eq!(roots1, roots2);
        assert!(!roots1.is_empty());
    }

    #[test]
    fn root_positions_vary_with_seed() {
        let roots1 = mangrove_root_positions((0, 64, 0), 1);
        let roots2 = mangrove_root_positions((0, 64, 0), 2);
        assert_ne!(roots1, roots2);
    }
}
