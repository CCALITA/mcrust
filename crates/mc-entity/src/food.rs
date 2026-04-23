/// Food item data and eating mechanics.
///
/// Provides a lookup table of food items with hunger restoration, saturation,
/// and eating duration values, plus helper functions for eating logic.

// ---------------------------------------------------------------------------
// FoodItem
// ---------------------------------------------------------------------------

/// Describes the nutritional properties of a single food item.
#[derive(Debug, Clone, PartialEq)]
pub struct FoodItem {
    pub item_id: u16,
    pub hunger: u32,
    pub saturation: f32,
    pub eat_duration: f32,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Standard eating duration in seconds for all food items.
const EAT_DURATION_SECS: f32 = 1.61;

/// Total number of food items in the registry.
const FOOD_COUNT: usize = 18;

// ---------------------------------------------------------------------------
// Food data table
// ---------------------------------------------------------------------------

/// All food items, ordered by item ID (3000..=3017).
const FOOD_TABLE: [FoodItem; FOOD_COUNT] = [
    FoodItem { item_id: 3000, hunger: 4,  saturation: 2.4,  eat_duration: EAT_DURATION_SECS }, // apple
    FoodItem { item_id: 3001, hunger: 5,  saturation: 6.0,  eat_duration: EAT_DURATION_SECS }, // baked_potato
    FoodItem { item_id: 3002, hunger: 8,  saturation: 12.8, eat_duration: EAT_DURATION_SECS }, // beef_cooked
    FoodItem { item_id: 3003, hunger: 5,  saturation: 6.0,  eat_duration: EAT_DURATION_SECS }, // bread
    FoodItem { item_id: 3004, hunger: 3,  saturation: 3.6,  eat_duration: EAT_DURATION_SECS }, // carrot
    FoodItem { item_id: 3005, hunger: 6,  saturation: 7.2,  eat_duration: EAT_DURATION_SECS }, // chicken_cooked
    FoodItem { item_id: 3006, hunger: 5,  saturation: 6.0,  eat_duration: EAT_DURATION_SECS }, // cod_cooked
    FoodItem { item_id: 3007, hunger: 4,  saturation: 9.6,  eat_duration: EAT_DURATION_SECS }, // golden_apple
    FoodItem { item_id: 3008, hunger: 6,  saturation: 14.4, eat_duration: EAT_DURATION_SECS }, // golden_carrot
    FoodItem { item_id: 3009, hunger: 2,  saturation: 1.2,  eat_duration: EAT_DURATION_SECS }, // melon_slice
    FoodItem { item_id: 3010, hunger: 6,  saturation: 7.2,  eat_duration: EAT_DURATION_SECS }, // mushroom_stew
    FoodItem { item_id: 3011, hunger: 8,  saturation: 12.8, eat_duration: EAT_DURATION_SECS }, // porkchop_cooked
    FoodItem { item_id: 3012, hunger: 1,  saturation: 0.6,  eat_duration: EAT_DURATION_SECS }, // potato
    FoodItem { item_id: 3013, hunger: 8,  saturation: 4.8,  eat_duration: EAT_DURATION_SECS }, // pumpkin_pie
    FoodItem { item_id: 3014, hunger: 10, saturation: 12.0, eat_duration: EAT_DURATION_SECS }, // rabbit_stew
    FoodItem { item_id: 3015, hunger: 6,  saturation: 9.6,  eat_duration: EAT_DURATION_SECS }, // salmon_cooked
    FoodItem { item_id: 3016, hunger: 2,  saturation: 0.4,  eat_duration: EAT_DURATION_SECS }, // sweet_berries
    FoodItem { item_id: 3017, hunger: 6,  saturation: 7.2,  eat_duration: EAT_DURATION_SECS }, // suspicious_stew
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Look up food data by item ID. Returns `None` for unknown items.
pub fn food_data(item_id: u16) -> Option<FoodItem> {
    FOOD_TABLE.iter().find(|f| f.item_id == item_id).cloned()
}

/// Whether the player can eat (hunger is below the maximum).
pub fn can_eat(current_hunger: u32, max_hunger: u32) -> bool {
    current_hunger < max_hunger
}

/// Standard eating animation/consumption duration in seconds.
pub fn eat_duration() -> f32 {
    EAT_DURATION_SECS
}

/// Total number of food items in the registry.
pub fn food_count() -> usize {
    FOOD_COUNT
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Lookup for each food item ------------------------------------------

    #[test]
    fn lookup_apple() {
        let food = food_data(3000).expect("apple should exist");
        assert_eq!(food.hunger, 4);
        assert!((food.saturation - 2.4).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_baked_potato() {
        let food = food_data(3001).expect("baked_potato should exist");
        assert_eq!(food.hunger, 5);
        assert!((food.saturation - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_beef_cooked() {
        let food = food_data(3002).expect("beef_cooked should exist");
        assert_eq!(food.hunger, 8);
        assert!((food.saturation - 12.8).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_bread() {
        let food = food_data(3003).expect("bread should exist");
        assert_eq!(food.hunger, 5);
        assert!((food.saturation - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_carrot() {
        let food = food_data(3004).expect("carrot should exist");
        assert_eq!(food.hunger, 3);
        assert!((food.saturation - 3.6).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_chicken_cooked() {
        let food = food_data(3005).expect("chicken_cooked should exist");
        assert_eq!(food.hunger, 6);
        assert!((food.saturation - 7.2).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_cod_cooked() {
        let food = food_data(3006).expect("cod_cooked should exist");
        assert_eq!(food.hunger, 5);
        assert!((food.saturation - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_golden_apple() {
        let food = food_data(3007).expect("golden_apple should exist");
        assert_eq!(food.hunger, 4);
        assert!((food.saturation - 9.6).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_golden_carrot() {
        let food = food_data(3008).expect("golden_carrot should exist");
        assert_eq!(food.hunger, 6);
        assert!((food.saturation - 14.4).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_melon_slice() {
        let food = food_data(3009).expect("melon_slice should exist");
        assert_eq!(food.hunger, 2);
        assert!((food.saturation - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_mushroom_stew() {
        let food = food_data(3010).expect("mushroom_stew should exist");
        assert_eq!(food.hunger, 6);
        assert!((food.saturation - 7.2).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_porkchop_cooked() {
        let food = food_data(3011).expect("porkchop_cooked should exist");
        assert_eq!(food.hunger, 8);
        assert!((food.saturation - 12.8).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_potato() {
        let food = food_data(3012).expect("potato should exist");
        assert_eq!(food.hunger, 1);
        assert!((food.saturation - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_pumpkin_pie() {
        let food = food_data(3013).expect("pumpkin_pie should exist");
        assert_eq!(food.hunger, 8);
        assert!((food.saturation - 4.8).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_rabbit_stew() {
        let food = food_data(3014).expect("rabbit_stew should exist");
        assert_eq!(food.hunger, 10);
        assert!((food.saturation - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_salmon_cooked() {
        let food = food_data(3015).expect("salmon_cooked should exist");
        assert_eq!(food.hunger, 6);
        assert!((food.saturation - 9.6).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_sweet_berries() {
        let food = food_data(3016).expect("sweet_berries should exist");
        assert_eq!(food.hunger, 2);
        assert!((food.saturation - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn lookup_suspicious_stew() {
        let food = food_data(3017).expect("suspicious_stew should exist");
        assert_eq!(food.hunger, 6);
        assert!((food.saturation - 7.2).abs() < f32::EPSILON);
    }

    // -- Unknown item -------------------------------------------------------

    #[test]
    fn unknown_item_returns_none() {
        assert!(food_data(0).is_none());
        assert!(food_data(9999).is_none());
        assert!(food_data(2999).is_none());
        assert!(food_data(3018).is_none());
    }

    // -- can_eat logic ------------------------------------------------------

    #[test]
    fn can_eat_when_hungry() {
        assert!(can_eat(10, 20));
    }

    #[test]
    fn can_eat_when_nearly_full() {
        assert!(can_eat(19, 20));
    }

    #[test]
    fn cannot_eat_when_full() {
        assert!(!can_eat(20, 20));
    }

    #[test]
    fn can_eat_when_starving() {
        assert!(can_eat(0, 20));
    }

    // -- eat_duration -------------------------------------------------------

    #[test]
    fn eat_duration_is_correct() {
        assert!((eat_duration() - 1.61).abs() < f32::EPSILON);
    }

    // -- food_count ---------------------------------------------------------

    #[test]
    fn food_count_matches_table() {
        assert_eq!(food_count(), 18);
    }

    // -- All items have correct eat_duration --------------------------------

    #[test]
    fn all_items_have_standard_eat_duration() {
        for id in 3000..=3017 {
            let food = food_data(id).unwrap_or_else(|| panic!("item {id} should exist"));
            assert!(
                (food.eat_duration - 1.61).abs() < f32::EPSILON,
                "item {id} should have eat_duration 1.61, got {}",
                food.eat_duration
            );
        }
    }
}
