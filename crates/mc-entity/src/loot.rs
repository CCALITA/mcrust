use mc_core::BlockId;

// ---------------------------------------------------------------------------
// Item IDs (mirrored from drops.rs for loot table references)
// ---------------------------------------------------------------------------

const COBBLESTONE: u16 = BlockId::Cobblestone as u16;
const DIRT: u16 = BlockId::Dirt as u16;
const COAL_ORE: u16 = BlockId::CoalOre as u16;
const IRON_ORE: u16 = BlockId::IronOre as u16;
const GOLD_ORE: u16 = BlockId::GoldOre as u16;
const GRAVEL: u16 = BlockId::Gravel as u16;

const STICK: u16 = 1000;
const COAL: u16 = 1001;
const DIAMOND: u16 = 1002;
const FLINT: u16 = 1003;
const LAPIS: u16 = 1004;
const EMERALD: u16 = 1005;
const REDSTONE_DUST_ITEM: u16 = 1006;

// Mob drop item IDs
const ROTTEN_FLESH: u16 = 2000;
const BONE: u16 = 2001;
const ARROW: u16 = 2002;
const GUNPOWDER: u16 = 2003;
const STRING_ITEM: u16 = 2004;
const RAW_PORKCHOP: u16 = 2005;
const LEATHER: u16 = 2006;
const RAW_BEEF: u16 = 2007;
const WOOL: u16 = 2008;
const FEATHER: u16 = 2009;
const RAW_CHICKEN: u16 = 2010;
const SPIDER_EYE: u16 = 2011;
const MUSIC_DISC: u16 = 2012;

// Mob kind discriminants (matches component::MobKind repr order)
const MOB_ZOMBIE: u8 = 0;
const MOB_SKELETON: u8 = 1;
const MOB_CREEPER: u8 = 2;
const MOB_SPIDER: u8 = 3;
const MOB_PIG: u8 = 4;
const MOB_COW: u8 = 5;
const MOB_SHEEP: u8 = 6;
const MOB_CHICKEN: u8 = 7;

// ---------------------------------------------------------------------------
// LootCondition
// ---------------------------------------------------------------------------

/// A condition that must be met for a loot entry to be eligible.
#[derive(Debug, Clone, PartialEq)]
pub enum LootCondition {
    /// Always passes.
    Always,
    /// Requires the mob to have been killed by a player (not environment).
    KilledByPlayer,
    /// Requires at least the given looting enchantment level.
    HasLooting(u8),
    /// Passes with the given probability in `[0.0, 1.0]`.
    RandomChance(f32),
    /// Requires the player to be using a Silk Touch tool.
    SilkTouch,
}

// ---------------------------------------------------------------------------
// LootEntry / LootPool / LootTable
// ---------------------------------------------------------------------------

/// A single possible item drop within a loot pool.
#[derive(Debug, Clone)]
pub struct LootEntry {
    pub item_id: u16,
    pub min_count: u8,
    pub max_count: u8,
    pub weight: u32,
    pub conditions: Vec<LootCondition>,
}

/// A pool of entries rolled a fixed number of times.
#[derive(Debug, Clone)]
pub struct LootPool {
    pub entries: Vec<LootEntry>,
    pub rolls: u8,
}

/// A complete loot table composed of one or more pools.
#[derive(Debug, Clone)]
pub struct LootTable {
    pub pools: Vec<LootPool>,
}

// ---------------------------------------------------------------------------
// LootContext
// ---------------------------------------------------------------------------

/// Runtime context passed when rolling a loot table.
#[derive(Debug, Clone)]
pub struct LootContext {
    pub killed_by_player: bool,
    pub looting_level: u8,
    pub silk_touch: bool,
    pub fortune_level: u8,
    pub seed: u64,
}

// ---------------------------------------------------------------------------
// Deterministic PRNG helpers (no external rand dependency for loot rolls)
// ---------------------------------------------------------------------------

/// Simple splitmix64-style hash for deterministic pseudo-random numbers.
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Returns a value in `[0.0, 1.0)`.
fn rand_f32(state: &mut u64) -> f32 {
    (splitmix(state) >> 40) as f32 / (1u64 << 24) as f32
}

