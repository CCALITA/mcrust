/// Shulker box — a portable storage block that preserves its inventory
/// when broken, unlike a regular chest. Supports 16 dye colors and an
/// open/close lid animation.

/// Total inventory slots in a shulker box (same as a single chest).
pub const SLOT_COUNT: usize = 27;

/// Maximum valid dye-color index (0–15 maps to the 16 Minecraft dye colors).
pub const MAX_COLOR: u8 = 15;

/// Slot content: `Some((item_id, count))` when occupied, `None` when empty.
pub type Slot = Option<(u16, u8)>;

/// A shulker box block entity.
///
/// Key difference from chests: when broken the block drops as an item
/// that retains its full inventory, and when placed that inventory is
/// restored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShulkerBox {
    /// Dye color index (0–15).
    pub color: u8,
    /// 27 inventory slots, matching a single chest.
    pub slots: [Slot; SLOT_COUNT],
}

/// Lid state of a shulker box (open / closed / animating).
#[derive(Debug, Clone, PartialEq)]
pub struct ShulkerOpenState {
    /// Number of players currently viewing this shulker box.
    pub viewers: u8,
    /// Lid animation progress in `[0.0, 1.0]`.
    /// `0.0` = fully closed, `1.0` = fully open.
    pub open_progress: f32,
}

/// An item representation of a shulker box that was broken.
/// Carries the color and the full inventory snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShulkerBoxItem {
    /// Dye color index (0–15).
    pub color: u8,
    /// Snapshot of the inventory at the time the box was broken.
    pub contents: [Slot; SLOT_COUNT],
}

// ---------------------------------------------------------------------------
// ShulkerBox
// ---------------------------------------------------------------------------

impl ShulkerBox {
    /// Create a new, empty shulker box with the given dye color.
    ///
    /// Colors 0–15 correspond to the 16 Minecraft dye colors.
    /// Values above 15 are clamped to 15.
    pub fn new(color: u8) -> Self {
        Self {
            color: color.min(MAX_COLOR),
            slots: [None; SLOT_COUNT],
        }
    }

    /// Returns `true` when every slot is empty.
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    /// Returns the number of occupied slots.
    pub fn occupied_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Break this shulker box and produce a droppable item that
    /// preserves the entire inventory.
    pub fn drop_as_item(&self) -> ShulkerBoxItem {
        ShulkerBoxItem {
            color: self.color,
            contents: self.slots,
        }
    }

    /// Place a shulker box from a previously dropped item, restoring
    /// its color and full inventory.
    pub fn place_from_item(item: &ShulkerBoxItem) -> Self {
        Self {
            color: item.color,
            slots: item.contents,
        }
    }
}

// ---------------------------------------------------------------------------
// ShulkerOpenState
// ---------------------------------------------------------------------------

/// Animation speed per second — the lid takes 0.5 s to fully open or close.
const ANIMATION_SPEED: f32 = 2.0;

impl ShulkerOpenState {
    /// Create a new, closed shulker open state with zero viewers.
    pub fn new() -> Self {
        Self {
            viewers: 0,
            open_progress: 0.0,
        }
    }

    /// Returns `true` when the lid is fully open.
    pub fn is_open(&self) -> bool {
        (self.open_progress - 1.0).abs() < f32::EPSILON
    }

    /// Returns `true` when the lid is fully closed.
    pub fn is_closed(&self) -> bool {
        self.open_progress.abs() < f32::EPSILON
    }
}

impl Default for ShulkerOpenState {
    fn default() -> Self {
        Self::new()
    }
}

/// Record that a player opened the shulker box.
/// Returns a new state with an incremented viewer count.
pub fn open_shulker(state: &ShulkerOpenState) -> ShulkerOpenState {
    ShulkerOpenState {
        viewers: state.viewers.saturating_add(1),
        open_progress: state.open_progress,
    }
}

/// Record that a player closed the shulker box.
/// Returns a new state with a decremented viewer count (floors at 0).
pub fn close_shulker(state: &ShulkerOpenState) -> ShulkerOpenState {
    ShulkerOpenState {
        viewers: state.viewers.saturating_sub(1),
        open_progress: state.open_progress,
    }
}

