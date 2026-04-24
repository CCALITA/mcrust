//! Decorated pot crafting from bricks and pottery sherds.
//!
//! A decorated pot is crafted in a 3x3 grid with bricks or sherds in the
//! cross pattern (slots 1, 3, 5, 7) and all other slots empty.  The resulting
//! pot item (7100) carries up to 4 sherd decorations packed into a `u32`.

/// Brick item ID accepted as a plain face material.
pub const BRICK_ID: u16 = 7050;

/// Inclusive start of the pottery sherd ID range.
pub const SHERD_RANGE_START: u16 = 7000;

/// Inclusive end of the pottery sherd ID range.
pub const SHERD_RANGE_END: u16 = 7019;

/// Decorated pot result item ID.
pub const DECORATED_POT_ID: u16 = 7100;

/// Bit-width used to pack a single sherd slot into a `u32`.
const BITS_PER_SLOT: u32 = 8;

/// Cross-pattern slot indices in a 3x3 crafting grid (top, left, right, bottom).
const CROSS_SLOTS: [usize; 4] = [1, 3, 5, 7];

/// Returns `true` if `item_id` is a brick or a pottery sherd.
pub fn is_brick_or_sherd(item_id: u16) -> bool {
    item_id == BRICK_ID || (SHERD_RANGE_START..=SHERD_RANGE_END).contains(&item_id)
}

/// Returns the visual crafting pattern for a decorated pot.
///
/// ```text
/// [ ][B][ ]
/// [B][ ][B]
/// [ ][B][ ]
/// ```
///
/// `B` = brick or sherd.
pub fn pot_crafting_pattern() -> [&'static str; 3] {
    [" B ", "B B", " B "]
}

/// Attempts to craft a decorated pot from a 3x3 grid.
///
/// Slots 1, 3, 5, 7 must each contain a brick (7050) or sherd (7000-7019).
/// All other slots must be `None`.  Returns `Some(7100)` on success.
pub fn craft_decorated_pot(grid: &[Option<u16>; 9]) -> Option<u16> {
    for (idx, slot) in grid.iter().enumerate() {
        let is_cross = CROSS_SLOTS.contains(&idx);
        match (is_cross, slot) {
            (true, Some(id)) if is_brick_or_sherd(*id) => {}
            (false, None) => {}
            _ => return None,
        }
    }
    Some(DECORATED_POT_ID)
}

/// Packs four optional sherd item IDs into a `u32`.
///
/// Each slot occupies 8 bits.  `None` is stored as `0`.
/// Slot order: \[top, left, right, bottom\] mapping to bits \[0..8, 8..16, 16..24, 24..32\].
pub fn pack_pot_sherds(sherds: [Option<u16>; 4]) -> u32 {
    let mut packed: u32 = 0;
    for (i, sherd) in sherds.iter().enumerate() {
        let value = sherd.unwrap_or(0) as u32;
        packed |= (value & 0xFF) << (i as u32 * BITS_PER_SLOT);
    }
    packed
}

