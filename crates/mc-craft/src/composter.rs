//! Composter block: converts compostable items into bone meal.

/// Maximum compost level before the composter is ready to harvest.
pub const MAX_LEVEL: u8 = 7;

/// Bone meal item ID.
const BONE_MEAL_ID: u16 = 1000;

/// Composter block state tracking the current compost level (0–7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComposterState {
    pub level: u8,
}

impl ComposterState {
    /// Create an empty composter.
    pub fn new() -> Self {
        Self { level: 0 }
    }
}

/// Returns the compost chance for an item (0.0 if not compostable).
///
/// Tiers follow vanilla Minecraft probabilities:
/// - 30%: seeds, leaves, saplings, grass
/// - 50%: flowers, small plants, dried kelp
/// - 65%: food items, mushroom blocks, vines
/// - 85%: baked goods, pumpkin/melon, hay bale
/// - 100%: cake, pumpkin pie
pub fn compost_chance(item_id: u16) -> f32 {
    match item_id {
        // 30% tier — seeds, leaves, saplings, grass
        100..=119 => 0.30,
        // 50% tier — flowers, small plants, dried kelp
        120..=139 => 0.50,
        // 65% tier — food items, mushroom blocks, vines
        140..=159 => 0.65,
        // 85% tier — baked goods, pumpkin/melon, hay bale
        160..=179 => 0.85,
        // 100% tier — cake, pumpkin pie
        180..=189 => 1.00,
        _ => 0.0,
    }
}

/// Returns `true` if the item can be composted.
pub fn is_compostable(item_id: u16) -> bool {
    compost_chance(item_id) > 0.0
}

/// Attempt to add an item to the composter.
///
/// Uses `seed` as a deterministic random value (0.0–1.0 range derived externally)
/// to decide whether the composting succeeds based on the item's chance.
///
/// Returns a new `ComposterState` and `true` if the level increased, or the
/// unchanged state and `false` otherwise.
pub fn add_item(state: ComposterState, item_id: u16, seed: f32) -> (ComposterState, bool) {
    let chance = compost_chance(item_id);
    if chance == 0.0 || state.level >= MAX_LEVEL {
        return (state, false);
    }
    if seed < chance {
        let new_state = ComposterState {
            level: state.level + 1,
        };
        (new_state, true)
    } else {
        (state, false)
    }
}

/// Harvest bone meal from a full composter (level 7).
///
/// Returns `Some(bone_meal_id)` and an empty composter if ready,
/// or `None` and the unchanged state if not at max level.
pub fn harvest(state: ComposterState) -> (ComposterState, Option<u16>) {
    if state.level == MAX_LEVEL {
        (ComposterState::new(), Some(BONE_MEAL_ID))
    } else {
        (state, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_composter_is_empty() {
        let state = ComposterState::new();
        assert_eq!(state.level, 0);
    }

    #[test]
    fn compost_chance_returns_correct_tiers() {
        assert_eq!(compost_chance(100), 0.30);
        assert_eq!(compost_chance(119), 0.30);
        assert_eq!(compost_chance(120), 0.50);
        assert_eq!(compost_chance(140), 0.65);
        assert_eq!(compost_chance(160), 0.85);
        assert_eq!(compost_chance(180), 1.00);
    }

    #[test]
    fn compost_chance_zero_for_unknown_items() {
        assert_eq!(compost_chance(0), 0.0);
        assert_eq!(compost_chance(999), 0.0);
    }

    #[test]
    fn is_compostable_matches_chance() {
        assert!(is_compostable(100));
        assert!(is_compostable(180));
        assert!(!is_compostable(0));
        assert!(!is_compostable(999));
    }

    #[test]
    fn add_item_succeeds_with_low_seed() {
        let state = ComposterState::new();
        let (new_state, success) = add_item(state, 100, 0.1); // 0.1 < 0.30
        assert!(success);
        assert_eq!(new_state.level, 1);
    }

    #[test]
    fn add_item_fails_with_high_seed() {
        let state = ComposterState::new();
        let (new_state, success) = add_item(state, 100, 0.5); // 0.5 >= 0.30
        assert!(!success);
        assert_eq!(new_state.level, 0);
    }

    #[test]
    fn add_item_always_succeeds_for_100_percent_tier() {
        let state = ComposterState::new();
        let (new_state, success) = add_item(state, 180, 0.99);
        assert!(success);
        assert_eq!(new_state.level, 1);
    }

    #[test]
    fn add_item_rejects_non_compostable() {
        let state = ComposterState::new();
        let (new_state, success) = add_item(state, 999, 0.0);
        assert!(!success);
        assert_eq!(new_state.level, 0);
    }

    #[test]
    fn add_item_rejects_when_full() {
        let state = ComposterState { level: MAX_LEVEL };
        let (new_state, success) = add_item(state, 180, 0.0);
        assert!(!success);
        assert_eq!(new_state.level, MAX_LEVEL);
    }

    #[test]
    fn harvest_at_max_level_returns_bone_meal() {
        let state = ComposterState { level: MAX_LEVEL };
        let (new_state, item) = harvest(state);
        assert_eq!(item, Some(1000));
        assert_eq!(new_state.level, 0);
    }

    #[test]
    fn harvest_below_max_returns_none() {
        let state = ComposterState { level: 5 };
        let (new_state, item) = harvest(state);
        assert_eq!(item, None);
        assert_eq!(new_state.level, 5);
    }

    #[test]
    fn full_composting_cycle() {
        let mut state = ComposterState::new();
        // Fill to max using 100% tier items
        for _ in 0..MAX_LEVEL {
            let (new_state, success) = add_item(state, 180, 0.0);
            assert!(success);
            state = new_state;
        }
        assert_eq!(state.level, MAX_LEVEL);

        let (new_state, item) = harvest(state);
        assert_eq!(item, Some(1000));
        assert_eq!(new_state.level, 0);
    }
}
