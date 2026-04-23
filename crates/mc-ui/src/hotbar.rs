//! Hotbar HUD data model and pixel-space layout.
//!
//! Provides [`HotbarSlot`] / [`HotbarData`] for the player's 9-slot hotbar state
//! plus [`hotbar_layout`] for converting that state into screen-pixel rectangles
//! consumed by the renderer.

/// Number of slots in the hotbar.
pub const HOTBAR_SLOTS: usize = 9;
/// Pixel width/height of a normal slot.
pub const SLOT_SIZE: f32 = 40.0;
/// Pixel gap between adjacent slots.
pub const SLOT_GAP: f32 = 4.0;
/// Pixel size of the highlight rectangle drawn around the selected slot.
pub const SELECTED_SIZE: f32 = 44.0;
/// Pixel margin from the bottom of the screen.
pub const BOTTOM_MARGIN: f32 = 8.0;

/// A single hotbar item slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotbarSlot {
    /// Item type identifier.
    pub item_id: u16,
    /// Stack count.
    pub count: u8,
    /// Optional durability bar value in 0.0..=1.0 (None for non-tools / non-damageable items).
    pub durability: Option<f32>,
}

/// Snapshot of the 9-slot hotbar plus the currently selected index.
#[derive(Debug, Clone, PartialEq)]
pub struct HotbarData {
    pub slots: [Option<HotbarSlot>; HOTBAR_SLOTS],
    pub selected: usize,
}

impl HotbarData {
    /// Empty hotbar with selection at slot 0.
    pub fn new() -> Self {
        Self {
            slots: [None; HOTBAR_SLOTS],
            selected: 0,
        }
    }

    /// Set the selected slot, clamped to the valid range `0..HOTBAR_SLOTS`.
    pub fn select(&mut self, slot: usize) {
        self.selected = slot.min(HOTBAR_SLOTS - 1);
    }

    /// Place an item in the given slot. No-op if `index` is out of range.
    pub fn set_slot(&mut self, index: usize, item: HotbarSlot) {
        if index < HOTBAR_SLOTS {
            self.slots[index] = Some(item);
        }
    }

    /// Clear the slot at `index`. No-op if out of range.
    pub fn clear_slot(&mut self, index: usize) {
        if index < HOTBAR_SLOTS {
            self.slots[index] = None;
        }
    }

    /// Returns a reference to the currently selected slot's item, if any.
    pub fn selected_item(&self) -> Option<&HotbarSlot> {
        self.slots.get(self.selected).and_then(|s| s.as_ref())
    }

    /// Swap the contents of two slots. No-op if either index is out of range.
    pub fn swap_slots(&mut self, a: usize, b: usize) {
        if a < HOTBAR_SLOTS && b < HOTBAR_SLOTS {
            self.slots.swap(a, b);
        }
    }
}

impl Default for HotbarData {
    fn default() -> Self {
        Self::new()
    }
}

/// Pixel-space rendering layout for one frame of the hotbar.
///
/// All rectangles are `(x, y, width, height)` in screen pixels with `(0, 0)` at the top-left.
#[derive(Debug, Clone, PartialEq)]
pub struct HotbarRenderInfo {
    /// Per-slot rectangles in slot-index order.
    pub slot_rects: [(f32, f32, f32, f32); HOTBAR_SLOTS],
    /// The (slightly larger) highlight rectangle around the selected slot.
    pub selected_rect: (f32, f32, f32, f32),
    /// Items present in the hotbar as `(slot_index, item_id, count, durability)`.
    pub items: Vec<(usize, u16, u8, Option<f32>)>,
}

/// Compute the pixel-space layout for the hotbar centered horizontally at the
/// bottom of a screen of size `screen_width` x `screen_height`.
pub fn hotbar_layout(screen_width: f32, screen_height: f32) -> HotbarRenderInfo {
    layout_with_data(screen_width, screen_height, &HotbarData::new())
}

