//! Enchantment compatibility checks using numeric IDs.
//!
//! Provides fast, `u8`-based lookups for enchantment names, max levels, and
//! mutual-exclusivity rules. Designed for anvil merging, enchanting-table
//! validation, and grindstone logic where numeric IDs are more convenient
//! than the enum-based [`crate::enchanting`] system.

/// Total number of recognized enchantments (IDs 0..36).
pub const TOTAL_ENCHANTMENTS: usize = 37;

/// Incompatible enchantment pairs.
///
/// Each entry `(a, b)` means enchantment `a` and `b` cannot coexist on the
/// same item. The check is symmetric: `are_compatible(a, b)` and
/// `are_compatible(b, a)` return the same result.
const INCOMPATIBLE_PAIRS: &[(u8, u8)] = &[
    // Damage group: sharpness(0) / smite(1) / bane_of_arthropods(2)
    (0, 1),
    (0, 2),
    (1, 2),
    // Protection group: protection(3) / fire_protection(4) / blast_protection(5) / projectile_protection(6)
    (3, 4),
    (3, 5),
    (3, 6),
    (4, 5),
    (4, 6),
    (5, 6),
    // Fortune(7) / silk_touch(8)
    (7, 8),
    // Infinity(9) / mending(10)
    (9, 10),
    // Depth_strider(11) / frost_walker(12)
    (11, 12),
    // Riptide(13) / loyalty(14)
    (13, 14),
    // Riptide(13) / channeling(15)
    (13, 15),
    // Multishot(16) / piercing(17)
    (16, 17),
];

/// Returns `true` when enchantments `a` and `b` can coexist on the same item.
///
/// Two identical IDs are always compatible (stacking the same enchantment is
/// handled elsewhere). Out-of-range IDs (>= [`TOTAL_ENCHANTMENTS`]) are
/// treated as compatible because they have no known conflict rules.
#[must_use]
pub fn are_compatible(a: u8, b: u8) -> bool {
    if a == b {
        return true;
    }
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    !INCOMPATIBLE_PAIRS
        .iter()
        .any(|&(x, y)| x == lo && y == hi)
}

/// Maximum level for the given enchantment ID.
///
/// Returns `0` for unrecognized IDs (>= [`TOTAL_ENCHANTMENTS`]).
#[must_use]
pub fn max_enchant_level(id: u8) -> u8 {
    match id {
        0 => 5,  // sharpness
        1 => 5,  // smite
        2 => 5,  // bane_of_arthropods
        3 => 4,  // protection
        4 => 4,  // fire_protection
        5 => 4,  // blast_protection
        6 => 4,  // projectile_protection
        7 => 3,  // fortune
        8 => 1,  // silk_touch
        9 => 1,  // infinity
        10 => 1, // mending
        11 => 3, // depth_strider
        12 => 2, // frost_walker
        13 => 3, // riptide
        14 => 3, // loyalty
        15 => 1, // channeling
        16 => 1, // multishot
        17 => 4, // piercing
        18 => 5, // power
        19 => 2, // punch
        20 => 1, // flame
        21 => 2, // knockback
        22 => 2, // fire_aspect
        23 => 3, // looting
        24 => 3, // sweeping_edge
        25 => 5, // efficiency
        26 => 3, // unbreaking
        27 => 3, // thorns
        28 => 3, // respiration
        29 => 1, // aqua_affinity
        30 => 3, // lure
        31 => 3, // luck_of_the_sea
        32 => 3, // impaling
        33 => 1, // curse_of_vanishing
        34 => 1, // curse_of_binding
        35 => 1, // soul_speed
        36 => 1, // swift_sneak
        _ => 0,
    }
}

/// Number of enchantments in the registry.
#[must_use]
pub const fn total_enchantments() -> usize {
    TOTAL_ENCHANTMENTS
}