/// Returns a value in `[min, max]` (inclusive on both ends).
fn rand_range_u8(state: &mut u64, min: u8, max: u8) -> u8 {
    if min >= max {
        return min;
    }
    let range = (max - min) as u64 + 1;
    let v = splitmix(state) % range;
    min + v as u8
}

// ---------------------------------------------------------------------------
// LootTable::roll
// ---------------------------------------------------------------------------

impl LootTable {
    /// Roll the loot table and return a list of `(item_id, count)` drops.
    pub fn roll(&self, ctx: &LootContext) -> Vec<(u16, u8)> {
        let mut results: Vec<(u16, u8)> = Vec::new();
        let mut rng_state = ctx.seed;

        for pool in &self.pools {
            for _ in 0..pool.rolls {
                // Filter eligible entries.
                let eligible: Vec<&LootEntry> = pool
                    .entries
                    .iter()
                    .filter(|e| check_conditions(&e.conditions, ctx, &mut rng_state))
                    .collect();

                if eligible.is_empty() {
                    continue;
                }

                // Weighted random selection.
                let total_weight: u32 = eligible.iter().map(|e| e.weight).sum();
                if total_weight == 0 {
                    continue;
                }

                let pick = (splitmix(&mut rng_state) % total_weight as u64) as u32;
                let mut cumulative = 0u32;
                let mut chosen: Option<&LootEntry> = None;
                for entry in &eligible {
                    cumulative += entry.weight;
                    if pick < cumulative {
                        chosen = Some(entry);
                        break;
                    }
                }

                if let Some(entry) = chosen {
                    // Looting extends max_count by looting_level.
                    let effective_max =
                        entry.max_count.saturating_add(ctx.looting_level);
                    let count = rand_range_u8(&mut rng_state, entry.min_count, effective_max);
                    if count > 0 {
                        results.push((entry.item_id, count));
                    }
                }
            }
        }

        results
    }
}

/// Evaluate all conditions for an entry. All must pass.
fn check_conditions(
    conditions: &[LootCondition],
    ctx: &LootContext,
    rng_state: &mut u64,
) -> bool {
    conditions.iter().all(|cond| match cond {
        LootCondition::Always => true,
        LootCondition::KilledByPlayer => ctx.killed_by_player,
        LootCondition::HasLooting(level) => ctx.looting_level >= *level,
        LootCondition::RandomChance(chance) => rand_f32(rng_state) < *chance,
        LootCondition::SilkTouch => ctx.silk_touch,
    })
}

// ---------------------------------------------------------------------------
// mob_loot_table — loot tables for all 8 mob types
// ---------------------------------------------------------------------------

