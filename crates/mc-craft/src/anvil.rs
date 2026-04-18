/// Anvil mechanics: combining, renaming, repairing, and enchantment merging.

/// The result of an anvil operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnvilResult {
    pub output_item: u16,
    pub output_count: u8,
    pub xp_cost: u32,
    pub output_name: Option<String>,
}

/// Maximum enchantment level cap used when merging enchantments.
const MAX_ENCHANT_LEVEL: u8 = 255;

/// Combine two items on an anvil.
///
/// - If `left_item == right_item`, the items are repaired and enchantments are merged.
/// - If `right_item` is a book (item id `387` — enchanted book), enchantments are
///   transferred from the book to the left item.
/// - Returns `None` if the items cannot be combined.
pub fn anvil_combine(
    left_item: u16,
    left_count: u8,
    right_item: u16,
    _right_count: u8,
    left_enchants: &[(u16, u8)],
    right_enchants: &[(u16, u8)],
) -> Option<AnvilResult> {
    const ENCHANTED_BOOK: u16 = 387;

    let is_same_type = left_item == right_item;
    let is_book_transfer = right_item == ENCHANTED_BOOK;

    if !is_same_type && !is_book_transfer {
        return None;
    }

    let merged = merge_enchantments(left_enchants, right_enchants);

    // Base XP cost: 2 for same-type repair, 1 for book transfer,
    // plus 1 per enchantment on the result.
    let base_cost: u32 = if is_same_type { 2 } else { 1 };
    let enchant_cost = merged.len() as u32;
    let xp_cost = base_cost + enchant_cost;

    Some(AnvilResult {
        output_item: left_item,
        output_count: left_count,
        xp_cost,
        output_name: None,
    })
}

/// Rename an item on the anvil for 1 XP level.
pub fn anvil_rename(item: u16, count: u8, name: &str) -> AnvilResult {
    AnvilResult {
        output_item: item,
        output_count: count,
        xp_cost: 1,
        output_name: Some(name.to_owned()),
    }
}

/// Calculate cumulative repair cost. Each prior repair doubles the cost,
/// starting at 1 for the first repair: `2^uses - 1`.
pub fn repair_cost(uses: u32) -> u32 {
    if uses == 0 {
        return 0;
    }
    // 2^uses - 1, saturating to avoid overflow for large values.
    2u32.saturating_pow(uses).saturating_sub(1)
}

/// Merge two sets of enchantments. For enchantments present on both items:
/// - If one level is higher, the higher level wins.
/// - If both levels are equal, the level is incremented by 1 (capped at
///   `MAX_ENCHANT_LEVEL`).
///
/// Enchantments only on the left or only on the right are kept as-is.
pub fn merge_enchantments(left: &[(u16, u8)], right: &[(u16, u8)]) -> Vec<(u16, u8)> {
    // Start with a copy of the left enchantments.
    let mut result: Vec<(u16, u8)> = left.to_vec();

    for &(r_id, r_level) in right {
        if let Some(entry) = result.iter_mut().find(|(id, _)| *id == r_id) {
            if r_level > entry.1 {
                entry.1 = r_level;
            } else if r_level == entry.1 {
                entry.1 = entry.1.saturating_add(1).min(MAX_ENCHANT_LEVEL);
            }
            // If r_level < entry.1, left already has the higher level — keep it.
        } else {
            result.push((r_id, r_level));
        }
    }

    result
}

