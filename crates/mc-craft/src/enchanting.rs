use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::SlotItem;

// Re-export data types so existing consumers of `enchanting::*` still work.
pub use crate::enchantment_data::{
    Enchantment, EnchantmentCategory, EnchantmentId, EnchantmentProperties, ENCHANTMENT_REGISTRY,
};

/// An item with enchantments applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantedItem {
    pub item_id: SlotItem,
    pub enchantments: Vec<Enchantment>,
}

impl EnchantedItem {
    /// Create an enchanted item with no enchantments.
    #[must_use]
    pub fn new(item_id: SlotItem) -> Self {
        Self { item_id, enchantments: Vec::new() }
    }

    /// Try to add an enchantment. Returns `false` if it is incompatible with
    /// an existing enchantment or already present.
    pub fn add_enchantment(&mut self, enchantment: Enchantment) -> bool {
        let new_props = enchantment.id.properties();
        for existing in &self.enchantments {
            if existing.id == enchantment.id {
                return false;
            }
            if new_props.incompatible_with.contains(&existing.id) {
                return false;
            }
            if existing.id.properties().incompatible_with.contains(&enchantment.id) {
                return false;
            }
        }
        self.enchantments.push(enchantment);
        true
    }

    /// Check whether the item has a specific enchantment, returning its level.
    #[must_use]
    pub fn enchantment_level(&self, id: EnchantmentId) -> Option<u8> {
        self.enchantments.iter().find(|e| e.id == id).map(|e| e.level)
    }
}

/// One of the three options presented on the enchanting table UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantOption {
    pub cost: u8,
    pub enchantments: Vec<Enchantment>,
    pub description: String,
}

fn deterministic_hash(seed: u64, extra: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    extra.hash(&mut hasher);
    hasher.finish()
}

/// Calculate the three-tier enchantment costs based on the number of bookshelves.
///
/// `bookshelves` is clamped to 0..=15. Returns `[tier1, tier2, tier3]` XP level costs.
#[must_use]
pub fn calculate_enchantment_cost(bookshelves: u8, seed: u64) -> [u8; 3] {
    let shelves = bookshelves.min(15) as u64;
    let base = shelves / 2;

    let r1 = (deterministic_hash(seed, 1) % 8) + 1;
    let tier1 = 1u64.max(r1 + base);

    let r2 = deterministic_hash(seed, 2) % 9;
    let raw2 = tier1 + r2 + base;
    let min2 = tier1 + 1;
    let max2 = (2 * shelves + 1).max(min2);
    let tier2 = raw2.clamp(min2, max2).min(29);

    let r3 = deterministic_hash(seed, 3) % 9;
    let raw3 = tier2 + r3 + base;
    let min3 = tier2 + 1;
    let max3 = (3 * shelves + 1).max(min3);
    let tier3 = raw3.clamp(min3, max3).min(30);

    [tier1 as u8, tier2 as u8, tier3 as u8]
}

/// Determine which enchantment categories apply to a given item type.
fn applicable_categories(item_type: u16) -> Vec<EnchantmentCategory> {
    match item_type {
        203 | 213 | 223 | 233 => vec![EnchantmentCategory::Weapon, EnchantmentCategory::General],
        200..=232 => vec![EnchantmentCategory::Tool, EnchantmentCategory::General],
        _ => vec![EnchantmentCategory::General],
    }
}

/// Collect all enchantments applicable to the given item type.
fn applicable_enchantments(item_type: u16) -> Vec<EnchantmentId> {
    let categories = applicable_categories(item_type);
    EnchantmentId::ALL
        .iter()
        .copied()
        .filter(|id: &EnchantmentId| categories.contains(&id.properties().category))
        .collect()
}