/// Human-readable name for the given enchantment ID.
///
/// Returns `"Unknown"` for unrecognized IDs (>= [`TOTAL_ENCHANTMENTS`]).
#[must_use]
pub fn enchantment_name(id: u8) -> &'static str {
    match id {
        0 => "Sharpness",
        1 => "Smite",
        2 => "Bane of Arthropods",
        3 => "Protection",
        4 => "Fire Protection",
        5 => "Blast Protection",
        6 => "Projectile Protection",
        7 => "Fortune",
        8 => "Silk Touch",
        9 => "Infinity",
        10 => "Mending",
        11 => "Depth Strider",
        12 => "Frost Walker",
        13 => "Riptide",
        14 => "Loyalty",
        15 => "Channeling",
        16 => "Multishot",
        17 => "Piercing",
        18 => "Power",
        19 => "Punch",
        20 => "Flame",
        21 => "Knockback",
        22 => "Fire Aspect",
        23 => "Looting",
        24 => "Sweeping Edge",
        25 => "Efficiency",
        26 => "Unbreaking",
        27 => "Thorns",
        28 => "Respiration",
        29 => "Aqua Affinity",
        30 => "Lure",
        31 => "Luck of the Sea",
        32 => "Impaling",
        33 => "Curse of Vanishing",
        34 => "Curse of Binding",
        35 => "Soul Speed",
        36 => "Swift Sneak",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── are_compatible ────────────────────────────────────────────────

    #[test]
    fn same_id_is_compatible() {
        for id in 0..TOTAL_ENCHANTMENTS as u8 {
            assert!(are_compatible(id, id), "id {id} should be compatible with itself");
        }
    }

    #[test]
    fn damage_group_mutually_exclusive() {
        // sharpness(0), smite(1), bane_of_arthropods(2)
        assert!(!are_compatible(0, 1));
        assert!(!are_compatible(0, 2));
        assert!(!are_compatible(1, 2));
    }

    #[test]
    fn protection_group_mutually_exclusive() {
        let ids = [3u8, 4, 5, 6];
        for (i, &a) in ids.iter().enumerate() {
            for &b in &ids[i + 1..] {
                assert!(
                    !are_compatible(a, b),
                    "{} and {} should be incompatible",
                    enchantment_name(a),
                    enchantment_name(b),
                );
            }
        }
    }

    #[test]
    fn fortune_silk_touch_incompatible() {
        assert!(!are_compatible(7, 8));
        assert!(!are_compatible(8, 7));
    }

    #[test]
    fn infinity_mending_incompatible() {
        assert!(!are_compatible(9, 10));
        assert!(!are_compatible(10, 9));
    }

    #[test]
    fn depth_strider_frost_walker_incompatible() {
        assert!(!are_compatible(11, 12));
        assert!(!are_compatible(12, 11));
    }

    #[test]
    fn riptide_loyalty_incompatible() {
        assert!(!are_compatible(13, 14));
        assert!(!are_compatible(14, 13));
    }

    #[test]
    fn riptide_channeling_incompatible() {
        assert!(!are_compatible(13, 15));
        assert!(!are_compatible(15, 13));
    }

    #[test]
    fn multishot_piercing_incompatible() {
        assert!(!are_compatible(16, 17));
        assert!(!are_compatible(17, 16));
    }

    #[test]
    fn compatible_pair_returns_true() {
        // sharpness(0) + knockback(21) are compatible
        assert!(are_compatible(0, 21));
        // fortune(7) + unbreaking(26) are compatible
        assert!(are_compatible(7, 26));
        // mending(10) + unbreaking(26) are compatible
        assert!(are_compatible(10, 26));
    }

    #[test]
    fn symmetry_holds_for_all_pairs() {
        for a in 0..TOTAL_ENCHANTMENTS as u8 {
            for b in 0..TOTAL_ENCHANTMENTS as u8 {
                assert_eq!(
                    are_compatible(a, b),
                    are_compatible(b, a),
                    "symmetry broken for ({a}, {b})",
                );
            }
        }
    }

    #[test]
    fn out_of_range_ids_are_compatible() {
        assert!(are_compatible(100, 200));
        assert!(are_compatible(0, 255));
    }

    // ── max_enchant_level ─────────────────────────────────────────────

    #[test]
    fn known_max_levels() {
        assert_eq!(max_enchant_level(0), 5);  // sharpness
        assert_eq!(max_enchant_level(3), 4);  // protection
        assert_eq!(max_enchant_level(8), 1);  // silk_touch
        assert_eq!(max_enchant_level(25), 5); // efficiency
        assert_eq!(max_enchant_level(36), 1); // swift_sneak
    }

    #[test]
    fn all_valid_ids_have_positive_max_level() {
        for id in 0..TOTAL_ENCHANTMENTS as u8 {
            assert!(
                max_enchant_level(id) >= 1,
                "id {id} ({}) has max_level 0",
                enchantment_name(id),
            );
        }
    }

    #[test]
    fn unknown_id_returns_zero() {
        assert_eq!(max_enchant_level(37), 0);
        assert_eq!(max_enchant_level(255), 0);
    }

    // ── total_enchantments ────────────────────────────────────────────

    #[test]
    fn total_is_37() {
        assert_eq!(total_enchantments(), 37);
    }

    // ── enchantment_name ──────────────────────────────────────────────

    #[test]
    fn known_names() {
        assert_eq!(enchantment_name(0), "Sharpness");
        assert_eq!(enchantment_name(2), "Bane of Arthropods");
        assert_eq!(enchantment_name(10), "Mending");
        assert_eq!(enchantment_name(33), "Curse of Vanishing");
        assert_eq!(enchantment_name(36), "Swift Sneak");
    }

    #[test]
    fn unknown_id_returns_unknown() {
        assert_eq!(enchantment_name(37), "Unknown");
        assert_eq!(enchantment_name(255), "Unknown");
    }

    #[test]
    fn all_valid_ids_have_non_empty_name() {
        for id in 0..TOTAL_ENCHANTMENTS as u8 {
            let name = enchantment_name(id);
            assert!(!name.is_empty(), "id {id} has empty name");
            assert_ne!(name, "Unknown", "id {id} has Unknown name");
        }
    }

    // ── Cross-checks ──────────────────────────────────────────────────

    #[test]
    fn loyalty_and_channeling_are_compatible() {
        // loyalty(14) and channeling(15) can coexist
        assert!(are_compatible(14, 15));
    }

    #[test]
    fn every_incompatible_pair_is_within_valid_range() {
        for &(a, b) in INCOMPATIBLE_PAIRS {
            assert!(
                (a as usize) < TOTAL_ENCHANTMENTS,
                "pair ({a}, {b}): {a} out of range",
            );
            assert!(
                (b as usize) < TOTAL_ENCHANTMENTS,
                "pair ({a}, {b}): {b} out of range",
            );
        }
    }
}