/// Returns `true` if the anvil degrades (12% chance).
/// `random` should be a value in `[0.0, 1.0)`.
pub fn anvil_degrades(random: f32) -> bool {
    random < 0.12
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── anvil_combine ──────────────────────────────────────────────────

    #[test]
    fn combine_same_type_repairs_and_merges_enchants() {
        let left_enchants = [(1, 2)];
        let right_enchants = [(1, 2), (2, 1)];

        let result = anvil_combine(
            100, 1, // left: iron sword
            100, 1, // right: iron sword
            &left_enchants,
            &right_enchants,
        );

        let result = result.expect("same-type combine should succeed");
        assert_eq!(result.output_item, 100);
        assert_eq!(result.output_count, 1);
        // base 2 (same type) + 2 enchantments on result = 4
        assert_eq!(result.xp_cost, 4);
        assert_eq!(result.output_name, None);
    }

    #[test]
    fn combine_book_transfers_enchants() {
        let left_enchants = [(1, 1)];
        let right_enchants = [(2, 3)];

        let result = anvil_combine(
            100, 1,   // left: iron sword
            387, 1,   // right: enchanted book
            &left_enchants,
            &right_enchants,
        );

        let result = result.expect("book transfer should succeed");
        assert_eq!(result.output_item, 100);
        // base 1 (book) + 2 enchantments = 3
        assert_eq!(result.xp_cost, 3);
    }

    #[test]
    fn combine_incompatible_items_returns_none() {
        let result = anvil_combine(100, 1, 200, 1, &[], &[]);
        assert!(result.is_none());
    }

    // ── anvil_rename ───────────────────────────────────────────────────

    #[test]
    fn rename_costs_one_xp() {
        let result = anvil_rename(100, 3, "Excalibur");
        assert_eq!(result.xp_cost, 1);
        assert_eq!(result.output_item, 100);
        assert_eq!(result.output_count, 3);
        assert_eq!(result.output_name.as_deref(), Some("Excalibur"));
    }

    // ── repair_cost ────────────────────────────────────────────────────

    #[test]
    fn repair_cost_doubles_each_use() {
        assert_eq!(repair_cost(0), 0);
        assert_eq!(repair_cost(1), 1);  // 2^1 - 1
        assert_eq!(repair_cost(2), 3);  // 2^2 - 1
        assert_eq!(repair_cost(3), 7);  // 2^3 - 1
        assert_eq!(repair_cost(4), 15); // 2^4 - 1
    }

    #[test]
    fn repair_cost_saturates_on_large_input() {
        // Should not panic; returns a saturated value.
        let cost = repair_cost(50);
        assert!(cost > 0);
    }

    // ── merge_enchantments ─────────────────────────────────────────────

    #[test]
    fn merge_higher_level_wins() {
        let left = [(1, 2)];
        let right = [(1, 3)];
        let merged = merge_enchantments(&left, &right);
        assert_eq!(merged, vec![(1, 3)]);
    }

    #[test]
    fn merge_equal_levels_increments() {
        let left = [(1, 2)];
        let right = [(1, 2)];
        let merged = merge_enchantments(&left, &right);
        assert_eq!(merged, vec![(1, 3)]);
    }

    #[test]
    fn merge_equal_at_max_caps() {
        let left = [(1, MAX_ENCHANT_LEVEL)];
        let right = [(1, MAX_ENCHANT_LEVEL)];
        let merged = merge_enchantments(&left, &right);
        assert_eq!(merged, vec![(1, MAX_ENCHANT_LEVEL)]);
    }

    #[test]
    fn merge_disjoint_enchantments_combines() {
        let left = [(1, 1)];
        let right = [(2, 3)];
        let merged = merge_enchantments(&left, &right);
        assert_eq!(merged, vec![(1, 1), (2, 3)]);
    }

    #[test]
    fn merge_empty_right_returns_left() {
        let left = [(5, 2), (6, 1)];
        let merged = merge_enchantments(&left, &[]);
        assert_eq!(merged, vec![(5, 2), (6, 1)]);
    }

    #[test]
    fn merge_empty_left_returns_right() {
        let right = [(3, 4)];
        let merged = merge_enchantments(&[], &right);
        assert_eq!(merged, vec![(3, 4)]);
    }

    // ── anvil_degrades ─────────────────────────────────────────────────

    #[test]
    fn degrades_below_threshold() {
        assert!(anvil_degrades(0.0));
        assert!(anvil_degrades(0.11));
    }

    #[test]
    fn does_not_degrade_above_threshold() {
        assert!(!anvil_degrades(0.12));
        assert!(!anvil_degrades(0.5));
        assert!(!anvil_degrades(0.99));
    }
}
