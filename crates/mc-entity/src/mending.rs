//! Mending enchantment: collected XP orbs repair held/equipped items instead of granting XP.
//!
//! In vanilla Minecraft, each XP point repairs 2 points of durability on a randomly
//! chosen mending-enchanted, damaged item. Any XP not consumed by mending falls
//! through to the player's normal XP pool.

/// Durability points repaired per XP point consumed.
pub const DURABILITY_PER_XP: u32 = 2;

/// Returns the durability that would be repaired for the given XP orb value,
/// assuming the entire orb is consumed by mending.
pub fn mending_repair_amount(xp_orb_value: u32) -> u32 {
    xp_orb_value.saturating_mul(DURABILITY_PER_XP)
}

/// Returns the XP needed to repair the given amount of durability,
/// rounded up (ceil(damage / 2)).
pub fn xp_used_for_repair(damage_to_repair: u32) -> u32 {
    damage_to_repair.div_ceil(DURABILITY_PER_XP)
}

/// A candidate item for the mending repair selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MendingItem {
    pub slot_index: usize,
    pub item_id: u16,
    pub damaged: u16,
}

/// Placeholder rule: tools/armor live in the item-id band 4000..=5000.
/// A handful of additional vanilla tool ids are recognized explicitly so callers
/// outside that range still benefit from mending.
pub fn is_mending_item(item_id: u16) -> bool {
    const EXTRA_TOOL_IDS: &[u16] = &[256, 257, 258, 267, 268, 269, 270, 271, 272, 273, 274, 275];
    if (4000..=5000).contains(&item_id) {
        return true;
    }
    EXTRA_TOOL_IDS.contains(&item_id)
}

/// Selects one damaged item from the candidate slice using a deterministic
/// seed-based pseudo-random pick. Returns the slot index of the chosen item,
/// or `None` if no item is damaged.
pub fn select_mending_item(items: &[MendingItem], seed: u64) -> Option<usize> {
    let damaged: Vec<&MendingItem> = items.iter().filter(|i| i.damaged > 0).collect();
    if damaged.is_empty() {
        return None;
    }
    // SplitMix64-style mix for a cheap deterministic index.
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let idx = (z as usize) % damaged.len();
    Some(damaged[idx].slot_index)
}

/// Applies mending to an item with the given current damage and available XP.
///
/// Returns `(new_damage, xp_consumed)` where `xp_consumed` is the XP used by
/// mending (the remainder is left to the caller for normal XP pickup).
pub fn apply_mending(item_damage: u16, xp_available: u32) -> (u16, u32) {
    if item_damage == 0 || xp_available == 0 {
        return (item_damage, 0);
    }
    let max_repair = mending_repair_amount(xp_available);
    let actual_repair = max_repair.min(item_damage as u32);
    let xp_consumed = xp_used_for_repair(actual_repair);
    let new_damage = item_damage.saturating_sub(actual_repair as u16);
    (new_damage, xp_consumed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_amount_is_double_xp() {
        assert_eq!(mending_repair_amount(0), 0);
        assert_eq!(mending_repair_amount(1), 2);
        assert_eq!(mending_repair_amount(7), 14);
    }

    #[test]
    fn xp_used_rounds_up() {
        assert_eq!(xp_used_for_repair(0), 0);
        assert_eq!(xp_used_for_repair(1), 1);
        assert_eq!(xp_used_for_repair(2), 1);
        assert_eq!(xp_used_for_repair(3), 2);
        assert_eq!(xp_used_for_repair(10), 5);
    }

    #[test]
    fn recognizes_tool_and_armor_ids() {
        assert!(is_mending_item(4000));
        assert!(is_mending_item(4500));
        assert!(is_mending_item(5000));
        assert!(is_mending_item(256)); // extra tool id
        assert!(!is_mending_item(0));
        assert!(!is_mending_item(3999));
        assert!(!is_mending_item(5001));
    }

    #[test]
    fn select_returns_none_when_no_damage() {
        let items = vec![
            MendingItem { slot_index: 0, item_id: 4001, damaged: 0 },
            MendingItem { slot_index: 1, item_id: 4002, damaged: 0 },
        ];
        assert_eq!(select_mending_item(&items, 42), None);
    }

    #[test]
    fn select_returns_none_for_empty() {
        assert_eq!(select_mending_item(&[], 42), None);
    }

    #[test]
    fn select_picks_only_damaged_slot() {
        let items = vec![
            MendingItem { slot_index: 7, item_id: 4001, damaged: 0 },
            MendingItem { slot_index: 9, item_id: 4002, damaged: 5 },
            MendingItem { slot_index: 11, item_id: 4003, damaged: 0 },
        ];
        for seed in 0..32u64 {
            assert_eq!(select_mending_item(&items, seed), Some(9));
        }
    }

    #[test]
    fn select_is_deterministic_for_same_seed() {
        let items = vec![
            MendingItem { slot_index: 1, item_id: 4001, damaged: 5 },
            MendingItem { slot_index: 2, item_id: 4002, damaged: 5 },
            MendingItem { slot_index: 3, item_id: 4003, damaged: 5 },
        ];
        let a = select_mending_item(&items, 12345);
        let b = select_mending_item(&items, 12345);
        assert_eq!(a, b);
        assert!(a.is_some());
        let chosen = a.unwrap();
        assert!([1, 2, 3].contains(&chosen));
    }

    #[test]
    fn apply_mending_no_op_when_undamaged() {
        let (dmg, xp) = apply_mending(0, 10);
        assert_eq!(dmg, 0);
        assert_eq!(xp, 0);
    }

    #[test]
    fn apply_mending_no_op_when_no_xp() {
        let (dmg, xp) = apply_mending(50, 0);
        assert_eq!(dmg, 50);
        assert_eq!(xp, 0);
    }

    #[test]
    fn apply_mending_partial_repair() {
        // 3 XP -> up to 6 durability repaired; 50 -> 44, all 3 XP consumed.
        let (dmg, xp) = apply_mending(50, 3);
        assert_eq!(dmg, 44);
        assert_eq!(xp, 3);
    }

    #[test]
    fn apply_mending_caps_at_full_repair() {
        // 10 XP would repair 20; only 5 damage exists -> consumes 3 XP (ceil(5/2)).
        let (dmg, xp) = apply_mending(5, 10);
        assert_eq!(dmg, 0);
        assert_eq!(xp, 3);
    }

    #[test]
    fn apply_mending_exact_match() {
        // 4 XP -> 8 durability; 8 damage -> fully repaired with 4 XP consumed.
        let (dmg, xp) = apply_mending(8, 4);
        assert_eq!(dmg, 0);
        assert_eq!(xp, 4);
    }
}
