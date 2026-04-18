use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ── Enchant Table State ───────────────────────────────────────────────────

/// A single enchantment option: `(cost, enchantments)` where enchantments is
/// a list of `(enchantment_id, level)` pairs.
pub type EnchantOption = Option<(u8, Vec<(u16, u8)>)>;

/// Represents the state of an enchanting table, including the item in the slot,
/// lapis lazuli count, surrounding bookshelves, generated enchantment options,
/// and the deterministic seed for option generation.
#[derive(Debug, Clone)]
pub struct EnchantTableState {
    /// The item placed in the enchanting slot (item ID), or `None` if empty.
    pub item_slot: Option<u16>,
    /// Number of lapis lazuli inserted (0..=3 typically consumed).
    pub lapis_count: u8,
    /// Number of bookshelves surrounding the table (effective max 15).
    pub bookshelves: u8,
    /// Three enchantment options.
    pub options: [EnchantOption; 3],
    /// Deterministic seed for reproducible option generation.
    pub seed: u64,
}

impl EnchantTableState {
    /// Create a new enchanting table state with no item and no options.
    #[must_use]
    pub fn new(bookshelves: u8, seed: u64) -> Self {
        Self {
            item_slot: None,
            lapis_count: 0,
            bookshelves,
            options: [None, None, None],
            seed,
        }
    }
}

// ── Bookshelf Power ───────────────────────────────────────────────────────

/// Calculate effective bookshelf power, capped at 15.
#[must_use]
pub fn bookshelf_power(count: u8) -> u8 {
    count.min(15)
}

// ── Deterministic hashing helper ──────────────────────────────────────────

fn deterministic_hash(seed: u64, extra: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    extra.hash(&mut hasher);
    hasher.finish()
}

// ── Option Generation ─────────────────────────────────────────────────────

/// Refresh the three enchantment options based on bookshelves and seed.
///
/// Each option has a cost tier and a list of `(enchantment_id, level)` pairs.
/// The cost tiers are strictly increasing (low, mid, high). Higher tiers
/// produce more and higher-level enchantments.
///
/// `bookshelves` is internally capped at 15.
pub fn refresh_options(state: &mut EnchantTableState) {
    let power = bookshelf_power(state.bookshelves) as u64;
    let base = power / 2;

    // Calculate three cost tiers (strictly increasing, tier3 capped at 30)
    let r1 = (deterministic_hash(state.seed, 1) % 8) + 1;
    let tier1 = 1u64.max(r1 + base);

    let r2 = deterministic_hash(state.seed, 2) % 9;
    let raw2 = tier1 + r2 + base;
    let min2 = tier1 + 1;
    let max2 = (2 * power + 1).max(min2);
    let tier2 = raw2.clamp(min2, max2).min(29);

    let r3 = deterministic_hash(state.seed, 3) % 9;
    let raw3 = tier2 + r3 + base;
    let min3 = tier2 + 1;
    let max3 = (3 * power + 1).max(min3);
    let tier3 = raw3.clamp(min3, max3).min(30);

    let costs = [tier1 as u8, tier2 as u8, tier3 as u8];

    // Generate enchantments for each tier
    for (slot, &cost) in costs.iter().enumerate() {
        let enchant_count: usize = match cost {
            0..=5 => 1,
            6..=15 => 2,
            _ => 3,
        };

        let mut selected: Vec<(u16, u8)> = Vec::new();
        for i in 0..enchant_count {
            let id_hash = deterministic_hash(state.seed, (slot as u64 + 1) * 100 + i as u64);
            // Generate enchantment ID in range 0..26 (matches EnchantmentId::ALL)
            let ench_id = (id_hash % 26) as u16;

            // Avoid duplicate enchantment IDs within the same option
            if selected.iter().any(|(id, _)| *id == ench_id) {
                let fallback_id = ((ench_id + 1) % 26) as u16;
                if !selected.iter().any(|(id, _)| *id == fallback_id) {
                    let level_hash =
                        deterministic_hash(state.seed, (slot as u64 + 1) * 200 + i as u64);
                    let level = ((level_hash % 5) + 1) as u8;
                    selected.push((fallback_id, level));
                }
            } else {
                let level_hash =
                    deterministic_hash(state.seed, (slot as u64 + 1) * 200 + i as u64);
                let level = ((level_hash % 5) + 1) as u8;
                selected.push((ench_id, level));
            }
        }

        state.options[slot] = Some((cost, selected));
    }
}

// ── Can Enchant ───────────────────────────────────────────────────────────