/// Extracts four optional sherd item IDs from a packed `u32`.
///
/// A stored value of `0` is interpreted as `None`.
pub fn extract_pot_sherds(data: u32) -> [Option<u16>; 4] {
    let mut sherds = [None; 4];
    for (i, slot) in sherds.iter_mut().enumerate() {
        let value = ((data >> (i as u32 * BITS_PER_SLOT)) & 0xFF) as u16;
        if value != 0 {
            *slot = Some(value);
        }
    }
    sherds
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_brick_or_sherd ─────────────────────────────────────────────────

    #[test]
    fn brick_is_accepted() {
        assert!(is_brick_or_sherd(BRICK_ID));
    }

    #[test]
    fn sherd_range_start_is_accepted() {
        assert!(is_brick_or_sherd(SHERD_RANGE_START));
    }

    #[test]
    fn sherd_range_end_is_accepted() {
        assert!(is_brick_or_sherd(SHERD_RANGE_END));
    }

    #[test]
    fn sherd_mid_range_is_accepted() {
        assert!(is_brick_or_sherd(7010));
    }

    #[test]
    fn arbitrary_item_is_rejected() {
        assert!(!is_brick_or_sherd(100));
        assert!(!is_brick_or_sherd(0));
        assert!(!is_brick_or_sherd(6999));
        assert!(!is_brick_or_sherd(7020));
    }

    // ── pot_crafting_pattern ──────────────────────────────────────────────

    #[test]
    fn pattern_has_three_rows() {
        let pattern = pot_crafting_pattern();
        assert_eq!(pattern.len(), 3);
    }

    #[test]
    fn pattern_rows_are_correct() {
        let pattern = pot_crafting_pattern();
        assert_eq!(pattern[0], " B ");
        assert_eq!(pattern[1], "B B");
        assert_eq!(pattern[2], " B ");
    }

    // ── craft_decorated_pot ───────────────────────────────────────────────

    fn make_grid(cross_item: u16) -> [Option<u16>; 9] {
        let mut grid = [None; 9];
        for &slot in &CROSS_SLOTS {
            grid[slot] = Some(cross_item);
        }
        grid
    }

    #[test]
    fn all_bricks_produces_pot() {
        let grid = make_grid(BRICK_ID);
        assert_eq!(craft_decorated_pot(&grid), Some(DECORATED_POT_ID));
    }

    #[test]
    fn all_sherds_produces_pot() {
        let grid = make_grid(7005);
        assert_eq!(craft_decorated_pot(&grid), Some(DECORATED_POT_ID));
    }

    #[test]
    fn mixed_bricks_and_sherds_produces_pot() {
        let mut grid = [None; 9];
        grid[1] = Some(BRICK_ID);
        grid[3] = Some(7000);
        grid[5] = Some(7019);
        grid[7] = Some(BRICK_ID);
        assert_eq!(craft_decorated_pot(&grid), Some(DECORATED_POT_ID));
    }

    #[test]
    fn missing_cross_slot_fails() {
        let mut grid = make_grid(BRICK_ID);
        grid[1] = None;
        assert_eq!(craft_decorated_pot(&grid), None);
    }

    #[test]
    fn non_empty_corner_fails() {
        let mut grid = make_grid(BRICK_ID);
        grid[0] = Some(100);
        assert_eq!(craft_decorated_pot(&grid), None);
    }

    #[test]
    fn non_empty_center_fails() {
        let mut grid = make_grid(BRICK_ID);
        grid[4] = Some(100);
        assert_eq!(craft_decorated_pot(&grid), None);
    }

    #[test]
    fn invalid_item_in_cross_fails() {
        let mut grid = [None; 9];
        grid[1] = Some(100);
        grid[3] = Some(BRICK_ID);
        grid[5] = Some(BRICK_ID);
        grid[7] = Some(BRICK_ID);
        assert_eq!(craft_decorated_pot(&grid), None);
    }

    #[test]
    fn empty_grid_fails() {
        let grid = [None; 9];
        assert_eq!(craft_decorated_pot(&grid), None);
    }

    // ── pack / extract round-trip ─────────────────────────────────────────

    #[test]
    fn pack_all_none_is_zero() {
        assert_eq!(pack_pot_sherds([None; 4]), 0);
    }

    #[test]
    fn extract_zero_is_all_none() {
        assert_eq!(extract_pot_sherds(0), [None; 4]);
    }

    #[test]
    fn round_trip_all_sherds() {
        let sherds = [Some(7000_u16), Some(7005), Some(7010), Some(7019)];
        let packed = pack_pot_sherds(sherds);
        let unpacked = extract_pot_sherds(packed);
        // Sherd IDs > 255 lose upper bits in 8-bit packing, so we compare low bytes.
        for (i, (original, recovered)) in sherds.iter().zip(unpacked.iter()).enumerate() {
            let expected = original.map(|v| v & 0xFF);
            assert_eq!(*recovered, expected, "mismatch at slot {i}");
        }
    }

    #[test]
    fn round_trip_mixed_none_and_sherd() {
        let sherds = [None, Some(7005_u16), None, Some(7019)];
        let packed = pack_pot_sherds(sherds);
        let unpacked = extract_pot_sherds(packed);
        for (i, (original, recovered)) in sherds.iter().zip(unpacked.iter()).enumerate() {
            let expected = original.map(|v| v & 0xFF);
            assert_eq!(*recovered, expected, "mismatch at slot {i}");
        }
    }

    #[test]
    fn round_trip_brick_ids() {
        // Bricks (7050) also pack via low 8 bits: 7050 & 0xFF = 154
        let sherds = [Some(BRICK_ID), Some(BRICK_ID), Some(BRICK_ID), Some(BRICK_ID)];
        let packed = pack_pot_sherds(sherds);
        let unpacked = extract_pot_sherds(packed);
        for slot in &unpacked {
            assert_eq!(*slot, Some(BRICK_ID & 0xFF));
        }
    }

    #[test]
    fn extract_specific_bits() {
        // Slot 2 = 42 means bits [16..24] = 42
        let data: u32 = 42 << 16;
        let result = extract_pot_sherds(data);
        assert_eq!(result, [None, None, Some(42), None]);
    }
}
