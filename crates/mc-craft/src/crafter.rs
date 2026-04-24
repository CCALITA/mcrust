//! Crafter block — automated crafting triggered by redstone.
//!
//! The crafter is a 3x3 grid block that can craft items when triggered by
//! a redstone signal. Individual slots can be disabled to support shaped
//! recipes. After crafting, the crafter enters a short cooldown.

// ── Constants ────────────────────────────────────────────────────────────

/// Cooldown after a successful craft (4 game ticks at 20 tps = 0.2 s).
const CRAFTER_COOLDOWN_SECS: f32 = 0.2;

/// Number of slots in the crafter grid.
const GRID_SIZE: usize = 9;

// ── Crafter state ────────────────────────────────────────────────────────

/// State of a crafter block.
#[derive(Debug, Clone, PartialEq)]
pub struct CrafterState {
    /// 3x3 crafting grid (row-major). `None` means the slot is empty.
    pub grid: [Option<u16>; GRID_SIZE],
    /// Per-slot disabled flag. Disabled slots cannot accept items.
    pub disabled_slots: [bool; GRID_SIZE],
    /// Whether the crafter is currently triggered by redstone.
    pub triggered: bool,
    /// Remaining cooldown in seconds before the crafter can craft again.
    pub cooldown: f32,
}

impl Default for CrafterState {
    fn default() -> Self {
        Self::new()
    }
}

impl CrafterState {
    /// Create a new crafter with all slots empty and enabled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            grid: [None; GRID_SIZE],
            disabled_slots: [false; GRID_SIZE],
            triggered: false,
            cooldown: 0.0,
        }
    }

    /// Place an item in the given slot.
    ///
    /// Does nothing if `idx >= 9` or the slot is disabled.
    pub fn set_slot(&mut self, idx: usize, item: u16) {
        if idx < GRID_SIZE && !self.disabled_slots[idx] {
            self.grid[idx] = Some(item);
        }
    }

    /// Remove any item from the given slot.
    ///
    /// Does nothing if `idx >= 9`.
    pub fn clear_slot(&mut self, idx: usize) {
        if idx < GRID_SIZE {
            self.grid[idx] = None;
        }
    }

    /// Toggle the disabled state of a slot.
    ///
    /// A disabled slot cannot accept items via [`set_slot`](Self::set_slot).
    /// Does nothing if `idx >= 9`.
    pub fn toggle_slot_disabled(&mut self, idx: usize) {
        if idx < GRID_SIZE {
            self.disabled_slots[idx] = !self.disabled_slots[idx];
        }
    }
}

// ── Free functions ───────────────────────────────────────────────────────

/// Attempt to craft using the crafter's current grid contents.
///
/// `recipe_lookup` maps a 3x3 grid to an optional output item id.
/// On success the consumed input slots are cleared, the cooldown is set,
/// and the output item id is returned.
pub fn trigger_craft(
    state: &mut CrafterState,
    recipe_lookup: &impl Fn(&[Option<u16>; GRID_SIZE]) -> Option<u16>,
) -> Option<u16> {
    if state.cooldown > 0.0 {
        return None;
    }

    let output = recipe_lookup(&state.grid)?;

    // Clear all non-empty slots that were consumed.
    for slot in &mut state.grid {
        if slot.is_some() {
            *slot = None;
        }
    }

    state.cooldown = CRAFTER_COOLDOWN_SECS;
    Some(output)
}

/// Return the crafter cooldown duration in seconds (4 game ticks = 0.2 s).
#[must_use]
pub const fn crafter_cooldown() -> f32 {
    CRAFTER_COOLDOWN_SECS
}

