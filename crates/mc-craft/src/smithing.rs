use crate::recipe;

// ── Item constants (smithing-specific) ────────────────────────────────────
// Diamond tools/armor are imported from `recipe::ITEM_*`.
// New items below are unique to smithing inputs/outputs.

/// Netherite upgrade smithing template.
const ITEM_NETHERITE_UPGRADE_TEMPLATE: u16 = 900;

/// Netherite ingot (addition material).
const ITEM_NETHERITE_INGOT: u16 = 901;

/// Diamond hoe (not yet in recipe.rs).
const ITEM_DIAMOND_HOE: u16 = 234;

// ── Netherite tools ──────────────────────────────────────────────────────
const ITEM_NETHERITE_SWORD: u16 = 240;
const ITEM_NETHERITE_PICKAXE: u16 = 241;
const ITEM_NETHERITE_AXE: u16 = 242;
const ITEM_NETHERITE_SHOVEL: u16 = 243;
const ITEM_NETHERITE_HOE: u16 = 244;

// ── Netherite armor ──────────────────────────────────────────────────────
const ITEM_NETHERITE_HELMET: u16 = 340;
const ITEM_NETHERITE_CHESTPLATE: u16 = 341;
const ITEM_NETHERITE_LEGGINGS: u16 = 342;
const ITEM_NETHERITE_BOOTS: u16 = 343;

// ── Smithing recipe ──────────────────────────────────────────────────────

/// A smithing table recipe: a template, base item, and addition combine to
/// produce an output item. In vanilla Minecraft this is used for
/// diamond → netherite upgrades.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmithingRecipe {
    pub template: u16,
    pub base: u16,
    pub addition: u16,
    pub output: u16,
}

// ── Default data ─────────────────────────────────────────────────────────

/// Return the 9 default diamond → netherite upgrade recipes.
#[must_use]
pub fn default_smithing_recipes() -> Vec<SmithingRecipe> {
    let t = ITEM_NETHERITE_UPGRADE_TEMPLATE;
    let a = ITEM_NETHERITE_INGOT;

    vec![
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_SWORD.0,      addition: a, output: ITEM_NETHERITE_SWORD },
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_PICKAXE.0,    addition: a, output: ITEM_NETHERITE_PICKAXE },
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_AXE.0,        addition: a, output: ITEM_NETHERITE_AXE },
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_SHOVEL.0,     addition: a, output: ITEM_NETHERITE_SHOVEL },
        SmithingRecipe { template: t, base: ITEM_DIAMOND_HOE,                  addition: a, output: ITEM_NETHERITE_HOE },
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_HELMET.0,     addition: a, output: ITEM_NETHERITE_HELMET },
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_CHESTPLATE.0, addition: a, output: ITEM_NETHERITE_CHESTPLATE },
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_LEGGINGS.0,   addition: a, output: ITEM_NETHERITE_LEGGINGS },
        SmithingRecipe { template: t, base: recipe::ITEM_DIAMOND_BOOTS.0,      addition: a, output: ITEM_NETHERITE_BOOTS },
    ]
}

// ── Smithing functions ───────────────────────────────────────────────────

/// Attempt to find a matching smithing recipe and return the output item ID.
///
/// Returns `None` if no recipe matches the given template, base, and addition.
#[must_use]
pub fn try_smith(template: u16, base: u16, addition: u16, recipes: &[SmithingRecipe]) -> Option<u16> {
    recipes
        .iter()
        .find(|r| r.template == template && r.base == base && r.addition == addition)
        .map(|r| r.output)
}

/// Clone all enchantments from the base item. In Minecraft, netherite
/// upgrades preserve every enchantment and its level.
#[must_use]
pub fn preserve_enchantments(enchants: &[(u16, u8)]) -> Vec<(u16, u8)> {
    enchants.to_vec()
}

// ── Smithing table ───────────────────────────────────────────────────────

/// A smithing table block with three input slots.
#[derive(Debug, Clone, Default)]
pub struct SmithingTable {
    pub template_slot: Option<u16>,
    pub base_slot: Option<u16>,
    pub addition_slot: Option<u16>,
}

impl SmithingTable {
    /// Check whether the current slot contents match any recipe.
    #[must_use]
    pub fn can_craft(&self, recipes: &[SmithingRecipe]) -> bool {
        match (self.template_slot, self.base_slot, self.addition_slot) {
            (Some(t), Some(b), Some(a)) => try_smith(t, b, a, recipes).is_some(),
            _ => false,
        }
    }

