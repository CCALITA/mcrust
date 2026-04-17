use std::collections::HashMap;

use mc_core::pos::BlockPos;

/// The type of block entity, used for identification and filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockEntityType {
    Chest,
    Furnace,
    Sign,
    BrewingStand,
    EnchantingTable,
    Hopper,
    Dispenser,
    Dropper,
}

/// Inventory data for a chest (27 slots).
/// Each slot is `Option<(item_id, count)>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChestData {
    pub slots: Vec<Option<(u16, u8)>>,
}

impl ChestData {
    pub fn new() -> Self {
        Self {
            slots: vec![None; 27],
        }
    }
}

impl Default for ChestData {
    fn default() -> Self {
        Self::new()
    }
}

/// Smelting data for a furnace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FurnaceData {
    pub input: Option<(u16, u8)>,
    pub fuel: Option<(u16, u8)>,
    pub output: Option<(u16, u8)>,
    pub burn_time: u32,
    pub cook_time: u32,
}

impl FurnaceData {
    pub fn new() -> Self {
        Self {
            input: None,
            fuel: None,
            output: None,
            burn_time: 0,
            cook_time: 0,
        }
    }
}

impl Default for FurnaceData {
    fn default() -> Self {
        Self::new()
    }
}

/// Text data for a sign (4 lines).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignData {
    pub lines: [String; 4],
}

impl SignData {
    pub fn new() -> Self {
        Self {
            lines: [String::new(), String::new(), String::new(), String::new()],
        }
    }
}

impl Default for SignData {
    fn default() -> Self {
        Self::new()
    }
}

/// Brewing stand data: 3 potion slots, 1 ingredient, fuel level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewingStandData {
    pub potions: [Option<u16>; 3],
    pub ingredient: Option<u16>,
    pub fuel: u8,
}

impl BrewingStandData {
    pub fn new() -> Self {
        Self {
            potions: [None; 3],
            ingredient: None,
            fuel: 0,
        }
    }
}

impl Default for BrewingStandData {
    fn default() -> Self {
        Self::new()
    }
}

/// Hopper data: 5 inventory slots and a transfer cooldown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HopperData {
    pub slots: Vec<Option<(u16, u8)>>,
    pub cooldown: u32,
}

impl HopperData {
    pub fn new() -> Self {
        Self {
            slots: vec![None; 5],
            cooldown: 0,
        }
    }
}

impl Default for HopperData {
    fn default() -> Self {
        Self::new()
    }
}

/// A block entity: an interactive block with associated data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockEntity {
    Chest(ChestData),
    Furnace(FurnaceData),
    Sign(SignData),
    BrewingStand(BrewingStandData),
    EnchantingTable,
    Hopper(HopperData),
    Dispenser,
    Dropper,
}

impl BlockEntity {
    /// Returns the type tag for this block entity.
    pub fn entity_type(&self) -> BlockEntityType {
        match self {
            Self::Chest(_) => BlockEntityType::Chest,
            Self::Furnace(_) => BlockEntityType::Furnace,
            Self::Sign(_) => BlockEntityType::Sign,
            Self::BrewingStand(_) => BlockEntityType::BrewingStand,
            Self::EnchantingTable => BlockEntityType::EnchantingTable,
            Self::Hopper(_) => BlockEntityType::Hopper,
            Self::Dispenser => BlockEntityType::Dispenser,
            Self::Dropper => BlockEntityType::Dropper,
        }
    }
}

/// Manages all block entities in the world.
#[derive(Debug, Clone)]
pub struct BlockEntityManager {
    entities: HashMap<BlockPos, BlockEntity>,
}

