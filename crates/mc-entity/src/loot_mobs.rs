use crate::loot::{LootCondition, LootEntry, LootPool, LootTable};

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
}