    /// Consume the slot contents and return the output item if a recipe matches.
    ///
    /// All three slots are cleared on success.
    pub fn craft(&mut self, recipes: &[SmithingRecipe]) -> Option<u16> {
        let t = self.template_slot?;
        let b = self.base_slot?;
        let a = self.addition_slot?;

        let output = try_smith(t, b, a, recipes)?;

        self.template_slot = None;
        self.base_slot = None;
        self.addition_slot = None;

        Some(output)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn recipes() -> Vec<SmithingRecipe> {
        default_smithing_recipes()
    }

    // ── All 9 upgrades ──────────────────────────────────────────────────

    #[test]
    fn upgrade_diamond_sword_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_SWORD.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_SWORD));
    }

    #[test]
    fn upgrade_diamond_pickaxe_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_PICKAXE.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_PICKAXE));
    }

    #[test]
    fn upgrade_diamond_axe_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_AXE.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_AXE));
    }

    #[test]
    fn upgrade_diamond_shovel_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_SHOVEL.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_SHOVEL));
    }

    #[test]
    fn upgrade_diamond_hoe_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, ITEM_DIAMOND_HOE, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_HOE));
    }

    #[test]
    fn upgrade_diamond_helmet_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_HELMET.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_HELMET));
    }

    #[test]
    fn upgrade_diamond_chestplate_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_CHESTPLATE.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_CHESTPLATE));
    }

    #[test]
    fn upgrade_diamond_leggings_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_LEGGINGS.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_LEGGINGS));
    }

    #[test]
    fn upgrade_diamond_boots_to_netherite() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_BOOTS.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, Some(ITEM_NETHERITE_BOOTS));
    }

    // ── Default recipe count ────────────────────────────────────────────

    #[test]
    fn default_recipes_has_nine_entries() {
        assert_eq!(recipes().len(), 9);
    }

    // ── Invalid combinations ────────────────────────────────────────────

    #[test]
    fn wrong_template_returns_none() {
        let r = recipes();
        let out = try_smith(9999, recipe::ITEM_DIAMOND_SWORD.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, None);
    }

    #[test]
    fn wrong_base_returns_none() {
        let r = recipes();
        // Iron sword is not a valid base for netherite upgrade.
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_IRON_SWORD.0, ITEM_NETHERITE_INGOT, &r);
        assert_eq!(out, None);
    }

    #[test]
    fn wrong_addition_returns_none() {
        let r = recipes();
        let out = try_smith(ITEM_NETHERITE_UPGRADE_TEMPLATE, recipe::ITEM_DIAMOND_SWORD.0, 9999, &r);
        assert_eq!(out, None);
    }

    #[test]
    fn all_slots_wrong_returns_none() {
        let r = recipes();
        let out = try_smith(0, 0, 0, &r);
        assert_eq!(out, None);
    }

    // ── Enchantment preservation ────────────────────────────────────────

    #[test]
    fn preserve_enchantments_clones_all() {
        let enchants: Vec<(u16, u8)> = vec![(1, 3), (5, 1), (12, 5)];
        let preserved = preserve_enchantments(&enchants);
        assert_eq!(preserved, enchants);
    }

    #[test]
    fn preserve_enchantments_empty() {
        let enchants: Vec<(u16, u8)> = vec![];
        let preserved = preserve_enchantments(&enchants);
        assert!(preserved.is_empty());
    }

    #[test]
    fn preserve_enchantments_single() {
        let enchants: Vec<(u16, u8)> = vec![(7, 2)];
        let preserved = preserve_enchantments(&enchants);
        assert_eq!(preserved, vec![(7, 2)]);
    }

    // ── SmithingTable empty slots ───────────────────────────────────────

    #[test]
    fn empty_table_cannot_craft() {
        let r = recipes();
        let table = SmithingTable::default();
        assert!(!table.can_craft(&r));
    }

    #[test]
    fn empty_table_craft_returns_none() {
        let r = recipes();
        let mut table = SmithingTable::default();
        assert_eq!(table.craft(&r), None);
    }

    #[test]
    fn partial_slots_cannot_craft() {
        let r = recipes();
        let table = SmithingTable {
            template_slot: Some(ITEM_NETHERITE_UPGRADE_TEMPLATE),
            base_slot: Some(recipe::ITEM_DIAMOND_SWORD.0),
            addition_slot: None,
        };
        assert!(!table.can_craft(&r));
    }

    #[test]
    fn missing_template_cannot_craft() {
        let r = recipes();
        let table = SmithingTable {
            template_slot: None,
            base_slot: Some(recipe::ITEM_DIAMOND_SWORD.0),
            addition_slot: Some(ITEM_NETHERITE_INGOT),
        };
        assert!(!table.can_craft(&r));
    }

    #[test]
    fn missing_base_cannot_craft() {
        let r = recipes();
        let table = SmithingTable {
            template_slot: Some(ITEM_NETHERITE_UPGRADE_TEMPLATE),
            base_slot: None,
            addition_slot: Some(ITEM_NETHERITE_INGOT),
        };
        assert!(!table.can_craft(&r));
    }

    // ── SmithingTable can_craft + craft ─────────────────────────────────

    #[test]
    fn table_can_craft_valid_recipe() {
        let r = recipes();
        let table = SmithingTable {
            template_slot: Some(ITEM_NETHERITE_UPGRADE_TEMPLATE),
            base_slot: Some(recipe::ITEM_DIAMOND_SWORD.0),
            addition_slot: Some(ITEM_NETHERITE_INGOT),
        };
        assert!(table.can_craft(&r));
    }

    #[test]
    fn table_craft_returns_output_and_clears_slots() {
        let r = recipes();
        let mut table = SmithingTable {
            template_slot: Some(ITEM_NETHERITE_UPGRADE_TEMPLATE),
            base_slot: Some(recipe::ITEM_DIAMOND_PICKAXE.0),
            addition_slot: Some(ITEM_NETHERITE_INGOT),
        };

        let output = table.craft(&r);
        assert_eq!(output, Some(ITEM_NETHERITE_PICKAXE));
        assert!(table.template_slot.is_none());
        assert!(table.base_slot.is_none());
        assert!(table.addition_slot.is_none());
    }

    #[test]
    fn table_craft_invalid_returns_none_and_preserves_slots() {
        let r = recipes();
        let mut table = SmithingTable {
            template_slot: Some(ITEM_NETHERITE_UPGRADE_TEMPLATE),
            base_slot: Some(recipe::ITEM_IRON_SWORD.0),
            addition_slot: Some(ITEM_NETHERITE_INGOT),
        };

        let output = table.craft(&r);
        assert_eq!(output, None);
        // Slots should remain untouched on failure.
        assert_eq!(table.template_slot, Some(ITEM_NETHERITE_UPGRADE_TEMPLATE));
        assert_eq!(table.base_slot, Some(recipe::ITEM_IRON_SWORD.0));
        assert_eq!(table.addition_slot, Some(ITEM_NETHERITE_INGOT));
    }
}
