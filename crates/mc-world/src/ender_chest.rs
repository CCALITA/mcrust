/// Ender chest inventory system — per-player shared inventory accessible
/// from any ender chest in any dimension.
///
/// In Minecraft, every player has exactly one 27-slot ender chest inventory.
/// Opening *any* ender chest block in the world shows the same inventory.
/// Items are stored as `(item_id, count)` pairs.

use std::collections::HashMap;

/// Number of slots in an ender chest inventory (3 rows x 9 columns).
pub const ENDER_CHEST_SLOTS: usize = 27;

/// A single player's ender chest inventory.
///
/// Each slot holds an optional `(item_id, count)` pair.
/// `None` means the slot is empty.
#[derive(Debug, Clone, PartialEq)]
pub struct EnderInventory {
    pub slots: [Option<(u16, u8)>; ENDER_CHEST_SLOTS],
}

impl EnderInventory {
    /// Create a new, empty ender inventory (all 27 slots empty).
    pub fn new() -> Self {
        Self {
            slots: [None; ENDER_CHEST_SLOTS],
        }
    }

    /// Set a slot to the given item. Returns a new inventory with the change applied.
    ///
    /// # Panics
    /// Panics if `slot >= ENDER_CHEST_SLOTS`.
    pub fn set_slot(&self, slot: usize, item: Option<(u16, u8)>) -> Self {
        assert!(slot < ENDER_CHEST_SLOTS, "slot index out of bounds");
        let mut new_slots = self.slots;
        new_slots[slot] = item;
        Self { slots: new_slots }
    }

    /// Returns the item in the given slot, or `None` if empty.
    ///
    /// # Panics
    /// Panics if `slot >= ENDER_CHEST_SLOTS`.
    pub fn get_slot(&self, slot: usize) -> Option<(u16, u8)> {
        assert!(slot < ENDER_CHEST_SLOTS, "slot index out of bounds");
        self.slots[slot]
    }

    /// Returns the number of occupied (non-empty) slots.
    pub fn occupied_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Returns `true` if all slots are empty.
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.is_none())
    }
}

impl Default for EnderInventory {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry of all per-player ender chest inventories.
///
/// Keyed by `player_id` (u64). Every player has at most one inventory;
/// a new empty inventory is created on first access.
#[derive(Debug, Clone, Default)]
pub struct EnderChestRegistry {
    inventories: HashMap<u64, EnderInventory>,
}

impl EnderChestRegistry {
    /// Create a new, empty registry.
    pub fn new() -> Self {
        Self {
            inventories: HashMap::new(),
        }
    }

    /// Get or create the ender inventory for the given player.
    ///
    /// If the player has no inventory yet, a new empty one is created and stored.
    /// Returns a reference to the player's inventory.
    pub fn get_or_create(&mut self, player_id: u64) -> &EnderInventory {
        self.inventories
            .entry(player_id)
            .or_insert_with(EnderInventory::new)
    }

    /// Returns the inventory for a player without creating one.
    /// Returns `None` if the player has never opened an ender chest.
    pub fn get(&self, player_id: u64) -> Option<&EnderInventory> {
        self.inventories.get(&player_id)
    }

    /// Update a player's inventory. Returns a new registry with the change applied.
    pub fn update(&self, player_id: u64, inventory: EnderInventory) -> Self {
        let mut new_inventories = self.inventories.clone();
        new_inventories.insert(player_id, inventory);
        Self {
            inventories: new_inventories,
        }
    }

