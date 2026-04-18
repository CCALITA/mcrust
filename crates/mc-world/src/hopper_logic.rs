/// Direction a hopper output faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopperDirection {
    Down,
    North,
    South,
    East,
    West,
}

/// Describes a single item transfer between two block positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HopperTransfer {
    pub from_pos: (i32, i32, i32),
    pub to_pos: (i32, i32, i32),
    pub item_id: u16,
    pub count: u8,
}

/// Number of game ticks a hopper must wait between consecutive transfers.
pub const HOPPER_COOLDOWN: u32 = 8;

/// Returns `true` if the hopper is eligible to transfer an item this tick.
///
/// A hopper may transfer when its cooldown has expired (reached zero) and it is
/// not locked by a redstone signal.
pub fn hopper_should_transfer(cooldown: u32, locked: bool) -> bool {
    cooldown == 0 && !locked
}

/// Attempts to pull one item from the container above into the hopper.
///
/// Scans `items_above` (the slot list of the container sitting on top) and
/// returns the index, item id, and count of the first non-empty slot. Returns
/// `None` if every slot is empty.
pub fn hopper_pull(items_above: &[(u16, u8)]) -> Option<(usize, u16, u8)> {
    items_above
        .iter()
        .enumerate()
        .find(|(_, (_, count))| *count > 0)
        .map(|(idx, (id, count))| (idx, *id, *count))
}

/// Attempts to push one item from the hopper into the container below/beside.
///
/// Iterates the hopper's own slots (`items_self`) looking for a non-empty slot,
/// then checks `dest_slots` for a compatible destination:
///   1. A slot containing the same `item_id` with room below `max_stack`.
///   2. An empty slot (`None`).
///
/// Returns `(source_index, item_id, count)` of the hopper slot that should be
/// decremented, or `None` if no transfer is possible.
pub fn hopper_push(
    items_self: &[(u16, u8)],
    dest_slots: &[Option<(u16, u8)>],
    max_stack: u8,
) -> Option<(usize, u16, u8)> {
    for (src_idx, &(item_id, count)) in items_self.iter().enumerate() {
        if count == 0 {
            continue;
        }

        // Check for a matching stack with room, then an empty slot.
        let has_room = dest_slots.iter().any(|slot| match slot {
            Some((dest_id, dest_count)) => *dest_id == item_id && *dest_count < max_stack,
            None => true,
        });

        if has_room {
            return Some((src_idx, item_id, count));
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- hopper_should_transfer -----------------------------------------------

    #[test]
    fn cooldown_blocks_transfer() {
        assert!(!hopper_should_transfer(4, false));
        assert!(!hopper_should_transfer(1, false));
        assert!(!hopper_should_transfer(HOPPER_COOLDOWN, false));
    }

    #[test]
    fn zero_cooldown_unlocked_allows_transfer() {
        assert!(hopper_should_transfer(0, false));
    }

    #[test]
    fn locked_hopper_blocks_transfer() {
        assert!(!hopper_should_transfer(0, true));
    }

    #[test]
    fn locked_with_cooldown_blocks_transfer() {
        assert!(!hopper_should_transfer(5, true));
    }

    // -- hopper_pull ----------------------------------------------------------

    #[test]
    fn pull_from_first_non_empty_slot() {
        let items = [(0, 0), (264, 10), (265, 5)];
        let result = hopper_pull(&items);
        assert_eq!(result, Some((1, 264, 10)));
    }

    #[test]
    fn pull_returns_none_when_all_empty() {
        let items = [(0, 0), (0, 0), (0, 0)];
        assert_eq!(hopper_pull(&items), None);
    }

    #[test]
    fn pull_from_very_first_slot() {
        let items = [(264, 3), (265, 5)];
        let result = hopper_pull(&items);
        assert_eq!(result, Some((0, 264, 3)));
    }

    #[test]
    fn pull_from_empty_slice() {
        let items: &[(u16, u8)] = &[];
        assert_eq!(hopper_pull(items), None);
    }

    // -- hopper_push ----------------------------------------------------------

    #[test]
    fn push_to_matching_stack() {
        let self_items = [(264, 10)];
        let dest = [Some((264, 50)), Some((265, 64))];
        let result = hopper_push(&self_items, &dest, 64);
        assert_eq!(result, Some((0, 264, 10)));
    }

    #[test]
    fn push_to_empty_slot() {
        let self_items = [(264, 10)];
        let dest = [Some((265, 64)), None];
        let result = hopper_push(&self_items, &dest, 64);
        assert_eq!(result, Some((0, 264, 10)));
    }

    #[test]
    fn push_fails_when_dest_full() {
        let self_items = [(264, 10)];
        let dest = [Some((265, 64)), Some((266, 64))];
        let result = hopper_push(&self_items, &dest, 64);
        assert_eq!(result, None);
    }

    #[test]
    fn push_skips_empty_self_slots() {
        let self_items = [(0, 0), (264, 5)];
        let dest = [None];
        let result = hopper_push(&self_items, &dest, 64);
        assert_eq!(result, Some((1, 264, 5)));
    }

    #[test]
    fn push_fails_when_self_empty() {
        let self_items = [(0, 0), (0, 0)];
        let dest = [None, None];
        let result = hopper_push(&self_items, &dest, 64);
        assert_eq!(result, None);
    }

    #[test]
    fn push_respects_max_stack() {
        let self_items = [(264, 5)];
        // Destination has matching item but is already at max_stack
        let dest = [Some((264, 16))];
        let result = hopper_push(&self_items, &dest, 16);
        assert_eq!(result, None);
    }

    // -- HopperTransfer struct ------------------------------------------------

    #[test]
    fn hopper_transfer_stores_positions_and_item() {
        let transfer = HopperTransfer {
            from_pos: (10, 64, -3),
            to_pos: (10, 63, -3),
            item_id: 264,
            count: 1,
        };
        assert_eq!(transfer.from_pos, (10, 64, -3));
        assert_eq!(transfer.to_pos, (10, 63, -3));
        assert_eq!(transfer.item_id, 264);
        assert_eq!(transfer.count, 1);
    }

    // -- HopperDirection enum -------------------------------------------------

    #[test]
    fn hopper_direction_variants_are_distinct() {
        let directions = [
            HopperDirection::Down,
            HopperDirection::North,
            HopperDirection::South,
            HopperDirection::East,
            HopperDirection::West,
        ];
        for (i, a) in directions.iter().enumerate() {
            for (j, b) in directions.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    // -- HOPPER_COOLDOWN constant ---------------------------------------------

    #[test]
    fn hopper_cooldown_is_eight() {
        assert_eq!(HOPPER_COOLDOWN, 8);
    }
}
