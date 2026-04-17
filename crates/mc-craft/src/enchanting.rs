use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::SlotItem;

// ── Enchantment ID ─────────────────────────────────────────────────────────

/// All enchantment types available in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnchantmentId {
    // Armor
    Protection,
    FireProtection,
    BlastProtection,
    ProjectileProtection,
    Thorns,
    Respiration,
    AquaAffinity,
    DepthStrider,
    FrostWalker,
    // Weapon
    Sharpness,
    Smite,
    BaneOfArthropods,
    Knockback,
    FireAspect,
    Looting,
    SweepingEdge,
    // Tool
    Efficiency,
    SilkTouch,
    Fortune,
    Unbreaking,
    // Bow
    Power,
    Punch,
    Flame,
    Infinity,
    // General
    Mending,
    CurseOfVanishing,
}

impl EnchantmentId {
    /// All enchantment variants, in declaration order.
    pub const ALL: [EnchantmentId; 26] = [
        EnchantmentId::Protection,
        EnchantmentId::FireProtection,
        EnchantmentId::BlastProtection,
        EnchantmentId::ProjectileProtection,
        EnchantmentId::Thorns,
        EnchantmentId::Respiration,
        EnchantmentId::AquaAffinity,
        EnchantmentId::DepthStrider,
        EnchantmentId::FrostWalker,
        EnchantmentId::Sharpness,
        EnchantmentId::Smite,
        EnchantmentId::BaneOfArthropods,
        EnchantmentId::Knockback,
        EnchantmentId::FireAspect,
        EnchantmentId::Looting,
        EnchantmentId::SweepingEdge,
        EnchantmentId::Efficiency,
        EnchantmentId::SilkTouch,
        EnchantmentId::Fortune,
        EnchantmentId::Unbreaking,
        EnchantmentId::Power,
        EnchantmentId::Punch,
        EnchantmentId::Flame,
        EnchantmentId::Infinity,
        EnchantmentId::Mending,
        EnchantmentId::CurseOfVanishing,
    ];

    /// Get the static properties for this enchantment.
    #[must_use]
    pub fn properties(self) -> &'static EnchantmentProperties {
        &ENCHANTMENT_REGISTRY[self as usize]
    }
}

// ── Enchantment Category ───────────────────────────────────────────────────

/// Broad category used to determine which items an enchantment can apply to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnchantmentCategory {
    Armor,
    Weapon,
    Tool,
    Bow,
    General,
}

// ── Enchantment Properties ─────────────────────────────────────────────────

/// Static data describing a single enchantment kind.
#[derive(Debug, Clone)]
pub struct EnchantmentProperties {
    pub name: &'static str,
    pub max_level: u8,
    pub category: EnchantmentCategory,
    pub incompatible_with: &'static [EnchantmentId],
}

// ── Static registry ────────────────────────────────────────────────────────

static ENCHANTMENT_REGISTRY: [EnchantmentProperties; 26] = [
    // Armor
    EnchantmentProperties {
        name: "Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::FireProtection,
            EnchantmentId::BlastProtection,
            EnchantmentId::ProjectileProtection,
        ],
    },
    EnchantmentProperties {
        name: "Fire Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::Protection,
            EnchantmentId::BlastProtection,
            EnchantmentId::ProjectileProtection,
        ],
    },
    EnchantmentProperties {
        name: "Blast Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::Protection,
            EnchantmentId::FireProtection,
            EnchantmentId::ProjectileProtection,
        ],
    },
    EnchantmentProperties {
        name: "Projectile Protection",
        max_level: 4,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[
            EnchantmentId::Protection,
            EnchantmentId::FireProtection,
            EnchantmentId::BlastProtection,
        ],
    },
    EnchantmentProperties {
        name: "Thorns",
        max_level: 3,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Respiration",
        max_level: 3,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Aqua Affinity",
        max_level: 1,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Depth Strider",
        max_level: 3,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[EnchantmentId::FrostWalker],
    },
    EnchantmentProperties {
        name: "Frost Walker",
        max_level: 2,
        category: EnchantmentCategory::Armor,
        incompatible_with: &[EnchantmentId::DepthStrider],
    },
    // Weapon
    EnchantmentProperties {
        name: "Sharpness",
        max_level: 5,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[EnchantmentId::Smite, EnchantmentId::BaneOfArthropods],
    },
    EnchantmentProperties {
        name: "Smite",
        max_level: 5,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[EnchantmentId::Sharpness, EnchantmentId::BaneOfArthropods],
    },
    EnchantmentProperties {
        name: "Bane of Arthropods",
        max_level: 5,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[EnchantmentId::Sharpness, EnchantmentId::Smite],
    },
    EnchantmentProperties {
        name: "Knockback",
        max_level: 2,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Fire Aspect",
        max_level: 2,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Looting",
        max_level: 3,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Sweeping Edge",
        max_level: 3,
        category: EnchantmentCategory::Weapon,
        incompatible_with: &[],
    },
    // Tool
    EnchantmentProperties {
        name: "Efficiency",
        max_level: 5,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Silk Touch",
        max_level: 1,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[EnchantmentId::Fortune],
    },
    EnchantmentProperties {
        name: "Fortune",
        max_level: 3,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[EnchantmentId::SilkTouch],
    },
    EnchantmentProperties {
        name: "Unbreaking",
        max_level: 3,
        category: EnchantmentCategory::Tool,
        incompatible_with: &[],
    },
    // Bow
    EnchantmentProperties {
        name: "Power",
        max_level: 5,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Punch",
        max_level: 2,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Flame",
        max_level: 1,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[],
    },
    EnchantmentProperties {
        name: "Infinity",
        max_level: 1,
        category: EnchantmentCategory::Bow,
        incompatible_with: &[EnchantmentId::Mending],
    },
    // General
    EnchantmentProperties {
        name: "Mending",
        max_level: 1,
        category: EnchantmentCategory::General,
        incompatible_with: &[EnchantmentId::Infinity],
    },
    EnchantmentProperties {
        name: "Curse of Vanishing",
        max_level: 1,
        category: EnchantmentCategory::General,
        incompatible_with: &[],
    },
];

