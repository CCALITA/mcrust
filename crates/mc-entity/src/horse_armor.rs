//! Horse armor types, protection values, and item ID mappings.

/// Represents the type of armor a horse can wear.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HorseArmorType {
    None,
    Iron,
    Gold,
    Diamond,
    Leather(u8),
}

/// Returns the protection value for the given horse armor type.
pub fn horse_armor_protection(armor: HorseArmorType) -> f32 {
    match armor {
        HorseArmorType::None => 0.0,
        HorseArmorType::Iron => 5.0,
        HorseArmorType::Gold => 7.0,
        HorseArmorType::Diamond => 11.0,
        HorseArmorType::Leather(_) => 3.0,
    }
}

/// Applies horse armor damage reduction to incoming damage.
///
/// Formula: `damage * (1.0 - protection * 0.04)`
pub fn apply_horse_armor_reduction(damage: f32, armor: HorseArmorType) -> f32 {
    let protection = horse_armor_protection(armor);
    damage * (1.0 - protection * 0.04)
}

/// Returns the item ID for the given horse armor type.
pub fn horse_armor_item_id(armor: HorseArmorType) -> u16 {
    match armor {
        HorseArmorType::None => 0,
        HorseArmorType::Iron => 8100,
        HorseArmorType::Gold => 8101,
        HorseArmorType::Diamond => 8102,
        HorseArmorType::Leather(_) => 8103,
    }
}

/// Creates a leather horse armor with the given dye color.
pub fn dyeable_leather_horse_armor(color: u8) -> HorseArmorType {
    HorseArmorType::Leather(color)
}

/// Returns whether the given item ID corresponds to a horse armor item.
pub fn is_horse_armor_item(item_id: u16) -> bool {
    (8100..=8103).contains(&item_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protection_values() {
        assert_eq!(horse_armor_protection(HorseArmorType::None), 0.0);
        assert_eq!(horse_armor_protection(HorseArmorType::Iron), 5.0);
        assert_eq!(horse_armor_protection(HorseArmorType::Gold), 7.0);
        assert_eq!(horse_armor_protection(HorseArmorType::Diamond), 11.0);
        assert_eq!(horse_armor_protection(HorseArmorType::Leather(0)), 3.0);
        assert_eq!(horse_armor_protection(HorseArmorType::Leather(255)), 3.0);
    }

    const TOLERANCE: f32 = 1e-5;

    #[test]
    fn damage_reduction_none() {
        let result = apply_horse_armor_reduction(20.0, HorseArmorType::None);
        assert!((result - 20.0).abs() < TOLERANCE);
    }

    #[test]
    fn damage_reduction_iron() {
        // 20.0 * (1.0 - 5.0 * 0.04) = 20.0 * 0.8 = 16.0
        let result = apply_horse_armor_reduction(20.0, HorseArmorType::Iron);
        assert!((result - 16.0).abs() < TOLERANCE);
    }

    #[test]
    fn damage_reduction_gold() {
        // 20.0 * (1.0 - 7.0 * 0.04) = 20.0 * 0.72 = 14.4
        let result = apply_horse_armor_reduction(20.0, HorseArmorType::Gold);
        assert!((result - 14.4).abs() < TOLERANCE);
    }

    #[test]
    fn damage_reduction_diamond() {
        // 20.0 * (1.0 - 11.0 * 0.04) = 20.0 * 0.56 = 11.2
        let result = apply_horse_armor_reduction(20.0, HorseArmorType::Diamond);
        assert!((result - 11.2).abs() < TOLERANCE);
    }

    #[test]
    fn damage_reduction_leather() {
        // 20.0 * (1.0 - 3.0 * 0.04) = 20.0 * 0.88 = 17.6
        let result = apply_horse_armor_reduction(20.0, HorseArmorType::Leather(42));
        assert!((result - 17.6).abs() < TOLERANCE);
    }

    #[test]
    fn damage_reduction_zero_damage() {
        let result = apply_horse_armor_reduction(0.0, HorseArmorType::Diamond);
        assert!((result - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn item_ids() {
        assert_eq!(horse_armor_item_id(HorseArmorType::None), 0);
        assert_eq!(horse_armor_item_id(HorseArmorType::Iron), 8100);
        assert_eq!(horse_armor_item_id(HorseArmorType::Gold), 8101);
        assert_eq!(horse_armor_item_id(HorseArmorType::Diamond), 8102);
        assert_eq!(horse_armor_item_id(HorseArmorType::Leather(0)), 8103);
    }

    #[test]
    fn dyeable_leather_creates_correct_variant() {
        let armor = dyeable_leather_horse_armor(128);
        assert_eq!(armor, HorseArmorType::Leather(128));
    }

    #[test]
    fn is_horse_armor_item_valid_ids() {
        assert!(is_horse_armor_item(8100));
        assert!(is_horse_armor_item(8101));
        assert!(is_horse_armor_item(8102));
        assert!(is_horse_armor_item(8103));
    }

    #[test]
    fn is_horse_armor_item_invalid_ids() {
        assert!(!is_horse_armor_item(0));
        assert!(!is_horse_armor_item(8099));
        assert!(!is_horse_armor_item(8104));
        assert!(!is_horse_armor_item(u16::MAX));
    }
}
