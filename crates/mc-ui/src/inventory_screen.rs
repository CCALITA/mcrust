/// Inventory screen layout and interaction logic for survival and crafting UIs.
///
/// The layout uses pixel-like coordinates matching the classic 176x166 inventory
/// texture, normalised so that the renderer can scale to any resolution.

// ---------------------------------------------------------------------------
// Layout constants (pixel-like coordinates for a 176x166 reference texture)
// ---------------------------------------------------------------------------

/// Width of one inventory slot.
const SLOT_W: f32 = 18.0;
/// Height of one inventory slot.
const SLOT_H: f32 = 18.0;

// -- Main inventory grid (9 columns x 4 rows: 3 rows + 1 hotbar row) -------

/// Top-left X of the main 9x3 grid.
const MAIN_GRID_X: f32 = 8.0;
/// Top-left Y of the main 9x3 grid.
const MAIN_GRID_Y: f32 = 84.0;
/// Top-left Y of the hotbar row (row index 3 of the main inventory).
const HOTBAR_Y: f32 = 142.0;

// -- Armor slots (4 vertical, left column) ----------------------------------

/// Top-left X of the armor column.
const ARMOR_X: f32 = 8.0;
/// Top-left Y of the first armor slot.
const ARMOR_Y: f32 = 8.0;

// -- Offhand slot -----------------------------------------------------------

/// Top-left X of the offhand (shield) slot.
const OFFHAND_X: f32 = 77.0;
/// Top-left Y of the offhand slot.
const OFFHAND_Y: f32 = 62.0;

// -- Crafting grid (2x2) and output -----------------------------------------

/// Top-left X of the 2x2 crafting input grid.
const CRAFT_INPUT_X: f32 = 98.0;
/// Top-left Y of the 2x2 crafting input grid.
const CRAFT_INPUT_Y: f32 = 18.0;
/// Top-left X of the crafting output slot.
const CRAFT_OUTPUT_X: f32 = 154.0;
/// Top-left Y of the crafting output slot.
const CRAFT_OUTPUT_Y: f32 = 28.0;

// -- Full crafting table (3x3) and output -----------------------------------

/// Top-left X of the 3x3 crafting input grid (crafting table).
const TABLE_CRAFT_INPUT_X: f32 = 30.0;
/// Top-left Y of the 3x3 crafting input grid.
const TABLE_CRAFT_INPUT_Y: f32 = 17.0;
/// Top-left X of the crafting-table output slot.
const TABLE_CRAFT_OUTPUT_X: f32 = 124.0;
/// Top-left Y of the crafting-table output slot.
const TABLE_CRAFT_OUTPUT_Y: f32 = 35.0;

// ---------------------------------------------------------------------------
// SlotPosition
// ---------------------------------------------------------------------------

/// Screen-space rectangle describing one inventory slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotPosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl SlotPosition {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

// ---------------------------------------------------------------------------
// InventoryLayout
// ---------------------------------------------------------------------------

/// Defines the slot counts for each section of the inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventoryLayout {
    /// Main inventory slots: 9 columns x 4 rows (3 storage rows + 1 hotbar).
    pub main_slots: usize,
    /// Armor slots (helmet, chestplate, leggings, boots).
    pub armor_slots: usize,
    /// Offhand (shield) slot.
    pub offhand_slots: usize,
    /// Crafting input slots (2x2 in survival, 3x3 at a crafting table).
    pub crafting_slots: usize,
    /// Crafting output slot.
    pub output_slots: usize,
}

impl InventoryLayout {
    /// Standard survival inventory layout.
    pub const SURVIVAL: Self = Self {
        main_slots: 36,
        armor_slots: 4,
        offhand_slots: 1,
        crafting_slots: 4,
        output_slots: 1,
    };

    /// Crafting-table layout (3x3 input grid).
    pub const CRAFTING_TABLE: Self = Self {
        main_slots: 36,
        armor_slots: 4,
        offhand_slots: 1,
        crafting_slots: 9,
        output_slots: 1,
    };

    /// Total number of slots across all sections.
    pub const fn total_slots(&self) -> usize {
        self.main_slots + self.armor_slots + self.offhand_slots + self.crafting_slots + self.output_slots
    }
}

// ---------------------------------------------------------------------------
// Layout generators
// ---------------------------------------------------------------------------

