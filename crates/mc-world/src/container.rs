/// Slot content: `Some((item_id, count))` when occupied, `None` when empty.
pub type SlotContent = Option<(u16, u8)>;

/// A generic container trait for block entities with inventory slots.
pub trait Container {
    /// Returns the total number of slots in this container.
    fn slot_count(&self) -> usize;

    /// Returns the content of the slot at `idx`, or `None` if the index is out of bounds.
    fn get_slot(&self, idx: usize) -> Option<SlotContent>;

    /// Sets the content of the slot at `idx`. Returns `false` if the index is out of bounds.
    fn set_slot(&mut self, idx: usize, content: SlotContent) -> bool;

    /// Returns `true` if every slot in the container is empty.
    fn is_empty(&self) -> bool {
        (0..self.slot_count()).all(|i| self.get_slot(i).map_or(true, |slot| slot.is_none()))
    }
}

// ---------------------------------------------------------------------------
// Concrete container types
// ---------------------------------------------------------------------------

/// A single chest with 27 slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChestContainer {
    pub slots: [SlotContent; 27],
}

impl ChestContainer {
    pub fn new() -> Self {
        Self { slots: [None; 27] }
    }
}

impl Default for ChestContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Container for ChestContainer {
    fn slot_count(&self) -> usize {
        27
    }

    fn get_slot(&self, idx: usize) -> Option<SlotContent> {
        self.slots.get(idx).copied()
    }

    fn set_slot(&mut self, idx: usize, content: SlotContent) -> bool {
        if idx < self.slots.len() {
            self.slots[idx] = content;
            true
        } else {
            false
        }
    }
}

/// A double chest with 54 slots (two single chests placed adjacent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleChestContainer {
    pub slots: [SlotContent; 54],
}

impl DoubleChestContainer {
    pub fn new() -> Self {
        Self { slots: [None; 54] }
    }
}

impl Default for DoubleChestContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Container for DoubleChestContainer {
    fn slot_count(&self) -> usize {
        54
    }

    fn get_slot(&self, idx: usize) -> Option<SlotContent> {
        self.slots.get(idx).copied()
    }

    fn set_slot(&mut self, idx: usize, content: SlotContent) -> bool {
        if idx < self.slots.len() {
            self.slots[idx] = content;
            true
        } else {
            false
        }
    }
}

/// A hopper with 5 slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HopperContainer {
    pub slots: [SlotContent; 5],
}

impl HopperContainer {
    pub fn new() -> Self {
        Self { slots: [None; 5] }
    }
}

impl Default for HopperContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Container for HopperContainer {
    fn slot_count(&self) -> usize {
        5
    }

    fn get_slot(&self, idx: usize) -> Option<SlotContent> {
        self.slots.get(idx).copied()
    }

    fn set_slot(&mut self, idx: usize, content: SlotContent) -> bool {
        if idx < self.slots.len() {
            self.slots[idx] = content;
            true
        } else {
            false
        }
    }
}

/// A dispenser (or dropper) with 9 slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispenserContainer {
    pub slots: [SlotContent; 9],
}

impl DispenserContainer {
    pub fn new() -> Self {
        Self { slots: [None; 9] }
    }
}

impl Default for DispenserContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Container for DispenserContainer {
    fn slot_count(&self) -> usize {
        9
    }

    fn get_slot(&self, idx: usize) -> Option<SlotContent> {
        self.slots.get(idx).copied()
    }

