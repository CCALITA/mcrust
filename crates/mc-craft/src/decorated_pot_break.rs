//! Decorated pot breaking, sherds, and pot contents.
//!
//! A decorated pot has up to 4 sherds (one per face) plus a single-stack inventory
//! of any item (max 64). Breaking it drops the sherds and contents; silk touch
//! drops the whole pot block intact.

/// Default decorated pot block/item ID.
pub const DEFAULT_POT_ID: u16 = 7100;

/// Maximum stack size for the pot's single-slot inventory.
pub const POT_MAX_CAPACITY: u32 = 64;

/// All 20 sherd item IDs, 7000..=7019.
const SHERD_IDS: [u16; 20] = [
    7000, 7001, 7002, 7003, 7004, 7005, 7006, 7007, 7008, 7009, 7010, 7011, 7012, 7013, 7014,
    7015, 7016, 7017, 7018, 7019,
];

/// Returns the list of valid sherd item IDs.
pub fn valid_sherd_item_ids() -> &'static [u16] {
    &SHERD_IDS
}

/// Returns the default decorated pot item/block ID.
pub fn default_pot_id() -> u16 {
    DEFAULT_POT_ID
}

/// Contents of a decorated pot — a single stack of any item, max 64.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotContents {
    pub items: Vec<(u16, u8)>,
    pub max_capacity: u32,
}

impl PotContents {
    /// Creates a new empty pot inventory with max capacity 64.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_capacity: POT_MAX_CAPACITY,
        }
    }

    /// Adds `count` of `item` to the pot. Returns false if the pot is full or
    /// already contains a different item. On success returns true and produces
    /// an updated stack (no-op if count is zero and slot is empty).
    pub fn add(&mut self, item: u16, count: u8) -> bool {
        if count == 0 {
            return false;
        }
        match self.items.first().copied() {
            None => {
                let clamped = (count as u32).min(self.max_capacity) as u8;
                if clamped == 0 {
                    return false;
                }
                self.items = vec![(item, clamped)];
                clamped == count
            }
            Some((existing_item, existing_count)) => {
                if existing_item != item {
                    return false;
                }
                let total = existing_count as u32 + count as u32;
                if total > self.max_capacity {
                    return false;
                }
                self.items = vec![(existing_item, total as u8)];
                true
            }
        }
    }
}

impl Default for PotContents {
    fn default() -> Self {
        Self::new()
    }
}

/// Drops produced when a decorated pot is broken without silk touch.
/// Returns the `Some` sherds (in face order) followed by the pot contents.
pub fn pot_break_drops(sherds: [Option<u16>; 4], contents: &PotContents) -> Vec<(u16, u8)> {
    let mut drops: Vec<(u16, u8)> = sherds
        .iter()
        .filter_map(|s| s.map(|id| (id, 1u8)))
        .collect();
    drops.extend(contents.items.iter().copied());
    drops
}

/// Drops when a decorated pot is broken with silk touch — returns the pot block intact.
pub fn silk_touch_pot_drops(pot_id: u16) -> Vec<(u16, u8)> {
    vec![(pot_id, 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pot_is_empty_with_cap_64() {
        let p = PotContents::new();
        assert!(p.items.is_empty());
        assert_eq!(p.max_capacity, 64);
    }

    #[test]
    fn add_to_empty_pot_succeeds() {
        let mut p = PotContents::new();
        assert!(p.add(42, 10));
        assert_eq!(p.items, vec![(42, 10)]);
    }

    #[test]
    fn add_same_item_stacks() {
        let mut p = PotContents::new();
        assert!(p.add(42, 10));
        assert!(p.add(42, 20));
        assert_eq!(p.items, vec![(42, 30)]);
    }

    #[test]
    fn add_different_item_fails() {
        let mut p = PotContents::new();
        assert!(p.add(42, 10));
        assert!(!p.add(99, 5));
        assert_eq!(p.items, vec![(42, 10)]);
    }

    #[test]
    fn add_over_capacity_fails() {
        let mut p = PotContents::new();
        assert!(p.add(42, 50));
        assert!(!p.add(42, 20));
        assert_eq!(p.items, vec![(42, 50)]);
    }

    #[test]
    fn add_zero_count_fails() {
        let mut p = PotContents::new();
        assert!(!p.add(42, 0));
        assert!(p.items.is_empty());
    }

    #[test]
    fn pot_break_drops_all_sherds_and_contents() {
        let mut p = PotContents::new();
        p.add(500, 3);
        let drops = pot_break_drops([Some(7000), Some(7005), Some(7010), Some(7019)], &p);
        assert_eq!(
            drops,
            vec![(7000, 1), (7005, 1), (7010, 1), (7019, 1), (500, 3)]
        );
    }

    #[test]
    fn pot_break_drops_skips_none_sherds() {
        let p = PotContents::new();
        let drops = pot_break_drops([Some(7000), None, Some(7010), None], &p);
        assert_eq!(drops, vec![(7000, 1), (7010, 1)]);
    }

    #[test]
    fn pot_break_empty_pot_no_sherds_returns_empty() {
        let p = PotContents::new();
        let drops = pot_break_drops([None, None, None, None], &p);
        assert!(drops.is_empty());
    }

    #[test]
    fn silk_touch_returns_pot_block() {
        assert_eq!(silk_touch_pot_drops(7100), vec![(7100, 1)]);
        assert_eq!(silk_touch_pot_drops(default_pot_id()), vec![(7100, 1)]);
    }

    #[test]
    fn valid_sherd_ids_has_20_entries_7000_to_7019() {
        let ids = valid_sherd_item_ids();
        assert_eq!(ids.len(), 20);
        assert_eq!(ids[0], 7000);
        assert_eq!(ids[19], 7019);
    }

    #[test]
    fn default_pot_id_is_7100() {
        assert_eq!(default_pot_id(), 7100);
    }
}
