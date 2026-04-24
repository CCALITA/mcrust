//! Beehive harvesting interactions.
//!
//! Models the player's interaction with a beehive or bee nest using a glass
//! bottle (to collect honey) or shears (to collect honeycombs). Honey only
//! drops when the hive is at maximum honey level. Without a campfire below,
//! disturbing the hive angers the resident bees.

/// Maximum honey level a beehive can reach.
pub const MAX_HONEY_LEVEL: u8 = 5;

/// Real-time seconds for a beehive to fill from level 0 to [`MAX_HONEY_LEVEL`].
/// Vanilla Minecraft fills a hive in roughly 24 minutes.
pub const HONEY_LEVEL_INCREMENT_TIME: f32 = 1200.0;

/// Vertical distance, in blocks, within which a campfire below the hive
/// suppresses bee anger when harvesting.
const CAMPFIRE_PACIFY_RANGE: u8 = 5;

/// Result of a single harvest interaction with a beehive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarvestResult {
    /// Number of honey bottles produced (0 or 1).
    pub honey_bottles: u8,
    /// Number of honeycombs produced (0 or 3).
    pub honeycombs: u8,
    /// Whether resident bees become hostile toward the player.
    pub anger_bees: bool,
    /// Whether the harvest produced any items.
    pub success: bool,
}

impl HarvestResult {
    /// Empty failed harvest (hive not full).
    fn failed() -> Self {
        Self {
            honey_bottles: 0,
            honeycombs: 0,
            anger_bees: false,
            success: false,
        }
    }
}

/// Attempt to harvest a beehive with a glass bottle.
///
/// Yields one honey bottle when the hive is at [`MAX_HONEY_LEVEL`]. A campfire
/// directly below the hive prevents the bees from being angered.
pub fn harvest_with_bottles(honey_level: u8, has_campfire_below: bool) -> HarvestResult {
    if honey_level < MAX_HONEY_LEVEL {
        return HarvestResult::failed();
    }
    HarvestResult {
        honey_bottles: 1,
        honeycombs: 0,
        anger_bees: !has_campfire_below,
        success: true,
    }
}

/// Attempt to harvest a beehive with shears.
///
/// Yields three honeycombs when the hive is at [`MAX_HONEY_LEVEL`]. A campfire
/// directly below the hive prevents the bees from being angered.
pub fn harvest_with_shears(honey_level: u8, has_campfire_below: bool) -> HarvestResult {
    if honey_level < MAX_HONEY_LEVEL {
        return HarvestResult::failed();
    }
    HarvestResult {
        honey_bottles: 0,
        honeycombs: 3,
        anger_bees: !has_campfire_below,
        success: true,
    }
}

/// Radius in blocks within which angered bees will pursue the player.
pub fn bee_anger_radius() -> f32 {
    8.0
}

/// Number of bees that emerge to attack when a hive is disturbed.
/// In vanilla, every bee currently inside the hive becomes hostile.
pub fn bees_attacking_count(hive_bee_count: u8) -> u8 {
    hive_bee_count
}

/// Returns true when a campfire directly below the hive is close enough to
/// pacify the bees during harvest.
pub fn campfire_below_prevents_anger(has_campfire: bool, distance_blocks: u8) -> bool {
    has_campfire && distance_blocks <= CAMPFIRE_PACIFY_RANGE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottles_at_full_honey_yields_one_bottle() {
        let result = harvest_with_bottles(MAX_HONEY_LEVEL, false);
        assert!(result.success);
        assert_eq!(result.honey_bottles, 1);
        assert_eq!(result.honeycombs, 0);
    }

    #[test]
    fn bottles_at_partial_honey_fails() {
        for level in 0..MAX_HONEY_LEVEL {
            let result = harvest_with_bottles(level, true);
            assert!(!result.success, "level {level} should fail");
            assert_eq!(result.honey_bottles, 0);
            assert!(!result.anger_bees);
        }
    }

    #[test]
    fn shears_at_full_honey_yields_three_combs() {
        let result = harvest_with_shears(MAX_HONEY_LEVEL, false);
        assert!(result.success);
        assert_eq!(result.honeycombs, 3);
        assert_eq!(result.honey_bottles, 0);
    }

    #[test]
    fn shears_at_partial_honey_fails() {
        let result = harvest_with_shears(4, false);
        assert!(!result.success);
        assert_eq!(result.honeycombs, 0);
    }

    #[test]
    fn campfire_below_prevents_anger_on_bottle_harvest() {
        let result = harvest_with_bottles(MAX_HONEY_LEVEL, true);
        assert!(result.success);
        assert!(!result.anger_bees);
    }

    #[test]
    fn campfire_below_prevents_anger_on_shear_harvest() {
        let result = harvest_with_shears(MAX_HONEY_LEVEL, true);
        assert!(result.success);
        assert!(!result.anger_bees);
    }

    #[test]
    fn no_campfire_angers_bees() {
        assert!(harvest_with_bottles(MAX_HONEY_LEVEL, false).anger_bees);
        assert!(harvest_with_shears(MAX_HONEY_LEVEL, false).anger_bees);
    }

    #[test]
    fn max_honey_level_constant() {
        assert_eq!(MAX_HONEY_LEVEL, 5);
    }

    #[test]
    fn fill_time_is_24_minutes() {
        assert_eq!(HONEY_LEVEL_INCREMENT_TIME, 1200.0);
    }

    #[test]
    fn anger_radius_is_eight_blocks() {
        assert_eq!(bee_anger_radius(), 8.0);
    }

    #[test]
    fn all_hive_bees_emerge_to_attack() {
        assert_eq!(bees_attacking_count(0), 0);
        assert_eq!(bees_attacking_count(3), 3);
    }

    #[test]
    fn campfire_within_range_pacifies() {
        assert!(campfire_below_prevents_anger(true, 1));
        assert!(campfire_below_prevents_anger(true, 5));
    }

    #[test]
    fn campfire_out_of_range_does_not_pacify() {
        assert!(!campfire_below_prevents_anger(true, 6));
    }

    #[test]
    fn no_campfire_never_pacifies() {
        assert!(!campfire_below_prevents_anger(false, 1));
    }
}
