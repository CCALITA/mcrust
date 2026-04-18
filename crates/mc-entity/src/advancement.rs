// ---------------------------------------------------------------------------
// Achievements / Advancements system
// ---------------------------------------------------------------------------

use std::collections::HashSet;

/// Unique identifier for each advancement in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AdvancementId {
    OpenInventory = 0,
    MineWood,
    CraftPlanks,
    MakePickaxe,
    MineCobblestone,
    BuildFurnace,
    SmeltIron,
    GetDiamond,
    MakeEnchantTable,
    EnterNether,
    EnterEnd,
    DefeatDragon,
    DefeatWither,
    GetBeacon,
    BrewPotion,
    FullDiamondArmor,
    EatGoldenApple,
    MakeCake,
    CoverDistance1000,
    Breed,
    CureZombie,
    TradeVillager,
    UseEnchantTable,
    CatchFish,
    MakeMap,
    PlaceBanner,
    GetTrident,
    FindStronghold,
    ActivatePortal,
    KillCreeper,
}

impl AdvancementId {
    /// Total number of advancements.
    pub const COUNT: usize = 30;
}

/// Static properties describing an advancement in the tree.
pub struct AdvancementProperties {
    pub name: &'static str,
    pub description: &'static str,
    pub parent: Option<AdvancementId>,
}

// ---------------------------------------------------------------------------
// Advancement registry (static)
// ---------------------------------------------------------------------------