// ── Enchantment (instance) ─────────────────────────────────────────────────

/// A specific enchantment applied at a given level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enchantment {
    pub id: EnchantmentId,
    pub level: u8,
}

impl Enchantment {
    /// Create an enchantment, clamping the level to the enchantment's max.
    #[must_use]
    pub fn new(id: EnchantmentId, level: u8) -> Self {
        let max = id.properties().max_level;
        Self {
            id,
            level: level.clamp(1, max),
        }
    }
}

// ── Enchanted Item ─────────────────────────────────────────────────────────

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
        Self {
            item_id,
            enchantments: Vec::new(),
        }
    }

    /// Try to add an enchantment. Returns `false` if it is incompatible with
    /// an existing enchantment or already present.
    pub fn add_enchantment(&mut self, enchantment: Enchantment) -> bool {
        let new_props = enchantment.id.properties();

        for existing in &self.enchantments {
            // Duplicate check
            if existing.id == enchantment.id {
                return false;
            }
            // Incompatibility check (bidirectional)
            if new_props.incompatible_with.contains(&existing.id) {
                return false;
            }
            let existing_props = existing.id.properties();
            if existing_props.incompatible_with.contains(&enchantment.id) {
                return false;
            }
        }

        self.enchantments.push(enchantment);
        true
    }

    /// Check whether the item has a specific enchantment, returning its level.
    #[must_use]
    pub fn enchantment_level(&self, id: EnchantmentId) -> Option<u8> {
        self.enchantments
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.level)
    }
}

// ── Enchant Option ─────────────────────────────────────────────────────────

/// One of the three options presented on the enchanting table UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantOption {
    pub cost: u8,
    pub enchantments: Vec<Enchantment>,
    pub description: String,
}

// ── Deterministic hashing helper ───────────────────────────────────────────

fn deterministic_hash(seed: u64, extra: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    extra.hash(&mut hasher);
    hasher.finish()
}

// ── Cost calculation ───────────────────────────────────────────────────────

/// Calculate the three-tier enchantment costs based on the number of bookshelves.
///
/// `bookshelves` is clamped to 0..=15. The returned array contains the XP level
/// cost for the low, mid, and high tier slots respectively.
///
/// Formula (deterministic, using `seed` for the random component):
/// - base = floor(bookshelves / 2)
/// - tier1 = max(1, hash_rand(1..8) + base)
/// - tier2 = tier1 + hash_rand(0..8) + base  (clamped to tier1+1 .. 2*bookshelves+1)
/// - tier3 = tier2 + hash_rand(0..8) + base  (clamped to tier2+1 .. 3*bookshelves+1, max 30)
#[must_use]
pub fn calculate_enchantment_cost(bookshelves: u8, seed: u64) -> [u8; 3] {
    let shelves = bookshelves.min(15) as u64;
    let base = shelves / 2;

    let r1 = (deterministic_hash(seed, 1) % 8) + 1; // 1..=8
    let tier1 = 1u64.max(r1 + base);

    let r2 = deterministic_hash(seed, 2) % 9; // 0..=8
    let raw2 = tier1 + r2 + base;
    let min2 = tier1 + 1;
    let max2 = (2 * shelves + 1).max(min2);
    let tier2 = raw2.clamp(min2, max2);

    // Ensure tier2 leaves room for tier3 (at least tier2 < 30)
    let tier2 = tier2.min(29);

    let r3 = deterministic_hash(seed, 3) % 9; // 0..=8
    let raw3 = tier2 + r3 + base;
    let min3 = tier2 + 1;
    let max3 = (3 * shelves + 1).max(min3);
    let tier3 = raw3.clamp(min3, max3).min(30);

    [tier1 as u8, tier2 as u8, tier3 as u8]
}

