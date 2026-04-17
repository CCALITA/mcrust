use crate::{ItemStack, SlotItem};
use std::collections::HashMap;

// ── Item constants ──────────────────────────────────────────────────────────
// These mirror conceptual Minecraft items. SlotItem(N) values are arbitrary
// identifiers; they will be reconciled with mc_core::item::ItemId later.
pub const ITEM_OAK_LOG: SlotItem = SlotItem(100);
pub const ITEM_OAK_PLANKS: SlotItem = SlotItem(101);
pub const ITEM_STICK: SlotItem = SlotItem(102);
pub const ITEM_CRAFTING_TABLE: SlotItem = SlotItem(103);
pub const ITEM_COBBLESTONE: SlotItem = SlotItem(104);
pub const ITEM_FURNACE: SlotItem = SlotItem(105);
pub const ITEM_CHEST: SlotItem = SlotItem(106);
pub const ITEM_COAL: SlotItem = SlotItem(107);
pub const ITEM_TORCH: SlotItem = SlotItem(108);
pub const ITEM_IRON_INGOT: SlotItem = SlotItem(109);
pub const ITEM_GOLD_INGOT: SlotItem = SlotItem(110);
pub const ITEM_DIAMOND: SlotItem = SlotItem(111);
pub const ITEM_WOODEN_PICKAXE: SlotItem = SlotItem(200);
pub const ITEM_WOODEN_AXE: SlotItem = SlotItem(201);
pub const ITEM_WOODEN_SHOVEL: SlotItem = SlotItem(202);
pub const ITEM_WOODEN_SWORD: SlotItem = SlotItem(203);
pub const ITEM_STONE_PICKAXE: SlotItem = SlotItem(210);
pub const ITEM_STONE_AXE: SlotItem = SlotItem(211);
pub const ITEM_STONE_SHOVEL: SlotItem = SlotItem(212);
pub const ITEM_STONE_SWORD: SlotItem = SlotItem(213);
pub const ITEM_IRON_PICKAXE: SlotItem = SlotItem(220);
pub const ITEM_IRON_AXE: SlotItem = SlotItem(221);
pub const ITEM_IRON_SHOVEL: SlotItem = SlotItem(222);
pub const ITEM_IRON_SWORD: SlotItem = SlotItem(223);
pub const ITEM_DIAMOND_PICKAXE: SlotItem = SlotItem(230);
pub const ITEM_DIAMOND_AXE: SlotItem = SlotItem(231);
pub const ITEM_DIAMOND_SHOVEL: SlotItem = SlotItem(232);
pub const ITEM_DIAMOND_SWORD: SlotItem = SlotItem(233);

// ── Recipe types ────────────────────────────────────────────────────────────

/// Describes the arrangement of ingredients in a crafting recipe.
#[derive(Debug, Clone)]
pub enum RecipePattern {
    /// A shaped recipe must match the grid at a specific offset.
    /// `pattern` is row-major, `width * height` elements.
    Shaped {
        width: u8,
        height: u8,
        pattern: Vec<Option<SlotItem>>,
    },
    /// A shapeless recipe matches when the grid contains exactly these
    /// ingredients in any arrangement.
    Shapeless { ingredients: Vec<SlotItem> },
}

/// A crafting recipe: pattern in, result out.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub pattern: RecipePattern,
    pub result: ItemStack,
}

/// The current contents of a crafting grid.
#[derive(Debug, Clone)]
pub struct CraftingGrid {
    /// Row-major slot data, `width * height` elements.
    pub slots: Vec<Option<SlotItem>>,
    /// 2 for the player 2x2 grid, 3 for a crafting table.
    pub width: u8,
}

impl CraftingGrid {
    /// Create an empty crafting grid of the given width (height = width).
    #[must_use]
    pub fn new(width: u8) -> Self {
        let size = (width as usize) * (width as usize);
        Self {
            slots: vec![None; size],
            width,
        }
    }

    #[must_use]
    fn height(&self) -> u8 {
        let w = self.width as usize;
        if w == 0 {
            return 0;
        }
        (self.slots.len() / w) as u8
    }

    /// Get the item at grid position (col, row).
    fn get(&self, col: u8, row: u8) -> Option<SlotItem> {
        let idx = (row as usize) * (self.width as usize) + (col as usize);
        self.slots.get(idx).copied().flatten()
    }
}

