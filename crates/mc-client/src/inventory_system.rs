use mc_core::block::BlockId;
use mc_craft::{Inventory, ItemStack, SlotItem};
use mc_entity::spawn_block_drops;

const HOTBAR_SIZE: usize = 9;

/// Bridge between the client game loop and the crafting inventory,
/// handling block-break drops, block placement, and hotbar selection.
pub struct PlayerInventory {
    inner: Inventory,
    selected_slot: usize,
}

impl PlayerInventory {
    /// Create a new player inventory with slot 0 selected.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Inventory::new(),
            selected_slot: 0,
        }
    }

    /// Try to add `count` items of `item_id` to the inventory.
    ///
    /// Returns the number of items that could **not** be stored (leftover).
    pub fn add_item(&mut self, item_id: u16, count: u8) -> u8 {
        let stack = ItemStack {
            item: SlotItem(item_id),
            count,
        };
        match self.inner.add_item(stack) {
            Some(remainder) => remainder.count,
            None => 0,
        }
    }

    /// Return `(item_id, count)` for the currently selected hotbar slot,
    /// or `None` if the slot is empty.
    #[must_use]
    pub fn selected_item(&self) -> Option<(u16, u8)> {
        self.inner.get_slot(self.selected_slot).as_ref().map(|s| (s.item.0, s.count))
    }

    /// Consume up to `count` items from the currently selected hotbar slot.
    pub fn consume_selected(&mut self, count: u8) {
        let _ = self.inner.remove_from_slot(self.selected_slot, count);
    }

    /// Select a hotbar slot (0-8). Values >= 9 are clamped to 8.
    pub fn select_slot(&mut self, slot: usize) {
        let clamped = slot.min(HOTBAR_SIZE - 1);
        self.inner.select_slot(clamped);
        self.selected_slot = clamped;
    }

    /// Handle a block being broken: convert the `block_id` to drops via the
    /// entity drop table and add each drop to the inventory.
    pub fn on_block_broken(&mut self, block_id: u16) {
        if let Some(block) = BlockId::from_raw(block_id) {
            let drops = spawn_block_drops(block);
            for (id, count) in drops {
                self.add_item(id, count);
            }
        }
    }

    /// Attempt to place a block: consume 1 item from the selected slot and
    /// return its `item_id` (the block to place). Returns `None` if the
    /// selected slot is empty.
    pub fn on_block_place(&mut self) -> Option<u16> {
        let item_id = self.selected_item()?.0;
        self.consume_selected(1);
        Some(item_id)
    }

    /// Check whether the inventory contains at least one of `item_id`.
    #[must_use]
    pub fn has_item(&self, item_id: u16) -> bool {
        self.item_count(item_id) > 0
    }

    /// Count the total number of `item_id` across all inventory slots.
    #[must_use]
    pub fn item_count(&self, item_id: u16) -> u8 {
        let target = SlotItem(item_id);
        let mut total: u16 = 0;
        for i in 0..36 {
            if let Some(stack) = self.inner.get_slot(i) {
                if stack.item == target {
                    total += stack.count as u16;
                }
            }
        }
        // Saturate to u8::MAX — in practice never exceeded with 36 slots * 64 max.
        total.min(u8::MAX as u16) as u8
    }
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_inventory_has_no_selected_item() {
        let inv = PlayerInventory::new();
        assert!(inv.selected_item().is_none());
    }

    #[test]
    fn add_item_returns_zero_leftover_when_space() {
        let mut inv = PlayerInventory::new();
        let leftover = inv.add_item(1, 10);
        assert_eq!(leftover, 0);
        assert!(inv.has_item(1));
        assert_eq!(inv.item_count(1), 10);
    }

    #[test]
    fn selected_item_returns_correct_slot() {
        let mut inv = PlayerInventory::new();
        inv.add_item(42, 5);
        // Item lands in slot 0, which is the default selected slot.
        assert_eq!(inv.selected_item(), Some((42, 5)));
    }

    #[test]
    fn consume_selected_reduces_count() {
        let mut inv = PlayerInventory::new();
        inv.add_item(42, 5);
        inv.consume_selected(2);
        assert_eq!(inv.selected_item(), Some((42, 3)));
    }

    #[test]
    fn consume_selected_clears_slot_when_exhausted() {
        let mut inv = PlayerInventory::new();
        inv.add_item(42, 3);
        inv.consume_selected(3);
        assert!(inv.selected_item().is_none());
    }

    #[test]
    fn select_slot_changes_held_item() {
        let mut inv = PlayerInventory::new();
        inv.add_item(10, 1);
        inv.select_slot(1);
        // Slot 1 is empty
        assert!(inv.selected_item().is_none());
        inv.select_slot(0);
        assert_eq!(inv.selected_item(), Some((10, 1)));
    }

    #[test]
    fn select_slot_clamps_to_max() {
        let mut inv = PlayerInventory::new();
        inv.select_slot(100);
        // Should clamp to 8
        assert_eq!(inv.selected_slot, 8);
    }

    #[test]
    fn on_block_broken_adds_drops() {
        let mut inv = PlayerInventory::new();
        // Stone drops Cobblestone (BlockId::Cobblestone as u16)
        inv.on_block_broken(BlockId::Stone as u16);
        let cobble_id = BlockId::Cobblestone as u16;
        assert!(inv.has_item(cobble_id));
        assert_eq!(inv.item_count(cobble_id), 1);
    }

    #[test]
    fn on_block_broken_with_invalid_id_does_nothing() {
        let mut inv = PlayerInventory::new();
        inv.on_block_broken(u16::MAX);
        // Nothing added
        assert!(inv.selected_item().is_none());
    }

    #[test]
    fn on_block_place_consumes_and_returns_id() {
        let mut inv = PlayerInventory::new();
        inv.add_item(7, 3);
        let placed = inv.on_block_place();
        assert_eq!(placed, Some(7));
        assert_eq!(inv.item_count(7), 2);
    }

    #[test]
    fn on_block_place_returns_none_when_empty() {
        let mut inv = PlayerInventory::new();
        assert!(inv.on_block_place().is_none());
    }

    #[test]
    fn has_item_returns_false_for_missing() {
        let inv = PlayerInventory::new();
        assert!(!inv.has_item(999));
    }

    #[test]
    fn item_count_sums_across_slots() {
        let mut inv = PlayerInventory::new();
        inv.add_item(5, 64);
        inv.add_item(5, 10);
        assert_eq!(inv.item_count(5), 74);
    }
}
