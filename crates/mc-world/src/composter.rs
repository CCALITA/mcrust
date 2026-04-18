//! Composter block mechanics.
//!
//! A composter turns organic items into bone meal through a 0-8 level
//! progression. Players add compostable items (levels 0-6), each with a
//! per-item success chance. Once level 7 is reached the composter is
//! "full" and transitions to level 8 (ready), at which point it can be
//! harvested for one bone meal.

use mc_core::ItemId;

/// Item ID for bone meal. No `ItemId::BoneMeal` variant exists yet, so we
/// reserve a placeholder raw value one past the current enum range.
pub const BONE_MEAL_ID: u16 = 200;

/// Outcome of attempting to add an item to a composter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompostResult {
    /// The item was consumed and the compost level increased.
    Added,
    /// The item was consumed but the compost level did not increase.
    Failed,
    /// The composter is at level 7 or 8; no more items can be added.
    AlreadyFull,
    /// The composter was at level 8 and has been harvested (bone meal produced).
    Harvested,
}

/// Returns the composting success chance for the given item.
///
/// A return value of `0.0` means the item is not compostable.
/// Chances follow the vanilla Minecraft tiers:
///   - 0.30 — seeds, leaves, saplings, grass
///   - 0.50 — melon, cactus, sugar cane, vines
///   - 0.65 — flowers, apples, carrots, potatoes, wheat, beetroot, cookies
///   - 0.85 — bread, baked potatoes, cooked food
///   - 1.00 — cake, golden apple
pub fn compost_chance(item_id: u16) -> f32 {
    // 30% tier — seeds, leaves, saplings, grass
    if item_id == ItemId::WheatSeeds as u16
        || item_id == ItemId::BeetrootSeeds as u16
        || item_id == ItemId::MelonSeeds as u16
        || item_id == ItemId::PumpkinSeeds as u16
        || item_id == ItemId::OakLeaves as u16
        || item_id == ItemId::GrassBlock as u16
    {
        return 0.3;
    }

    // 50% tier — melon slice, sugar cane (block), cactus (block)
    if item_id == ItemId::Melon as u16 {
        return 0.5;
    }

    // 65% tier — flowers, apple, carrot, potato, wheat, beetroot, cookie
    if item_id == ItemId::Apple as u16
        || item_id == ItemId::Carrot as u16
        || item_id == ItemId::Potato as u16
        || item_id == ItemId::WheatItem as u16
        || item_id == ItemId::Beetroot as u16
        || item_id == ItemId::Cookie as u16
    {
        return 0.65;
    }

    // 85% tier — bread, baked potato, cooked food
    if item_id == ItemId::Bread as u16
        || item_id == ItemId::BakedPotato as u16
        || item_id == ItemId::CookedPorkchop as u16
        || item_id == ItemId::CookedBeef as u16
        || item_id == ItemId::CookedChicken as u16
        || item_id == ItemId::CookedMutton as u16
    {
        return 0.85;
    }

    // 100% tier — cake, golden apple
    if item_id == ItemId::Cake as u16 || item_id == ItemId::GoldenApple as u16 {
        return 1.0;
    }

    0.0
}

/// Attempt to compost an item in a composter at the given `level`.
///
/// # Rules
/// - Levels 0-6: the item's `compost_chance` is compared against `random`
///   (a uniform value in `0.0..1.0`). On success the level increments; on
///   failure the item is still consumed but the level stays the same.
///   Items with a chance of `0.0` (non-compostable) always return `Failed`.
/// - Level 7: the composter automatically transitions to level 8 (ready)
///   and returns `AlreadyFull`.
/// - Level 8: the composter is ready to be harvested; returns `AlreadyFull`.
pub fn try_compost(level: &mut u8, item_id: u16, random: f32) -> CompostResult {
    if *level >= 7 {
        return CompostResult::AlreadyFull;
    }

    let chance = compost_chance(item_id);

    if chance <= 0.0 {
        return CompostResult::Failed;
    }

    if random < chance {
        *level += 1;
        if *level == 7 {
            // Transition to "ready" state.
            *level = 8;
        }
        CompostResult::Added
    } else {
        CompostResult::Failed
    }
}