/// Generate three enchantment options for the enchanting table UI.
#[must_use]
pub fn generate_enchantment_options(
    item_type: u16,
    bookshelves: u8,
    seed: u64,
) -> [EnchantOption; 3] {
    let costs = calculate_enchantment_cost(bookshelves, seed);
    let pool = applicable_enchantments(item_type);

    let build_option = |tier: usize, cost: u8| -> EnchantOption {
        if pool.is_empty() {
            return EnchantOption {
                cost,
                enchantments: Vec::new(),
                description: String::from("No applicable enchantments"),
            };
        }
        let enchant_count = match cost {
            0..=5 => 1usize,
            6..=15 => 2,
            _ => 3,
        };
        let mut selected: Vec<Enchantment> = Vec::new();
        for i in 0..enchant_count.min(pool.len()) {
            let hash = deterministic_hash(seed, (tier as u64 + 1) * 100 + i as u64);
            let idx = (hash as usize) % pool.len();
            let candidate = pool[idx];

            let dominated = selected.iter().any(|s| {
                s.id == candidate
                    || s.id.properties().incompatible_with.contains(&candidate)
                    || candidate.properties().incompatible_with.contains(&s.id)
            });

            if dominated {
                let fallback_idx = (idx + 1) % pool.len();
                let fallback = pool[fallback_idx];
                let still_bad = selected.iter().any(|s| {
                    s.id == fallback
                        || s.id.properties().incompatible_with.contains(&fallback)
                        || fallback.properties().incompatible_with.contains(&s.id)
                });
                if !still_bad {
                    let level_hash = deterministic_hash(seed, (tier as u64 + 1) * 200 + i as u64);
                    let max_lvl = fallback.properties().max_level as u64;
                    let level = ((level_hash % max_lvl) + 1) as u8;
                    selected.push(Enchantment::new(fallback, level));
                }
            } else {
                let level_hash = deterministic_hash(seed, (tier as u64 + 1) * 200 + i as u64);
                let max_lvl = candidate.properties().max_level as u64;
                let level = ((level_hash % max_lvl) + 1) as u8;
                selected.push(Enchantment::new(candidate, level));
            }
        }
        let description = selected
            .iter()
            .map(|e| format!("{} {}", e.id.properties().name, roman_numeral(e.level)))
            .collect::<Vec<_>>()
            .join(", ");
        EnchantOption { cost, enchantments: selected, description }
    };

    [build_option(0, costs[0]), build_option(1, costs[1]), build_option(2, costs[2])]
}

fn roman_numeral(n: u8) -> &'static str {
    match n {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        _ => "?",
    }
}