/// Returns the loot table for a mob of the given kind (u8 discriminant of `MobKind`).
pub fn mob_loot_table(kind: u8) -> LootTable {
    match kind {
        MOB_ZOMBIE => LootTable {
            pools: vec![
                LootPool {
                    entries: vec![LootEntry {
                        item_id: ROTTEN_FLESH,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
            ],
        },
        MOB_SKELETON => LootTable {
            pools: vec![
                LootPool {
                    entries: vec![LootEntry {
                        item_id: BONE,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
                LootPool {
                    entries: vec![LootEntry {
                        item_id: ARROW,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
            ],
        },
        MOB_CREEPER => LootTable {
            pools: vec![
                LootPool {
                    entries: vec![LootEntry {
                        item_id: GUNPOWDER,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
                // Rare music disc drop when killed by skeleton
                LootPool {
                    entries: vec![LootEntry {
                        item_id: MUSIC_DISC,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![
                            LootCondition::KilledByPlayer,
                            LootCondition::RandomChance(0.05),
                        ],
                    }],
                    rolls: 1,
                },
            ],
        },
        MOB_SPIDER => LootTable {
            pools: vec![
                LootPool {
                    entries: vec![LootEntry {
                        item_id: STRING_ITEM,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
                // Rare spider eye (player kill only)
                LootPool {
                    entries: vec![LootEntry {
                        item_id: SPIDER_EYE,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![
                            LootCondition::KilledByPlayer,
                            LootCondition::RandomChance(0.33),
                        ],
                    }],
                    rolls: 1,
                },
            ],
        },
        MOB_PIG => LootTable {
            pools: vec![LootPool {
                entries: vec![LootEntry {
                    item_id: RAW_PORKCHOP,
                    min_count: 1,
                    max_count: 3,
                    weight: 1,
                    conditions: vec![LootCondition::Always],
                }],
                rolls: 1,
            }],
        },
        MOB_COW => LootTable {
            pools: vec![
                LootPool {
                    entries: vec![LootEntry {
                        item_id: RAW_BEEF,
                        min_count: 1,
                        max_count: 3,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
                LootPool {
                    entries: vec![LootEntry {
                        item_id: LEATHER,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
            ],
        },
        MOB_SHEEP => LootTable {
            pools: vec![LootPool {
                entries: vec![LootEntry {
                    item_id: WOOL,
                    min_count: 1,
                    max_count: 1,
                    weight: 1,
                    conditions: vec![LootCondition::Always],
                }],
                rolls: 1,
            }],
        },
        MOB_CHICKEN => LootTable {
            pools: vec![
                LootPool {
                    entries: vec![LootEntry {
                        item_id: RAW_CHICKEN,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
                LootPool {
                    entries: vec![LootEntry {
                        item_id: FEATHER,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
            ],
        },
        // Unknown mob kind — empty table.
        _ => LootTable { pools: vec![] },
    }
}

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
// Fortune helper for block loot
// ---------------------------------------------------------------------------

impl LootTable {
    /// Roll a block loot table with fortune support.
    ///
    /// Fortune increases effective `max_count` on ore-type entries by
    /// `fortune_level`, similar to how looting works for mobs. The caller
    /// should pass a `LootContext` with `fortune_level` set.
    pub fn roll_block(&self, ctx: &LootContext) -> Vec<(u16, u8)> {
        let mut results: Vec<(u16, u8)> = Vec::new();
        let mut rng_state = ctx.seed;

        for pool in &self.pools {
            for _ in 0..pool.rolls {
                let eligible: Vec<&LootEntry> = pool
                    .entries
                    .iter()
                    .filter(|e| check_conditions(&e.conditions, ctx, &mut rng_state))
                    .collect();

                if eligible.is_empty() {
                    continue;
                }

                let total_weight: u32 = eligible.iter().map(|e| e.weight).sum();
                if total_weight == 0 {
                    continue;
                }

                let pick = (splitmix(&mut rng_state) % total_weight as u64) as u32;
                let mut cumulative = 0u32;
                let mut chosen: Option<&LootEntry> = None;
                for entry in &eligible {
                    cumulative += entry.weight;
                    if pick < cumulative {
                        chosen = Some(entry);
                        break;
                    }
                }

                if let Some(entry) = chosen {
                    // Fortune extends max_count for block drops.
                    let effective_max =
                        entry.max_count.saturating_add(ctx.fortune_level);
                    let count = rand_range_u8(&mut rng_state, entry.min_count, effective_max);
                    if count > 0 {
                        results.push((entry.item_id, count));
                    }
                }
            }
        }

        results
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_ctx(seed: u64) -> LootContext {
        LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: false,
            fortune_level: 0,
            seed,
        }
    }

    // -- Weighted selection ---------------------------------------------------

    #[test]
    fn weighted_selection_respects_weights() {
        let table = LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: 1,
                        min_count: 1,
                        max_count: 1,
                        weight: 100,
                        conditions: vec![LootCondition::Always],
                    },
                    LootEntry {
                        item_id: 2,
                        min_count: 1,
                        max_count: 1,
                        weight: 0,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        };

        // With weight 100 vs 0, item 1 should always be selected.
        for seed in 0..50 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            assert_eq!(drops.len(), 1);
            assert_eq!(drops[0].0, 1);
        }
    }

    #[test]
    fn weighted_selection_covers_both_items() {
        let table = LootTable {
            pools: vec![LootPool {
                entries: vec![
                    LootEntry {
                        item_id: 10,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                    LootEntry {
                        item_id: 20,
                        min_count: 1,
                        max_count: 1,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    },
                ],
                rolls: 1,
            }],
        };

        let mut saw_10 = false;
        let mut saw_20 = false;
        for seed in 0..200 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            assert_eq!(drops.len(), 1);
            match drops[0].0 {
                10 => saw_10 = true,
                20 => saw_20 = true,
                other => panic!("unexpected item {other}"),
            }
        }
        assert!(saw_10, "should have seen item 10");
        assert!(saw_20, "should have seen item 20");
    }

    // -- Looting extends max count -------------------------------------------

    #[test]
    fn looting_extends_max_count() {
        let table = LootTable {
            pools: vec![LootPool {
                entries: vec![LootEntry {
                    item_id: ROTTEN_FLESH,
                    min_count: 1,
                    max_count: 2,
                    weight: 1,
                    conditions: vec![LootCondition::Always],
                }],
                rolls: 1,
            }],
        };

        let mut found_above_base_max = false;
        for seed in 0..500 {
            let ctx = LootContext {
                killed_by_player: true,
                looting_level: 3,
                silk_touch: false,
                fortune_level: 0,
                seed,
            };
            let drops = table.roll(&ctx);
            assert_eq!(drops.len(), 1);
            let count = drops[0].1;
            // With looting 3, effective max = 2 + 3 = 5.
            assert!(count >= 1 && count <= 5, "count {count} out of range [1,5]");
            if count > 2 {
                found_above_base_max = true;
            }
        }
        assert!(
            found_above_base_max,
            "looting should sometimes produce counts above the base max"
        );
    }

    // -- Conditions -----------------------------------------------------------

    #[test]
    fn killed_by_player_condition_filters() {
        let table = LootTable {
            pools: vec![LootPool {
                entries: vec![LootEntry {
                    item_id: SPIDER_EYE,
                    min_count: 1,
                    max_count: 1,
                    weight: 1,
                    conditions: vec![LootCondition::KilledByPlayer],
                }],
                rolls: 1,
            }],
        };

        // Not killed by player — should drop nothing.
        let ctx = LootContext {
            killed_by_player: false,
            looting_level: 0,
            silk_touch: false,
            fortune_level: 0,
            seed: 42,
        };
        let drops = table.roll(&ctx);
        assert!(drops.is_empty());

        // Killed by player — should drop.
        let ctx_player = LootContext {
            killed_by_player: true,
            looting_level: 0,
            silk_touch: false,
            fortune_level: 0,
            seed: 42,
        };
        let drops_player = table.roll(&ctx_player);
        assert_eq!(drops_player.len(), 1);
        assert_eq!(drops_player[0].0, SPIDER_EYE);
    }

    #[test]
    fn has_looting_condition_requires_level() {
        let table = LootTable {
            pools: vec![LootPool {
                entries: vec![LootEntry {
                    item_id: 99,
                    min_count: 1,
                    max_count: 1,
                    weight: 1,
                    conditions: vec![LootCondition::HasLooting(2)],
                }],
                rolls: 1,
            }],
        };

        // Looting 1 — not enough.
        let ctx_low = LootContext {
            killed_by_player: true,
            looting_level: 1,
            silk_touch: false,
            fortune_level: 0,
            seed: 1,
        };
        assert!(table.roll(&ctx_low).is_empty());

        // Looting 2 — passes.
        let ctx_ok = LootContext {
            killed_by_player: true,
            looting_level: 2,
            silk_touch: false,
            fortune_level: 0,
            seed: 1,
        };
        assert_eq!(table.roll(&ctx_ok).len(), 1);
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

    // -- Mob loot tables smoke tests ------------------------------------------

    #[test]
    fn mob_loot_zombie_drops_rotten_flesh_or_nothing() {
        let table = mob_loot_table(MOB_ZOMBIE);
        let mut found_flesh = false;
        for seed in 0..100 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            for (id, count) in &drops {
                assert_eq!(*id, ROTTEN_FLESH);
                assert!(*count <= 2);
                found_flesh = true;
            }
        }
        assert!(found_flesh, "zombie should drop rotten flesh sometimes");
    }

    #[test]
    fn mob_loot_skeleton_can_drop_bones_and_arrows() {
        let table = mob_loot_table(MOB_SKELETON);
        let mut saw_bone = false;
        let mut saw_arrow = false;
        for seed in 0..200 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            for (id, _) in &drops {
                if *id == BONE {
                    saw_bone = true;
                }
                if *id == ARROW {
                    saw_arrow = true;
                }
            }
        }
        assert!(saw_bone, "skeleton should drop bones sometimes");
        assert!(saw_arrow, "skeleton should drop arrows sometimes");
    }

    #[test]
    fn mob_loot_pig_always_drops_porkchop() {
        let table = mob_loot_table(MOB_PIG);
        for seed in 0..50 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            assert!(!drops.is_empty(), "pig should always drop porkchop");
            assert_eq!(drops[0].0, RAW_PORKCHOP);
            assert!(drops[0].1 >= 1 && drops[0].1 <= 3);
        }
    }

    #[test]
    fn mob_loot_cow_always_drops_beef() {
        let table = mob_loot_table(MOB_COW);
        for seed in 0..50 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            assert!(!drops.is_empty());
            assert_eq!(drops[0].0, RAW_BEEF);
        }
    }

    #[test]
    fn mob_loot_sheep_drops_wool() {
        let table = mob_loot_table(MOB_SHEEP);
        let ctx = default_ctx(42);
        let drops = table.roll(&ctx);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].0, WOOL);
        assert_eq!(drops[0].1, 1);
    }

    #[test]
    fn mob_loot_chicken_drops_chicken_and_feathers() {
        let table = mob_loot_table(MOB_CHICKEN);
        let mut saw_chicken = false;
        let mut saw_feather = false;
        for seed in 0..200 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            for (id, _) in &drops {
                if *id == RAW_CHICKEN {
                    saw_chicken = true;
                }
                if *id == FEATHER {
                    saw_feather = true;
                }
            }
        }
        assert!(saw_chicken, "chicken should drop raw chicken");
        assert!(saw_feather, "chicken should drop feathers sometimes");
    }

    #[test]
    fn mob_loot_unknown_kind_returns_empty() {
        let table = mob_loot_table(255);
        let ctx = default_ctx(0);
        let drops = table.roll(&ctx);
        assert!(drops.is_empty());
    }

    #[test]
    fn mob_loot_creeper_rare_music_disc() {
        let table = mob_loot_table(MOB_CREEPER);
        let mut saw_disc = false;
        for seed in 0..2000 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            for (id, _) in &drops {
                if *id == MUSIC_DISC {
                    saw_disc = true;
                }
            }
        }
        assert!(saw_disc, "creeper should rarely drop a music disc");
    }

    #[test]
    fn mob_loot_spider_rare_spider_eye() {
        let table = mob_loot_table(MOB_SPIDER);
        let mut saw_eye = false;
        for seed in 0..500 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            for (id, _) in &drops {
                if *id == SPIDER_EYE {
                    saw_eye = true;
                }
            }
        }
        assert!(saw_eye, "spider should sometimes drop a spider eye");
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

    // -- Deterministic PRNG --------------------------------------------------

    #[test]
    fn same_seed_produces_same_results() {
        let table = mob_loot_table(MOB_ZOMBIE);
        let ctx1 = default_ctx(12345);
        let ctx2 = default_ctx(12345);
        let drops1 = table.roll(&ctx1);
        let drops2 = table.roll(&ctx2);
        assert_eq!(drops1, drops2);
    }

    #[test]
    fn different_seeds_produce_different_results() {
        let table = mob_loot_table(MOB_SKELETON);
        let mut results = std::collections::HashSet::new();
        for seed in 0..100 {
            let ctx = default_ctx(seed);
            let drops = table.roll(&ctx);
            results.insert(format!("{drops:?}"));
        }
        // With 100 different seeds we should see more than 1 distinct outcome.
        assert!(results.len() > 1, "different seeds should vary results");
    }
}