/// Harvest bone meal from a composter at level 8.
///
/// Returns `Some(BONE_MEAL_ID)` when the composter is ready, resetting
/// the level to 0. Returns `None` if the level is not 8.
pub fn harvest(level: &mut u8) -> Option<u16> {
    if *level == 8 {
        *level = 0;
        Some(BONE_MEAL_ID)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- compost_chance -----------------------------------------------------

    #[test]
    fn seeds_have_30_percent_chance() {
        assert_eq!(compost_chance(ItemId::WheatSeeds as u16), 0.3);
        assert_eq!(compost_chance(ItemId::BeetrootSeeds as u16), 0.3);
        assert_eq!(compost_chance(ItemId::MelonSeeds as u16), 0.3);
        assert_eq!(compost_chance(ItemId::PumpkinSeeds as u16), 0.3);
    }

    #[test]
    fn leaves_have_30_percent_chance() {
        assert_eq!(compost_chance(ItemId::OakLeaves as u16), 0.3);
    }

    #[test]
    fn melon_has_50_percent_chance() {
        assert_eq!(compost_chance(ItemId::Melon as u16), 0.5);
    }

    #[test]
    fn food_crops_have_65_percent_chance() {
        assert_eq!(compost_chance(ItemId::Apple as u16), 0.65);
        assert_eq!(compost_chance(ItemId::Carrot as u16), 0.65);
        assert_eq!(compost_chance(ItemId::Potato as u16), 0.65);
        assert_eq!(compost_chance(ItemId::WheatItem as u16), 0.65);
        assert_eq!(compost_chance(ItemId::Beetroot as u16), 0.65);
        assert_eq!(compost_chance(ItemId::Cookie as u16), 0.65);
    }

    #[test]
    fn cooked_food_has_85_percent_chance() {
        assert_eq!(compost_chance(ItemId::Bread as u16), 0.85);
        assert_eq!(compost_chance(ItemId::BakedPotato as u16), 0.85);
        assert_eq!(compost_chance(ItemId::CookedPorkchop as u16), 0.85);
        assert_eq!(compost_chance(ItemId::CookedBeef as u16), 0.85);
        assert_eq!(compost_chance(ItemId::CookedChicken as u16), 0.85);
        assert_eq!(compost_chance(ItemId::CookedMutton as u16), 0.85);
    }

    #[test]
    fn cake_and_golden_apple_have_100_percent_chance() {
        assert_eq!(compost_chance(ItemId::Cake as u16), 1.0);
        assert_eq!(compost_chance(ItemId::GoldenApple as u16), 1.0);
    }

    #[test]
    fn unknown_item_has_zero_chance() {
        // Use an item ID that is not compostable (e.g. diamond sword).
        assert_eq!(compost_chance(ItemId::DiamondSword as u16), 0.0);
        // Completely invalid ID.
        assert_eq!(compost_chance(9999), 0.0);
    }

    // ---- try_compost --------------------------------------------------------

    #[test]
    fn compost_succeeds_when_random_below_chance() {
        let mut level = 0u8;
        let result = try_compost(&mut level, ItemId::Bread as u16, 0.1);
        assert_eq!(result, CompostResult::Added);
        assert_eq!(level, 1);
    }

    #[test]
    fn compost_fails_when_random_above_chance() {
        let mut level = 0u8;
        let result = try_compost(&mut level, ItemId::WheatSeeds as u16, 0.5);
        assert_eq!(result, CompostResult::Failed);
        assert_eq!(level, 0);
    }

    #[test]
    fn compost_fails_for_non_compostable_item() {
        let mut level = 0u8;
        let result = try_compost(&mut level, ItemId::DiamondSword as u16, 0.0);
        assert_eq!(result, CompostResult::Failed);
        assert_eq!(level, 0);
    }

    #[test]
    fn compost_100_percent_item_always_succeeds() {
        let mut level = 0u8;
        let result = try_compost(&mut level, ItemId::Cake as u16, 0.99);
        assert_eq!(result, CompostResult::Added);
        assert_eq!(level, 1);
    }

    #[test]
    fn level_progresses_from_0_to_ready() {
        let mut level = 0u8;
        // Use cake (1.0 chance) so every attempt succeeds.
        for expected in 1..=6 {
            let result = try_compost(&mut level, ItemId::Cake as u16, 0.0);
            assert_eq!(result, CompostResult::Added);
            assert_eq!(level, expected);
        }
        // The 7th successful compost jumps from 6 -> 8 (ready).
        let result = try_compost(&mut level, ItemId::Cake as u16, 0.0);
        assert_eq!(result, CompostResult::Added);
        assert_eq!(level, 8);
    }

    #[test]
    fn already_full_at_level_7() {
        let mut level = 7u8;
        let result = try_compost(&mut level, ItemId::Cake as u16, 0.0);
        assert_eq!(result, CompostResult::AlreadyFull);
        assert_eq!(level, 7);
    }

    #[test]
    fn already_full_at_level_8() {
        let mut level = 8u8;
        let result = try_compost(&mut level, ItemId::Cake as u16, 0.0);
        assert_eq!(result, CompostResult::AlreadyFull);
        assert_eq!(level, 8);
    }

    // ---- harvest ------------------------------------------------------------

    #[test]
    fn harvest_at_level_8_yields_bone_meal() {
        let mut level = 8u8;
        let result = harvest(&mut level);
        assert_eq!(result, Some(BONE_MEAL_ID));
        assert_eq!(level, 0);
    }

    #[test]
    fn harvest_at_level_below_8_yields_nothing() {
        for start_level in 0..=7 {
            let mut level = start_level;
            let result = harvest(&mut level);
            assert_eq!(result, None);
            assert_eq!(level, start_level);
        }
    }

    // ---- full cycle ---------------------------------------------------------

    #[test]
    fn full_cycle_compost_to_harvest() {
        let mut level = 0u8;
        // Fill the composter to level 8 using cake (always succeeds).
        for _ in 0..7 {
            try_compost(&mut level, ItemId::Cake as u16, 0.0);
        }
        assert_eq!(level, 8);

        // Cannot add more items.
        let result = try_compost(&mut level, ItemId::Cake as u16, 0.0);
        assert_eq!(result, CompostResult::AlreadyFull);

        // Harvest.
        let item = harvest(&mut level);
        assert_eq!(item, Some(BONE_MEAL_ID));
        assert_eq!(level, 0);

        // Can compost again after harvest.
        let result = try_compost(&mut level, ItemId::Cake as u16, 0.0);
        assert_eq!(result, CompostResult::Added);
        assert_eq!(level, 1);
    }
}