pub static ADVANCEMENT_REGISTRY: [AdvancementProperties; AdvancementId::COUNT] = [
    // 0 — OpenInventory (root)
    AdvancementProperties {
        name: "Taking Inventory",
        description: "Open your inventory",
        parent: None,
    },
    // 1 — MineWood
    AdvancementProperties {
        name: "Getting Wood",
        description: "Mine a log with your hand",
        parent: Some(AdvancementId::OpenInventory),
    },
    // 2 — CraftPlanks
    AdvancementProperties {
        name: "Benchmarking",
        description: "Craft planks from a log",
        parent: Some(AdvancementId::MineWood),
    },
    // 3 — MakePickaxe
    AdvancementProperties {
        name: "Time to Mine!",
        description: "Craft a wooden pickaxe",
        parent: Some(AdvancementId::CraftPlanks),
    },
    // 4 — MineCobblestone
    AdvancementProperties {
        name: "Stone Age",
        description: "Mine cobblestone with a pickaxe",
        parent: Some(AdvancementId::MakePickaxe),
    },
    // 5 — BuildFurnace
    AdvancementProperties {
        name: "Hot Topic",
        description: "Build a furnace",
        parent: Some(AdvancementId::MineCobblestone),
    },
    // 6 — SmeltIron
    AdvancementProperties {
        name: "Acquire Hardware",
        description: "Smelt an iron ingot",
        parent: Some(AdvancementId::BuildFurnace),
    },
    // 7 — GetDiamond
    AdvancementProperties {
        name: "Diamonds!",
        description: "Acquire a diamond",
        parent: Some(AdvancementId::SmeltIron),
    },
    // 8 — MakeEnchantTable
    AdvancementProperties {
        name: "Enchanter",
        description: "Craft an enchanting table",
        parent: Some(AdvancementId::GetDiamond),
    },
    // 9 — EnterNether
    AdvancementProperties {
        name: "We Need to Go Deeper",
        description: "Enter the Nether",
        parent: Some(AdvancementId::GetDiamond),
    },
    // 10 — EnterEnd
    AdvancementProperties {
        name: "The End?",
        description: "Enter the End",
        parent: Some(AdvancementId::EnterNether),
    },
    // 11 — DefeatDragon
    AdvancementProperties {
        name: "Free the End",
        description: "Defeat the Ender Dragon",
        parent: Some(AdvancementId::EnterEnd),
    },
    // 12 — DefeatWither
    AdvancementProperties {
        name: "The Beginning?",
        description: "Defeat the Wither",
        parent: Some(AdvancementId::EnterNether),
    },
    // 13 — GetBeacon
    AdvancementProperties {
        name: "Beaconator",
        description: "Create and power a full beacon",
        parent: Some(AdvancementId::DefeatWither),
    },
    // 14 — BrewPotion
    AdvancementProperties {
        name: "Local Brewery",
        description: "Brew a potion",
        parent: Some(AdvancementId::EnterNether),
    },
    // 15 — FullDiamondArmor
    AdvancementProperties {
        name: "Cover Me with Diamonds",
        description: "Wear a full set of diamond armor",
        parent: Some(AdvancementId::GetDiamond),
    },
    // 16 — EatGoldenApple
    AdvancementProperties {
        name: "Overpowered",
        description: "Eat an enchanted golden apple",
        parent: Some(AdvancementId::SmeltIron),
    },
    // 17 — MakeCake
    AdvancementProperties {
        name: "The Lie",
        description: "Craft a cake",
        parent: Some(AdvancementId::CraftPlanks),
    },
    // 18 — CoverDistance1000
    AdvancementProperties {
        name: "Adventuring Time",
        description: "Travel 1000 blocks from spawn",
        parent: Some(AdvancementId::OpenInventory),
    },
    // 19 — Breed
    AdvancementProperties {
        name: "The Parrots and the Bats",
        description: "Breed two animals",
        parent: Some(AdvancementId::OpenInventory),
    },
    // 20 — CureZombie
    AdvancementProperties {
        name: "Zombie Doctor",
        description: "Cure a zombie villager",
        parent: Some(AdvancementId::BrewPotion),
    },
    // 21 — TradeVillager
    AdvancementProperties {
        name: "What a Deal!",
        description: "Trade with a villager",
        parent: Some(AdvancementId::OpenInventory),
    },
    // 22 — UseEnchantTable
    AdvancementProperties {
        name: "Enchantment Master",
        description: "Enchant an item at an enchanting table",
        parent: Some(AdvancementId::MakeEnchantTable),
    },
    // 23 — CatchFish
    AdvancementProperties {
        name: "Fishy Business",
        description: "Catch a fish with a fishing rod",
        parent: Some(AdvancementId::OpenInventory),
    },
    // 24 — MakeMap
    AdvancementProperties {
        name: "Map Room",
        description: "Craft a map",
        parent: Some(AdvancementId::CraftPlanks),
    },
    // 25 — PlaceBanner
    AdvancementProperties {
        name: "Volun-tier",
        description: "Place a banner",
        parent: Some(AdvancementId::CraftPlanks),
    },
    // 26 — GetTrident
    AdvancementProperties {
        name: "A Throwaway Joke",
        description: "Obtain a trident",
        parent: Some(AdvancementId::OpenInventory),
    },
    // 27 — FindStronghold
    AdvancementProperties {
        name: "Eye Spy",
        description: "Find a stronghold",
        parent: Some(AdvancementId::EnterNether),
    },
    // 28 — ActivatePortal
    AdvancementProperties {
        name: "Into Fire",
        description: "Activate a Nether portal",
        parent: Some(AdvancementId::SmeltIron),
    },
    // 29 — KillCreeper
    AdvancementProperties {
        name: "Monster Hunter",
        description: "Kill a creeper",
        parent: Some(AdvancementId::OpenInventory),
    },
];

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

/// Events that can trigger advancement progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancementTrigger {
    /// A block was mined (block id as `u16`).
    BlockMined(u16),
    /// An item was crafted (item id as `u16`).
    ItemCrafted(u16),
    /// A mob was killed (mob kind discriminant as `u8`).
    MobKilled(u8),
    /// Player entered a dimension (dimension id as `u8`).
    DimensionEntered(u8),
    /// An item was used/consumed (item id as `u16`).
    ItemUsed(u16),
    /// An item was obtained (item id as `u16`).
    ItemObtained(u16),
    /// Player traveled a distance (cumulative, in blocks).
    DistanceTraveled(u32),
}

// ---------------------------------------------------------------------------
// Tracker
// ---------------------------------------------------------------------------

/// Per-player tracker that records unlocked advancements and pending triggers.
#[derive(Debug, Clone)]
pub struct AdvancementTracker {
    unlocked: HashSet<AdvancementId>,
    pending_triggers: Vec<AdvancementTrigger>,
}