// ── Item-type to category mapping ──────────────────────────────────────────

/// Determine which enchantment categories apply to a given item type.
///
/// Uses the `SlotItem(u16)` value ranges established in `recipe.rs`:
/// - 200..=233  tools (pickaxe, axe, shovel, sword)
/// - Swords (x03, x13, x23, x33 suffixes) map to Weapon
/// - Other tools map to Tool
/// - All tools also get General
///
/// Items outside these ranges receive only General enchantments.
fn applicable_categories(item_type: u16) -> Vec<EnchantmentCategory> {
    match item_type {
        // Swords: 203, 213, 223, 233
        203 | 213 | 223 | 233 => vec![
            EnchantmentCategory::Weapon,
            EnchantmentCategory::General,
        ],
        // Pickaxes, axes, shovels
        200..=232 => vec![
            EnchantmentCategory::Tool,
            EnchantmentCategory::General,
        ],
        _ => vec![EnchantmentCategory::General],
    }
}

/// Collect all enchantments applicable to the given item type.
fn applicable_enchantments(item_type: u16) -> Vec<EnchantmentId> {
    let categories = applicable_categories(item_type);
    EnchantmentId::ALL
        .iter()
        .copied()
        .filter(|id| categories.contains(&id.properties().category))
        .collect()
}

// ── Enchantment option generation ──────────────────────────────────────────

