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
                    let effective_max = entry.max_count.saturating_add(ctx.looting_level);
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
fn check_conditions(conditions: &[LootCondition], ctx: &LootContext, rng_state: &mut u64) -> bool {
    conditions.iter().all(|cond| match cond {
        LootCondition::Always => true,
        LootCondition::KilledByPlayer => ctx.killed_by_player,
        LootCondition::HasLooting(level) => ctx.looting_level >= *level,
        LootCondition::RandomChance(chance) => rand_f32(rng_state) < *chance,
        LootCondition::SilkTouch => ctx.silk_touch,
    })
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

                // When silk touch is active, prefer entries whose conditions
                // include SilkTouch over generic Always entries.
                let final_entries = prefer_silk_touch_entries(&eligible, ctx.silk_touch);

                let total_weight: u32 = final_entries.iter().map(|e| e.weight).sum();
                if total_weight == 0 {
                    continue;
                }

                let pick = (splitmix(&mut rng_state) % total_weight as u64) as u32;
                let mut cumulative = 0u32;
                let mut chosen: Option<&LootEntry> = None;
                for entry in &final_entries {
                    cumulative += entry.weight;
                    if pick < cumulative {
                        chosen = Some(entry);
                        break;
                    }
                }

                if let Some(entry) = chosen {
                    // Fortune extends max_count for block drops.
                    let effective_max = entry.max_count.saturating_add(ctx.fortune_level);
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

/// When silk touch is active, filter to only silk-touch-conditioned entries
/// (if any exist). This prevents `Always` entries from competing with
/// silk-touch-specific drops.
fn prefer_silk_touch_entries<'a>(
    eligible: &[&'a LootEntry],
    silk_touch: bool,
) -> Vec<&'a LootEntry> {
    if !silk_touch {
        return eligible.to_vec();
    }
    let silk: Vec<&LootEntry> = eligible
        .iter()
        .filter(|e| {
            e.conditions
                .iter()
                .any(|c| matches!(c, LootCondition::SilkTouch))
        })
        .copied()
        .collect();
    if silk.is_empty() {
        eligible.to_vec()
    } else {
        silk
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
                    item_id: 2000, // ROTTEN_FLESH
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
                    item_id: 2011, // SPIDER_EYE
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
        assert_eq!(drops_player[0].0, 2011);
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

    // -- Deterministic PRNG --------------------------------------------------

    #[test]
    fn same_seed_produces_same_results() {
        let table = LootTable {
            pools: vec![LootPool {
                entries: vec![LootEntry {
                    item_id: 2000,
                    min_count: 0,
                    max_count: 2,
                    weight: 1,
                    conditions: vec![LootCondition::Always],
                }],
                rolls: 1,
            }],
        };
        let ctx1 = default_ctx(12345);
        let ctx2 = default_ctx(12345);
        let drops1 = table.roll(&ctx1);
        let drops2 = table.roll(&ctx2);
        assert_eq!(drops1, drops2);
    }

    #[test]
    fn different_seeds_produce_different_results() {
        let table = LootTable {
            pools: vec![
                LootPool {
                    entries: vec![LootEntry {
                        item_id: 2001,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
                LootPool {
                    entries: vec![LootEntry {
                        item_id: 2002,
                        min_count: 0,
                        max_count: 2,
                        weight: 1,
                        conditions: vec![LootCondition::Always],
                    }],
                    rolls: 1,
                },
            ],
        };
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