impl AdvancementTracker {
    pub fn new() -> Self {
        Self {
            unlocked: HashSet::new(),
            pending_triggers: Vec::new(),
        }
    }

    /// Queue a trigger event for later processing.
    pub fn push_trigger(&mut self, trigger: AdvancementTrigger) {
        self.pending_triggers.push(trigger);
    }

    /// Process all pending triggers, check conditions, unlock new
    /// advancements, and return the list of newly unlocked IDs.
    ///
    /// An advancement is unlocked only if its parent is already unlocked
    /// (or it has no parent).
    pub fn check_triggers(&mut self) -> Vec<AdvancementId> {
        let triggers: Vec<AdvancementTrigger> = self.pending_triggers.drain(..).collect();
        let mut newly_unlocked = Vec::new();

        for trigger in &triggers {
            let candidates = match_trigger(trigger);
            for id in candidates {
                if self.unlocked.contains(&id) {
                    continue;
                }
                let props = &ADVANCEMENT_REGISTRY[id as u8 as usize];
                let parent_ok = match props.parent {
                    None => true,
                    Some(parent) => self.unlocked.contains(&parent),
                };
                if parent_ok {
                    self.unlocked.insert(id);
                    newly_unlocked.push(id);
                }
            }
        }

        newly_unlocked
    }

    /// Whether a specific advancement has been unlocked.
    pub fn is_unlocked(&self, id: AdvancementId) -> bool {
        self.unlocked.contains(&id)
    }

    /// How many advancements the player has unlocked.
    pub fn unlock_count(&self) -> usize {
        self.unlocked.len()
    }

    /// Total number of advancements in the game.
    pub fn total_count() -> usize {
        AdvancementId::COUNT
    }
}

impl Default for AdvancementTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Trigger -> AdvancementId mapping
// ---------------------------------------------------------------------------

/// Block IDs (from mc-core::BlockId repr values).
const BLOCK_OAK_LOG: u16 = 8;
const BLOCK_COBBLESTONE: u16 = 11;
const BLOCK_DIAMOND_ORE: u16 = 15;

/// Item IDs for crafted / obtained items (from mc-core::ItemId repr values).
const ITEM_OAK_PLANKS: u16 = 10;
const ITEM_WOODEN_PICKAXE: u16 = 100; // placeholder
const ITEM_FURNACE: u16 = 19;
const ITEM_IRON_INGOT: u16 = 84;
const ITEM_DIAMOND: u16 = 87;
const ITEM_ENCHANTING_TABLE: u16 = 101; // placeholder
const ITEM_DIAMOND_ARMOR_SET: u16 = 102; // placeholder
const ITEM_GOLDEN_APPLE: u16 = 103; // placeholder
const ITEM_CAKE: u16 = 104; // placeholder
const ITEM_POTION: u16 = 105; // placeholder
const ITEM_BEACON: u16 = 106; // placeholder
const ITEM_MAP: u16 = 107; // placeholder
const ITEM_BANNER: u16 = 108; // placeholder
const ITEM_TRIDENT: u16 = 109; // placeholder
const ITEM_FISH: u16 = 110; // placeholder

/// Mob kind discriminants (from component::MobKind).
const MOB_CREEPER: u8 = 2;

/// Dimension IDs.
const DIM_NETHER: u8 = 1;
const DIM_END: u8 = 2;

