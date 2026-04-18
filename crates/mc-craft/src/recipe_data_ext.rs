//! Default recipe definitions for the crafting system (part 2).
//!
//! Utility, weapon, redstone, and miscellaneous recipe definitions.
//! Called from `recipe_data::default_recipes()`.

use crate::ItemStack;
use crate::SlotItem;
use crate::item_ids::*;
use crate::recipe::{Recipe, RecipePattern, RecipeRegistry};

fn s(item: SlotItem) -> Option<SlotItem> {
    Some(item)
}

const N: Option<SlotItem> = None;

// ── Utility recipes ────────────────────────────────────────────────────────

pub(crate) fn add_utility_recipes(reg: &mut RecipeRegistry) {
    // Bucket (3x2: I_I / _I_)
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 2,
            pattern: vec![
                s(ITEM_IRON_INGOT),
                N,
                s(ITEM_IRON_INGOT),
                N,
                s(ITEM_IRON_INGOT),
                N,
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
                N,
                s(ITEM_IRON_INGOT),
                N,
                s(ITEM_IRON_INGOT),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_IRON_INGOT),
                N,
                s(ITEM_IRON_INGOT),
                N,
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
                N,
                s(ITEM_GOLD_INGOT),
                N,
                s(ITEM_GOLD_INGOT),
                s(ITEM_REDSTONE_DUST),
                s(ITEM_GOLD_INGOT),
                N,
                s(ITEM_GOLD_INGOT),
                N,
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
            pattern: vec![N, s(ITEM_IRON_INGOT), s(ITEM_IRON_INGOT), N],
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
                N,
                N,
                s(ITEM_STICK),
                N,
                s(ITEM_STICK),
                s(ITEM_STRING),
                s(ITEM_STICK),
                N,
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
}

// ── Weapon recipes ─────────────────────────────────────────────────────────

pub(crate) fn add_weapon_recipes(reg: &mut RecipeRegistry) {
    // Bow (3x3: _SM / S_M / _SM) -- sticks + string
    reg.add(Recipe {
        pattern: RecipePattern::Shaped {
            width: 3,
            height: 3,
            pattern: vec![
                N,
                s(ITEM_STICK),
                s(ITEM_STRING),
                s(ITEM_STICK),
                N,
                s(ITEM_STRING),
                N,
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
                N,
                s(ITEM_OAK_PLANKS),
                N,
            ],
        },
        result: ItemStack {
            item: ITEM_SHIELD,
            count: 1,
        },
    });
}

// ── Redstone recipes ───────────────────────────────────────────────────────

pub(crate) fn add_redstone_recipes(reg: &mut RecipeRegistry) {
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
                N,
                s(ITEM_REDSTONE_TORCH),
                N,
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
                N,
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
                N,
                s(ITEM_IRON_INGOT),
                s(ITEM_IRON_INGOT),
                s(ITEM_CHEST),
                s(ITEM_IRON_INGOT),
                N,
                s(ITEM_IRON_INGOT),
                N,
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
}

// ── Misc recipes ───────────────────────────────────────────────────────────

pub(crate) fn add_misc_recipes(reg: &mut RecipeRegistry) {
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
                N,
                s(ITEM_IRON_INGOT),
                s(ITEM_IRON_INGOT),
                s(ITEM_STICK),
                s(ITEM_IRON_INGOT),
                s(ITEM_IRON_INGOT),
                N,
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
            pattern: vec![s(ITEM_BRICK), N, s(ITEM_BRICK), N, s(ITEM_BRICK), N],
        },
        result: ItemStack {
            item: ITEM_FLOWER_POT,
            count: 1,
        },
    });
}
