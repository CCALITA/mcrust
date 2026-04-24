//! Frost Walker enchantment — freezes water under the player's feet.
//!
//! Frost Walker turns water source blocks into frosted ice within a
//! radius around the player. The ice decays after a short time.
//! The enchantment also prevents magma block damage and does not
//! function in the Nether (dimension 1).

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum enchantment level for Frost Walker.
pub const MAX_FROST_WALKER_LEVEL: u8 = 2;

/// Dimension id for the Overworld.
const OVERWORLD: u8 = 0;

/// Dimension id for the Nether.
const NETHER: u8 = 1;

/// Dimension id for the End.
const END: u8 = 2;

// ---------------------------------------------------------------------------
// Radius
// ---------------------------------------------------------------------------

/// Returns the freeze radius for a given Frost Walker level.
///
/// - Level 0 returns 0 (no freezing).
/// - Level 1 returns 3 (2 + 1).
/// - Level 2 returns 4 (2 + 2).
pub fn frost_walker_radius(level: u8) -> u8 {
    if level == 0 {
        return 0;
    }
    2 + level
}

// ---------------------------------------------------------------------------
// Water freezing positions
// ---------------------------------------------------------------------------

/// Returns all block positions in a circle of [`frost_walker_radius`] at
/// `y = py - 1` centred on `(px, pz)`.
///
/// Positions `(bx, by, bz)` are included when the squared horizontal
/// distance from `(px, pz)` satisfies `dx*dx + dz*dz <= radius*radius`.
pub fn freeze_water_positions(px: i32, py: i32, pz: i32, level: u8) -> Vec<(i32, i32, i32)> {
    let r = frost_walker_radius(level) as i32;
    if r == 0 {
        return Vec::new();
    }
    let by = py - 1;
    let r_sq = r * r;
    let mut positions = Vec::new();
    for dx in -r..=r {
        for dz in -r..=r {
            if dx * dx + dz * dz <= r_sq {
                positions.push((px + dx, by, pz + dz));
            }
        }
    }
    positions
}

// ---------------------------------------------------------------------------
// Frosted ice decay
// ---------------------------------------------------------------------------

/// Returns the base decay time in seconds for frosted ice blocks.
///
/// In vanilla Minecraft frosted ice decays after a short random delay;
/// this returns the base value of 3.0 seconds.
pub fn frosted_ice_decay_time() -> f32 {
    3.0
}

// ---------------------------------------------------------------------------
// Magma damage prevention
// ---------------------------------------------------------------------------

/// Returns `true` — Frost Walker prevents magma block damage.
pub fn prevents_magma_damage() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Dimension check
// ---------------------------------------------------------------------------

/// Returns whether Frost Walker functions in the given dimension.
///
/// - Dimension 0 (Overworld): `true`
/// - Dimension 1 (Nether): `false` — water cannot exist in the Nether.
/// - Dimension 2 (End): `true`
pub fn frost_walker_works_in_dimension(dim: u8) -> bool {
    match dim {
        OVERWORLD | END => true,
        NETHER => false,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- frost_walker_radius -------------------------------------------------

    #[test]
    fn radius_level_zero_returns_zero() {
        assert_eq!(frost_walker_radius(0), 0);
    }

    #[test]
    fn radius_level_one_returns_three() {
        assert_eq!(frost_walker_radius(1), 3);
    }

    #[test]
    fn radius_level_two_returns_four() {
        assert_eq!(frost_walker_radius(2), 4);
    }

    #[test]
    fn radius_level_three_returns_five() {
        // Even though MAX_FROST_WALKER_LEVEL is 2, the function computes 2+level.
        assert_eq!(frost_walker_radius(3), 5);
    }

    // -- freeze_water_positions ----------------------------------------------

    #[test]
    fn freeze_positions_empty_at_level_zero() {
        let positions = freeze_water_positions(0, 64, 0, 0);
        assert!(positions.is_empty());
    }

    #[test]
    fn freeze_positions_all_at_y_minus_one() {
        let positions = freeze_water_positions(10, 64, 20, 1);
        assert!(!positions.is_empty());
        for &(_, by, _) in &positions {
            assert_eq!(by, 63);
        }
    }

    #[test]
    fn freeze_positions_contains_center() {
        let positions = freeze_water_positions(5, 70, 5, 1);
        assert!(positions.contains(&(5, 69, 5)));
    }

    #[test]
    fn freeze_positions_within_radius() {
        let px = 0;
        let pz = 0;
        let level = 2;
        let r = frost_walker_radius(level) as i32;
        let r_sq = r * r;
        let positions = freeze_water_positions(px, 64, pz, level);
        for &(bx, _, bz) in &positions {
            let dx = bx - px;
            let dz = bz - pz;
            assert!(
                dx * dx + dz * dz <= r_sq,
                "position ({bx}, {bz}) is outside radius {r}"
            );
        }
    }

    #[test]
    fn freeze_positions_does_not_include_outside_radius() {
        let px = 0;
        let pz = 0;
        let level = 1;
        let r = frost_walker_radius(level) as i32;
        let positions = freeze_water_positions(px, 64, pz, level);
        // Corner (r, r) should be outside the circle for radius 3.
        assert!(
            !positions.contains(&(r, 63, r)),
            "corner ({r}, 63, {r}) should be outside radius {r}"
        );
    }

    #[test]
    fn freeze_positions_count_level_one() {
        // Radius 3 circle: count positions where dx^2+dz^2 <= 9.
        let positions = freeze_water_positions(0, 64, 0, 1);
        let expected = (-3i32..=3)
            .flat_map(|dx| (-3i32..=3).map(move |dz| (dx, dz)))
            .filter(|&(dx, dz)| dx * dx + dz * dz <= 9)
            .count();
        assert_eq!(positions.len(), expected);
    }

    // -- frosted_ice_decay_time ----------------------------------------------

    #[test]
    fn decay_time_is_three_seconds() {
        assert!((frosted_ice_decay_time() - 3.0).abs() < f32::EPSILON);
    }

    // -- prevents_magma_damage -----------------------------------------------

    #[test]
    fn magma_damage_is_prevented() {
        assert!(prevents_magma_damage());
    }

    // -- frost_walker_works_in_dimension -------------------------------------

    #[test]
    fn works_in_overworld() {
        assert!(frost_walker_works_in_dimension(0));
    }

    #[test]
    fn does_not_work_in_nether() {
        assert!(!frost_walker_works_in_dimension(1));
    }

    #[test]
    fn works_in_end() {
        assert!(frost_walker_works_in_dimension(2));
    }

    #[test]
    fn unknown_dimension_returns_false() {
        assert!(!frost_walker_works_in_dimension(255));
    }

    // -- MAX_FROST_WALKER_LEVEL ----------------------------------------------

    #[test]
    fn max_level_is_two() {
        assert_eq!(MAX_FROST_WALKER_LEVEL, 2);
    }
}