/// Apply an enchantment's effect to a base value and return the modified value.
#[must_use]
pub fn apply_enchantment_effect(enchantment: &Enchantment, base_value: f32) -> f32 {
    let level = enchantment.level as f32;
    match enchantment.id {
        EnchantmentId::Sharpness | EnchantmentId::Smite | EnchantmentId::BaneOfArthropods => {
            base_value + 1.25 * level
        }
        EnchantmentId::Protection
        | EnchantmentId::FireProtection
        | EnchantmentId::BlastProtection
        | EnchantmentId::ProjectileProtection => base_value * (1.0 - 0.04 * level),
        EnchantmentId::Efficiency => base_value + level * level + 1.0,
        EnchantmentId::Unbreaking => base_value / (level + 1.0),
        EnchantmentId::Power => base_value + 0.5 * (level + 1.0),
        EnchantmentId::Knockback | EnchantmentId::Punch => base_value + 0.5 * level,
        _ => base_value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_add_duplicate_enchantment() {
        let mut item = EnchantedItem::new(200);
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 3)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 5)));
    }

    #[test]
    fn cannot_add_incompatible_enchantment() {
        let mut item = EnchantedItem::new(203);
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Sharpness, 5)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Smite, 3)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::BaneOfArthropods, 3)));
    }

    #[test]
    fn can_add_compatible_enchantments() {
        let mut item = EnchantedItem::new(200);
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 3)));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Unbreaking, 3)));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Fortune, 2)));
        assert_eq!(item.enchantments.len(), 3);
    }

    #[test]
    fn silk_touch_and_fortune_are_incompatible() {
        let mut item = EnchantedItem::new(200);
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::SilkTouch, 1)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Fortune, 3)));
    }

    #[test]
    fn infinity_and_mending_are_incompatible() {
        let mut item = EnchantedItem::new(200);
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Infinity, 1)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Mending, 1)));
    }

    #[test]
    fn depth_strider_and_frost_walker_are_incompatible() {
        let mut item = EnchantedItem::new(200);
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::DepthStrider, 3)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::FrostWalker, 2)));
    }

    #[test]
    fn enchantment_level_query() {
        let mut item = EnchantedItem::new(200);
        item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 4));
        assert_eq!(item.enchantment_level(EnchantmentId::Efficiency), Some(4));
        assert_eq!(item.enchantment_level(EnchantmentId::Fortune), None);
    }

    #[test]
    fn cost_tiers_are_strictly_increasing() {
        for seed in 0..100u64 {
            let c = calculate_enchantment_cost(15, seed);
            assert!(c[0] < c[1] && c[1] < c[2], "seed {seed}: {c:?} not increasing");
        }
    }

    #[test]
    fn cost_tier1_is_at_least_1() {
        for seed in 0..100u64 {
            let c = calculate_enchantment_cost(0, seed);
            assert!(c[0] >= 1, "seed {seed}: tier1 = {}", c[0]);
        }
    }

    #[test]
    fn cost_tier3_is_at_most_30() {
        for seed in 0..200u64 {
            let c = calculate_enchantment_cost(15, seed);
            assert!(c[2] <= 30, "seed {seed}: tier3 = {} exceeds 30", c[2]);
        }
    }

    #[test]
    fn zero_bookshelves_produces_low_costs() {
        for seed in 0..50u64 {
            let c = calculate_enchantment_cost(0, seed);
            assert!(c[0] <= 8, "seed {seed}: tier1 = {}", c[0]);
            assert!(c[2] <= 25, "seed {seed}: tier3 = {}", c[2]);
        }
    }

    #[test]
    fn bookshelves_clamped_to_15() {
        assert_eq!(
            calculate_enchantment_cost(15, 42),
            calculate_enchantment_cost(30, 42),
        );
    }

    #[test]
    fn deterministic_cost_same_seed() {
        assert_eq!(
            calculate_enchantment_cost(10, 12345),
            calculate_enchantment_cost(10, 12345),
        );
    }

    #[test]
    fn sharpness_adds_damage() {
        let e = Enchantment::new(EnchantmentId::Sharpness, 3);
        assert!((apply_enchantment_effect(&e, 7.0) - 10.75).abs() < f32::EPSILON);
    }

    #[test]
    fn protection_reduces_damage() {
        let e = Enchantment::new(EnchantmentId::Protection, 4);
        assert!((apply_enchantment_effect(&e, 100.0) - 84.0).abs() < f32::EPSILON);
    }

    #[test]
    fn efficiency_boosts_mining_speed() {
        let e = Enchantment::new(EnchantmentId::Efficiency, 5);
        assert!((apply_enchantment_effect(&e, 8.0) - 34.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unbreaking_reduces_durability_usage() {
        let e = Enchantment::new(EnchantmentId::Unbreaking, 3);
        assert!((apply_enchantment_effect(&e, 1.0) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn power_adds_bow_damage() {
        let e = Enchantment::new(EnchantmentId::Power, 5);
        assert!((apply_enchantment_effect(&e, 9.0) - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn knockback_adds_distance() {
        let e = Enchantment::new(EnchantmentId::Knockback, 2);
        assert!((apply_enchantment_effect(&e, 3.0) - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unknown_effect_returns_base() {
        let e = Enchantment::new(EnchantmentId::AquaAffinity, 1);
        assert!((apply_enchantment_effect(&e, 10.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn generate_options_returns_three() {
        assert_eq!(generate_enchantment_options(200, 15, 42).len(), 3);
    }

    #[test]
    fn generate_options_costs_match_tier_costs() {
        let costs = calculate_enchantment_cost(15, 42);
        let opts = generate_enchantment_options(200, 15, 42);
        assert_eq!(opts[0].cost, costs[0]);
        assert_eq!(opts[1].cost, costs[1]);
        assert_eq!(opts[2].cost, costs[2]);
    }

    #[test]
    fn generate_options_deterministic() {
        assert_eq!(
            generate_enchantment_options(200, 10, 999),
            generate_enchantment_options(200, 10, 999),
        );
    }

    #[test]
    fn generate_options_each_has_at_least_one_enchantment() {
        for opt in &generate_enchantment_options(200, 15, 42) {
            assert!(!opt.enchantments.is_empty(), "cost {} has no enchantments", opt.cost);
        }
    }

    #[test]
    fn generate_options_descriptions_are_nonempty() {
        for opt in &generate_enchantment_options(200, 15, 42) {
            assert!(!opt.description.is_empty(), "cost {} has empty description", opt.cost);
        }
    }

    #[test]
    fn generate_options_no_duplicate_enchantments_in_option() {
        for seed in 0..50u64 {
            for opt in &generate_enchantment_options(203, 15, seed) {
                let ids: Vec<EnchantmentId> = opt.enchantments.iter().map(|e| e.id).collect();
                for (i, id) in ids.iter().enumerate() {
                    assert!(!ids[i + 1..].contains(id), "seed {seed}: duplicate {:?}", id);
                }
            }
        }
    }

    #[test]
    fn roman_numerals() {
        assert_eq!(roman_numeral(1), "I");
        assert_eq!(roman_numeral(2), "II");
        assert_eq!(roman_numeral(3), "III");
        assert_eq!(roman_numeral(4), "IV");
        assert_eq!(roman_numeral(5), "V");
        assert_eq!(roman_numeral(0), "?");
        assert_eq!(roman_numeral(6), "?");
    }

    #[test]
    fn sword_gets_weapon_and_general_enchantments() {
        let pool = applicable_enchantments(203);
        assert!(pool.contains(&EnchantmentId::Sharpness));
        assert!(pool.contains(&EnchantmentId::Mending));
        assert!(!pool.contains(&EnchantmentId::Efficiency));
    }

    #[test]
    fn pickaxe_gets_tool_and_general_enchantments() {
        let pool = applicable_enchantments(200);
        assert!(pool.contains(&EnchantmentId::Efficiency));
        assert!(pool.contains(&EnchantmentId::Unbreaking));
        assert!(pool.contains(&EnchantmentId::Mending));
        assert!(!pool.contains(&EnchantmentId::Sharpness));
    }

    #[test]
    fn non_tool_gets_only_general_enchantments() {
        let pool = applicable_enchantments(100);
        assert!(pool.contains(&EnchantmentId::Mending));
        assert!(pool.contains(&EnchantmentId::CurseOfVanishing));
        assert!(!pool.contains(&EnchantmentId::Efficiency));
        assert!(!pool.contains(&EnchantmentId::Sharpness));
    }

    #[test]
    fn protection_variants_are_mutually_exclusive() {
        let protections = [
            EnchantmentId::Protection,
            EnchantmentId::FireProtection,
            EnchantmentId::BlastProtection,
            EnchantmentId::ProjectileProtection,
        ];
        for (i, &a) in protections.iter().enumerate() {
            for (j, &b) in protections.iter().enumerate() {
                if i == j {
                    continue;
                }
                let mut item = EnchantedItem::new(200);
                item.add_enchantment(Enchantment::new(a, 1));
                assert!(
                    !item.add_enchantment(Enchantment::new(b, 1)),
                    "{:?} and {:?} should be incompatible",
                    a,
                    b
                );
            }
        }
    }
}
