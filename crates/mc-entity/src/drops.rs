use glam::Vec3;

use crate::component::MobKind;

// ---------------------------------------------------------------------------
// Drop data types
// ---------------------------------------------------------------------------

/// An item drop in the world (e.g. after breaking a block or killing a mob).
#[derive(Debug, Clone, PartialEq)]
pub struct ItemDrop {
    /// Name / identifier of the dropped item.
    pub item: String,
    /// Quantity of items in this stack.
    pub count: u32,
    /// World position where the drop spawned.
    pub position: Vec3,
}

/// An experience orb in the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XpOrb {
    /// Amount of XP this orb grants.
    pub amount: u32,
    /// World position where the orb spawned.
    pub position: Vec3,
}

// ---------------------------------------------------------------------------
// Drop system
// ---------------------------------------------------------------------------

/// Manages pending item drops and XP orbs before they are picked up.
#[derive(Debug, Default)]
pub struct DropSystem {
    pub item_drops: Vec<ItemDrop>,
    pub xp_orbs: Vec<XpOrb>,
}

impl DropSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove and return all pending item drops.
    pub fn drain_items(&mut self) -> Vec<ItemDrop> {
        std::mem::take(&mut self.item_drops)
    }

    /// Remove and return all pending XP orbs.
    pub fn drain_xp(&mut self) -> Vec<XpOrb> {
        std::mem::take(&mut self.xp_orbs)
    }
}

// ---------------------------------------------------------------------------
// Spawning helpers
// ---------------------------------------------------------------------------

/// Spawn drops for a broken block.
///
/// This is a simplified version — in a full implementation each block type
/// would have its own loot table. Here we use sensible defaults.
pub fn spawn_block_drops(system: &mut DropSystem, block_name: &str, position: Vec3) {
    let item = match block_name {
        "Stone" => "Cobblestone",
        "CoalOre" => "Coal",
        "DiamondOre" => "Diamond",
        "IronOre" => "RawIron",
        "GoldOre" => "RawGold",
        "GrassBlock" => "Dirt",
        "OakLeaves" | "BirchLeaves" | "SpruceLeaves" | "JungleLeaves" | "DarkOakLeaves" => {
            // Leaves usually drop nothing; saplings would be a random chance.
            return;
        }
        other => other, // most blocks drop themselves
    };

    system.item_drops.push(ItemDrop {
        item: item.to_string(),
        count: 1,
        position,
    });

    // Ore blocks also drop XP.
    let xp = match block_name {
        "CoalOre" => 1,
        "DiamondOre" => 7,
        "LapisOre" => 5,
        "EmeraldOre" => 7,
        "RedstoneOre" => 3,
        _ => 0,
    };
    if xp > 0 {
        system.xp_orbs.push(XpOrb {
            amount: xp,
            position,
        });
    }
}

/// Spawn drops for a killed mob.
pub fn spawn_mob_drops(system: &mut DropSystem, kind: MobKind, position: Vec3) {
    let (item, count) = match kind {
        MobKind::Zombie => ("RottenFlesh", 1),
        MobKind::Skeleton => ("Bone", 1),
        MobKind::Creeper => ("Gunpowder", 1),
        MobKind::Spider => ("String", 1),
        MobKind::Pig => ("RawPorkchop", 1),
        MobKind::Cow => ("RawBeef", 1),
        MobKind::Sheep => ("Wool", 1),
        MobKind::Chicken => ("RawChicken", 1),
    };

    system.item_drops.push(ItemDrop {
        item: item.to_string(),
        count,
        position,
    });

    // Hostile mobs drop XP.
    let xp = match kind {
        MobKind::Zombie | MobKind::Skeleton | MobKind::Creeper | MobKind::Spider => 5,
        _ => 0,
    };
    if xp > 0 {
        system.xp_orbs.push(XpOrb {
            amount: xp,
            position,
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_drops_stone_yields_cobblestone() {
        let mut sys = DropSystem::new();
        spawn_block_drops(&mut sys, "Stone", Vec3::ZERO);
        assert_eq!(sys.item_drops.len(), 1);
        assert_eq!(sys.item_drops[0].item, "Cobblestone");
    }

    #[test]
    fn block_drops_diamond_ore_yields_xp() {
        let mut sys = DropSystem::new();
        spawn_block_drops(&mut sys, "DiamondOre", Vec3::ZERO);
        assert_eq!(sys.xp_orbs.len(), 1);
        assert_eq!(sys.xp_orbs[0].amount, 7);
    }

    #[test]
    fn leaves_drop_nothing() {
        let mut sys = DropSystem::new();
        spawn_block_drops(&mut sys, "OakLeaves", Vec3::ZERO);
        assert!(sys.item_drops.is_empty());
    }

    #[test]
    fn mob_drops_zombie() {
        let mut sys = DropSystem::new();
        spawn_mob_drops(&mut sys, MobKind::Zombie, Vec3::ZERO);
        assert_eq!(sys.item_drops.len(), 1);
        assert_eq!(sys.item_drops[0].item, "RottenFlesh");
        assert_eq!(sys.xp_orbs.len(), 1);
        assert_eq!(sys.xp_orbs[0].amount, 5);
    }

    #[test]
    fn mob_drops_passive_no_xp() {
        let mut sys = DropSystem::new();
        spawn_mob_drops(&mut sys, MobKind::Pig, Vec3::ZERO);
        assert_eq!(sys.item_drops.len(), 1);
        assert!(sys.xp_orbs.is_empty());
    }

    #[test]
    fn drain_clears_drops() {
        let mut sys = DropSystem::new();
        spawn_block_drops(&mut sys, "CoalOre", Vec3::ZERO);
        spawn_mob_drops(&mut sys, MobKind::Cow, Vec3::ZERO);

        let items = sys.drain_items();
        let xp = sys.drain_xp();

        assert_eq!(items.len(), 2);
        assert_eq!(xp.len(), 1);
        assert!(sys.item_drops.is_empty());
        assert!(sys.xp_orbs.is_empty());
    }
}