/// Generate slot positions for the survival inventory screen.
///
/// Slot ordering (46 total):
///   0..36  — main inventory (rows 0-2 storage, row 3 hotbar)
///  36..40  — armor (helmet → boots)
///  40      — offhand
///  41..45  — crafting 2x2 input (row-major)
///  45      — crafting output
pub fn generate_survival_layout() -> Vec<SlotPosition> {
    let mut slots = Vec::with_capacity(InventoryLayout::SURVIVAL.total_slots());

    // -- Main inventory: 3 storage rows (slots 0..27) -----------------------
    for row in 0..3 {
        for col in 0..9 {
            slots.push(SlotPosition::new(
                MAIN_GRID_X + col as f32 * SLOT_W,
                MAIN_GRID_Y + row as f32 * SLOT_H,
                SLOT_W,
                SLOT_H,
            ));
        }
    }

    // -- Hotbar row (slots 27..36) ------------------------------------------
    for col in 0..9 {
        slots.push(SlotPosition::new(
            MAIN_GRID_X + col as f32 * SLOT_W,
            HOTBAR_Y,
            SLOT_W,
            SLOT_H,
        ));
    }

    // -- Armor (slots 36..40) -----------------------------------------------
    for i in 0..4 {
        slots.push(SlotPosition::new(
            ARMOR_X,
            ARMOR_Y + i as f32 * SLOT_H,
            SLOT_W,
            SLOT_H,
        ));
    }

    // -- Offhand (slot 40) --------------------------------------------------
    slots.push(SlotPosition::new(OFFHAND_X, OFFHAND_Y, SLOT_W, SLOT_H));

    // -- Crafting 2x2 input (slots 41..45) ----------------------------------
    for row in 0..2 {
        for col in 0..2 {
            slots.push(SlotPosition::new(
                CRAFT_INPUT_X + col as f32 * SLOT_W,
                CRAFT_INPUT_Y + row as f32 * SLOT_H,
                SLOT_W,
                SLOT_H,
            ));
        }
    }

    // -- Crafting output (slot 45) ------------------------------------------
    slots.push(SlotPosition::new(
        CRAFT_OUTPUT_X,
        CRAFT_OUTPUT_Y,
        SLOT_W,
        SLOT_H,
    ));

    slots
}

/// Generate slot positions for the crafting-table screen.
///
/// Slot ordering (51 total):
///   0..36  — main inventory (same as survival)
///  36..40  — armor
///  40      — offhand
///  41..50  — crafting 3x3 input (row-major)
///  50      — crafting output
pub fn generate_crafting_layout() -> Vec<SlotPosition> {
    let mut slots = Vec::with_capacity(InventoryLayout::CRAFTING_TABLE.total_slots());

    // -- Main inventory (identical to survival) -----------------------------
    for row in 0..3 {
        for col in 0..9 {
            slots.push(SlotPosition::new(
                MAIN_GRID_X + col as f32 * SLOT_W,
                MAIN_GRID_Y + row as f32 * SLOT_H,
                SLOT_W,
                SLOT_H,
            ));
        }
    }
    for col in 0..9 {
        slots.push(SlotPosition::new(
            MAIN_GRID_X + col as f32 * SLOT_W,
            HOTBAR_Y,
            SLOT_W,
            SLOT_H,
        ));
    }

    // -- Armor (slots 36..40) -----------------------------------------------
    for i in 0..4 {
        slots.push(SlotPosition::new(
            ARMOR_X,
            ARMOR_Y + i as f32 * SLOT_H,
            SLOT_W,
            SLOT_H,
        ));
    }

    // -- Offhand (slot 40) --------------------------------------------------
    slots.push(SlotPosition::new(OFFHAND_X, OFFHAND_Y, SLOT_W, SLOT_H));

    // -- Crafting 3x3 input (slots 41..50) ----------------------------------
    for row in 0..3 {
        for col in 0..3 {
            slots.push(SlotPosition::new(
                TABLE_CRAFT_INPUT_X + col as f32 * SLOT_W,
                TABLE_CRAFT_INPUT_Y + row as f32 * SLOT_H,
                SLOT_W,
                SLOT_H,
            ));
        }
    }

    // -- Crafting output (slot 50) ------------------------------------------
    slots.push(SlotPosition::new(
        TABLE_CRAFT_OUTPUT_X,
        TABLE_CRAFT_OUTPUT_Y,
        SLOT_W,
        SLOT_H,
    ));

    slots
}

// ---------------------------------------------------------------------------
// Mouse button
// ---------------------------------------------------------------------------

/// Mouse button used for a slot click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// DragState
// ---------------------------------------------------------------------------

/// Tracks the item currently held on the cursor during drag operations.
///
/// `held_item` is `(item_id, stack_count)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragState {
    pub held_item: Option<(u16, u8)>,
    pub source_slot: usize,
}

impl DragState {
    pub const fn empty() -> Self {
        Self {
            held_item: None,
            source_slot: 0,
        }
    }

    pub const fn holding(item_id: u16, count: u8, source: usize) -> Self {
        Self {
            held_item: Some((item_id, count)),
            source_slot: source,
        }
    }
}

