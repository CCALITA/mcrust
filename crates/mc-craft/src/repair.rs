/// Crafting-grid repair: combining two identical damageable items.
///
/// Unlike anvil repair, grid repair **removes all enchantments** from the
/// resulting item. The output durability is the sum of both inputs plus a
/// 5% bonus of `max_dur`, capped at `max_dur`.

/// Item-id ranges that correspond to damageable equipment (tools, armor,
/// weapons, and utility items such as shears/fishing rods/shields).
const DAMAGEABLE_RANGES: &[(u16, u16)] = &[
    (200, 233), // tools: wooden through diamond pickaxe/axe/shovel/sword
    (300, 333), // armor: leather through diamond helmet/chest/legs/boots
    (500, 505), // utility: bucket, redstone_dust… shears, fishing_rod
    (600, 604), // weapons: bow, arrow, flint, feather, shield
];

/// Returns `true` if the given item id represents a damageable item.
#[must_use]
pub fn is_damageable(item: u16) -> bool {
    DAMAGEABLE_RANGES
        .iter()
        .any(|&(lo, hi)| item >= lo && item <= hi)
}

/// Returns `true` if two items can be repaired together in a crafting grid.
///
/// Both items must share the same item id **and** that id must be damageable.
#[must_use]
pub fn can_repair_in_grid(item1: u16, item2: u16) -> bool {
    item1 == item2 && is_damageable(item1)
}

/// Calculate the resulting durability when two items are combined.
///
/// Formula: `min(dur1 + dur2 + floor(max_dur * 0.05), max_dur)`.
#[must_use]
pub fn calculate_repaired_durability(dur1: u32, dur2: u32, max_dur: u32) -> u32 {
    let bonus = max_dur / 20; // 5% of max, integer division
    dur1.saturating_add(dur2)
        .saturating_add(bonus)
        .min(max_dur)
}

/// Attempt to repair two items in a crafting grid.
///
/// Returns `Some((output_item, output_durability))` on success.
/// The output item id equals `item1` (== `item2`). Enchantments are
/// **not** preserved (unlike anvil repair).
///
/// Returns `None` when:
/// - The two item ids differ, or
/// - The item is not damageable.
#[must_use]
pub fn repair_in_grid(
    item1: u16,
    dur1: u32,
    item2: u16,
    dur2: u32,
    max_dur: u32,
) -> Option<(u16, u32)> {
    if !can_repair_in_grid(item1, item2) {
        return None;
    }
    let repaired = calculate_repaired_durability(dur1, dur2, max_dur);
    Some((item1, repaired))
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_damageable ─────────────────────────────────────────────────

    #[test]
    fn tools_are_damageable() {
        assert!(is_damageable(200)); // wooden pickaxe
        assert!(is_damageable(223)); // iron sword
        assert!(is_damageable(233)); // diamond sword
    }

    #[test]
    fn armor_is_damageable() {
        assert!(is_damageable(300)); // leather helmet
        assert!(is_damageable(333)); // diamond boots
    }

    #[test]
    fn raw_materials_are_not_damageable() {
        assert!(!is_damageable(100)); // oak log
        assert!(!is_damageable(109)); // iron ingot
        assert!(!is_damageable(410)); // wool
    }

    // ── can_repair_in_grid ────────────────────────────────────────────

    #[test]
    fn same_damageable_items_can_repair() {
        assert!(can_repair_in_grid(220, 220)); // two iron pickaxes
    }

    #[test]
    fn different_items_cannot_repair() {
        assert!(!can_repair_in_grid(220, 221)); // iron pickaxe vs iron axe
    }

    #[test]
    fn non_damageable_same_id_cannot_repair() {
        assert!(!can_repair_in_grid(100, 100)); // two oak logs
    }

    // ── calculate_repaired_durability ─────────────────────────────────

    #[test]
    fn durability_adds_with_five_percent_bonus() {
        // 20 + 20 + floor(100 * 0.05) = 45
        assert_eq!(calculate_repaired_durability(20, 20, 100), 45);
    }

    #[test]
    fn durability_caps_at_max() {
        assert_eq!(calculate_repaired_durability(90, 90, 100), 100);
    }

    #[test]
    fn zero_durability_inputs_get_only_bonus() {
        // 0 + 0 + floor(200 * 0.05) = 10
        assert_eq!(calculate_repaired_durability(0, 0, 200), 10);
    }

    // ── repair_in_grid ────────────────────────────────────────────────

    #[test]
    fn repair_returns_combined_durability() {
        let result = repair_in_grid(220, 30, 220, 25, 250);
        // 30 + 25 + floor(250 * 0.05) = 30 + 25 + 12 = 67
        assert_eq!(result, Some((220, 67)));
    }

    #[test]
    fn repair_caps_result_at_max_durability() {
        let result = repair_in_grid(220, 200, 220, 200, 250);
        assert_eq!(result, Some((220, 250)));
    }

    #[test]
    fn repair_rejects_different_item_types() {
        let result = repair_in_grid(220, 30, 221, 25, 250);
        assert!(result.is_none());
    }

    #[test]
    fn repair_rejects_non_damageable_items() {
        let result = repair_in_grid(100, 30, 100, 25, 250);
        assert!(result.is_none());
    }
}
