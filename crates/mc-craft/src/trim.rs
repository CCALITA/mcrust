//! Armor trim patterns and materials.
//!
//! In Minecraft, armor trims are cosmetic decorations applied via the
//! smithing table using a smithing template (pattern), an armor piece, and
//! a material item. This module models the 16 trim patterns, 10 trim
//! materials, and the logic to apply a trim to an armor item.

use crate::item_ids;

// ── Trim pattern smithing template item IDs ──────────────────────────────

const TEMPLATE_SENTRY: u16 = 910;
const TEMPLATE_DUNE: u16 = 911;
const TEMPLATE_COAST: u16 = 912;
const TEMPLATE_WILD: u16 = 913;
const TEMPLATE_WARD: u16 = 914;
const TEMPLATE_EYE: u16 = 915;
const TEMPLATE_VEX: u16 = 916;
const TEMPLATE_TIDE: u16 = 917;
const TEMPLATE_SNOUT: u16 = 918;
const TEMPLATE_RIB: u16 = 919;
const TEMPLATE_WAYFINDER: u16 = 920;
const TEMPLATE_SHAPER: u16 = 921;
const TEMPLATE_SILENCE: u16 = 922;
const TEMPLATE_RAISER: u16 = 923;
const TEMPLATE_HOST: u16 = 924;
const TEMPLATE_FLOW: u16 = 925;

// ── Trim material item IDs ───────────────────────────────────────────────

const MATERIAL_IRON_INGOT: u16 = item_ids::ITEM_IRON_INGOT;
const MATERIAL_COPPER_INGOT: u16 = 113;
const MATERIAL_GOLD_INGOT: u16 = item_ids::ITEM_GOLD_INGOT;
const MATERIAL_LAPIS_LAZULI: u16 = 114;
const MATERIAL_EMERALD: u16 = 115;
const MATERIAL_DIAMOND: u16 = item_ids::ITEM_DIAMOND;
const MATERIAL_NETHERITE_INGOT: u16 = 901;
const MATERIAL_REDSTONE_DUST: u16 = item_ids::ITEM_REDSTONE_DUST;
const MATERIAL_AMETHYST_SHARD: u16 = 116;
const MATERIAL_QUARTZ: u16 = item_ids::ITEM_QUARTZ;

// ── Armor item sets (trimmable) ──────────────────────────────────────────

const TRIMMABLE_ARMOR: &[u16] = &[
    // Leather
    item_ids::ITEM_LEATHER_HELMET,
    item_ids::ITEM_LEATHER_CHESTPLATE,
    item_ids::ITEM_LEATHER_LEGGINGS,
    item_ids::ITEM_LEATHER_BOOTS,
    // Iron
    item_ids::ITEM_IRON_HELMET,
    item_ids::ITEM_IRON_CHESTPLATE,
    item_ids::ITEM_IRON_LEGGINGS,
    item_ids::ITEM_IRON_BOOTS,
    // Gold
    item_ids::ITEM_GOLD_HELMET,
    item_ids::ITEM_GOLD_CHESTPLATE,
    item_ids::ITEM_GOLD_LEGGINGS,
    item_ids::ITEM_GOLD_BOOTS,
    // Diamond
    item_ids::ITEM_DIAMOND_HELMET,
    item_ids::ITEM_DIAMOND_CHESTPLATE,
    item_ids::ITEM_DIAMOND_LEGGINGS,
    item_ids::ITEM_DIAMOND_BOOTS,
];

// ── Enums ────────────────────────────────────────────────────────────────

/// The 16 armor trim patterns available in Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrimPattern {
    Sentry,
    Dune,
    Coast,
    Wild,
    Ward,
    Eye,
    Vex,
    Tide,
    Snout,
    Rib,
    Wayfinder,
    Shaper,
    Silence,
    Raiser,
    Host,
    Flow,
}

impl TrimPattern {
    /// All trim patterns in declaration order.
    pub const ALL: &[TrimPattern] = &[
        TrimPattern::Sentry,
        TrimPattern::Dune,
        TrimPattern::Coast,
        TrimPattern::Wild,
        TrimPattern::Ward,
        TrimPattern::Eye,
        TrimPattern::Vex,
        TrimPattern::Tide,
        TrimPattern::Snout,
        TrimPattern::Rib,
        TrimPattern::Wayfinder,
        TrimPattern::Shaper,
        TrimPattern::Silence,
        TrimPattern::Raiser,
        TrimPattern::Host,
        TrimPattern::Flow,
    ];