/// Generate three enchantment options for the enchanting table UI.
///
/// `item_type` is the `SlotItem.0` value of the item being enchanted.
/// `bookshelves` is clamped to 0..=15.
/// `seed` drives deterministic "random" selection.
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

        // Higher cost → more enchantments
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

            // Skip duplicates and incompatible enchantments
            let dominated = selected.iter().any(|s| {
                s.id == candidate
                    || s.id.properties().incompatible_with.contains(&candidate)
                    || candidate.properties().incompatible_with.contains(&s.id)
            });

            if dominated {
                // Try next in pool
                let fallback_idx = (idx + 1) % pool.len();
                let fallback = pool[fallback_idx];
                let still_bad = selected.iter().any(|s| {
                    s.id == fallback
                        || s.id.properties().incompatible_with.contains(&fallback)
                        || fallback.properties().incompatible_with.contains(&s.id)
                });
                if !still_bad {
                    let level_hash =
                        deterministic_hash(seed, (tier as u64 + 1) * 200 + i as u64);
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

        EnchantOption {
            cost,
            enchantments: selected,
            description,
        }
    };

    [
        build_option(0, costs[0]),
        build_option(1, costs[1]),
        build_option(2, costs[2]),
    ]
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

// ── Effect application ─────────────────────────────────────────────────────

/// Apply an enchantment's effect to a base value and return the modified value.
///
/// - **Sharpness**: `base + 1.25 * level` (extra damage)
/// - **Protection**: `base * (1.0 - 0.04 * level)` (damage reduction)
/// - **Efficiency**: `base + level^2 + 1` (mining speed bonus)
/// - **Unbreaking**: `base * (1.0 / (level + 1))` (durability usage chance)
/// - **Power**: `base + 0.5 * (level + 1)` (bow damage bonus)
/// - **Knockback**: `base + 0.5 * level` (knockback distance)
/// - Others return `base` unchanged.
#[must_use]
pub fn apply_enchantment_effect(enchantment: &Enchantment, base_value: f32) -> f32 {
    let level = enchantment.level as f32;
    match enchantment.id {
        EnchantmentId::Sharpness
        | EnchantmentId::Smite
        | EnchantmentId::BaneOfArthropods => base_value + 1.25 * level,

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

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── EnchantmentId basics ───────────────────────────────────────────

    #[test]
    fn all_contains_26_variants() {
        assert_eq!(EnchantmentId::ALL.len(), 26);
    }

    #[test]
    fn properties_lookup_returns_correct_name() {
        assert_eq!(EnchantmentId::Sharpness.properties().name, "Sharpness");
        assert_eq!(
            EnchantmentId::CurseOfVanishing.properties().name,
            "Curse of Vanishing"
        );
    }

    #[test]
    fn max_levels_are_within_1_to_5() {
        for id in &EnchantmentId::ALL {
            let max = id.properties().max_level;
            assert!(
                (1..=5).contains(&max),
                "{:?} has max_level {} outside 1..=5",
                id,
                max
            );
        }
    }

    // ── Enchantment level clamping ─────────────────────────────────────

    #[test]
    fn enchantment_new_clamps_level_to_max() {
        let e = Enchantment::new(EnchantmentId::AquaAffinity, 5);
        assert_eq!(e.level, 1); // max_level is 1
    }

    #[test]
    fn enchantment_new_clamps_level_to_minimum_1() {
        let e = Enchantment::new(EnchantmentId::Sharpness, 0);
        assert_eq!(e.level, 1);
    }

    #[test]
    fn enchantment_new_preserves_valid_level() {
        let e = Enchantment::new(EnchantmentId::Sharpness, 3);
        assert_eq!(e.level, 3);
    }

    // ── EnchantedItem incompatibility ──────────────────────────────────

    #[test]
    fn cannot_add_duplicate_enchantment() {
        let mut item = EnchantedItem::new(SlotItem(200));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 3)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 5)));
    }

    #[test]
    fn cannot_add_incompatible_enchantment() {
        let mut item = EnchantedItem::new(SlotItem(203));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Sharpness, 5)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Smite, 3)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::BaneOfArthropods, 3)));
    }

    #[test]
    fn can_add_compatible_enchantments() {
        let mut item = EnchantedItem::new(SlotItem(200));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 3)));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Unbreaking, 3)));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Fortune, 2)));
        assert_eq!(item.enchantments.len(), 3);
    }

    #[test]
    fn silk_touch_and_fortune_are_incompatible() {
        let mut item = EnchantedItem::new(SlotItem(200));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::SilkTouch, 1)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Fortune, 3)));
    }

    #[test]
    fn infinity_and_mending_are_incompatible() {
        let mut item = EnchantedItem::new(SlotItem(200));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::Infinity, 1)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::Mending, 1)));
    }

    #[test]
    fn depth_strider_and_frost_walker_are_incompatible() {
        let mut item = EnchantedItem::new(SlotItem(200));
        assert!(item.add_enchantment(Enchantment::new(EnchantmentId::DepthStrider, 3)));
        assert!(!item.add_enchantment(Enchantment::new(EnchantmentId::FrostWalker, 2)));
    }

    #[test]
    fn enchantment_level_query() {
        let mut item = EnchantedItem::new(SlotItem(200));
        item.add_enchantment(Enchantment::new(EnchantmentId::Efficiency, 4));
        assert_eq!(item.enchantment_level(EnchantmentId::Efficiency), Some(4));
        assert_eq!(item.enchantment_level(EnchantmentId::Fortune), None);
    }

    // ── Cost calculation ───────────────────────────────────────────────

    #[test]
    fn cost_tiers_are_strictly_increasing() {
        for seed in 0..100u64 {
            let costs = calculate_enchantment_cost(15, seed);
            assert!(
                costs[0] < costs[1] && costs[1] < costs[2],
                "seed {seed}: costs {:?} are not strictly increasing",
                costs,
            );
        }
    }

    #[test]
    fn cost_tier1_is_at_least_1() {
        for seed in 0..100u64 {
            let costs = calculate_enchantment_cost(0, seed);
            assert!(costs[0] >= 1, "seed {seed}: tier1 = {}", costs[0]);
        }
    }

    #[test]
    fn cost_tier3_is_at_most_30() {
        for seed in 0..200u64 {
            let costs = calculate_enchantment_cost(15, seed);
            assert!(
                costs[2] <= 30,
                "seed {seed}: tier3 = {} exceeds 30",
                costs[2]
            );
        }
    }

    #[test]
    fn zero_bookshelves_produces_low_costs() {
        for seed in 0..50u64 {
            let costs = calculate_enchantment_cost(0, seed);
            assert!(costs[0] <= 8, "seed {seed}: tier1 = {}", costs[0]);
            assert!(costs[2] <= 25, "seed {seed}: tier3 = {}", costs[2]);
        }
    }

    #[test]
    fn bookshelves_clamped_to_15() {
        let a = calculate_enchantment_cost(15, 42);
        let b = calculate_enchantment_cost(30, 42);
        assert_eq!(a, b, "bookshelves > 15 should clamp to 15");
    }

    #[test]
    fn deterministic_cost_same_seed() {
        let a = calculate_enchantment_cost(10, 12345);
        let b = calculate_enchantment_cost(10, 12345);
        assert_eq!(a, b);
    }

    // ── Effect application ─────────────────────────────────────────────

    #[test]
    fn sharpness_adds_damage() {
        let e = Enchantment::new(EnchantmentId::Sharpness, 3);
        let result = apply_enchantment_effect(&e, 7.0);
        // 7.0 + 1.25 * 3 = 10.75
        assert!((result - 10.75).abs() < f32::EPSILON);
    }

    #[test]
    fn protection_reduces_damage() {
        let e = Enchantment::new(EnchantmentId::Protection, 4);
        let result = apply_enchantment_effect(&e, 100.0);
        // 100.0 * (1.0 - 0.04 * 4) = 100.0 * 0.84 = 84.0
        assert!((result - 84.0).abs() < f32::EPSILON);
    }

    #[test]
    fn efficiency_boosts_mining_speed() {
        let e = Enchantment::new(EnchantmentId::Efficiency, 5);
        let result = apply_enchantment_effect(&e, 8.0);
        // 8.0 + 5^2 + 1 = 8.0 + 26 = 34.0
        assert!((result - 34.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unbreaking_reduces_durability_usage() {
        let e = Enchantment::new(EnchantmentId::Unbreaking, 3);
        let result = apply_enchantment_effect(&e, 1.0);
        // 1.0 / (3 + 1) = 0.25
        assert!((result - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn power_adds_bow_damage() {
        let e = Enchantment::new(EnchantmentId::Power, 5);
        let result = apply_enchantment_effect(&e, 9.0);
        // 9.0 + 0.5 * (5 + 1) = 9.0 + 3.0 = 12.0
        assert!((result - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn knockback_adds_distance() {
        let e = Enchantment::new(EnchantmentId::Knockback, 2);
        let result = apply_enchantment_effect(&e, 3.0);
        // 3.0 + 0.5 * 2 = 4.0
        assert!((result - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unknown_effect_returns_base() {
        let e = Enchantment::new(EnchantmentId::AquaAffinity, 1);
        let result = apply_enchantment_effect(&e, 10.0);
        assert!((result - 10.0).abs() < f32::EPSILON);
    }

    // ── Enchantment option generation ──────────────────────────────────

    #[test]
    fn generate_options_returns_three() {
        let options = generate_enchantment_options(200, 15, 42);
        assert_eq!(options.len(), 3);
    }

    #[test]
    fn generate_options_costs_match_tier_costs() {
        let seed = 42u64;
        let costs = calculate_enchantment_cost(15, seed);
        let options = generate_enchantment_options(200, 15, seed);
        assert_eq!(options[0].cost, costs[0]);
        assert_eq!(options[1].cost, costs[1]);
        assert_eq!(options[2].cost, costs[2]);
    }

    #[test]
    fn generate_options_deterministic() {
        let a = generate_enchantment_options(200, 10, 999);
        let b = generate_enchantment_options(200, 10, 999);
        assert_eq!(a, b);
    }

    #[test]
    fn generate_options_each_has_at_least_one_enchantment() {
        let options = generate_enchantment_options(200, 15, 42);
        for opt in &options {
            assert!(
                !opt.enchantments.is_empty(),
                "option with cost {} has no enchantments",
                opt.cost
            );
        }
    }

    #[test]
    fn generate_options_descriptions_are_nonempty() {
        let options = generate_enchantment_options(200, 15, 42);
        for opt in &options {
            assert!(
                !opt.description.is_empty(),
                "option with cost {} has empty description",
                opt.cost
            );
        }
    }

    #[test]
    fn generate_options_no_duplicate_enchantments_in_option() {
        for seed in 0..50u64 {
            let options = generate_enchantment_options(203, 15, seed);
            for opt in &options {
                let ids: Vec<EnchantmentId> = opt.enchantments.iter().map(|e| e.id).collect();
                for (i, id) in ids.iter().enumerate() {
                    assert!(
                        !ids[i + 1..].contains(id),
                        "seed {seed}: duplicate {:?} in option",
                        id
                    );
                }
            }
        }
    }

    // ── Roman numeral helper ───────────────────────────────────────────

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

    // ── Applicable enchantments by item type ───────────────────────────

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

    // ── Protection enchantment group incompatibility ───────────────────

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
                let mut item = EnchantedItem::new(SlotItem(200));
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