    /// Returns the number of players who have an ender inventory.
    pub fn player_count(&self) -> usize {
        self.inventories.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- EnderInventory -------------------------------------------------------

    #[test]
    fn new_inventory_has_all_empty_slots() {
        let inv = EnderInventory::new();
        assert!(inv.is_empty());
        assert_eq!(inv.occupied_slots(), 0);
        for slot in 0..ENDER_CHEST_SLOTS {
            assert_eq!(inv.get_slot(slot), None);
        }
    }

    #[test]
    fn set_slot_returns_new_inventory_with_item() {
        let inv = EnderInventory::new();
        let updated = inv.set_slot(0, Some((42, 16)));
        // Original unchanged (immutability)
        assert!(inv.is_empty());
        // Updated has the item
        assert_eq!(updated.get_slot(0), Some((42, 16)));
        assert_eq!(updated.occupied_slots(), 1);
    }

    #[test]
    fn set_slot_can_clear_a_slot() {
        let inv = EnderInventory::new().set_slot(5, Some((10, 1)));
        assert_eq!(inv.occupied_slots(), 1);
        let cleared = inv.set_slot(5, None);
        assert!(cleared.is_empty());
    }

    #[test]
    fn occupied_slots_counts_correctly() {
        let inv = EnderInventory::new()
            .set_slot(0, Some((1, 1)))
            .set_slot(13, Some((2, 64)))
            .set_slot(26, Some((3, 32)));
        assert_eq!(inv.occupied_slots(), 3);
        assert!(!inv.is_empty());
    }

    #[test]
    #[should_panic(expected = "slot index out of bounds")]
    fn get_slot_panics_on_out_of_bounds() {
        let inv = EnderInventory::new();
        inv.get_slot(ENDER_CHEST_SLOTS);
    }

    #[test]
    #[should_panic(expected = "slot index out of bounds")]
    fn set_slot_panics_on_out_of_bounds() {
        let inv = EnderInventory::new();
        inv.set_slot(ENDER_CHEST_SLOTS, Some((1, 1)));
    }

    #[test]
    fn default_is_same_as_new() {
        assert_eq!(EnderInventory::default(), EnderInventory::new());
    }

    // -- EnderChestRegistry ---------------------------------------------------

    #[test]
    fn new_registry_has_no_players() {
        let reg = EnderChestRegistry::new();
        assert_eq!(reg.player_count(), 0);
    }

    #[test]
    fn get_or_create_creates_empty_inventory_for_new_player() {
        let mut reg = EnderChestRegistry::new();
        let inv = reg.get_or_create(1);
        assert!(inv.is_empty());
        assert_eq!(reg.player_count(), 1);
    }

    #[test]
    fn get_or_create_returns_same_inventory_on_second_call() {
        let mut reg = EnderChestRegistry::new();
        reg.get_or_create(42);
        // Simulate placing an item via update
        let inv = reg.get(42).unwrap().set_slot(0, Some((100, 5)));
        reg = reg.update(42, inv);
        let inv = reg.get_or_create(42);
        assert_eq!(inv.get_slot(0), Some((100, 5)));
    }

    #[test]
    fn get_returns_none_for_unknown_player() {
        let reg = EnderChestRegistry::new();
        assert!(reg.get(999).is_none());
    }

    #[test]
    fn update_returns_new_registry_with_changed_inventory() {
        let mut reg = EnderChestRegistry::new();
        reg.get_or_create(1);
        let inv = EnderInventory::new().set_slot(10, Some((50, 32)));
        let updated = reg.update(1, inv.clone());
        // Original registry still has empty inventory
        assert!(reg.get(1).unwrap().is_empty());
        // Updated registry has the item
        assert_eq!(updated.get(1).unwrap().get_slot(10), Some((50, 32)));
    }

    #[test]
    fn multiple_players_have_separate_inventories() {
        let mut reg = EnderChestRegistry::new();
        reg.get_or_create(1);
        reg.get_or_create(2);
        let inv1 = EnderInventory::new().set_slot(0, Some((10, 1)));
        let inv2 = EnderInventory::new().set_slot(0, Some((20, 2)));
        let reg = reg.update(1, inv1).update(2, inv2);
        assert_eq!(reg.get(1).unwrap().get_slot(0), Some((10, 1)));
        assert_eq!(reg.get(2).unwrap().get_slot(0), Some((20, 2)));
        assert_eq!(reg.player_count(), 2);
    }

    #[test]
    fn all_ender_chests_access_same_player_inventory() {
        // Simulate: player 1 puts item in ender chest A, opens ender chest B,
        // sees the same item. This is guaranteed by the per-player keying.
        let mut reg = EnderChestRegistry::new();
        reg.get_or_create(1);
        let inv = EnderInventory::new().set_slot(0, Some((42, 10)));
        let reg = reg.update(1, inv);
        // "Open another ender chest" — same get call, same inventory
        let from_chest_b = reg.get(1).unwrap();
        assert_eq!(from_chest_b.get_slot(0), Some((42, 10)));
    }
}
