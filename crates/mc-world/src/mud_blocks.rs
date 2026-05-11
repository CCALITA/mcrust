//! Mud block types and mechanics.

/// Types of mud blocks available in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MudBlockType {
    Mud,
    PackedMud,
    MudBricks,
    MuddyMangroveRoots,
}

/// Returns the slowdown factor applied when walking on mud.
pub fn mud_slowdown_factor() -> f32 {
    0.4
}

/// Returns true if a water bottle can convert the given block to mud.
/// Only dirt (id 3) can be converted.
pub fn mud_from_water_bottle(dirt_id: u16) -> bool {
    dirt_id == 3
}

/// Returns true if packed mud can be crafted (requires wheat + mud).
pub fn packed_mud_from_mud(has_wheat: bool) -> bool {
    has_wheat
}

/// Returns the recipe output for mud bricks: (item_id for packed_mud, quantity needed).
pub fn mud_bricks_recipe() -> (u16, u8) {
    (4, 4)
}

/// Returns true if muddy mangrove roots can be crafted from mud and roots.
pub fn muddy_roots_from_mud_and_roots(has_mud: bool, has_roots: bool) -> bool {
    has_mud && has_roots
}

/// Returns the hardness value for a given mud block type.
pub fn mud_block_hardness(block: MudBlockType) -> f32 {
    match block {
        MudBlockType::Mud => 0.5,
        MudBlockType::PackedMud => 1.0,
        MudBlockType::MudBricks => 1.5,
        MudBlockType::MuddyMangroveRoots => 0.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mud_slowdown_factor() {
        assert!((mud_slowdown_factor() - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mud_from_water_bottle_dirt() {
        assert!(mud_from_water_bottle(3));
    }

    #[test]
    fn test_mud_from_water_bottle_non_dirt() {
        assert!(!mud_from_water_bottle(1));
        assert!(!mud_from_water_bottle(0));
        assert!(!mud_from_water_bottle(4));
    }

    #[test]
    fn test_packed_mud_from_mud_with_wheat() {
        assert!(packed_mud_from_mud(true));
    }

    #[test]
    fn test_packed_mud_from_mud_without_wheat() {
        assert!(!packed_mud_from_mud(false));
    }

    #[test]
    fn test_mud_bricks_recipe() {
        let (id, qty) = mud_bricks_recipe();
        assert_eq!(id, 4);
        assert_eq!(qty, 4);
    }

    #[test]
    fn test_muddy_roots_both_present() {
        assert!(muddy_roots_from_mud_and_roots(true, true));
    }

    #[test]
    fn test_muddy_roots_missing_mud() {
        assert!(!muddy_roots_from_mud_and_roots(false, true));
    }

    #[test]
    fn test_muddy_roots_missing_roots() {
        assert!(!muddy_roots_from_mud_and_roots(true, false));
    }

    #[test]
    fn test_mud_block_hardness() {
        assert!((mud_block_hardness(MudBlockType::Mud) - 0.5).abs() < f32::EPSILON);
        assert!((mud_block_hardness(MudBlockType::PackedMud) - 1.0).abs() < f32::EPSILON);
        assert!((mud_block_hardness(MudBlockType::MudBricks) - 1.5).abs() < f32::EPSILON);
        assert!((mud_block_hardness(MudBlockType::MuddyMangroveRoots) - 0.7).abs() < f32::EPSILON);
    }
}
