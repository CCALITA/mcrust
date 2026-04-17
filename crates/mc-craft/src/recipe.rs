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

// ── Armor items ────────────────────────────────────────────────────────────
pub const ITEM_LEATHER: SlotItem = SlotItem(112);
pub const ITEM_LEATHER_HELMET: SlotItem = SlotItem(300);
pub const ITEM_LEATHER_CHESTPLATE: SlotItem = SlotItem(301);
pub const ITEM_LEATHER_LEGGINGS: SlotItem = SlotItem(302);
pub const ITEM_LEATHER_BOOTS: SlotItem = SlotItem(303);
pub const ITEM_IRON_HELMET: SlotItem = SlotItem(310);
pub const ITEM_IRON_CHESTPLATE: SlotItem = SlotItem(311);
pub const ITEM_IRON_LEGGINGS: SlotItem = SlotItem(312);
pub const ITEM_IRON_BOOTS: SlotItem = SlotItem(313);
pub const ITEM_GOLD_HELMET: SlotItem = SlotItem(320);
pub const ITEM_GOLD_CHESTPLATE: SlotItem = SlotItem(321);
pub const ITEM_GOLD_LEGGINGS: SlotItem = SlotItem(322);
pub const ITEM_GOLD_BOOTS: SlotItem = SlotItem(323);
pub const ITEM_DIAMOND_HELMET: SlotItem = SlotItem(330);
pub const ITEM_DIAMOND_CHESTPLATE: SlotItem = SlotItem(331);
pub const ITEM_DIAMOND_LEGGINGS: SlotItem = SlotItem(332);
pub const ITEM_DIAMOND_BOOTS: SlotItem = SlotItem(333);

// ── Building items ─────────────────────────────────────────────────────────
pub const ITEM_OAK_STAIRS: SlotItem = SlotItem(400);
pub const ITEM_COBBLESTONE_STAIRS: SlotItem = SlotItem(401);
pub const ITEM_OAK_SLAB: SlotItem = SlotItem(402);
pub const ITEM_COBBLESTONE_SLAB: SlotItem = SlotItem(403);
pub const ITEM_OAK_FENCE: SlotItem = SlotItem(404);
pub const ITEM_OAK_FENCE_GATE: SlotItem = SlotItem(405);
pub const ITEM_OAK_DOOR: SlotItem = SlotItem(406);
pub const ITEM_OAK_TRAPDOOR: SlotItem = SlotItem(407);
pub const ITEM_LADDER: SlotItem = SlotItem(408);
pub const ITEM_OAK_SIGN: SlotItem = SlotItem(409);
pub const ITEM_WOOL: SlotItem = SlotItem(410);
pub const ITEM_BED: SlotItem = SlotItem(411);

// ── Utility items ──────────────────────────────────────────────────────────
pub const ITEM_BUCKET: SlotItem = SlotItem(500);
pub const ITEM_REDSTONE_DUST: SlotItem = SlotItem(501);
pub const ITEM_COMPASS: SlotItem = SlotItem(502);
pub const ITEM_CLOCK: SlotItem = SlotItem(503);
pub const ITEM_SHEARS: SlotItem = SlotItem(504);
pub const ITEM_FISHING_ROD: SlotItem = SlotItem(505);
pub const ITEM_STRING: SlotItem = SlotItem(506);
pub const ITEM_BOOKSHELF: SlotItem = SlotItem(507);
pub const ITEM_BOOK: SlotItem = SlotItem(508);
pub const ITEM_PAPER: SlotItem = SlotItem(509);
pub const ITEM_SUGAR_CANE: SlotItem = SlotItem(510);
pub const ITEM_GUNPOWDER: SlotItem = SlotItem(511);
pub const ITEM_SAND: SlotItem = SlotItem(512);
pub const ITEM_TNT: SlotItem = SlotItem(513);
pub const ITEM_PUMPKIN: SlotItem = SlotItem(514);
pub const ITEM_JACK_O_LANTERN: SlotItem = SlotItem(515);