/// Return the block-offset direction a crafter ejects items, based on its
/// facing value.
///
/// | facing | direction | offset        |
/// |--------|-----------|---------------|
/// | 0      | south     | ( 0, 0,  1)   |
/// | 1      | west      | (-1, 0,  0)   |
/// | 2      | north     | ( 0, 0, -1)   |
/// | 3      | east      | ( 1, 0,  0)   |
/// | 4      | up        | ( 0, 1,  0)   |
/// | 5      | down      | ( 0,-1,  0)   |
///
/// Unknown values default to south.
#[must_use]
pub const fn crafter_eject_direction(facing: u8) -> (i32, i32, i32) {
    match facing {
        0 => (0, 0, 1),   // south
        1 => (-1, 0, 0),  // west
        2 => (0, 0, -1),  // north
        3 => (1, 0, 0),   // east
        4 => (0, 1, 0),   // up
        5 => (0, -1, 0),  // down
        _ => (0, 0, 1),   // default south
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction ─────────────────────────────────────────────────

    #[test]
    fn new_crafter_has_empty_enabled_slots() {
        let crafter = CrafterState::new();
        assert!(crafter.grid.iter().all(|s| s.is_none()));
        assert!(crafter.disabled_slots.iter().all(|&d| !d));
        assert!(!crafter.triggered);
        assert!((crafter.cooldown - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_matches_new() {
        let a = CrafterState::new();
        let b = CrafterState::default();
        assert_eq!(a, b);
    }

    // ── Slot operations ──────────────────────────────────────────────

    #[test]
    fn set_slot_places_item() {
        let mut crafter = CrafterState::new();
        crafter.set_slot(0, 42);
        assert_eq!(crafter.grid[0], Some(42));
    }

    #[test]
    fn set_slot_ignores_out_of_bounds() {
        let mut crafter = CrafterState::new();
        crafter.set_slot(9, 42);
        crafter.set_slot(100, 42);
        assert!(crafter.grid.iter().all(|s| s.is_none()));
    }

    #[test]
    fn set_slot_ignores_disabled_slot() {
        let mut crafter = CrafterState::new();
        crafter.disabled_slots[3] = true;
        crafter.set_slot(3, 42);
        assert_eq!(crafter.grid[3], None);
    }

    #[test]
    fn clear_slot_removes_item() {
        let mut crafter = CrafterState::new();
        crafter.set_slot(4, 10);
        assert_eq!(crafter.grid[4], Some(10));
        crafter.clear_slot(4);
        assert_eq!(crafter.grid[4], None);
    }

    #[test]
    fn clear_slot_ignores_out_of_bounds() {
        let mut crafter = CrafterState::new();
        crafter.clear_slot(9);
        crafter.clear_slot(100);
        // No panic is the assertion.
    }

    #[test]
    fn toggle_disabled_flips_state() {
        let mut crafter = CrafterState::new();
        assert!(!crafter.disabled_slots[2]);
        crafter.toggle_slot_disabled(2);
        assert!(crafter.disabled_slots[2]);
        crafter.toggle_slot_disabled(2);
        assert!(!crafter.disabled_slots[2]);
    }

    #[test]
    fn toggle_disabled_ignores_out_of_bounds() {
        let mut crafter = CrafterState::new();
        crafter.toggle_slot_disabled(9);
        crafter.toggle_slot_disabled(100);
        // No panic is the assertion.
    }

    // ── Crafting ─────────────────────────────────────────────────────

    fn dummy_lookup(grid: &[Option<u16>; 9]) -> Option<u16> {
        // Recognises a single "recipe": slot 0 = 1, slot 1 = 2 → output 99
        if grid[0] == Some(1) && grid[1] == Some(2) {
            Some(99)
        } else {
            None
        }
    }

    #[test]
    fn trigger_craft_succeeds_with_matching_recipe() {
        let mut crafter = CrafterState::new();
        crafter.set_slot(0, 1);
        crafter.set_slot(1, 2);

        let result = trigger_craft(&mut crafter, &dummy_lookup);
        assert_eq!(result, Some(99));

        // Input slots should be cleared.
        assert!(crafter.grid.iter().all(|s| s.is_none()));

        // Cooldown should be set.
        assert!((crafter.cooldown - CRAFTER_COOLDOWN_SECS).abs() < f32::EPSILON);
    }

    #[test]
    fn trigger_craft_returns_none_with_no_recipe() {
        let mut crafter = CrafterState::new();
        crafter.set_slot(0, 50);

        let result = trigger_craft(&mut crafter, &dummy_lookup);
        assert_eq!(result, None);

        // Slot should remain unchanged.
        assert_eq!(crafter.grid[0], Some(50));
        assert!((crafter.cooldown - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn trigger_craft_blocked_during_cooldown() {
        let mut crafter = CrafterState::new();
        crafter.set_slot(0, 1);
        crafter.set_slot(1, 2);
        crafter.cooldown = 0.1;

        let result = trigger_craft(&mut crafter, &dummy_lookup);
        assert_eq!(result, None);

        // Slots should not be consumed.
        assert_eq!(crafter.grid[0], Some(1));
        assert_eq!(crafter.grid[1], Some(2));
    }

    // ── Cooldown constant ────────────────────────────────────────────

    #[test]
    fn crafter_cooldown_is_four_ticks() {
        assert!((crafter_cooldown() - 0.2).abs() < f32::EPSILON);
    }

    // ── Eject direction ──────────────────────────────────────────────

    #[test]
    fn eject_direction_all_facings() {
        assert_eq!(crafter_eject_direction(0), (0, 0, 1));   // south
        assert_eq!(crafter_eject_direction(1), (-1, 0, 0));  // west
        assert_eq!(crafter_eject_direction(2), (0, 0, -1));  // north
        assert_eq!(crafter_eject_direction(3), (1, 0, 0));   // east
        assert_eq!(crafter_eject_direction(4), (0, 1, 0));   // up
        assert_eq!(crafter_eject_direction(5), (0, -1, 0));  // down
    }

    #[test]
    fn eject_direction_unknown_defaults_to_south() {
        assert_eq!(crafter_eject_direction(6), (0, 0, 1));
        assert_eq!(crafter_eject_direction(255), (0, 0, 1));
    }
}
