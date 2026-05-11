//! Cat morning gift mechanics.
//!
//! When a tamed cat sleeps with the player, it has a 70% chance to bring a gift item.

/// Items a cat can bring as a morning gift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatGiftItem {
    RawCod,
    RawSalmon,
    RabbitHide,
    RabbitFoot,
    StringItem,
    RottenFlesh,
    Feather,
    PhantomMembrane,
}

/// Returns the weight for the given gift item in the loot table.
pub fn cat_gift_weight(item: CatGiftItem) -> f32 {
    match item {
        CatGiftItem::RawCod => 0.1661,
        CatGiftItem::RawSalmon => 0.1661,
        CatGiftItem::RabbitHide => 0.1661,
        CatGiftItem::RabbitFoot => 0.1661,
        CatGiftItem::StringItem => 0.1661,
        CatGiftItem::RottenFlesh => 0.1661,
        CatGiftItem::Feather => 0.0069,
        CatGiftItem::PhantomMembrane => 0.0069,
    }
}

/// Selects a gift item using a weighted random selection based on the given seed.
pub fn select_cat_gift(seed: u64) -> CatGiftItem {
    const ITEMS: [CatGiftItem; 8] = [
        CatGiftItem::RawCod,
        CatGiftItem::RawSalmon,
        CatGiftItem::RabbitHide,
        CatGiftItem::RabbitFoot,
        CatGiftItem::StringItem,
        CatGiftItem::RottenFlesh,
        CatGiftItem::Feather,
        CatGiftItem::PhantomMembrane,
    ];

    let total_weight: f32 = ITEMS.iter().map(|i| cat_gift_weight(*i)).sum();
    // Simple hash to get a value in [0, 1)
    let hash = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let normalized = (hash as f64 / u64::MAX as f64) as f32;
    let target = normalized * total_weight;

    let mut cumulative = 0.0;
    for item in &ITEMS {
        cumulative += cat_gift_weight(*item);
        if target < cumulative {
            return *item;
        }
    }

    CatGiftItem::PhantomMembrane
}

/// Returns the chance (0.0 to 1.0) that a cat will give a gift in the morning.
pub fn cat_gives_gift_chance() -> f32 {
    0.7
}

/// Returns the item ID for a cat gift item.
pub fn cat_gift_item_id(item: CatGiftItem) -> u16 {
    match item {
        CatGiftItem::RawCod => 800,
        CatGiftItem::RawSalmon => 801,
        CatGiftItem::RabbitHide => 802,
        CatGiftItem::RabbitFoot => 803,
        CatGiftItem::StringItem => 804,
        CatGiftItem::RottenFlesh => 805,
        CatGiftItem::Feather => 806,
        CatGiftItem::PhantomMembrane => 807,
    }
}

/// Returns the total number of distinct gift types.
pub fn total_gift_types() -> usize {
    8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_gift_types() {
        assert_eq!(total_gift_types(), 8);
    }

    #[test]
    fn test_cat_gives_gift_chance() {
        assert!((cat_gives_gift_chance() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cat_gift_weight_common_items() {
        assert!((cat_gift_weight(CatGiftItem::RawCod) - 0.1661).abs() < f32::EPSILON);
        assert!((cat_gift_weight(CatGiftItem::RawSalmon) - 0.1661).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cat_gift_weight_rare_items() {
        assert!((cat_gift_weight(CatGiftItem::Feather) - 0.0069).abs() < f32::EPSILON);
        assert!((cat_gift_weight(CatGiftItem::PhantomMembrane) - 0.0069).abs() < f32::EPSILON);
    }

    #[test]
    fn test_select_cat_gift_deterministic() {
        let gift1 = select_cat_gift(42);
        let gift2 = select_cat_gift(42);
        assert_eq!(gift1, gift2);
    }

    #[test]
    fn test_select_cat_gift_varies_with_seed() {
        // With enough different seeds we should get at least 2 distinct items
        let mut seen = std::collections::HashSet::new();
        for seed in 0..100 {
            seen.insert(select_cat_gift(seed));
        }
        assert!(seen.len() >= 2);
    }

    #[test]
    fn test_cat_gift_item_id_unique() {
        let items = [
            CatGiftItem::RawCod,
            CatGiftItem::RawSalmon,
            CatGiftItem::RabbitHide,
            CatGiftItem::RabbitFoot,
            CatGiftItem::StringItem,
            CatGiftItem::RottenFlesh,
            CatGiftItem::Feather,
            CatGiftItem::PhantomMembrane,
        ];
        let ids: std::collections::HashSet<u16> = items.iter().map(|i| cat_gift_item_id(*i)).collect();
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn test_weights_sum_to_approximately_one() {
        let items = [
            CatGiftItem::RawCod,
            CatGiftItem::RawSalmon,
            CatGiftItem::RabbitHide,
            CatGiftItem::RabbitFoot,
            CatGiftItem::StringItem,
            CatGiftItem::RottenFlesh,
            CatGiftItem::Feather,
            CatGiftItem::PhantomMembrane,
        ];
        let total: f32 = items.iter().map(|i| cat_gift_weight(*i)).sum();
        assert!((total - 1.0).abs() < 0.02);
    }
}