// ── Weapon items ───────────────────────────────────────────────────────────
pub const ITEM_BOW: SlotItem = SlotItem(600);
pub const ITEM_ARROW: SlotItem = SlotItem(601);
pub const ITEM_FLINT: SlotItem = SlotItem(602);
pub const ITEM_FEATHER: SlotItem = SlotItem(603);
pub const ITEM_SHIELD: SlotItem = SlotItem(604);

// ── Redstone items ─────────────────────────────────────────────────────────
pub const ITEM_REDSTONE_TORCH: SlotItem = SlotItem(700);
pub const ITEM_REPEATER: SlotItem = SlotItem(701);
pub const ITEM_COMPARATOR: SlotItem = SlotItem(702);
pub const ITEM_QUARTZ: SlotItem = SlotItem(703);
pub const ITEM_PISTON: SlotItem = SlotItem(704);
pub const ITEM_OBSERVER: SlotItem = SlotItem(705);
pub const ITEM_DISPENSER: SlotItem = SlotItem(706);
pub const ITEM_DROPPER: SlotItem = SlotItem(707);
pub const ITEM_HOPPER: SlotItem = SlotItem(708);
pub const ITEM_LEVER: SlotItem = SlotItem(709);
pub const ITEM_STONE_BUTTON: SlotItem = SlotItem(710);
pub const ITEM_STONE: SlotItem = SlotItem(711);

// ── Misc items ─────────────────────────────────────────────────────────────
pub const ITEM_NOTE_BLOCK: SlotItem = SlotItem(800);
pub const ITEM_RAIL: SlotItem = SlotItem(801);
pub const ITEM_PAINTING: SlotItem = SlotItem(802);
pub const ITEM_ITEM_FRAME: SlotItem = SlotItem(803);
pub const ITEM_FLOWER_POT: SlotItem = SlotItem(804);
pub const ITEM_BRICK: SlotItem = SlotItem(805);

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

