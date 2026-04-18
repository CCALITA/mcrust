use crate::recipe;

// ── Item constants (workstation-specific) ─────────────────────────────────
// Items already defined in recipe.rs are imported via `recipe::ITEM_*`.
// New items below are unique to workstation inputs/outputs.

const ITEM_STONE_STAIRS: u16 = 900;
const ITEM_STONE_SLAB: u16 = 901;
const ITEM_STONE_BRICKS: u16 = 902;
const ITEM_STONE_BRICK_STAIRS: u16 = 903;
const ITEM_STONE_BRICK_SLAB: u16 = 904;
const ITEM_STONE_BRICK_WALL: u16 = 905;
const ITEM_STONE_WALL: u16 = 906;
const ITEM_SMOOTH_STONE_SLAB: u16 = 907;
const ITEM_COBBLESTONE_WALL: u16 = 908;
const ITEM_GRANITE: u16 = 909;
const ITEM_GRANITE_STAIRS: u16 = 910;
const ITEM_GRANITE_SLAB: u16 = 911;
const ITEM_GRANITE_WALL: u16 = 912;
const ITEM_POLISHED_GRANITE: u16 = 913;
const ITEM_POLISHED_GRANITE_STAIRS: u16 = 914;
const ITEM_POLISHED_GRANITE_SLAB: u16 = 915;
const ITEM_DIORITE: u16 = 916;
const ITEM_DIORITE_STAIRS: u16 = 917;
const ITEM_DIORITE_SLAB: u16 = 918;
const ITEM_DIORITE_WALL: u16 = 919;
const ITEM_SMOOTH_STONE: u16 = 920;

/// Maximum number of banner layers allowed by the loom.
const MAX_LOOM_LAYERS: usize = 6;

// ── Stonecutter ───────────────────────────────────────────────────────────

/// A stonecutter recipe: one input item produces `count` output items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StonecutterRecipe {
    pub input: u16,
    pub output: u16,
    pub count: u8,
}

/// Return 20 default stonecutter recipes covering stone variants
/// (stairs, slabs, bricks, walls).
#[must_use]
pub fn default_stonecutter_recipes() -> Vec<StonecutterRecipe> {
    vec![
        // Stone variants
        StonecutterRecipe { input: recipe::ITEM_STONE.0,       output: ITEM_STONE_STAIRS,          count: 1 },
        StonecutterRecipe { input: recipe::ITEM_STONE.0,       output: ITEM_STONE_SLAB,            count: 2 },
        StonecutterRecipe { input: recipe::ITEM_STONE.0,       output: ITEM_STONE_BRICKS,          count: 1 },
        StonecutterRecipe { input: recipe::ITEM_STONE.0,       output: ITEM_STONE_BRICK_STAIRS,    count: 1 },
        StonecutterRecipe { input: recipe::ITEM_STONE.0,       output: ITEM_STONE_BRICK_SLAB,      count: 2 },
        StonecutterRecipe { input: recipe::ITEM_STONE.0,       output: ITEM_STONE_BRICK_WALL,      count: 1 },
        StonecutterRecipe { input: recipe::ITEM_STONE.0,       output: ITEM_STONE_WALL,            count: 1 },
        // Smooth stone
        StonecutterRecipe { input: ITEM_SMOOTH_STONE,          output: ITEM_SMOOTH_STONE_SLAB,     count: 2 },
        // Cobblestone variants
        StonecutterRecipe { input: recipe::ITEM_COBBLESTONE.0, output: recipe::ITEM_COBBLESTONE_STAIRS.0, count: 1 },
        StonecutterRecipe { input: recipe::ITEM_COBBLESTONE.0, output: recipe::ITEM_COBBLESTONE_SLAB.0,   count: 2 },
        StonecutterRecipe { input: recipe::ITEM_COBBLESTONE.0, output: ITEM_COBBLESTONE_WALL,      count: 1 },
        // Granite variants
        StonecutterRecipe { input: ITEM_GRANITE,               output: ITEM_GRANITE_STAIRS,        count: 1 },
        StonecutterRecipe { input: ITEM_GRANITE,               output: ITEM_GRANITE_SLAB,          count: 2 },
        StonecutterRecipe { input: ITEM_GRANITE,               output: ITEM_GRANITE_WALL,          count: 1 },
        StonecutterRecipe { input: ITEM_GRANITE,               output: ITEM_POLISHED_GRANITE,      count: 1 },
        StonecutterRecipe { input: ITEM_POLISHED_GRANITE,      output: ITEM_POLISHED_GRANITE_STAIRS, count: 1 },
        StonecutterRecipe { input: ITEM_POLISHED_GRANITE,      output: ITEM_POLISHED_GRANITE_SLAB, count: 2 },
        // Diorite variants
        StonecutterRecipe { input: ITEM_DIORITE,               output: ITEM_DIORITE_STAIRS,        count: 1 },
        StonecutterRecipe { input: ITEM_DIORITE,               output: ITEM_DIORITE_SLAB,          count: 2 },
        StonecutterRecipe { input: ITEM_DIORITE,               output: ITEM_DIORITE_WALL,          count: 1 },
    ]
}

// ── Grindstone ────────────────────────────────────────────────────────────

/// Result of a grindstone disenchant operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrindstoneResult {
    pub output_item: u16,
    pub output_count: u8,
    pub xp_returned: u32,
}

/// Disenchant an item via the grindstone, stripping all enchantments.
///
/// Each enchantment returns 2 XP on average. The output item is the same
/// item type with the same count, but without enchantments.
#[must_use]
pub fn grindstone_disenchant(item: u16, count: u8, enchant_count: u8) -> GrindstoneResult {
    let xp_returned = u32::from(enchant_count) * 2;
    GrindstoneResult {
        output_item: item,
        output_count: count,
        xp_returned,
    }
}

