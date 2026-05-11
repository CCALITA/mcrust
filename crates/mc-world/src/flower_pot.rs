//! Flower pot block logic: placing and removing plants from pots.

/// A flower pot that can hold a single plant item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowerPot {
    pub content: Option<u16>,
}

/// Item IDs that can be placed in a flower pot.
pub const POTTABLE_PLANTS: &[u16] = &[
    6,   // oak sapling
    7,   // spruce sapling
    8,   // birch sapling
    9,   // jungle sapling
    10,  // acacia sapling
    11,  // dark oak sapling
    37,  // dandelion
    38,  // poppy
    39,  // brown mushroom
    40,  // red mushroom
    81,  // cactus
    175, // fern
    31,  // dead bush
    32,  // bamboo
    199, // azalea
];

impl FlowerPot {
    /// Creates a new empty flower pot.
    pub fn new() -> Self {
        Self { content: None }
    }
}

impl Default for FlowerPot {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns whether the given item ID can be placed in a flower pot.
pub fn can_pot_plant(item_id: u16) -> bool {
    POTTABLE_PLANTS.contains(&item_id)
}

/// Attempts to place a plant in the pot. Returns `true` if successful.
pub fn place_in_pot(pot: &mut FlowerPot, plant: u16) -> bool {
    if pot.content.is_some() || !can_pot_plant(plant) {
        return false;
    }
    pot.content = Some(plant);
    true
}

/// Removes the plant from the pot, returning the item ID if one was present.
pub fn remove_from_pot(pot: &mut FlowerPot) -> Option<u16> {
    pot.content.take()
}

/// Returns the AABB collision box for a flower pot: [min_x, min_y, min_z, max_x, max_y, max_z].
pub fn pot_collision_box() -> [f32; 6] {
    [0.3125, 0.0, 0.3125, 0.6875, 0.375, 0.6875]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pot_is_empty() {
        let pot = FlowerPot::new();
        assert_eq!(pot.content, None);
    }

    #[test]
    fn pottable_plants_has_15_entries() {
        assert_eq!(POTTABLE_PLANTS.len(), 15);
    }

    #[test]
    fn can_pot_valid_plant() {
        assert!(can_pot_plant(37));
        assert!(can_pot_plant(6));
    }

    #[test]
    fn cannot_pot_invalid_plant() {
        assert!(!can_pot_plant(1));
        assert!(!can_pot_plant(999));
    }

    #[test]
    fn place_in_empty_pot() {
        let mut pot = FlowerPot::new();
        assert!(place_in_pot(&mut pot, 37));
        assert_eq!(pot.content, Some(37));
    }

    #[test]
    fn place_in_occupied_pot_fails() {
        let mut pot = FlowerPot { content: Some(37) };
        assert!(!place_in_pot(&mut pot, 38));
        assert_eq!(pot.content, Some(37));
    }

    #[test]
    fn place_invalid_plant_fails() {
        let mut pot = FlowerPot::new();
        assert!(!place_in_pot(&mut pot, 1));
        assert_eq!(pot.content, None);
    }

    #[test]
    fn remove_from_pot_returns_content() {
        let mut pot = FlowerPot { content: Some(81) };
        assert_eq!(remove_from_pot(&mut pot), Some(81));
        assert_eq!(pot.content, None);
    }

    #[test]
    fn remove_from_empty_pot_returns_none() {
        let mut pot = FlowerPot::new();
        assert_eq!(remove_from_pot(&mut pot), None);
    }

    #[test]
    fn collision_box_dimensions() {
        let bb = pot_collision_box();
        assert_eq!(bb.len(), 6);
        assert!(bb[3] > bb[0]); // max_x > min_x
        assert!(bb[4] > bb[1]); // max_y > min_y
        assert!(bb[5] > bb[2]); // max_z > min_z
    }
}