    /// Convert a smithing template item ID to a `TrimPattern`.
    #[must_use]
    pub fn from_template_id(id: u16) -> Option<TrimPattern> {
        match id {
            TEMPLATE_SENTRY => Some(TrimPattern::Sentry),
            TEMPLATE_DUNE => Some(TrimPattern::Dune),
            TEMPLATE_COAST => Some(TrimPattern::Coast),
            TEMPLATE_WILD => Some(TrimPattern::Wild),
            TEMPLATE_WARD => Some(TrimPattern::Ward),
            TEMPLATE_EYE => Some(TrimPattern::Eye),
            TEMPLATE_VEX => Some(TrimPattern::Vex),
            TEMPLATE_TIDE => Some(TrimPattern::Tide),
            TEMPLATE_SNOUT => Some(TrimPattern::Snout),
            TEMPLATE_RIB => Some(TrimPattern::Rib),
            TEMPLATE_WAYFINDER => Some(TrimPattern::Wayfinder),
            TEMPLATE_SHAPER => Some(TrimPattern::Shaper),
            TEMPLATE_SILENCE => Some(TrimPattern::Silence),
            TEMPLATE_RAISER => Some(TrimPattern::Raiser),
            TEMPLATE_HOST => Some(TrimPattern::Host),
            TEMPLATE_FLOW => Some(TrimPattern::Flow),
            _ => None,
        }
    }
}

/// The 10 armor trim materials available in Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrimMaterial {
    Iron,
    Copper,
    Gold,
    Lapis,
    Emerald,
    Diamond,
    Netherite,
    Redstone,
    Amethyst,
    Quartz,
}

impl TrimMaterial {
    /// All trim materials in declaration order.
    pub const ALL: &[TrimMaterial] = &[
        TrimMaterial::Iron,
        TrimMaterial::Copper,
        TrimMaterial::Gold,
        TrimMaterial::Lapis,
        TrimMaterial::Emerald,
        TrimMaterial::Diamond,
        TrimMaterial::Netherite,
        TrimMaterial::Redstone,
        TrimMaterial::Amethyst,
        TrimMaterial::Quartz,
    ];

    /// Convert a material item ID to a `TrimMaterial`.
    #[must_use]
    pub fn from_item_id(id: u16) -> Option<TrimMaterial> {
        match id {
            MATERIAL_IRON_INGOT => Some(TrimMaterial::Iron),
            MATERIAL_COPPER_INGOT => Some(TrimMaterial::Copper),
            MATERIAL_GOLD_INGOT => Some(TrimMaterial::Gold),
            MATERIAL_LAPIS_LAZULI => Some(TrimMaterial::Lapis),
            MATERIAL_EMERALD => Some(TrimMaterial::Emerald),
            MATERIAL_DIAMOND => Some(TrimMaterial::Diamond),
            MATERIAL_NETHERITE_INGOT => Some(TrimMaterial::Netherite),
            MATERIAL_REDSTONE_DUST => Some(TrimMaterial::Redstone),
            MATERIAL_AMETHYST_SHARD => Some(TrimMaterial::Amethyst),
            MATERIAL_QUARTZ => Some(TrimMaterial::Quartz),
            _ => None,
        }
    }
}

// ── Trimmed armor ────────────────────────────────────────────────────────

/// An armor piece with a cosmetic trim applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimmedArmor {
    pub armor_item: u16,
    pub pattern: TrimPattern,
    pub material: TrimMaterial,
}

// ── Public API ───────────────────────────────────────────────────────────

/// Returns `true` if the given item ID is a trimmable armor piece.
#[must_use]
pub fn is_trimmable(item_id: u16) -> bool {
    TRIMMABLE_ARMOR.contains(&item_id)
}