/// Compute the pixel-space layout populated with `data`'s items and selection.
pub fn layout_with_data(
    screen_width: f32,
    screen_height: f32,
    data: &HotbarData,
) -> HotbarRenderInfo {
    let total_width = SLOT_SIZE * HOTBAR_SLOTS as f32 + SLOT_GAP * (HOTBAR_SLOTS as f32 - 1.0);
    let start_x = (screen_width - total_width) * 0.5;
    let y = screen_height - SLOT_SIZE - BOTTOM_MARGIN;

    let mut slot_rects = [(0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32); HOTBAR_SLOTS];
    for (i, rect) in slot_rects.iter_mut().enumerate() {
        let x = start_x + (SLOT_SIZE + SLOT_GAP) * i as f32;
        *rect = (x, y, SLOT_SIZE, SLOT_SIZE);
    }

    let selected = data.selected.min(HOTBAR_SLOTS - 1);
    let (sx, sy, _, _) = slot_rects[selected];
    // Center the larger highlight on top of the slot.
    let offset = (SELECTED_SIZE - SLOT_SIZE) * 0.5;
    let selected_rect = (sx - offset, sy - offset, SELECTED_SIZE, SELECTED_SIZE);

    let items = data
        .slots
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| slot.map(|s| (i, s.item_id, s.count, s.durability)))
        .collect();

    HotbarRenderInfo {
        slot_rects,
        selected_rect,
        items,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(id: u16) -> HotbarSlot {
        HotbarSlot {
            item_id: id,
            count: 1,
            durability: None,
        }
    }

    #[test]
    fn new_hotbar_is_empty_with_selection_zero() {
        let hb = HotbarData::new();
        assert_eq!(hb.selected, 0);
        assert!(hb.slots.iter().all(|s| s.is_none()));
    }

    #[test]
    fn select_clamps_out_of_range() {
        let mut hb = HotbarData::new();
        hb.select(3);
        assert_eq!(hb.selected, 3);
        hb.select(99);
        assert_eq!(hb.selected, HOTBAR_SLOTS - 1);
    }

    #[test]
    fn set_slot_within_range_stores_item() {
        let mut hb = HotbarData::new();
        let item = HotbarSlot {
            item_id: 42,
            count: 7,
            durability: Some(0.5),
        };
        hb.set_slot(2, item);
        assert_eq!(hb.slots[2], Some(item));
    }

    #[test]
    fn set_slot_out_of_range_is_noop() {
        let mut hb = HotbarData::new();
        hb.set_slot(99, sample_item(1));
        assert!(hb.slots.iter().all(|s| s.is_none()));
    }

    #[test]
    fn clear_slot_removes_item() {
        let mut hb = HotbarData::new();
        hb.set_slot(4, sample_item(99));
        hb.clear_slot(4);
        assert!(hb.slots[4].is_none());
    }

    #[test]
    fn clear_slot_out_of_range_is_noop() {
        let mut hb = HotbarData::new();
        hb.set_slot(0, sample_item(1));
        hb.clear_slot(100);
        assert!(hb.slots[0].is_some());
    }

    #[test]
    fn selected_item_returns_current_selection() {
        let mut hb = HotbarData::new();
        hb.set_slot(5, sample_item(123));
        hb.select(5);
        assert_eq!(hb.selected_item(), Some(&sample_item(123)));
    }

    #[test]
    fn selected_item_none_when_empty() {
        let hb = HotbarData::new();
        assert!(hb.selected_item().is_none());
    }

    #[test]
    fn swap_slots_exchanges_items() {
        let mut hb = HotbarData::new();
        hb.set_slot(0, sample_item(1));
        hb.set_slot(8, sample_item(2));
        hb.swap_slots(0, 8);
        assert_eq!(hb.slots[0], Some(sample_item(2)));
        assert_eq!(hb.slots[8], Some(sample_item(1)));
    }

    #[test]
    fn swap_slots_out_of_range_is_noop() {
        let mut hb = HotbarData::new();
        hb.set_slot(0, sample_item(1));
        hb.swap_slots(0, 100);
        assert_eq!(hb.slots[0], Some(sample_item(1)));
    }

    #[test]
    fn layout_centers_horizontally_at_bottom() {
        let info = hotbar_layout(1920.0, 1080.0);
        let total_width = SLOT_SIZE * HOTBAR_SLOTS as f32 + SLOT_GAP * (HOTBAR_SLOTS as f32 - 1.0);
        let expected_start = (1920.0 - total_width) * 0.5;
        let expected_y = 1080.0 - SLOT_SIZE - BOTTOM_MARGIN;

        assert_eq!(info.slot_rects[0].0, expected_start);
        assert_eq!(info.slot_rects[0].1, expected_y);
        // All slots share the same y and have correct width/height.
        for rect in &info.slot_rects {
            assert_eq!(rect.1, expected_y);
            assert_eq!(rect.2, SLOT_SIZE);
            assert_eq!(rect.3, SLOT_SIZE);
        }
        // Adjacent slots are spaced by SLOT_SIZE + SLOT_GAP.
        for i in 1..HOTBAR_SLOTS {
            let dx = info.slot_rects[i].0 - info.slot_rects[i - 1].0;
            assert!((dx - (SLOT_SIZE + SLOT_GAP)).abs() < 1e-3);
        }
    }

    #[test]
    fn layout_selected_rect_is_44px_centered_on_first_slot() {
        let info = hotbar_layout(800.0, 600.0);
        let (sx, sy, sw, sh) = info.selected_rect;
        assert_eq!(sw, SELECTED_SIZE);
        assert_eq!(sh, SELECTED_SIZE);

        let (slot_x, slot_y, _, _) = info.slot_rects[0];
        let offset = (SELECTED_SIZE - SLOT_SIZE) * 0.5;
        assert!((sx - (slot_x - offset)).abs() < 1e-3);
        assert!((sy - (slot_y - offset)).abs() < 1e-3);
    }

    #[test]
    fn layout_with_data_includes_items_and_tracks_selection() {
        let mut hb = HotbarData::new();
        hb.set_slot(
            0,
            HotbarSlot {
                item_id: 1,
                count: 64,
                durability: None,
            },
        );
        hb.set_slot(
            3,
            HotbarSlot {
                item_id: 256,
                count: 1,
                durability: Some(0.75),
            },
        );
        hb.select(3);

        let info = layout_with_data(1920.0, 1080.0, &hb);
        assert_eq!(info.items.len(), 2);
        assert!(info.items.contains(&(0, 1, 64, None)));
        assert!(info.items.contains(&(3, 256, 1, Some(0.75))));

        // Selected highlight should be centered on slot 3.
        let (slot_x, slot_y, _, _) = info.slot_rects[3];
        let offset = (SELECTED_SIZE - SLOT_SIZE) * 0.5;
        assert!((info.selected_rect.0 - (slot_x - offset)).abs() < 1e-3);
        assert!((info.selected_rect.1 - (slot_y - offset)).abs() < 1e-3);
    }

    #[test]
    fn layout_with_empty_data_produces_no_items() {
        let info = hotbar_layout(1024.0, 768.0);
        assert!(info.items.is_empty());
    }
}