/// Check whether the player can apply the enchantment in the given slot.
///
/// Requirements:
/// - `slot` must be 0, 1, or 2
/// - An item must be in the enchanting slot
/// - The option at `slot` must exist
/// - The player must have enough XP levels (>= option cost)
/// - The player must have enough lapis lazuli (>= slot index + 1)
#[must_use]
pub fn can_enchant(state: &EnchantTableState, slot: usize, player_level: u32, lapis: u8) -> bool {
    if slot > 2 {
        return false;
    }
    if state.item_slot.is_none() {
        return false;
    }
    let option = match &state.options[slot] {
        Some(opt) => opt,
        None => return false,
    };
    let required_lapis = (slot as u8) + 1;
    let required_level = option.0 as u32;

    player_level >= required_level && lapis >= required_lapis
}

// ── Apply Enchant ─────────────────────────────────────────────────────────

/// Apply the enchantment from the given slot, consuming lapis lazuli.
///
/// Returns the list of `(enchantment_id, level)` pairs if successful,
/// or `None` if the slot is invalid or has no option.
///
/// On success:
/// - The lapis count is decremented by `slot + 1`
/// - All three options are cleared (a new seed/refresh is needed)
/// - The item slot is cleared
pub fn apply_enchant(state: &mut EnchantTableState, slot: usize) -> Option<Vec<(u16, u8)>> {
    if slot > 2 {
        return None;
    }
    let option = state.options[slot].take()?;
    let lapis_cost = (slot as u8) + 1;
    state.lapis_count = state.lapis_count.saturating_sub(lapis_cost);

    // Clear remaining options and item
    state.options = [None, None, None];
    state.item_slot = None;

    Some(option.1)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Bookshelf power ───────────────────────────────────────────────

    #[test]
    fn bookshelf_power_caps_at_15() {
        assert_eq!(bookshelf_power(0), 0);
        assert_eq!(bookshelf_power(10), 10);
        assert_eq!(bookshelf_power(15), 15);
        assert_eq!(bookshelf_power(20), 15);
        assert_eq!(bookshelf_power(255), 15);
    }

    // ── Option generation ─────────────────────────────────────────────

    #[test]
    fn refresh_generates_three_options() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        refresh_options(&mut state);

        for (i, opt) in state.options.iter().enumerate() {
            assert!(opt.is_some(), "option {i} should be Some after refresh");
        }
    }

    #[test]
    fn refresh_costs_are_strictly_increasing() {
        for seed in 0..100u64 {
            let mut state = EnchantTableState::new(15, seed);
            state.item_slot = Some(200);
            refresh_options(&mut state);

            let costs: Vec<u8> = state
                .options
                .iter()
                .map(|o| o.as_ref().unwrap().0)
                .collect();
            assert!(
                costs[0] < costs[1] && costs[1] < costs[2],
                "seed {seed}: costs {costs:?} not strictly increasing"
            );
        }
    }

    #[test]
    fn refresh_tier3_capped_at_30() {
        for seed in 0..200u64 {
            let mut state = EnchantTableState::new(15, seed);
            state.item_slot = Some(200);
            refresh_options(&mut state);

            let cost3 = state.options[2].as_ref().unwrap().0;
            assert!(cost3 <= 30, "seed {seed}: tier3 cost {cost3} exceeds 30");
        }
    }

    #[test]
    fn refresh_deterministic_same_seed() {
        let mut a = EnchantTableState::new(10, 12345);
        a.item_slot = Some(200);
        refresh_options(&mut a);

        let mut b = EnchantTableState::new(10, 12345);
        b.item_slot = Some(200);
        refresh_options(&mut b);

        for i in 0..3 {
            assert_eq!(
                a.options[i], b.options[i],
                "option {i} differs for same seed"
            );
        }
    }

    #[test]
    fn refresh_each_option_has_enchantments() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        refresh_options(&mut state);

        for (i, opt) in state.options.iter().enumerate() {
            let enchants = &opt.as_ref().unwrap().1;
            assert!(
                !enchants.is_empty(),
                "option {i} should have at least one enchantment"
            );
        }
    }

    #[test]
    fn refresh_no_duplicate_enchantments_within_option() {
        for seed in 0..50u64 {
            let mut state = EnchantTableState::new(15, seed);
            state.item_slot = Some(200);
            refresh_options(&mut state);

            for (slot_idx, opt) in state.options.iter().enumerate() {
                let enchants = &opt.as_ref().unwrap().1;
                for (i, (id, _)) in enchants.iter().enumerate() {
                    assert!(
                        !enchants[i + 1..].iter().any(|(other_id, _)| other_id == id),
                        "seed {seed}, slot {slot_idx}: duplicate enchantment id {id}"
                    );
                }
            }
        }
    }

    // ── Level / lapis requirements ────────────────────────────────────

    #[test]
    fn can_enchant_requires_item_in_slot() {
        let mut state = EnchantTableState::new(15, 42);
        refresh_options(&mut state);
        // No item in slot
        assert!(!can_enchant(&state, 0, 30, 3));
    }

    #[test]
    fn can_enchant_requires_valid_slot() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        refresh_options(&mut state);
        assert!(!can_enchant(&state, 3, 30, 3));
    }

    #[test]
    fn can_enchant_requires_sufficient_level() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        refresh_options(&mut state);

        let cost = state.options[2].as_ref().unwrap().0;
        // Player level below cost should fail
        assert!(!can_enchant(&state, 2, (cost as u32) - 1, 3));
        // Player level at cost should succeed
        assert!(can_enchant(&state, 2, cost as u32, 3));
    }

    #[test]
    fn can_enchant_requires_sufficient_lapis() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        refresh_options(&mut state);

        // Slot 0 needs 1 lapis, slot 1 needs 2, slot 2 needs 3
        assert!(can_enchant(&state, 0, 30, 1));
        assert!(!can_enchant(&state, 1, 30, 1));
        assert!(can_enchant(&state, 1, 30, 2));
        assert!(!can_enchant(&state, 2, 30, 2));
        assert!(can_enchant(&state, 2, 30, 3));
    }

    #[test]
    fn can_enchant_slot0_needs_1_lapis() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        refresh_options(&mut state);

        assert!(!can_enchant(&state, 0, 30, 0));
        assert!(can_enchant(&state, 0, 30, 1));
    }

    // ── Bookshelf capping ─────────────────────────────────────────────

    #[test]
    fn bookshelf_capping_produces_same_options() {
        let mut a = EnchantTableState::new(15, 99);
        a.item_slot = Some(200);
        refresh_options(&mut a);

        let mut b = EnchantTableState::new(30, 99);
        b.item_slot = Some(200);
        refresh_options(&mut b);

        for i in 0..3 {
            assert_eq!(
                a.options[i], b.options[i],
                "option {i} should be identical for 15 and 30 bookshelves"
            );
        }
    }

    // ── Apply enchant ─────────────────────────────────────────────────

    #[test]
    fn apply_enchant_returns_enchantments_and_clears_state() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        state.lapis_count = 3;
        refresh_options(&mut state);

        let expected = state.options[1].as_ref().unwrap().1.clone();
        let result = apply_enchant(&mut state, 1);
        assert_eq!(result, Some(expected));

        // State should be cleared
        assert!(state.item_slot.is_none());
        assert!(state.options.iter().all(|o| o.is_none()));
        // Lapis decremented by slot+1 = 2
        assert_eq!(state.lapis_count, 1);
    }

    #[test]
    fn apply_enchant_invalid_slot_returns_none() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        refresh_options(&mut state);

        assert_eq!(apply_enchant(&mut state, 3), None);
    }

    #[test]
    fn apply_enchant_empty_option_returns_none() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        // Options not refreshed, all None
        assert_eq!(apply_enchant(&mut state, 0), None);
    }

    #[test]
    fn apply_enchant_lapis_saturates_at_zero() {
        let mut state = EnchantTableState::new(15, 42);
        state.item_slot = Some(200);
        state.lapis_count = 0;
        refresh_options(&mut state);

        // Even with 0 lapis, apply_enchant itself does not check —
        // that is can_enchant's job. apply_enchant just saturating-subtracts.
        let result = apply_enchant(&mut state, 2);
        assert!(result.is_some());
        assert_eq!(state.lapis_count, 0);
    }

    #[test]
    fn zero_bookshelves_produces_low_costs() {
        for seed in 0..50u64 {
            let mut state = EnchantTableState::new(0, seed);
            state.item_slot = Some(200);
            refresh_options(&mut state);

            let cost1 = state.options[0].as_ref().unwrap().0;
            let cost3 = state.options[2].as_ref().unwrap().0;
            assert!(cost1 <= 8, "seed {seed}: tier1 cost {cost1}");
            assert!(cost3 <= 25, "seed {seed}: tier3 cost {cost3}");
        }
    }
}