// ---------------------------------------------------------------------------
// ClickResult
// ---------------------------------------------------------------------------

/// Outcome of clicking an inventory slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickResult {
    /// Nothing happened (e.g. clicking an empty slot with no held item).
    Nothing,
    /// Picked up the full stack from the slot.
    PickUp {
        item_id: u16,
        count: u8,
        from_slot: usize,
    },
    /// Placed the held stack into the empty slot.
    Place {
        item_id: u16,
        count: u8,
        into_slot: usize,
    },
    /// Swapped the held item with the item in the slot.
    Swap {
        placed_id: u16,
        placed_count: u8,
        picked_id: u16,
        picked_count: u8,
        slot: usize,
    },
    /// Split: right-clicked to place a single item from the held stack.
    Split {
        item_id: u16,
        placed_count: u8,
        remaining_count: u8,
        into_slot: usize,
    },
}

// ---------------------------------------------------------------------------
// click_slot
// ---------------------------------------------------------------------------

/// Process a click on `slot_idx` given the current `slot_contents` and `drag` state.
///
/// `slot_contents` is `None` for an empty slot, or `Some((item_id, count))`.
///
/// Returns a `ClickResult` describing what happened, plus an updated `DragState`.
pub fn click_slot(
    slot_idx: usize,
    button: MouseButton,
    slot_contents: Option<(u16, u8)>,
    drag: &DragState,
) -> (ClickResult, DragState) {
    match (drag.held_item, slot_contents, button) {
        // --- Nothing held, click on empty slot → nothing -------------------
        (None, None, _) => (ClickResult::Nothing, DragState::empty()),

        // --- Nothing held, left-click on occupied slot → pick up full stack
        (None, Some((id, count)), MouseButton::Left) => (
            ClickResult::PickUp {
                item_id: id,
                count,
                from_slot: slot_idx,
            },
            DragState::holding(id, count, slot_idx),
        ),

        // --- Nothing held, right-click on occupied slot → pick up half -----
        (None, Some((id, count)), MouseButton::Right) => {
            let picked = (count + 1) / 2; // ceil half
            let remaining = count - picked;
            if remaining == 0 {
                (
                    ClickResult::PickUp {
                        item_id: id,
                        count: picked,
                        from_slot: slot_idx,
                    },
                    DragState::holding(id, picked, slot_idx),
                )
            } else {
                // Pick up half; the other half stays in the slot.
                // Modeled as a PickUp of the taken portion.
                (
                    ClickResult::PickUp {
                        item_id: id,
                        count: picked,
                        from_slot: slot_idx,
                    },
                    DragState::holding(id, picked, slot_idx),
                )
            }
        }

        // --- Holding item, left-click on empty slot → place full stack -----
        (Some((id, count)), None, MouseButton::Left) => (
            ClickResult::Place {
                item_id: id,
                count,
                into_slot: slot_idx,
            },
            DragState::empty(),
        ),

        // --- Holding item, right-click on empty slot → place one -----------
        (Some((id, count)), None, MouseButton::Right) => {
            if count <= 1 {
                (
                    ClickResult::Place {
                        item_id: id,
                        count: 1,
                        into_slot: slot_idx,
                    },
                    DragState::empty(),
                )
            } else {
                (
                    ClickResult::Split {
                        item_id: id,
                        placed_count: 1,
                        remaining_count: count - 1,
                        into_slot: slot_idx,
                    },
                    DragState::holding(id, count - 1, drag.source_slot),
                )
            }
        }

        // --- Holding item, left-click on occupied slot → swap --------------
        (Some((held_id, held_count)), Some((slot_id, slot_count)), MouseButton::Left) => (
            ClickResult::Swap {
                placed_id: held_id,
                placed_count: held_count,
                picked_id: slot_id,
                picked_count: slot_count,
                slot: slot_idx,
            },
            DragState::holding(slot_id, slot_count, slot_idx),
        ),

        // --- Holding item, right-click on occupied slot → place one if same
        (Some((held_id, held_count)), Some((slot_id, _slot_count)), MouseButton::Right) => {
            if held_id == slot_id && held_count > 1 {
                (
                    ClickResult::Split {
                        item_id: held_id,
                        placed_count: 1,
                        remaining_count: held_count - 1,
                        into_slot: slot_idx,
                    },
                    DragState::holding(held_id, held_count - 1, drag.source_slot),
                )
            } else {
                // Different item or only 1 held → swap
                (
                    ClickResult::Swap {
                        placed_id: held_id,
                        placed_count: held_count,
                        picked_id: slot_id,
                        picked_count: _slot_count,
                        slot: slot_idx,
                    },
                    DragState::holding(slot_id, _slot_count, slot_idx),
                )
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

    // -- Layout tests -------------------------------------------------------

    #[test]
    fn survival_layout_has_correct_slot_count() {
        let layout = generate_survival_layout();
        assert_eq!(
            layout.len(),
            InventoryLayout::SURVIVAL.total_slots(),
            "survival layout should have 46 slots"
        );
    }

    #[test]
    fn crafting_layout_has_correct_slot_count() {
        let layout = generate_crafting_layout();
        assert_eq!(
            layout.len(),
            InventoryLayout::CRAFTING_TABLE.total_slots(),
            "crafting layout should have 51 slots"
        );
    }

    #[test]
    fn all_slot_positions_have_positive_dimensions() {
        for pos in generate_survival_layout() {
            assert!(pos.width > 0.0, "slot width must be positive: {pos:?}");
            assert!(pos.height > 0.0, "slot height must be positive: {pos:?}");
        }
        for pos in generate_crafting_layout() {
            assert!(pos.width > 0.0, "slot width must be positive: {pos:?}");
            assert!(pos.height > 0.0, "slot height must be positive: {pos:?}");
        }
    }

    #[test]
    fn survival_and_crafting_share_main_inventory() {
        let survival = generate_survival_layout();
        let crafting = generate_crafting_layout();
        // First 36 slots (main inventory) should be identical.
        assert_eq!(
            &survival[..36],
            &crafting[..36],
            "main inventory slots should match between survival and crafting"
        );
    }

    // -- Click interaction tests --------------------------------------------

    #[test]
    fn click_empty_slot_with_nothing_held_is_noop() {
        let drag = DragState::empty();
        let (result, new_drag) = click_slot(5, MouseButton::Left, None, &drag);
        assert_eq!(result, ClickResult::Nothing);
        assert_eq!(new_drag.held_item, None);
    }

    #[test]
    fn left_click_picks_up_full_stack() {
        let drag = DragState::empty();
        let (result, new_drag) = click_slot(10, MouseButton::Left, Some((42, 16)), &drag);
        assert_eq!(
            result,
            ClickResult::PickUp {
                item_id: 42,
                count: 16,
                from_slot: 10,
            }
        );
        assert_eq!(new_drag.held_item, Some((42, 16)));
        assert_eq!(new_drag.source_slot, 10);
    }

    #[test]
    fn right_click_picks_up_half_stack() {
        let drag = DragState::empty();
        let (result, new_drag) = click_slot(3, MouseButton::Right, Some((7, 10)), &drag);
        assert_eq!(
            result,
            ClickResult::PickUp {
                item_id: 7,
                count: 5,
                from_slot: 3,
            }
        );
        assert_eq!(new_drag.held_item, Some((7, 5)));
    }

    #[test]
    fn left_click_places_held_into_empty_slot() {
        let drag = DragState::holding(99, 8, 0);
        let (result, new_drag) = click_slot(20, MouseButton::Left, None, &drag);
        assert_eq!(
            result,
            ClickResult::Place {
                item_id: 99,
                count: 8,
                into_slot: 20,
            }
        );
        assert_eq!(new_drag.held_item, None);
    }

    #[test]
    fn right_click_splits_one_into_empty_slot() {
        let drag = DragState::holding(5, 4, 0);
        let (result, new_drag) = click_slot(15, MouseButton::Right, None, &drag);
        assert_eq!(
            result,
            ClickResult::Split {
                item_id: 5,
                placed_count: 1,
                remaining_count: 3,
                into_slot: 15,
            }
        );
        assert_eq!(new_drag.held_item, Some((5, 3)));
    }

    #[test]
    fn left_click_swaps_held_with_slot() {
        let drag = DragState::holding(10, 2, 0);
        let (result, new_drag) = click_slot(7, MouseButton::Left, Some((20, 5)), &drag);
        assert_eq!(
            result,
            ClickResult::Swap {
                placed_id: 10,
                placed_count: 2,
                picked_id: 20,
                picked_count: 5,
                slot: 7,
            }
        );
        assert_eq!(new_drag.held_item, Some((20, 5)));
    }

    #[test]
    fn right_click_places_last_held_item() {
        let drag = DragState::holding(5, 1, 0);
        let (result, new_drag) = click_slot(12, MouseButton::Right, None, &drag);
        assert_eq!(
            result,
            ClickResult::Place {
                item_id: 5,
                count: 1,
                into_slot: 12,
            }
        );
        assert_eq!(new_drag.held_item, None);
    }

    #[test]
    fn inventory_layout_total_slots() {
        assert_eq!(InventoryLayout::SURVIVAL.total_slots(), 46);
        assert_eq!(InventoryLayout::CRAFTING_TABLE.total_slots(), 51);
    }
}