/// Registry holding all known crafting recipes.
#[derive(Debug, Clone)]
pub struct RecipeRegistry {
    recipes: Vec<Recipe>,
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeRegistry {
    /// Create an empty recipe registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    /// Register a new recipe.
    pub fn add(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Try to find a recipe matching the current crafting grid contents.
    /// Returns the output `ItemStack` if a match is found.
    #[must_use]
    pub fn find_match(&self, grid: &CraftingGrid) -> Option<ItemStack> {
        for recipe in &self.recipes {
            match &recipe.pattern {
                RecipePattern::Shaped {
                    width,
                    height,
                    pattern,
                } => {
                    if try_match_shaped(grid, *width, *height, pattern) {
                        return Some(recipe.result.clone());
                    }
                }
                RecipePattern::Shapeless { ingredients } => {
                    if try_match_shapeless(grid, ingredients) {
                        return Some(recipe.result.clone());
                    }
                }
            }
        }
        None
    }

    /// Access the list of recipes.
    #[must_use]
    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }
}

// ── Matching helpers ────────────────────────────────────────────────────────

/// Check whether a shaped pattern matches the grid at any valid offset.
fn try_match_shaped(
    grid: &CraftingGrid,
    pat_w: u8,
    pat_h: u8,
    pattern: &[Option<SlotItem>],
) -> bool {
    let grid_w = grid.width;
    let grid_h = grid.height();

    if pat_w > grid_w || pat_h > grid_h {
        return false;
    }

    // Try every valid offset where the pattern could be placed.
    for offset_x in 0..=(grid_w - pat_w) {
        for offset_y in 0..=(grid_h - pat_h) {
            if matches_at(grid, pat_w, pat_h, pattern, offset_x, offset_y) {
                return true;
            }
        }
    }

    false
}

/// Check whether the pattern matches at a specific (offset_x, offset_y) position
/// in the grid, and that all other grid cells are empty.
fn matches_at(
    grid: &CraftingGrid,
    pat_w: u8,
    pat_h: u8,
    pattern: &[Option<SlotItem>],
    offset_x: u8,
    offset_y: u8,
) -> bool {
    let grid_w = grid.width;
    let grid_h = grid.height();

    for row in 0..grid_h {
        for col in 0..grid_w {
            let in_pattern = col >= offset_x
                && col < offset_x + pat_w
                && row >= offset_y
                && row < offset_y + pat_h;

            let grid_item = grid.get(col, row);

            if in_pattern {
                let pat_col = col - offset_x;
                let pat_row = row - offset_y;
                let pat_idx = (pat_row as usize) * (pat_w as usize) + (pat_col as usize);
                let pat_item = pattern[pat_idx];
                if grid_item != pat_item {
                    return false;
                }
            } else {
                // Cells outside the pattern region must be empty.
                if grid_item.is_some() {
                    return false;
                }
            }
        }
    }

    true
}

/// Check whether the grid contains exactly the required shapeless ingredients
/// (order-independent, but counts must match).
fn try_match_shapeless(grid: &CraftingGrid, ingredients: &[SlotItem]) -> bool {
    // Count ingredients required.
    let mut required: HashMap<SlotItem, usize> = HashMap::new();
    for &item in ingredients {
        *required.entry(item).or_insert(0) += 1;
    }

    // Count items present in the grid.
    let mut present: HashMap<SlotItem, usize> = HashMap::new();
    for item in grid.slots.iter().flatten() {
        *present.entry(*item).or_insert(0) += 1;
    }

    required == present
}

// ── Default recipes ─────────────────────────────────────────────────────────

/// Populate a registry with ~20 default Minecraft-style recipes.
#[must_use]
pub fn default_recipes() -> RecipeRegistry {
    let mut reg = RecipeRegistry::new();

    // Helper closures to reduce boilerplate.
    let s = |item: SlotItem| -> Option<SlotItem> { Some(item) };
    let n: Option<SlotItem> = None;

    // ── Shapeless recipes ───────────────────────────────────────────────

    // 4 Oak Planks from 1 Oak Log
    reg.add(Recipe {
        pattern: RecipePattern::Shapeless {
            ingredients: vec![ITEM_OAK_LOG],
        },
        result: ItemStack {
            item: ITEM_OAK_PLANKS,
            count: 4,
        },
    });

    // ── Shaped recipes ──────────────────────────────────────────────────

    // 4 Sticks from 2 Oak Planks (1x2 vertical)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 1,
            height: 2,
            pattern: vec![s(ITEM_OAK_PLANKS), s(ITEM_OAK_PLANKS)],
        },
        result: ItemStack {
            item: ITEM_STICK,
            count: 4,
        },
    });

    // Crafting Table from 4 Oak Planks (2x2)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 2,
            height: 2,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_CRAFTING_TABLE,
            count: 1,
        },
    });

    // Furnace from 8 Cobblestone (3x3 ring)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                n,
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
            ],
        },
        result: ItemStack {
            item: ITEM_FURNACE,
            count: 1,
        },
    });

    // Chest from 8 Oak Planks (3x3 ring)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                n,
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_CHEST,
            count: 1,
        },
    });

    // Torch from Coal + Stick (1x2)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 1,
            height: 2,
            pattern: vec![s(ITEM_COAL), s(ITEM_STICK)],
        },
        result: ItemStack {
            item: ITEM_TORCH,
            count: 4,
        },
    });

    // ── Tool recipes ────────────────────────────────────────────────────

    // Pickaxes (3x3: MMM / _S_ / _S_)
    for (material, result_item) in [
        (ITEM_OAK_PLANKS, ITEM_WOODEN_PICKAXE),
        (ITEM_COBBLESTONE, ITEM_STONE_PICKAXE),
        (ITEM_IRON_INGOT, ITEM_IRON_PICKAXE),
        (ITEM_DIAMOND, ITEM_DIAMOND_PICKAXE),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 3,
                height: 3,
                pattern: vec![
                    s(material),
                    s(material),
                    s(material),
                    n,
                    s(ITEM_STICK),
                    n,
                    n,
                    s(ITEM_STICK),
                    n,
                ],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    // Axes (3x3: MM_ / MS_ / _S_)
    for (material, result_item) in [
        (ITEM_OAK_PLANKS, ITEM_WOODEN_AXE),
        (ITEM_COBBLESTONE, ITEM_STONE_AXE),
        (ITEM_IRON_INGOT, ITEM_IRON_AXE),
        (ITEM_DIAMOND, ITEM_DIAMOND_AXE),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 2,
                height: 3,
                pattern: vec![
                    s(material),
                    s(material),
                    s(material),
                    s(ITEM_STICK),
                    n,
                    s(ITEM_STICK),
                ],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    // Shovels (1x3: M / S / S)
    for (material, result_item) in [
        (ITEM_OAK_PLANKS, ITEM_WOODEN_SHOVEL),
        (ITEM_COBBLESTONE, ITEM_STONE_SHOVEL),
        (ITEM_IRON_INGOT, ITEM_IRON_SHOVEL),
        (ITEM_DIAMOND, ITEM_DIAMOND_SHOVEL),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 1,
                height: 3,
                pattern: vec![s(material), s(ITEM_STICK), s(ITEM_STICK)],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    // Swords (1x3: M / M / S)
    for (material, result_item) in [
        (ITEM_OAK_PLANKS, ITEM_WOODEN_SWORD),
        (ITEM_COBBLESTONE, ITEM_STONE_SWORD),
        (ITEM_IRON_INGOT, ITEM_IRON_SWORD),
        (ITEM_DIAMOND, ITEM_DIAMOND_SWORD),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 1,
                height: 3,
                pattern: vec![s(material), s(material), s(ITEM_STICK)],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapeless_planks_from_log() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(2);
        grid.slots[0] = Some(ITEM_OAK_LOG);
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.item, ITEM_OAK_PLANKS);
        assert_eq!(result.count, 4);
    }

    #[test]
    fn shapeless_planks_any_position() {
        let reg = default_recipes();
        // Place log in bottom-right of a 3x3 grid.
        let mut grid = CraftingGrid::new(3);
        grid.slots[8] = Some(ITEM_OAK_LOG);
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_OAK_PLANKS);
    }

    #[test]
    fn shapeless_does_not_match_extra_items() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(2);
        grid.slots[0] = Some(ITEM_OAK_LOG);
        grid.slots[1] = Some(ITEM_COBBLESTONE);
        let result = reg.find_match(&grid);
        assert!(result.is_none());
    }

    #[test]
    fn shaped_sticks_from_planks() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        // Place two planks in a vertical column.
        grid.slots[1] = Some(ITEM_OAK_PLANKS); // col 1, row 0
        grid.slots[4] = Some(ITEM_OAK_PLANKS); // col 1, row 1
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.item, ITEM_STICK);
        assert_eq!(result.count, 4);
    }

    #[test]
    fn shaped_crafting_table() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(2);
        for slot in &mut grid.slots {
            *slot = Some(ITEM_OAK_PLANKS);
        }
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_CRAFTING_TABLE);
    }

    #[test]
    fn shaped_furnace_3x3_ring() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let cobble = Some(ITEM_COBBLESTONE);
        grid.slots = vec![
            cobble, cobble, cobble, cobble, None, cobble, cobble, cobble, cobble,
        ];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_FURNACE);
    }

    #[test]
    fn shaped_pickaxe_pattern() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let m = Some(ITEM_OAK_PLANKS);
        let stick = Some(ITEM_STICK);
        grid.slots = vec![m, m, m, None, stick, None, None, stick, None];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_WOODEN_PICKAXE);
    }

    #[test]
    fn shaped_sword_pattern() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let m = Some(ITEM_DIAMOND);
        let stick = Some(ITEM_STICK);
        // Sword is 1x3, placed in column 0.
        grid.slots = vec![m, None, None, m, None, None, stick, None, None];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_DIAMOND_SWORD);
    }

    #[test]
    fn shaped_pattern_offset_in_grid() {
        let reg = default_recipes();
        // Place a 2x2 crafting table pattern at offset (1,1) in a 3x3 grid.
        let mut grid = CraftingGrid::new(3);
        let p = Some(ITEM_OAK_PLANKS);
        grid.slots = vec![None, None, None, None, p, p, None, p, p];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_CRAFTING_TABLE);
    }

    #[test]
    fn no_match_on_empty_grid() {
        let reg = default_recipes();
        let grid = CraftingGrid::new(3);
        assert!(reg.find_match(&grid).is_none());
    }

    #[test]
    fn torch_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        grid.slots[0] = Some(ITEM_COAL);
        grid.slots[3] = Some(ITEM_STICK);
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.item, ITEM_TORCH);
        assert_eq!(result.count, 4);
    }

    #[test]
    fn registry_recipe_count() {
        let reg = default_recipes();
        // 1 shapeless + 5 basic shaped + 4 * 4 tools = 22 recipes
        assert_eq!(reg.recipes().len(), 22);
    }
}