impl BlockEntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    /// Place a block entity at the given position, replacing any existing one.
    pub fn place(&mut self, pos: BlockPos, entity: BlockEntity) {
        self.entities.insert(pos, entity);
    }

    /// Remove and return the block entity at the given position.
    pub fn remove(&mut self, pos: BlockPos) -> Option<BlockEntity> {
        self.entities.remove(&pos)
    }

    /// Get a reference to the block entity at the given position.
    pub fn get(&self, pos: BlockPos) -> Option<&BlockEntity> {
        self.entities.get(&pos)
    }

    /// Get a mutable reference to the block entity at the given position.
    pub fn get_mut(&mut self, pos: BlockPos) -> Option<&mut BlockEntity> {
        self.entities.get_mut(&pos)
    }

    /// Tick all block entities that have time-based behavior.
    ///
    /// - Furnaces: decrements `burn_time` and `cook_time` when active.
    /// - Hoppers: decrements `cooldown` when non-zero.
    pub fn tick(&mut self) {
        for entity in self.entities.values_mut() {
            match entity {
                BlockEntity::Furnace(data) => {
                    if data.burn_time > 0 {
                        data.burn_time -= 1;
                    }
                    if data.cook_time > 0 {
                        data.cook_time -= 1;
                    }
                }
                BlockEntity::Hopper(data) => {
                    if data.cooldown > 0 {
                        data.cooldown -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// Iterate over all block entities and their positions.
    pub fn iter(&self) -> impl Iterator<Item = (&BlockPos, &BlockEntity)> {
        self.entities.iter()
    }
}

impl Default for BlockEntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_get_remove_lifecycle() {
        let mut manager = BlockEntityManager::new();
        let pos = BlockPos::new(10, 64, 20);
        let chest = BlockEntity::Chest(ChestData::new());

        manager.place(pos, chest.clone());

        let retrieved = manager.get(pos);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().entity_type(), BlockEntityType::Chest);

        let removed = manager.remove(pos);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), chest);

        assert!(manager.get(pos).is_none());
    }

    #[test]
    fn chest_slot_manipulation() {
        let mut manager = BlockEntityManager::new();
        let pos = BlockPos::new(0, 65, 0);
        manager.place(pos, BlockEntity::Chest(ChestData::new()));

        if let Some(BlockEntity::Chest(data)) = manager.get_mut(pos) {
            assert_eq!(data.slots.len(), 27);
            assert!(data.slots.iter().all(|s| s.is_none()));

            // Place an item (diamond, id=264, count=16) in slot 0
            data.slots[0] = Some((264, 16));
            // Place an item (iron, id=265, count=32) in slot 13
            data.slots[13] = Some((265, 32));
        } else {
            panic!("expected chest entity");
        }

        if let Some(BlockEntity::Chest(data)) = manager.get(pos) {
            assert_eq!(data.slots[0], Some((264, 16)));
            assert_eq!(data.slots[13], Some((265, 32)));
            assert!(data.slots[1].is_none());
        } else {
            panic!("expected chest entity");
        }
    }

    #[test]
    fn furnace_tick_decrements() {
        let mut manager = BlockEntityManager::new();
        let pos = BlockPos::new(5, 64, 5);

        let mut furnace = FurnaceData::new();
        furnace.burn_time = 10;
        furnace.cook_time = 5;
        manager.place(pos, BlockEntity::Furnace(furnace));

        manager.tick();

        if let Some(BlockEntity::Furnace(data)) = manager.get(pos) {
            assert_eq!(data.burn_time, 9);
            assert_eq!(data.cook_time, 4);
        } else {
            panic!("expected furnace entity");
        }

        // Tick until cook_time reaches zero
        for _ in 0..4 {
            manager.tick();
        }

        if let Some(BlockEntity::Furnace(data)) = manager.get(pos) {
            assert_eq!(data.burn_time, 5);
            assert_eq!(data.cook_time, 0);
        } else {
            panic!("expected furnace entity");
        }

        // cook_time should stay at zero
        manager.tick();
        if let Some(BlockEntity::Furnace(data)) = manager.get(pos) {
            assert_eq!(data.cook_time, 0);
        } else {
            panic!("expected furnace entity");
        }
    }

    #[test]
    fn hopper_cooldown_decrements() {
        let mut manager = BlockEntityManager::new();
        let pos = BlockPos::new(3, 64, 3);

        let mut hopper = HopperData::new();
        hopper.cooldown = 3;
        manager.place(pos, BlockEntity::Hopper(hopper));

        manager.tick();
        if let Some(BlockEntity::Hopper(data)) = manager.get(pos) {
            assert_eq!(data.cooldown, 2);
            assert_eq!(data.slots.len(), 5);
        } else {
            panic!("expected hopper entity");
        }

        manager.tick();
        manager.tick();
        if let Some(BlockEntity::Hopper(data)) = manager.get(pos) {
            assert_eq!(data.cooldown, 0);
        } else {
            panic!("expected hopper entity");
        }

        // Cooldown stays at zero
        manager.tick();
        if let Some(BlockEntity::Hopper(data)) = manager.get(pos) {
            assert_eq!(data.cooldown, 0);
        } else {
            panic!("expected hopper entity");
        }
    }

    #[test]
    fn iter_returns_all_entities() {
        let mut manager = BlockEntityManager::new();
        manager.place(BlockPos::new(0, 0, 0), BlockEntity::Chest(ChestData::new()));
        manager.place(BlockPos::new(1, 0, 0), BlockEntity::Sign(SignData::new()));
        manager.place(BlockPos::new(2, 0, 0), BlockEntity::EnchantingTable);

        let count = manager.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn place_replaces_existing_entity() {
        let mut manager = BlockEntityManager::new();
        let pos = BlockPos::new(0, 64, 0);

        manager.place(pos, BlockEntity::Chest(ChestData::new()));
        assert_eq!(
            manager.get(pos).unwrap().entity_type(),
            BlockEntityType::Chest
        );

        manager.place(pos, BlockEntity::Furnace(FurnaceData::new()));
        assert_eq!(
            manager.get(pos).unwrap().entity_type(),
            BlockEntityType::Furnace
        );
    }

    #[test]
    fn remove_nonexistent_returns_none() {
        let mut manager = BlockEntityManager::new();
        assert!(manager.remove(BlockPos::new(99, 99, 99)).is_none());
    }
}