/// Map a single trigger to the advancement(s) it may unlock.
fn match_trigger(trigger: &AdvancementTrigger) -> Vec<AdvancementId> {
    match trigger {
        AdvancementTrigger::BlockMined(block_id) => match *block_id {
            BLOCK_OAK_LOG => vec![AdvancementId::MineWood],
            BLOCK_COBBLESTONE => vec![AdvancementId::MineCobblestone],
            BLOCK_DIAMOND_ORE => vec![AdvancementId::GetDiamond],
            _ => vec![],
        },
        AdvancementTrigger::ItemCrafted(item_id) => match *item_id {
            ITEM_OAK_PLANKS => vec![AdvancementId::CraftPlanks],
            ITEM_WOODEN_PICKAXE => vec![AdvancementId::MakePickaxe],
            ITEM_FURNACE => vec![AdvancementId::BuildFurnace],
            ITEM_ENCHANTING_TABLE => vec![AdvancementId::MakeEnchantTable],
            ITEM_CAKE => vec![AdvancementId::MakeCake],
            ITEM_MAP => vec![AdvancementId::MakeMap],
            _ => vec![],
        },
        AdvancementTrigger::MobKilled(mob_kind) => match *mob_kind {
            MOB_CREEPER => vec![AdvancementId::KillCreeper],
            _ => vec![],
        },
        AdvancementTrigger::DimensionEntered(dim_id) => match *dim_id {
            DIM_NETHER => vec![AdvancementId::EnterNether],
            DIM_END => vec![AdvancementId::EnterEnd],
            _ => vec![],
        },
        AdvancementTrigger::ItemUsed(item_id) => match *item_id {
            ITEM_GOLDEN_APPLE => vec![AdvancementId::EatGoldenApple],
            ITEM_POTION => vec![AdvancementId::BrewPotion],
            ITEM_BANNER => vec![AdvancementId::PlaceBanner],
            _ => vec![],
        },
        AdvancementTrigger::ItemObtained(item_id) => match *item_id {
            ITEM_IRON_INGOT => vec![AdvancementId::SmeltIron],
            ITEM_DIAMOND => vec![AdvancementId::GetDiamond],
            ITEM_DIAMOND_ARMOR_SET => vec![AdvancementId::FullDiamondArmor],
            ITEM_BEACON => vec![AdvancementId::GetBeacon],
            ITEM_TRIDENT => vec![AdvancementId::GetTrident],
            ITEM_FISH => vec![AdvancementId::CatchFish],
            _ => vec![],
        },
        AdvancementTrigger::DistanceTraveled(distance) => {
            if *distance >= 1000 {
                vec![AdvancementId::CoverDistance1000]
            } else {
                vec![]
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_correct_count() {
        assert_eq!(ADVANCEMENT_REGISTRY.len(), AdvancementId::COUNT);
    }

    #[test]
    fn root_advancement_has_no_parent() {
        assert!(
            ADVANCEMENT_REGISTRY[AdvancementId::OpenInventory as usize]
                .parent
                .is_none()
        );
    }

    #[test]
    fn mine_wood_unlocks_from_block_mined() {
        let mut tracker = AdvancementTracker::new();
        // OpenInventory is root (no parent) — unlock it first so MineWood's
        // parent requirement is satisfied.
        tracker.unlocked.insert(AdvancementId::OpenInventory);

        tracker.push_trigger(AdvancementTrigger::BlockMined(BLOCK_OAK_LOG));
        let unlocked = tracker.check_triggers();

        assert!(unlocked.contains(&AdvancementId::MineWood));
        assert!(tracker.is_unlocked(AdvancementId::MineWood));
    }

    #[test]
    fn chain_unlocks_mine_wood_then_craft_planks() {
        let mut tracker = AdvancementTracker::new();
        tracker.unlocked.insert(AdvancementId::OpenInventory);

        // First trigger: mine wood
        tracker.push_trigger(AdvancementTrigger::BlockMined(BLOCK_OAK_LOG));
        let first = tracker.check_triggers();
        assert!(first.contains(&AdvancementId::MineWood));

        // Second trigger: craft planks (parent MineWood now unlocked)
        tracker.push_trigger(AdvancementTrigger::ItemCrafted(ITEM_OAK_PLANKS));
        let second = tracker.check_triggers();
        assert!(second.contains(&AdvancementId::CraftPlanks));
        assert!(tracker.is_unlocked(AdvancementId::CraftPlanks));
    }

    #[test]
    fn cannot_unlock_without_parent() {
        let mut tracker = AdvancementTracker::new();
        // Do NOT unlock OpenInventory — MineWood's parent is missing.
        tracker.push_trigger(AdvancementTrigger::BlockMined(BLOCK_OAK_LOG));
        let unlocked = tracker.check_triggers();

        assert!(unlocked.is_empty());
        assert!(!tracker.is_unlocked(AdvancementId::MineWood));
    }

    #[test]
    fn triggers_clear_after_processing() {
        let mut tracker = AdvancementTracker::new();
        tracker.push_trigger(AdvancementTrigger::BlockMined(BLOCK_OAK_LOG));

        let _ = tracker.check_triggers();
        // Second call should return nothing — triggers were drained.
        let second = tracker.check_triggers();
        assert!(second.is_empty());
    }

    #[test]
    fn duplicate_trigger_does_not_double_unlock() {
        let mut tracker = AdvancementTracker::new();
        tracker.unlocked.insert(AdvancementId::OpenInventory);

        tracker.push_trigger(AdvancementTrigger::BlockMined(BLOCK_OAK_LOG));
        tracker.push_trigger(AdvancementTrigger::BlockMined(BLOCK_OAK_LOG));
        let unlocked = tracker.check_triggers();

        // MineWood should appear only once.
        assert_eq!(
            unlocked
                .iter()
                .filter(|&&id| id == AdvancementId::MineWood)
                .count(),
            1
        );
    }

    #[test]
    fn unlock_count_tracks_correctly() {
        let mut tracker = AdvancementTracker::new();
        assert_eq!(tracker.unlock_count(), 0);

        tracker.unlocked.insert(AdvancementId::OpenInventory);
        assert_eq!(tracker.unlock_count(), 1);

        tracker.push_trigger(AdvancementTrigger::BlockMined(BLOCK_OAK_LOG));
        let _ = tracker.check_triggers();
        assert_eq!(tracker.unlock_count(), 2);
    }

    #[test]
    fn total_count_returns_30() {
        assert_eq!(AdvancementTracker::total_count(), 30);
    }

    #[test]
    fn distance_trigger_unlocks_at_threshold() {
        let mut tracker = AdvancementTracker::new();
        tracker.unlocked.insert(AdvancementId::OpenInventory);

        // Below threshold — nothing happens.
        tracker.push_trigger(AdvancementTrigger::DistanceTraveled(999));
        let unlocked = tracker.check_triggers();
        assert!(unlocked.is_empty());

        // At threshold — unlocks.
        tracker.push_trigger(AdvancementTrigger::DistanceTraveled(1000));
        let unlocked = tracker.check_triggers();
        assert!(unlocked.contains(&AdvancementId::CoverDistance1000));
    }

    #[test]
    fn kill_creeper_unlocks_from_mob_killed() {
        let mut tracker = AdvancementTracker::new();
        tracker.unlocked.insert(AdvancementId::OpenInventory);

        tracker.push_trigger(AdvancementTrigger::MobKilled(MOB_CREEPER));
        let unlocked = tracker.check_triggers();
        assert!(unlocked.contains(&AdvancementId::KillCreeper));
    }

    #[test]
    fn enter_nether_requires_get_diamond_parent() {
        let mut tracker = AdvancementTracker::new();
        // EnterNether's parent is GetDiamond — skip the chain.
        tracker.push_trigger(AdvancementTrigger::DimensionEntered(DIM_NETHER));
        let unlocked = tracker.check_triggers();
        assert!(unlocked.is_empty());

        // Now give the full chain.
        tracker.unlocked.insert(AdvancementId::OpenInventory);
        tracker.unlocked.insert(AdvancementId::MineWood);
        tracker.unlocked.insert(AdvancementId::CraftPlanks);
        tracker.unlocked.insert(AdvancementId::MakePickaxe);
        tracker.unlocked.insert(AdvancementId::MineCobblestone);
        tracker.unlocked.insert(AdvancementId::BuildFurnace);
        tracker.unlocked.insert(AdvancementId::SmeltIron);
        tracker.unlocked.insert(AdvancementId::GetDiamond);

        tracker.push_trigger(AdvancementTrigger::DimensionEntered(DIM_NETHER));
        let unlocked = tracker.check_triggers();
        assert!(unlocked.contains(&AdvancementId::EnterNether));
    }
}