/// Advance the lid animation toward its target:
/// - `1.0` when `viewers > 0` (opening)
/// - `0.0` when `viewers == 0` (closing)
///
/// Returns a new state with the updated `open_progress`, clamped to `[0.0, 1.0]`.
pub fn tick_shulker_animation(state: &ShulkerOpenState, dt: f32) -> ShulkerOpenState {
    let target = if state.viewers > 0 { 1.0 } else { 0.0 };
    let delta = ANIMATION_SPEED * dt;
    let new_progress = if state.open_progress < target {
        (state.open_progress + delta).min(target)
    } else {
        (state.open_progress - delta).max(target)
    };
    ShulkerOpenState {
        viewers: state.viewers,
        open_progress: new_progress,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- construction --------------------------------------------------------

    #[test]
    fn new_shulker_box_is_empty() {
        let sb = ShulkerBox::new(0);
        assert!(sb.is_empty());
        assert_eq!(sb.occupied_slots(), 0);
    }

    #[test]
    fn new_shulker_box_stores_color() {
        let sb = ShulkerBox::new(5);
        assert_eq!(sb.color, 5);
    }

    #[test]
    fn color_clamped_to_max() {
        let sb = ShulkerBox::new(200);
        assert_eq!(sb.color, MAX_COLOR);
    }

    // -- drop_as_item / place_from_item round-trip --------------------------

    #[test]
    fn drop_preserves_inventory() {
        let mut sb = ShulkerBox::new(3);
        sb.slots[0] = Some((42, 64));
        sb.slots[13] = Some((7, 1));

        let item = sb.drop_as_item();
        assert_eq!(item.color, 3);
        assert_eq!(item.contents[0], Some((42, 64)));
        assert_eq!(item.contents[13], Some((7, 1)));
        // other slots remain empty
        assert_eq!(item.contents[1], None);
    }

    #[test]
    fn place_from_item_restores_contents() {
        let mut sb = ShulkerBox::new(9);
        sb.slots[26] = Some((100, 16));
        let item = sb.drop_as_item();

        let restored = ShulkerBox::place_from_item(&item);
        assert_eq!(restored.color, 9);
        assert_eq!(restored.slots[26], Some((100, 16)));
        assert_eq!(restored.slots[0], None);
    }

    #[test]
    fn round_trip_preserves_full_inventory() {
        let mut sb = ShulkerBox::new(14);
        for i in 0..SLOT_COUNT {
            sb.slots[i] = Some((i as u16, (i + 1) as u8));
        }

        let item = sb.drop_as_item();
        let restored = ShulkerBox::place_from_item(&item);
        assert_eq!(restored, sb);
    }

    // -- empty / occupied helpers -------------------------------------------

    #[test]
    fn is_empty_false_when_occupied() {
        let mut sb = ShulkerBox::new(0);
        sb.slots[5] = Some((1, 1));
        assert!(!sb.is_empty());
    }

    #[test]
    fn occupied_slots_counts_correctly() {
        let mut sb = ShulkerBox::new(0);
        sb.slots[0] = Some((1, 1));
        sb.slots[10] = Some((2, 2));
        sb.slots[26] = Some((3, 3));
        assert_eq!(sb.occupied_slots(), 3);
    }

    // -- open / close state --------------------------------------------------

    #[test]
    fn open_increments_viewers() {
        let state = ShulkerOpenState::new();
        let opened = open_shulker(&state);
        assert_eq!(opened.viewers, 1);
    }

    #[test]
    fn close_decrements_viewers() {
        let state = ShulkerOpenState {
            viewers: 2,
            open_progress: 0.5,
        };
        let closed = close_shulker(&state);
        assert_eq!(closed.viewers, 1);
    }

    #[test]
    fn close_does_not_underflow() {
        let state = ShulkerOpenState::new();
        let closed = close_shulker(&state);
        assert_eq!(closed.viewers, 0);
    }

    #[test]
    fn open_does_not_overflow() {
        let state = ShulkerOpenState {
            viewers: 255,
            open_progress: 1.0,
        };
        let opened = open_shulker(&state);
        assert_eq!(opened.viewers, 255);
    }

    // -- lid animation -------------------------------------------------------

    #[test]
    fn animation_opens_when_viewers_present() {
        let state = ShulkerOpenState {
            viewers: 1,
            open_progress: 0.0,
        };
        let next = tick_shulker_animation(&state, 0.1);
        assert!(
            next.open_progress > 0.0,
            "progress should increase toward 1.0"
        );
    }

    #[test]
    fn animation_closes_when_no_viewers() {
        let state = ShulkerOpenState {
            viewers: 0,
            open_progress: 0.8,
        };
        let next = tick_shulker_animation(&state, 0.1);
        assert!(
            next.open_progress < 0.8,
            "progress should decrease toward 0.0"
        );
    }

    #[test]
    fn is_open_true_at_full_progress() {
        let state = ShulkerOpenState {
            viewers: 1,
            open_progress: 1.0,
        };
        assert!(state.is_open());
    }

    #[test]
    fn is_closed_true_at_zero_progress() {
        let state = ShulkerOpenState::new();
        assert!(state.is_closed());
    }

    #[test]
    fn drop_empty_shulker_yields_empty_item() {
        let sb = ShulkerBox::new(0);
        let item = sb.drop_as_item();
        assert!(item.contents.iter().all(|s| s.is_none()));
    }
}
