use mc_core::BlockId;

use crate::loot::{LootCondition, LootEntry, LootPool, LootTable};

// ---------------------------------------------------------------------------
// Item IDs (mirrored from drops.rs for loot table references)
// ---------------------------------------------------------------------------

const COBBLESTONE: u16 = BlockId::Cobblestone as u16;
const DIRT: u16 = BlockId::Dirt as u16;
const COAL_ORE: u16 = BlockId::CoalOre as u16;
const GRAVEL: u16 = BlockId::Gravel as u16;

const STICK: u16 = 1000;
const COAL: u16 = 1001;
const DIAMOND: u16 = 1002;
const FLINT: u16 = 1003;
const LAPIS: u16 = 1004;
const EMERALD: u16 = 1005;
const REDSTONE_DUST_ITEM: u16 = 1006;

// ---------------------------------------------------------------------------
// block_loot_table — loot tables for blocks with fortune / silk touch support
// ---------------------------------------------------------------------------

/// Returns the loot table for a block. Supports fortune and silk-touch mechanics.
pub fn block_loot_table(block_id: u16) -> LootTable {
    match BlockId::from_raw(block_id) {
        Some(BlockId::Stone) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    // Silk touch: drop the stone block itself.
                    LootEntry {
                        item_id: BlockId::Stone as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    // Normal: drop cobblestone.
                    LootEntry {
                        item_id: COBBLESTONE,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::DiamondOre) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    // Silk touch: drop the ore block.
                    LootEntry {
                        item_id: BlockId::DiamondOre as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    // Normal: 1 diamond (fortune can extend max).
                    LootEntry {
                        item_id: DIAMOND,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::CoalOre) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: COAL_ORE,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    LootEntry {
                        item_id: COAL,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::LapisOre) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: BlockId::LapisOre as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    LootEntry {
                        item_id: LAPIS,
                        min_count: 4,
                        max_count: 9,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::EmeraldOre) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: BlockId::EmeraldOre as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    LootEntry {
                        item_id: EMERALD,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::RedstoneOre) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: BlockId::RedstoneOre as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    LootEntry {
                        item_id: REDSTONE_DUST_ITEM,
                        min_count: 4,
                        max_count: 5,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::GrassBlock) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: BlockId::GrassBlock as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    LootEntry {
                        item_id: DIRT,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::OakLeaves) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: BlockId::OakLeaves as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                    LootEntry {
                        item_id: STICK,
                        min_count: 1,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::RandomChance(0.1)],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::Gravel) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: FLINT,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        // Fortune increases flint chance — model as higher weight.
                        conditions: vec![LootCondition::RandomChance(0.1)],
                    },
                    LootEntry {
                        item_id: GRAVEL,
                        min_count: 1,
                        max_count: 1,
                        weight: 9,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::Glass) => LootTable {
            pools: vec![LootPool {
                entries: vec![
                    // Silk touch only — glass drops nothing without it.
                    LootEntry {
                        item_id: BlockId::Glass as u16,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::SilkTouch],
                    },
                ],
                rolls: 1,
            }],
        },
        Some(BlockId::Air) | Some(BlockId::Bedrock) | Some(BlockId::Water) => {
            LootTable { pools: vec![] }
        }
        // Default: drop the block itself.
        _ => LootTable {
            pools: vec![LootPool {
                entries: vec![LootEntry {
                    item_id: block_id,
                    min_count: 1,
                    max_count: 1,
                    weight: 1,
                    conditions: vec![LootCondition::Always],
                }],
                rolls: 1,
            }],
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loot::LootContext;

    fn default_ctx(seed: u64) -> LootContext {
        LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: false,
            fortune_level: 0,
            seed,
        }
    }

    // -- Silk Touch on blocks -------------------------------------------------

    #[test]
    fn silk_touch_drops_ore_block_instead_of_item() {
        let table = block_loot_table(BlockId::DiamondOre as u16);

        // With silk touch, should drop the ore block.
        let ctx = LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: true,
            fortune_level: 0,
            seed: 42,
        };
        let drops = table.roll_block(&ctx);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].0, BlockId::DiamondOre as u16);
        assert_eq!(drops[0].1, 1);
    }

    #[test]
    fn no_silk_touch_drops_diamond_from_diamond_ore() {
        let table = block_loot_table(BlockId::DiamondOre as u16);

        let ctx = LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: false,
            fortune_level: 0,
            seed: 42,
        };
        let drops = table.roll_block(&ctx);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].0, DIAMOND);
    }

    #[test]
    fn silk_touch_glass_drops_glass() {
        let table = block_loot_table(BlockId::Glass as u16);

        let ctx_silk = LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: true,
            fortune_level: 0,
            seed: 1,
        };
        let drops = table.roll_block(&ctx_silk);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].0, BlockId::Glass as u16);
    }

    #[test]
    fn no_silk_touch_glass_drops_nothing() {
        let table = block_loot_table(BlockId::Glass as u16);

        let ctx = LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: false,
            fortune_level: 0,
            seed: 1,
        };
        let drops = table.roll_block(&ctx);
        assert!(drops.is_empty());
    }

    // -- Fortune on blocks ----------------------------------------------------

    #[test]
    fn fortune_extends_diamond_ore_max_count() {
        let table = block_loot_table(BlockId::DiamondOre as u16);

        let mut found_above_one = false;
        for seed in 0..500 {
            let ctx = LootContext {
                killed_by_player: true,
                looting_level: 0,
                silk_touch: false,
                fortune_level: 3,
                seed,
            };
            let drops = table.roll_block(&ctx);
            assert_eq!(drops.len(), 1);
            assert_eq!(drops[0].0, DIAMOND);
            // fortune 3 -> effective max = 1 + 3 = 4
            let count = drops[0].1;
            assert!(count >= 1 && count <= 4, "count {count} out of [1,4]");
            if count > 1 {
                found_above_one = true;
            }
        }
        assert!(
            found_above_one,
            "fortune should sometimes produce more than 1 diamond"
        );
    }

    #[test]
    fn fortune_extends_lapis_ore_max_count() {
        let table = block_loot_table(BlockId::LapisOre as u16);

        let mut found_above_nine = false;
        for seed in 0..500 {
            let ctx = LootContext {
                killed_by_player: true,
                looting_level: 0,
                silk_touch: false,
                fortune_level: 3,
                seed,
            };
            let drops = table.roll_block(&ctx);
            assert_eq!(drops.len(), 1);
            assert_eq!(drops[0].0, LAPIS);
            let count = drops[0].1;
            // fortune 3 -> effective max = 9 + 3 = 12
            assert!(count >= 4 && count <= 12, "count {count} out of [4,12]");
            if count > 9 {
                found_above_nine = true;
            }
        }
        assert!(
            found_above_nine,
            "fortune should sometimes produce more than 9 lapis"
        );
    }

    // -- Block loot table defaults --------------------------------------------

    #[test]
    fn block_loot_air_drops_nothing() {
        let table = block_loot_table(BlockId::Air as u16);
        let ctx = default_ctx(0);
        assert!(table.roll_block(&ctx).is_empty());
    }

    #[test]
    fn block_loot_generic_drops_self() {
        let table = block_loot_table(BlockId::OakPlanks as u16);
        let ctx = default_ctx(42);
        let drops = table.roll_block(&ctx);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0], (BlockId::OakPlanks as u16, 1));
    }

    #[test]
    fn block_loot_stone_without_silk_touch_drops_cobblestone() {
        let table = block_loot_table(BlockId::Stone as u16);
        let ctx = default_ctx(42);
        let drops = table.roll_block(&ctx);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].0, COBBLESTONE);
    }

    #[test]
    fn block_loot_stone_with_silk_touch_drops_stone() {
        let table = block_loot_table(BlockId::Stone as u16);
        let ctx = LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: true,
            fortune_level: 0,
            seed: 42,
        };
        let drops = table.roll_block(&ctx);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].0, BlockId::Stone as u16);
    }
}
