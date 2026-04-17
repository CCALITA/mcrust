use crate::ItemStack;

const MAIN_SLOTS: usize = 36;
const ARMOR_SLOTS: usize = 4;
const HOTBAR_SIZE: usize = 9;
const MAX_STACK_SIZE: u8 = 64;

/// Player inventory with 36 main slots (0-8 hotbar), 4 armor slots, and an offhand slot.
#[derive(Debug, Clone)]
pub struct Inventory {
    slots: [Option<ItemStack>; MAIN_SLOTS],
    armor: [Option<ItemStack>; ARMOR_SLOTS],
    offhand: Option<ItemStack>,
    selected_slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    /// Create an empty inventory with hotbar slot 0 selected.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
            armor: std::array::from_fn(|_| None),
            offhand: None,
            selected_slot: 0,
        }
    }

    /// Get a reference to a main inventory slot.
    ///
    /// # Panics
    /// Panics if `idx >= 36`.
    #[must_use]
    pub fn get_slot(&self, idx: usize) -> &Option<ItemStack> {
        assert!(idx < MAIN_SLOTS, "slot index {idx} out of range (0..{MAIN_SLOTS})");
        &self.slots[idx]
    }

    /// Set a main inventory slot to the given stack (or `None` to clear).
    ///
    /// # Panics
    /// Panics if `idx >= 36`.
    pub fn set_slot(&mut self, idx: usize, stack: Option<ItemStack>) {
        assert!(idx < MAIN_SLOTS, "slot index {idx} out of range (0..{MAIN_SLOTS})");
        self.slots[idx] = stack;
    }

    /// Try to add an `ItemStack` to the inventory.
    ///
    /// Merges into existing stacks of the same item first, then fills empty slots.
    /// Returns `Some(remainder)` if the inventory could not absorb all items,
    /// or `None` if everything was placed.
    pub fn add_item(&mut self, stack: ItemStack) -> Option<ItemStack> {
        let mut remaining = stack.count;
        let item = stack.item;

        // First pass: merge into existing stacks of the same item.
        for slot in &mut self.slots {
            if remaining == 0 {
                return None;
            }
            if let Some(existing) = slot {
                if existing.item == item && existing.count < MAX_STACK_SIZE {
                    let space = MAX_STACK_SIZE - existing.count;
                    let transfer = remaining.min(space);
                    existing.count += transfer;
                    remaining -= transfer;
                }
            }
        }

        // Second pass: place into empty slots.
        for slot in &mut self.slots {
            if remaining == 0 {
                return None;
            }
            if slot.is_none() {
                let transfer = remaining.min(MAX_STACK_SIZE);
                *slot = Some(ItemStack { item, count: transfer });
                remaining -= transfer;
            }
        }

        if remaining > 0 {
            Some(ItemStack { item, count: remaining })
        } else {
            None
        }
    }

    /// Remove `count` items from the given slot.
    ///
    /// Returns the removed `ItemStack`, or `None` if the slot is empty or has
    /// fewer items than requested.
    ///
    /// # Panics
    /// Panics if `idx >= 36`.
    pub fn remove_from_slot(&mut self, idx: usize, count: u8) -> Option<ItemStack> {
        assert!(idx < MAIN_SLOTS, "slot index {idx} out of range (0..{MAIN_SLOTS})");

        let slot = &mut self.slots[idx];
        let existing = slot.as_mut()?;

        if existing.count < count {
            return None;
        }

        let item = existing.item;
        existing.count -= count;

        if existing.count == 0 {
            *slot = None;
        }

        Some(ItemStack { item, count })
    }

    /// Swap the contents of two main inventory slots.
    ///
    /// # Panics
    /// Panics if either index is `>= 36`.
    pub fn swap_slots(&mut self, a: usize, b: usize) {
        assert!(a < MAIN_SLOTS, "slot index {a} out of range (0..{MAIN_SLOTS})");
        assert!(b < MAIN_SLOTS, "slot index {b} out of range (0..{MAIN_SLOTS})");
        self.slots.swap(a, b);
    }

    /// The first 9 slots (hotbar).
    #[must_use]
    pub fn hotbar(&self) -> &[Option<ItemStack>] {
        &self.slots[..HOTBAR_SIZE]
    }

    /// The item in the currently selected hotbar slot.
    #[must_use]
    pub fn held_item(&self) -> &Option<ItemStack> {
        &self.slots[self.selected_slot]
    }

    /// The currently selected hotbar slot index (0-8).
    #[must_use]
    pub fn selected_slot(&self) -> usize {
        self.selected_slot
    }

    /// Select a hotbar slot.
    ///
    /// # Panics
    /// Panics if `slot >= 9`.
    pub fn select_slot(&mut self, slot: usize) {
        assert!(slot < HOTBAR_SIZE, "hotbar slot {slot} out of range (0..{HOTBAR_SIZE})");
        self.selected_slot = slot;
    }

    /// Get a reference to an armor slot.
    ///
    /// # Panics
    /// Panics if `idx >= 4`.
    #[must_use]
    pub fn armor_slot(&self, idx: usize) -> &Option<ItemStack> {
        assert!(idx < ARMOR_SLOTS, "armor index {idx} out of range (0..{ARMOR_SLOTS})");
        &self.armor[idx]
    }

    /// Get a reference to the offhand slot.
    #[must_use]
    pub fn offhand(&self) -> &Option<ItemStack> {
        &self.offhand
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SlotItem;

    fn stone() -> SlotItem {
        SlotItem(1)
    }

    fn dirt() -> SlotItem {
        SlotItem(2)
    }

    #[test]
    fn new_inventory_is_empty() {
        let inv = Inventory::new();
        for i in 0..MAIN_SLOTS {
            assert!(inv.get_slot(i).is_none());
        }
        assert_eq!(inv.selected_slot(), 0);
    }

    #[test]
    fn add_item_fills_empty_slot() {
        let mut inv = Inventory::new();
        let remainder = inv.add_item(ItemStack { item: stone(), count: 10 });
        assert!(remainder.is_none());
        assert_eq!(inv.get_slot(0).as_ref().unwrap().item, stone());
        assert_eq!(inv.get_slot(0).as_ref().unwrap().count, 10);
    }

    #[test]
    fn add_item_merges_into_existing_stack() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item: stone(), count: 50 }));
        let remainder = inv.add_item(ItemStack { item: stone(), count: 10 });
        assert!(remainder.is_none());
        assert_eq!(inv.get_slot(0).as_ref().unwrap().count, 60);
    }

    #[test]
    fn add_item_overflow_goes_to_next_slot() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item: stone(), count: 60 }));
        let remainder = inv.add_item(ItemStack { item: stone(), count: 10 });
        assert!(remainder.is_none());
        // 4 merged into slot 0, 6 into slot 1
        assert_eq!(inv.get_slot(0).as_ref().unwrap().count, 64);
        assert_eq!(inv.get_slot(1).as_ref().unwrap().count, 6);
    }

    #[test]
    fn add_item_full_inventory_returns_remainder() {
        let mut inv = Inventory::new();
        for i in 0..MAIN_SLOTS {
            inv.set_slot(i, Some(ItemStack { item: stone(), count: MAX_STACK_SIZE }));
        }
        let remainder = inv.add_item(ItemStack { item: stone(), count: 5 });
        assert!(remainder.is_some());
        assert_eq!(remainder.unwrap().count, 5);
    }

    #[test]
    fn add_item_skips_different_item_stacks_when_merging() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item: dirt(), count: 30 }));
        let remainder = inv.add_item(ItemStack { item: stone(), count: 10 });
        assert!(remainder.is_none());
        // dirt unchanged in slot 0, stone placed in slot 1
        assert_eq!(inv.get_slot(0).as_ref().unwrap().item, dirt());
        assert_eq!(inv.get_slot(1).as_ref().unwrap().item, stone());
        assert_eq!(inv.get_slot(1).as_ref().unwrap().count, 10);
    }

    #[test]
    fn remove_from_slot_partial() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item: stone(), count: 10 }));
        let removed = inv.remove_from_slot(0, 3);
        assert!(removed.is_some());
        let removed = removed.unwrap();
        assert_eq!(removed.item, stone());
        assert_eq!(removed.count, 3);
        assert_eq!(inv.get_slot(0).as_ref().unwrap().count, 7);
    }

    #[test]
    fn remove_from_slot_entire_stack() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item: stone(), count: 5 }));
        let removed = inv.remove_from_slot(0, 5);
        assert!(removed.is_some());
        assert!(inv.get_slot(0).is_none());
    }

    #[test]
    fn remove_from_empty_slot_returns_none() {
        let mut inv = Inventory::new();
        assert!(inv.remove_from_slot(0, 1).is_none());
    }

    #[test]
    fn remove_more_than_available_returns_none() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item: stone(), count: 3 }));
        assert!(inv.remove_from_slot(0, 5).is_none());
    }

    #[test]
    fn swap_slots_works() {
        let mut inv = Inventory::new();
        inv.set_slot(0, Some(ItemStack { item: stone(), count: 10 }));
        inv.set_slot(1, Some(ItemStack { item: dirt(), count: 5 }));
        inv.swap_slots(0, 1);
        assert_eq!(inv.get_slot(0).as_ref().unwrap().item, dirt());
        assert_eq!(inv.get_slot(1).as_ref().unwrap().item, stone());
    }

    #[test]
    fn hotbar_returns_first_nine_slots() {
        let mut inv = Inventory::new();
        inv.set_slot(8, Some(ItemStack { item: stone(), count: 1 }));
        assert_eq!(inv.hotbar().len(), HOTBAR_SIZE);
        assert!(inv.hotbar()[8].is_some());
    }

    #[test]
    fn held_item_follows_selection() {
        let mut inv = Inventory::new();
        inv.set_slot(3, Some(ItemStack { item: stone(), count: 1 }));
        assert!(inv.held_item().is_none());
        inv.select_slot(3);
        assert!(inv.held_item().is_some());
    }

    #[test]
    #[should_panic(expected = "slot index 36 out of range")]
    fn get_slot_out_of_bounds_panics() {
        let inv = Inventory::new();
        let _ = inv.get_slot(36);
    }

    #[test]
    #[should_panic(expected = "hotbar slot 9 out of range")]
    fn select_slot_out_of_bounds_panics() {
        let mut inv = Inventory::new();
        inv.select_slot(9);
    }
}
