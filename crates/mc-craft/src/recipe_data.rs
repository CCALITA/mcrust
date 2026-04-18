//! Default recipe definitions for the crafting system (part 1).
//!
//! Basic, tool, armor, and building recipe definitions. The remaining
//! categories live in `recipe_data_ext.rs`.

use crate::ItemStack;
use crate::SlotItem;
use crate::item_ids::*;
use crate::recipe::{Recipe, RecipePattern, RecipeRegistry};

use crate::recipe_data_ext;

/// Populate a registry with 80+ default Minecraft-style recipes.
#[must_use]
pub fn default_recipes() -> RecipeRegistry {
    let mut reg = RecipeRegistry::new();
    add_basic_and_tool_recipes(&mut reg);
    add_armor_recipes(&mut reg);
    add_building_recipes(&mut reg);
    recipe_data_ext::add_utility_recipes(&mut reg);
    recipe_data_ext::add_weapon_recipes(&mut reg);
    recipe_data_ext::add_redstone_recipes(&mut reg);
    recipe_data_ext::add_misc_recipes(&mut reg);
    reg
}

fn s(item: SlotItem) -> Option<SlotItem> {
    Some(item)
}

const N: Option<SlotItem> = None;

// ── Basic & tool recipes ───────────────────────────────────────────────────

fn add_basic_and_tool_recipes(reg: &mut RecipeRegistry) {
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
                N,
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
                N,
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
                    N,
                    s(ITEM_STICK),
                    N,
                    N,
                    s(ITEM_STICK),
                    N,
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
                    N,
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
}

// ── Armor recipes ──────────────────────────────────────────────────────────

fn add_armor_recipes(reg: &mut RecipeRegistry) {
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
                    N,
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
                    N,
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
                    N,
                    s(material),
                    s(material),
                    N,
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
                pattern: vec![s(material), N, s(material), s(material), N, s(material)],
            },
            result: ItemStack {
                item: result_item,
                count: 1,
            },
        });
    }
}

// ── Building recipes ───────────────────────────────────────────────────────

fn add_building_recipes(reg: &mut RecipeRegistry) {
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
                    N,
                    N,
                    s(material),
                    s(material),
                    N,
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
                N,
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                s(ITEM_STICK),
                N,
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
                N,
                s(ITEM_STICK),
                N,
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
}