    fn set_slot(&mut self, idx: usize, content: SlotContent) -> bool {
        if idx < self.slots.len() {
            self.slots[idx] = content;
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Transfers one item from `from_slot` in `from` into the first available slot
/// in `to` (stacking with matching items up to `max_stack`, or placing in an
/// empty slot).
///
/// Returns `true` if the transfer succeeded, `false` if the source slot was
/// empty or the destination container is full.
pub fn transfer_item(
    from: &mut dyn Container,
    from_slot: usize,
    to: &mut dyn Container,
    max_stack: u8,
) -> bool {
    let src = match from.get_slot(from_slot) {
        Some(Some(item)) => item,
        _ => return false,
    };

    let (item_id, count) = src;
    if count == 0 {
        return false;
    }

    // Try to stack into an existing matching slot first
    for i in 0..to.slot_count() {
        if let Some(Some((existing_id, existing_count))) = to.get_slot(i) {
            if existing_id == item_id && existing_count < max_stack {
                let space = max_stack - existing_count;
                let moved = count.min(space);
                to.set_slot(i, Some((item_id, existing_count + moved)));
                let remaining = count - moved;
                if remaining == 0 {
                    from.set_slot(from_slot, None);
                } else {
                    from.set_slot(from_slot, Some((item_id, remaining)));
                }
                return true;
            }
        }
    }

    // Try to place in an empty slot
    for i in 0..to.slot_count() {
        if let Some(None) = to.get_slot(i) {
            let moved = count.min(max_stack);
            to.set_slot(i, Some((item_id, moved)));
            let remaining = count - moved;
            if remaining == 0 {
                from.set_slot(from_slot, None);
            } else {
                from.set_slot(from_slot, Some((item_id, remaining)));
            }
            return true;
        }
    }

    false
}

/// Finds the best slot index in `container` for adding items with the given
/// `item_id`. Prefers an existing stack of the same item that has room (below
/// `max_stack`), falling back to the first empty slot.
///
/// Returns `None` if the container has no room for this item.
pub fn find_slot_for_item(container: &dyn Container, item_id: u16, max_stack: u8) -> Option<usize> {
    let mut first_empty: Option<usize> = None;

    for i in 0..container.slot_count() {
        match container.get_slot(i) {
            Some(Some((existing_id, existing_count)))
                if existing_id == item_id && existing_count < max_stack =>
            {
                return Some(i);
            }
            Some(None) if first_empty.is_none() => {
                first_empty = Some(i);
            }
            _ => {}
        }
    }

    first_empty
}

/// Adds `count` items of `item_id` into `container`, respecting `max_stack`
/// per slot.
///
/// Returns the number of leftover items that could not be placed (0 means
/// everything was inserted successfully).
pub fn add_to_container(
    container: &mut dyn Container,
    item_id: u16,
    count: u8,
    max_stack: u8,
) -> u8 {
    let mut remaining = count;

    // First pass: fill existing stacks of the same item
    for i in 0..container.slot_count() {
        if remaining == 0 {
            break;
        }
        if let Some(Some((existing_id, existing_count))) = container.get_slot(i) {
            if existing_id == item_id && existing_count < max_stack {
                let space = max_stack - existing_count;
                let added = remaining.min(space);
                container.set_slot(i, Some((item_id, existing_count + added)));
                remaining -= added;
            }
        }
    }

    // Second pass: use empty slots
    for i in 0..container.slot_count() {
        if remaining == 0 {
            break;
        }
        if let Some(None) = container.get_slot(i) {
            let added = remaining.min(max_stack);
            container.set_slot(i, Some((item_id, added)));
            remaining -= added;
        }
    }

    remaining
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ChestContainer CRUD -------------------------------------------------

    #[test]
    fn chest_starts_empty() {
        let chest = ChestContainer::new();
        assert_eq!(chest.slot_count(), 27);
        assert!(chest.is_empty());
        for i in 0..27 {
            assert_eq!(chest.get_slot(i), Some(None));
        }
    }

    #[test]
    fn chest_set_and_get_slot() {
        let mut chest = ChestContainer::new();
        assert!(chest.set_slot(0, Some((264, 16))));
        assert_eq!(chest.get_slot(0), Some(Some((264, 16))));
        assert!(!chest.is_empty());
    }

    #[test]
    fn chest_clear_slot() {
        let mut chest = ChestContainer::new();
        chest.set_slot(5, Some((1, 64)));
        chest.set_slot(5, None);
        assert_eq!(chest.get_slot(5), Some(None));
        assert!(chest.is_empty());
    }

    #[test]
    fn chest_out_of_bounds() {
        let mut chest = ChestContainer::new();
        assert!(!chest.set_slot(27, Some((1, 1))));
        assert_eq!(chest.get_slot(27), None);
    }

    // -- DoubleChestContainer ------------------------------------------------

    #[test]
    fn double_chest_has_54_slots() {
        let dc = DoubleChestContainer::new();
        assert_eq!(dc.slot_count(), 54);
        assert!(dc.is_empty());
    }

    #[test]
    fn double_chest_crud() {
        let mut dc = DoubleChestContainer::new();
        assert!(dc.set_slot(53, Some((100, 32))));
        assert_eq!(dc.get_slot(53), Some(Some((100, 32))));
        assert!(!dc.is_empty());

        dc.set_slot(53, None);
        assert!(dc.is_empty());
    }

    #[test]
    fn double_chest_out_of_bounds() {
        let mut dc = DoubleChestContainer::new();
        assert!(!dc.set_slot(54, Some((1, 1))));
        assert_eq!(dc.get_slot(54), None);
    }

    // -- HopperContainer -----------------------------------------------------

    #[test]
    fn hopper_has_5_slots() {
        let hopper = HopperContainer::new();
        assert_eq!(hopper.slot_count(), 5);
        assert!(hopper.is_empty());
    }

    #[test]
    fn hopper_crud() {
        let mut hopper = HopperContainer::new();
        hopper.set_slot(0, Some((10, 3)));
        assert_eq!(hopper.get_slot(0), Some(Some((10, 3))));
        assert!(!hopper.is_empty());
    }

    // -- DispenserContainer --------------------------------------------------

    #[test]
    fn dispenser_has_9_slots() {
        let disp = DispenserContainer::new();
        assert_eq!(disp.slot_count(), 9);
        assert!(disp.is_empty());
    }

    #[test]
    fn dispenser_crud() {
        let mut disp = DispenserContainer::new();
        disp.set_slot(8, Some((262, 64)));
        assert_eq!(disp.get_slot(8), Some(Some((262, 64))));
        assert!(!disp.is_empty());
    }

    // -- transfer_item -------------------------------------------------------

    #[test]
    fn transfer_moves_item_to_empty_slot() {
        let mut from = ChestContainer::new();
        let mut to = ChestContainer::new();
        from.set_slot(0, Some((264, 10)));

        assert!(transfer_item(&mut from, 0, &mut to, 64));
        assert_eq!(from.get_slot(0), Some(None));
        assert_eq!(to.get_slot(0), Some(Some((264, 10))));
    }

    #[test]
    fn transfer_stacks_with_matching_item() {
        let mut from = ChestContainer::new();
        let mut to = ChestContainer::new();
        from.set_slot(0, Some((264, 10)));
        to.set_slot(0, Some((264, 50)));

        assert!(transfer_item(&mut from, 0, &mut to, 64));
        // 50 + 10 = 60, fits within max_stack 64
        assert_eq!(to.get_slot(0), Some(Some((264, 60))));
        assert_eq!(from.get_slot(0), Some(None));
    }

    #[test]
    fn transfer_partial_when_stack_nearly_full() {
        let mut from = ChestContainer::new();
        let mut to = HopperContainer::new();
        from.set_slot(0, Some((264, 20)));
        // Fill all hopper slots with the same item, leaving 2 spaces in the first
        for i in 0..5 {
            to.set_slot(i, Some((264, 62)));
        }

        assert!(transfer_item(&mut from, 0, &mut to, 64));
        // Only 2 can fit in the first slot
        assert_eq!(to.get_slot(0), Some(Some((264, 64))));
        assert_eq!(from.get_slot(0), Some(Some((264, 18))));
    }

    #[test]
    fn transfer_fails_when_source_empty() {
        let mut from = ChestContainer::new();
        let mut to = ChestContainer::new();
        assert!(!transfer_item(&mut from, 0, &mut to, 64));
    }

    #[test]
    fn transfer_fails_when_dest_full() {
        let mut from = ChestContainer::new();
        let mut to = HopperContainer::new();
        from.set_slot(0, Some((264, 10)));
        // Fill hopper completely with a different item
        for i in 0..5 {
            to.set_slot(i, Some((265, 64)));
        }
        assert!(!transfer_item(&mut from, 0, &mut to, 64));
        // Source should be unchanged
        assert_eq!(from.get_slot(0), Some(Some((264, 10))));
    }

    // -- find_slot_for_item --------------------------------------------------

    #[test]
    fn find_slot_prefers_existing_stack() {
        let mut chest = ChestContainer::new();
        chest.set_slot(5, Some((264, 32)));

        let idx = find_slot_for_item(&chest, 264, 64);
        assert_eq!(idx, Some(5));
    }

    #[test]
    fn find_slot_falls_back_to_empty() {
        let mut chest = ChestContainer::new();
        // Slot 0 has a full stack
        chest.set_slot(0, Some((264, 64)));

        let idx = find_slot_for_item(&chest, 264, 64);
        // Slot 0 is full, so it should find the first empty slot (1)
        assert_eq!(idx, Some(1));
    }

    #[test]
    fn find_slot_returns_none_when_full() {
        let mut hopper = HopperContainer::new();
        // Fill with full stacks of different item
        for i in 0..5 {
            hopper.set_slot(i, Some((265, 64)));
        }
        assert_eq!(find_slot_for_item(&hopper, 264, 64), None);
    }

    #[test]
    fn find_slot_skips_full_matching_stacks() {
        let mut chest = ChestContainer::new();
        chest.set_slot(0, Some((264, 64))); // full
        chest.set_slot(1, Some((264, 32))); // has room

        assert_eq!(find_slot_for_item(&chest, 264, 64), Some(1));
    }

    // -- add_to_container ----------------------------------------------------

    #[test]
    fn add_to_empty_container() {
        let mut chest = ChestContainer::new();
        let leftover = add_to_container(&mut chest, 264, 32, 64);
        assert_eq!(leftover, 0);
        assert_eq!(chest.get_slot(0), Some(Some((264, 32))));
    }

    #[test]
    fn add_stacks_with_existing_items() {
        let mut chest = ChestContainer::new();
        chest.set_slot(0, Some((264, 50)));

        let leftover = add_to_container(&mut chest, 264, 10, 64);
        assert_eq!(leftover, 0);
        assert_eq!(chest.get_slot(0), Some(Some((264, 60))));
    }

    #[test]
    fn add_overflows_to_new_slot() {
        let mut chest = ChestContainer::new();
        chest.set_slot(0, Some((264, 60)));

        let leftover = add_to_container(&mut chest, 264, 10, 64);
        assert_eq!(leftover, 0);
        // 4 fit in slot 0, 6 overflow to slot 1
        assert_eq!(chest.get_slot(0), Some(Some((264, 64))));
        assert_eq!(chest.get_slot(1), Some(Some((264, 6))));
    }

    #[test]
    fn add_returns_leftover_when_full() {
        let mut hopper = HopperContainer::new();
        // Fill all 5 slots to max
        for i in 0..5 {
            hopper.set_slot(i, Some((264, 64)));
        }

        let leftover = add_to_container(&mut hopper, 264, 10, 64);
        assert_eq!(leftover, 10);
    }

    #[test]
    fn add_partial_leftover() {
        let mut hopper = HopperContainer::new();
        // Fill 4 slots completely, leave 1 slot with room for 3
        for i in 0..4 {
            hopper.set_slot(i, Some((264, 64)));
        }
        hopper.set_slot(4, Some((264, 61)));

        let leftover = add_to_container(&mut hopper, 264, 10, 64);
        // Only 3 fit
        assert_eq!(leftover, 7);
        assert_eq!(hopper.get_slot(4), Some(Some((264, 64))));
    }

    #[test]
    fn add_zero_count_is_noop() {
        let mut chest = ChestContainer::new();
        let leftover = add_to_container(&mut chest, 264, 0, 64);
        assert_eq!(leftover, 0);
        assert!(chest.is_empty());
    }

    // -- Cross-container transfer scenarios ----------------------------------

    #[test]
    fn transfer_between_different_container_types() {
        let mut chest = ChestContainer::new();
        let mut hopper = HopperContainer::new();
        chest.set_slot(0, Some((264, 16)));

        assert!(transfer_item(&mut chest, 0, &mut hopper, 64));
        assert_eq!(chest.get_slot(0), Some(None));
        assert_eq!(hopper.get_slot(0), Some(Some((264, 16))));
    }

    #[test]
    fn double_chest_fill_and_find() {
        let mut dc = DoubleChestContainer::new();
        // Fill first 53 slots
        for i in 0..53 {
            dc.set_slot(i, Some((1, 64)));
        }
        // Last slot should still be empty
        assert_eq!(find_slot_for_item(&dc, 264, 64), Some(53));

        dc.set_slot(53, Some((264, 32)));
        // Now searching for 264 should prefer the partially filled slot 53
        assert_eq!(find_slot_for_item(&dc, 264, 64), Some(53));

        // Fill it up
        dc.set_slot(53, Some((264, 64)));
        // Container of different items is full — no room for 264
        assert_eq!(find_slot_for_item(&dc, 264, 64), None);
    }
}