/// Attempt to repair two items of the same type via the grindstone.
///
/// Both items must share the same item id. The resulting durability is
/// the sum of both remaining durabilities plus a 5% bonus (of 64, the
/// simplified max durability), capped at 64. Returns `None` if the item
/// types differ.
#[must_use]
pub fn grindstone_repair(item1: (u16, u8), item2: (u16, u8)) -> Option<(u16, u8)> {
    if item1.0 != item2.0 {
        return None;
    }

    let max_durability: u16 = 64;
    let bonus = (max_durability * 5 / 100) as u8; // 5% of max
    let combined = item1
        .1
        .saturating_add(item2.1)
        .saturating_add(bonus)
        .min(max_durability as u8);

    Some((item1.0, combined))
}

// ── Loom ──────────────────────────────────────────────────────────────────

/// Apply a new pattern layer to a banner via the loom.
///
/// `existing` contains the current layers as `(pattern_id, color)` pairs.
/// Returns `None` if the banner already has the maximum of 6 layers.
/// Otherwise returns a new vector with the added layer.
#[must_use]
pub fn loom_apply(
    existing: &[(u8, u8)],
    new_pattern: u8,
    new_color: u8,
) -> Option<Vec<(u8, u8)>> {
    if existing.len() >= MAX_LOOM_LAYERS {
        return None;
    }

    let mut layers = existing.to_vec();
    layers.push((new_pattern, new_color));
    Some(layers)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Stonecutter ───────────────────────────────────────────────────

    #[test]
    fn default_stonecutter_recipes_has_twenty_entries() {
        let recipes = default_stonecutter_recipes();
        assert_eq!(recipes.len(), 20);
    }

    #[test]
    fn stonecutter_stone_to_stairs() {
        let recipes = default_stonecutter_recipes();
        let r = recipes
            .iter()
            .find(|r| r.input == recipe::ITEM_STONE.0 && r.output == ITEM_STONE_STAIRS);
        assert!(r.is_some());
        assert_eq!(r.expect("stone stairs recipe").count, 1);
    }

    #[test]
    fn stonecutter_slab_recipes_yield_two() {
        let recipes = default_stonecutter_recipes();
        let slab_recipes: Vec<_> = recipes.iter().filter(|r| r.count == 2).collect();
        assert!(
            slab_recipes.len() >= 2,
            "expected at least 2 slab recipes yielding 2"
        );
    }

    // ── Grindstone disenchant ─────────────────────────────────────────

    #[test]
    fn disenchant_returns_correct_xp() {
        let result = grindstone_disenchant(recipe::ITEM_DIAMOND_SWORD.0, 1, 3);
        assert_eq!(result.xp_returned, 6); // 3 enchantments * 2 xp each
        assert_eq!(result.output_item, recipe::ITEM_DIAMOND_SWORD.0);
        assert_eq!(result.output_count, 1);
    }

    #[test]
    fn disenchant_zero_enchantments_returns_zero_xp() {
        let result = grindstone_disenchant(recipe::ITEM_IRON_PICKAXE.0, 1, 0);
        assert_eq!(result.xp_returned, 0);
    }

    // ── Grindstone repair ─────────────────────────────────────────────

    #[test]
    fn repair_combines_durability_with_bonus() {
        let result = grindstone_repair(
            (recipe::ITEM_IRON_SWORD.0, 20),
            (recipe::ITEM_IRON_SWORD.0, 20),
        );
        // 20 + 20 + 5% of 64 = 20 + 20 + 3 = 43
        assert_eq!(result, Some((recipe::ITEM_IRON_SWORD.0, 43)));
    }

    #[test]
    fn repair_caps_at_max_durability() {
        let result = grindstone_repair(
            (recipe::ITEM_IRON_SWORD.0, 60),
            (recipe::ITEM_IRON_SWORD.0, 60),
        );
        assert_eq!(result, Some((recipe::ITEM_IRON_SWORD.0, 64)));
    }

    #[test]
    fn repair_rejects_different_items() {
        let result = grindstone_repair(
            (recipe::ITEM_IRON_SWORD.0, 20),
            (recipe::ITEM_DIAMOND_SWORD.0, 20),
        );
        assert!(result.is_none());
    }

    // ── Loom ──────────────────────────────────────────────────────────

    #[test]
    fn loom_adds_layer_to_empty_banner() {
        let result = loom_apply(&[], 1, 5);
        assert_eq!(result, Some(vec![(1, 5)]));
    }

    #[test]
    fn loom_rejects_seventh_layer() {
        let existing: Vec<(u8, u8)> = vec![(1, 0), (2, 1), (3, 2), (4, 3), (5, 4), (6, 5)];
        let result = loom_apply(&existing, 7, 6);
        assert!(result.is_none());
    }

    #[test]
    fn loom_allows_up_to_six_layers() {
        let mut banner: Vec<(u8, u8)> = Vec::new();
        for i in 0..6 {
            let result = loom_apply(&banner, i, i);
            assert!(result.is_some(), "layer {i} should be allowed");
            banner = result.expect("layer should succeed");
        }
        assert_eq!(banner.len(), 6);

        // Seventh layer should fail
        let result = loom_apply(&banner, 6, 6);
        assert!(result.is_none(), "7th layer should be rejected");
    }

    #[test]
    fn loom_does_not_mutate_existing() {
        let existing: Vec<(u8, u8)> = vec![(1, 2)];
        let result = loom_apply(&existing, 3, 4);
        assert_eq!(existing.len(), 1, "original should be unchanged");
        assert_eq!(result.expect("should succeed").len(), 2);
    }
}