/// Attempt to apply a trim to an armor piece using a smithing template and
/// a material item.
///
/// Returns `None` if:
/// - The template ID does not correspond to a known trim pattern.
/// - The material ID does not correspond to a known trim material.
/// - The armor item is not a trimmable armor piece.
#[must_use]
pub fn apply_trim(armor: u16, template: u16, material: u16) -> Option<TrimmedArmor> {
    let pattern = TrimPattern::from_template_id(template)?;
    let trim_material = TrimMaterial::from_item_id(material)?;

    if !is_trimmable(armor) {
        return None;
    }

    Some(TrimmedArmor {
        armor_item: armor,
        pattern,
        material: trim_material,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_trim_valid_combination() {
        let result = apply_trim(
            item_ids::ITEM_DIAMOND_CHESTPLATE,
            TEMPLATE_SENTRY,
            MATERIAL_GOLD_INGOT,
        );
        let trimmed = result.expect("should produce a trimmed armor");
        assert_eq!(trimmed.armor_item, item_ids::ITEM_DIAMOND_CHESTPLATE);
        assert_eq!(trimmed.pattern, TrimPattern::Sentry);
        assert_eq!(trimmed.material, TrimMaterial::Gold);
    }

    #[test]
    fn apply_trim_invalid_template_returns_none() {
        let result = apply_trim(
            item_ids::ITEM_IRON_HELMET,
            9999,
            MATERIAL_DIAMOND,
        );
        assert!(result.is_none());
    }

    #[test]
    fn apply_trim_invalid_material_returns_none() {
        let result = apply_trim(
            item_ids::ITEM_IRON_HELMET,
            TEMPLATE_WILD,
            9999,
        );
        assert!(result.is_none());
    }

    #[test]
    fn apply_trim_non_armor_item_returns_none() {
        // A sword is not armor.
        let result = apply_trim(
            item_ids::ITEM_DIAMOND_SWORD,
            TEMPLATE_COAST,
            MATERIAL_EMERALD,
        );
        assert!(result.is_none());
    }

    #[test]
    fn apply_trim_all_patterns_recognized() {
        for (i, &pattern) in TrimPattern::ALL.iter().enumerate() {
            let template_id = TEMPLATE_SENTRY + i as u16;
            let parsed = TrimPattern::from_template_id(template_id);
            assert_eq!(parsed, Some(pattern), "pattern index {i} should be recognized");
        }
    }

    #[test]
    fn apply_trim_all_materials_recognized() {
        let material_ids = [
            MATERIAL_IRON_INGOT,
            MATERIAL_COPPER_INGOT,
            MATERIAL_GOLD_INGOT,
            MATERIAL_LAPIS_LAZULI,
            MATERIAL_EMERALD,
            MATERIAL_DIAMOND,
            MATERIAL_NETHERITE_INGOT,
            MATERIAL_REDSTONE_DUST,
            MATERIAL_AMETHYST_SHARD,
            MATERIAL_QUARTZ,
        ];
        for (i, &id) in material_ids.iter().enumerate() {
            let parsed = TrimMaterial::from_item_id(id);
            assert_eq!(parsed, Some(TrimMaterial::ALL[i]), "material index {i} should be recognized");
        }
    }

    #[test]
    fn all_leather_armor_is_trimmable() {
        assert!(is_trimmable(item_ids::ITEM_LEATHER_HELMET));
        assert!(is_trimmable(item_ids::ITEM_LEATHER_CHESTPLATE));
        assert!(is_trimmable(item_ids::ITEM_LEATHER_LEGGINGS));
        assert!(is_trimmable(item_ids::ITEM_LEATHER_BOOTS));
    }

    #[test]
    fn all_diamond_armor_is_trimmable() {
        assert!(is_trimmable(item_ids::ITEM_DIAMOND_HELMET));
        assert!(is_trimmable(item_ids::ITEM_DIAMOND_CHESTPLATE));
        assert!(is_trimmable(item_ids::ITEM_DIAMOND_LEGGINGS));
        assert!(is_trimmable(item_ids::ITEM_DIAMOND_BOOTS));
    }

    #[test]
    fn non_armor_is_not_trimmable() {
        assert!(!is_trimmable(item_ids::ITEM_DIAMOND_SWORD));
        assert!(!is_trimmable(item_ids::ITEM_STICK));
        assert!(!is_trimmable(0));
    }

    #[test]
    fn trim_pattern_count_is_sixteen() {
        assert_eq!(TrimPattern::ALL.len(), 16);
    }

    #[test]
    fn trim_material_count_is_ten() {
        assert_eq!(TrimMaterial::ALL.len(), 10);
    }
}