/// Populate a registry with 80+ default Minecraft-style recipes.
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

    // ── Armor recipes ──────────────────────────────────────────────────────

    // Helmets (3x2: MMM / M_M)
    for (material, result_item) in [
        (ITEM_LEATHER, ITEM_LEATHER_HELMET),
        (ITEM_IRON_INGOT, ITEM_IRON_HELMET),
        (ITEM_GOLD_INGOT, ITEM_GOLD_HELMET),
        (ITEM_DIAMOND, ITEM_DIAMOND_HELMET),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 3,
                height: 2,
                pattern: vec![
                    s(material),
                    s(material),
                    s(material),
                    s(material),
                    n,
                    s(material),
                ],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    // Chestplates (3x3: M_M / MMM / MMM)
    for (material, result_item) in [
        (ITEM_LEATHER, ITEM_LEATHER_CHESTPLATE),
        (ITEM_IRON_INGOT, ITEM_IRON_CHESTPLATE),
        (ITEM_GOLD_INGOT, ITEM_GOLD_CHESTPLATE),
        (ITEM_DIAMOND, ITEM_DIAMOND_CHESTPLATE),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 3,
                height: 3,
                pattern: vec![
                    s(material),
                    n,
                    s(material),
                    s(material),
                    s(material),
                    s(material),
                    s(material),
                    s(material),
                    s(material),
                ],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    // Leggings (3x3: MMM / M_M / M_M)
    for (material, result_item) in [
        (ITEM_LEATHER, ITEM_LEATHER_LEGGINGS),
        (ITEM_IRON_INGOT, ITEM_IRON_LEGGINGS),
        (ITEM_GOLD_INGOT, ITEM_GOLD_LEGGINGS),
        (ITEM_DIAMOND, ITEM_DIAMOND_LEGGINGS),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 3,
                height: 3,
                pattern: vec![
                    s(material),
                    s(material),
                    s(material),
                    s(material),
                    n,
                    s(material),
                    s(material),
                    n,
                    s(material),
                ],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    // Boots (3x2: M_M / M_M)
    for (material, result_item) in [
        (ITEM_LEATHER, ITEM_LEATHER_BOOTS),
        (ITEM_IRON_INGOT, ITEM_IRON_BOOTS),
        (ITEM_GOLD_INGOT, ITEM_GOLD_BOOTS),
        (ITEM_DIAMOND, ITEM_DIAMOND_BOOTS),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 3,
                height: 2,
                pattern: vec![s(material), n, s(material), s(material), n, s(material)],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }

    // ── Building recipes ───────────────────────────────────────────────────

    // Stairs (3x3: M__ / MM_ / MMM) -- yields 4
    for (material, result_item) in [
        (ITEM_OAK_PLANKS, ITEM_OAK_STAIRS),
        (ITEM_COBBLESTONE, ITEM_COBBLESTONE_STAIRS),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 3,
                height: 3,
                pattern: vec![
                    s(material),
                    n,
                    n,
                    s(material),
                    s(material),
                    n,
                    s(material),
                    s(material),
                    s(material),
                ],
            },
            result: ItemStack {
                item: result_item,
                count: 4,
            },
        });
    }

    // Slabs (3x1: MMM) -- yields 6
    for (material, result_item) in [
        (ITEM_OAK_PLANKS, ITEM_OAK_SLAB),
        (ITEM_COBBLESTONE, ITEM_COBBLESTONE_SLAB),
    ] {
        reg.add(Recipe {
            pattern: RecipePattern::Shaped {
                width: 3,
                height: 1,
                pattern: vec![s(material), s(material), s(material)],
            },
            result: ItemStack {
                item: result_item,
                count: 6,
            },
        });
    }

    // Fence (3x2: PSP / PSP) -- yields 3
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_STICK),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_STICK),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_OAK_FENCE,
            count: 3,
        },
    });

    // Fence Gate (3x2: SPS / SPS)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![
                s(ITEM_STICK),
                s(ITEM_OAK_PLANKS),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_OAK_PLANKS),
                s(ITEM_STICK),
            ],
        },
        result: ItemStack {
            item: ITEM_OAK_FENCE_GATE,
            count: 1,
        },
    });

    // Door (2x3: PP / PP / PP) -- yields 3
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 2,
            height: 3,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_OAK_DOOR,
            count: 3,
        },
    });

    // Trapdoor (3x2: PPP / PPP) -- yields 2
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_OAK_TRAPDOOR,
            count: 2,
        },
    });

    // Ladder (3x3: S_S / SSS / S_S) -- yields 3
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_STICK),
                n,
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                n,
                s(ITEM_STICK),
            ],
        },
        result: ItemStack {
            item: ITEM_LADDER,
            count: 3,
        },
    });

    // Sign (3x3: PPP / PPP / _S_) -- yields 3
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                n,
                s(ITEM_STICK),
                n,
            ],
        },
        result: ItemStack {
            item: ITEM_OAK_SIGN,
            count: 3,
        },
    });

    // Bed (3x2: WWW / PPP)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![
                s(ITEM_WOOL),
                s(ITEM_WOOL),
                s(ITEM_WOOL),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_BED,
            count: 1,
        },
    });

    // ── Utility recipes ────────────────────────────────────────────────────

    // Bucket (3x2: I_I / _I_)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
                n,
            ],
        },
        result: ItemStack {
            item: ITEM_BUCKET,
            count: 1,
        },
    });

    // Compass (3x3: _I_ / IRI / _I_)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                n,
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
                n,
            ],
        },
        result: ItemStack {
            item: ITEM_COMPASS,
            count: 1,
        },
    });

    // Clock (3x3: _G_ / GRG / _G_)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                n,
                s(ITEM_GOLD_INGOT),
                n,
                s(ITEM_GOLD_INGOT),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_GOLD_INGOT),
                n,
                s(ITEM_GOLD_INGOT),
                n,
            ],
        },
        result: ItemStack {
            item: ITEM_CLOCK,
            count: 1,
        },
    });

    // Shears (2x2: _I / I_)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 2,
            height: 2,
            pattern: vec![n, s(ITEM_IRON_INGOT), s(ITEM_IRON_INGOT), n],
        },
        result: ItemStack {
            item: ITEM_SHEARS,
            count: 1,
        },
    });

    // Fishing Rod (3x3: __S / _SI / S_I)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                n,
                n,
                s(ITEM_STICK),
                n,
                s(ITEM_STICK),
                s(ITEM_STRING),
                s(ITEM_STICK),
                n,
                s(ITEM_STRING),
            ],
        },
        result: ItemStack {
            item: ITEM_FISHING_ROD,
            count: 1,
        },
    });

    // Bookshelf (3x3: PPP / BBB / PPP)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_BOOK),
                s(ITEM_BOOK),
                s(ITEM_BOOK),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_BOOKSHELF,
            count: 1,
        },
    });

    // Book (shapeless: 3 Paper + 1 Leather)
    reg.add(Recipe {
        pattern: RecipePattern::Shapeless {
            ingredients: vec![ITEM_PAPER, ITEM_PAPER, ITEM_PAPER, ITEM_LEATHER],
        },
        result: ItemStack {
            item: ITEM_BOOK,
            count: 1,
        },
    });

    // Paper (3x1: CCC) -- sugar cane -- yields 3
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 1,
            pattern: vec![s(ITEM_SUGAR_CANE), s(ITEM_SUGAR_CANE), s(ITEM_SUGAR_CANE)],
        },
        result: ItemStack {
            item: ITEM_PAPER,
            count: 3,
        },
    });

    // TNT (3x3: GSG / SGS / GSG)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_GUNPOWDER),
                s(ITEM_SAND),
                s(ITEM_GUNPOWDER),
                s(ITEM_SAND),
                s(ITEM_GUNPOWDER),
                s(ITEM_SAND),
                s(ITEM_GUNPOWDER),
                s(ITEM_SAND),
                s(ITEM_GUNPOWDER),
            ],
        },
        result: ItemStack {
            item: ITEM_TNT,
            count: 1,
        },
    });

    // Jack o'Lantern (shapeless: Pumpkin + Torch)
    reg.add(Recipe {
        pattern: RecipePattern::Shapeless {
            ingredients: vec![ITEM_PUMPKIN, ITEM_TORCH],
        },
        result: ItemStack {
            item: ITEM_JACK_O_LANTERN,
            count: 1,
        },
    });

    // ── Weapon recipes ─────────────────────────────────────────────────────

    // Bow (3x3: _SM / S_M / _SM) -- sticks + string
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                n,
                s(ITEM_STICK),
                s(ITEM_STRING),
                s(ITEM_STICK),
                n,
                s(ITEM_STRING),
                n,
                s(ITEM_STICK),
                s(ITEM_STRING),
            ],
        },
        result: ItemStack {
            item: ITEM_BOW,
            count: 1,
        },
    });

    // Arrow (1x3: F / S / Fe) -- flint, stick, feather -- yields 4
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 1,
            height: 3,
            pattern: vec![s(ITEM_FLINT), s(ITEM_STICK), s(ITEM_FEATHER)],
        },
        result: ItemStack {
            item: ITEM_ARROW,
            count: 4,
        },
    });

    // Shield (3x3: PIP / PPP / _P_)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_IRON_INGOT),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                n,
                s(ITEM_OAK_PLANKS),
                n,
            ],
        },
        result: ItemStack {
            item: ITEM_SHIELD,
            count: 1,
        },
    });

    // ── Redstone recipes ───────────────────────────────────────────────────

    // Redstone Torch (1x2: R / S)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 1,
            height: 2,
            pattern: vec![s(ITEM_REDSTONE_DUST), s(ITEM_STICK)],
        },
        result: ItemStack {
            item: ITEM_REDSTONE_TORCH,
            count: 1,
        },
    });

    // Repeater (3x2: TRT / SSS)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![
                s(ITEM_REDSTONE_TORCH),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_REDSTONE_TORCH),
                s(ITEM_STONE),
                s(ITEM_STONE),
                s(ITEM_STONE),
            ],
        },
        result: ItemStack {
            item: ITEM_REPEATER,
            count: 1,
        },
    });

    // Comparator (3x3: _T_ / TQT / SSS)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                n,
                s(ITEM_REDSTONE_TORCH),
                n,
                s(ITEM_REDSTONE_TORCH),
                s(ITEM_QUARTZ),
                s(ITEM_REDSTONE_TORCH),
                s(ITEM_STONE),
                s(ITEM_STONE),
                s(ITEM_STONE),
            ],
        },
        result: ItemStack {
            item: ITEM_COMPARATOR,
            count: 1,
        },
    });

    // Piston (3x3: PPP / CIC / CRC)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_COBBLESTONE),
                s(ITEM_IRON_INGOT),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_COBBLESTONE),
            ],
        },
        result: ItemStack {
            item: ITEM_PISTON,
            count: 1,
        },
    });

    // Observer (3x3: CCC / RRQ / CCC)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_QUARTZ),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
            ],
        },
        result: ItemStack {
            item: ITEM_OBSERVER,
            count: 1,
        },
    });

    // Dispenser (3x3: CCC / CBC / CRC)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_BOW),
                s(ITEM_COBBLESTONE),
                s(ITEM_COBBLESTONE),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_COBBLESTONE),
            ],
        },
        result: ItemStack {
            item: ITEM_DISPENSER,
            count: 1,
        },
    });

    // Dropper (3x3: CCC / C_C / CRC)
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
                s(ITEM_REDSTONE_DUST),
                s(ITEM_COBBLESTONE),
            ],
        },
        result: ItemStack {
            item: ITEM_DROPPER,
            count: 1,
        },
    });

    // Hopper (3x3: I_I / ICI / _I_)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
                s(ITEM_IRON_INGOT),
                s(ITEM_CHEST),
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
                n,
            ],
        },
        result: ItemStack {
            item: ITEM_HOPPER,
            count: 1,
        },
    });

    // Lever (shapeless: Stick + Cobblestone)
    reg.add(Recipe {
        pattern: RecipePattern::Shapeless {
            ingredients: vec![ITEM_STICK, ITEM_COBBLESTONE],
        },
        result: ItemStack {
            item: ITEM_LEVER,
            count: 1,
        },
    });

    // Stone Button (shapeless: 1 Stone)
    reg.add(Recipe {
        pattern: RecipePattern::Shapeless {
            ingredients: vec![ITEM_STONE],
        },
        result: ItemStack {
            item: ITEM_STONE_BUTTON,
            count: 1,
        },
    });

    // ── Misc recipes ───────────────────────────────────────────────────────

    // Note Block (3x3: PPP / PRP / PPP)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
                s(ITEM_OAK_PLANKS),
            ],
        },
        result: ItemStack {
            item: ITEM_NOTE_BLOCK,
            count: 1,
        },
    });

    // Rail (3x3: I_I / ISI / I_I) -- yields 16
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
                s(ITEM_IRON_INGOT),
                s(ITEM_STICK),
                s(ITEM_IRON_INGOT),
                s(ITEM_IRON_INGOT),
                n,
                s(ITEM_IRON_INGOT),
            ],
        },
        result: ItemStack {
            item: ITEM_RAIL,
            count: 16,
        },
    });

    // Painting (3x3: SSS / SWS / SSS) -- sticks + wool
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_WOOL),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
            ],
        },
        result: ItemStack {
            item: ITEM_PAINTING,
            count: 1,
        },
    });

    // Item Frame (3x3: SSS / SLS / SSS) -- sticks + leather
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_LEATHER),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
            ],
        },
        result: ItemStack {
            item: ITEM_ITEM_FRAME,
            count: 1,
        },
    });

    // Flower Pot (3x2: B_B / _B_) -- 3 bricks
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![s(ITEM_BRICK), n, s(ITEM_BRICK), n, s(ITEM_BRICK), n],
        },
        result: ItemStack {
            item: ITEM_FLOWER_POT,
            count: 1,
        },
    });

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
