use crate::ItemStack;
use crate::SlotItem;
pub use crate::item_ids::*;
use std::collections::HashMap;

pub use crate::recipe_data::default_recipes;

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
        // 22 original + 16 armor + 11 building + 10 utility + 3 weapons
        // + 10 redstone + 5 misc = 77
        assert_eq!(reg.recipes().len(), 77);
    }

    // ── New recipe tests ───────────────────────────────────────────────────

    #[test]
    fn iron_helmet_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let i = Some(ITEM_IRON_INGOT);
        // Helmet is 3x2: III / I_I -- placed at top of 3x3
        grid.slots = vec![i, i, i, i, None, i, None, None, None];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_IRON_HELMET);
    }

    #[test]
    fn diamond_chestplate_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let d = Some(ITEM_DIAMOND);
        // Chestplate: D_D / DDD / DDD
        grid.slots = vec![d, None, d, d, d, d, d, d, d];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_DIAMOND_CHESTPLATE);
    }

    #[test]
    fn gold_leggings_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let g = Some(ITEM_GOLD_INGOT);
        // Leggings: GGG / G_G / G_G
        grid.slots = vec![g, g, g, g, None, g, g, None, g];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_GOLD_LEGGINGS);
    }

    #[test]
    fn leather_boots_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let l = Some(ITEM_LEATHER);
        // Boots: L_L / L_L -- placed at top of 3x3
        grid.slots = vec![l, None, l, l, None, l, None, None, None];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_LEATHER_BOOTS);
    }

    #[test]
    fn oak_stairs_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let p = Some(ITEM_OAK_PLANKS);
        // Stairs: P__ / PP_ / PPP
        grid.slots = vec![p, None, None, p, p, None, p, p, p];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.item, ITEM_OAK_STAIRS);
        assert_eq!(r.count, 4);
    }

    #[test]
    fn bed_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let w = Some(ITEM_WOOL);
        let p = Some(ITEM_OAK_PLANKS);
        // Bed: WWW / PPP -- placed at top of 3x3
        grid.slots = vec![w, w, w, p, p, p, None, None, None];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_BED);
    }

    #[test]
    fn compass_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let i = Some(ITEM_IRON_INGOT);
        let r = Some(ITEM_REDSTONE_DUST);
        // Compass: _I_ / IRI / _I_
        grid.slots = vec![None, i, None, i, r, i, None, i, None];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_COMPASS);
    }

    #[test]
    fn bow_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let st = Some(ITEM_STICK);
        let sr = Some(ITEM_STRING);
        // Bow: _SM / S_M / _SM  (M = string here)
        grid.slots = vec![None, st, sr, st, None, sr, None, st, sr];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_BOW);
    }

    #[test]
    fn piston_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let p = Some(ITEM_OAK_PLANKS);
        let c = Some(ITEM_COBBLESTONE);
        let i = Some(ITEM_IRON_INGOT);
        let r = Some(ITEM_REDSTONE_DUST);
        // Piston: PPP / CIC / CRC
        grid.slots = vec![p, p, p, c, i, c, c, r, c];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_PISTON);
    }

    #[test]
    fn note_block_recipe() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        let p = Some(ITEM_OAK_PLANKS);
        let r = Some(ITEM_REDSTONE_DUST);
        // Note Block: PPP / PRP / PPP
        grid.slots = vec![p, p, p, p, r, p, p, p, p];
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_NOTE_BLOCK);
    }

    #[test]
    fn jack_o_lantern_shapeless() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        grid.slots[2] = Some(ITEM_PUMPKIN);
        grid.slots[5] = Some(ITEM_TORCH);
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_JACK_O_LANTERN);
    }

    #[test]
    fn lever_shapeless() {
        let reg = default_recipes();
        let mut grid = CraftingGrid::new(3);
        grid.slots[4] = Some(ITEM_STICK);
        grid.slots[7] = Some(ITEM_COBBLESTONE);
        let result = reg.find_match(&grid);
        assert!(result.is_some());
        assert_eq!(result.unwrap().item, ITEM_LEVER);
    }
}
