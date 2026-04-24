//! Scaffolding climbing system — climb velocity, placement rules, and collapse chain.
//!
//! Scaffolding blocks allow players to climb vertically (jump to ascend, sneak to
//! descend). When the base block supporting a horizontal run of scaffolding is
//! broken, all horizontally connected scaffolding within [`MAX_HORIZONTAL_DISTANCE`]
//! collapses. Scaffolding may only be placed on another scaffolding block or on a
//! solid block below.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum horizontal flood-fill distance for scaffolding collapse chains.
///
/// Matches vanilla Minecraft's limit of 6 horizontally-connected scaffolding
/// blocks before the chain stops propagating support.
pub const MAX_HORIZONTAL_DISTANCE: u8 = 6;

/// Upward climb velocity when jumping on scaffolding (blocks/tick).
pub const CLIMB_UP_VELOCITY: f32 = 0.4;

/// Downward climb velocity when sneaking on scaffolding (blocks/tick).
pub const CLIMB_DOWN_VELOCITY: f32 = -0.4;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Per-entity scaffolding climbing state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaffoldingState {
    /// Whether the entity is currently occupying a scaffolding block.
    pub on_scaffolding: bool,
    /// Current vertical climb speed in blocks/tick (positive = up, negative = down).
    pub climb_speed: f32,
}

impl ScaffoldingState {
    /// Create a default state — not on scaffolding, zero climb speed.
    pub fn new() -> Self {
        Self {
            on_scaffolding: false,
            climb_speed: 0.0,
        }
    }
}

impl Default for ScaffoldingState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Climbing rules
// ---------------------------------------------------------------------------

/// Whether a player may climb the scaffolding they are currently in.
///
/// Climbing requires being inside a scaffolding block (`at_pos`) and not
/// sneaking — sneaking prevents vertical movement to allow horizontal traversal.
pub fn can_climb_scaffolding(at_pos: bool, sneaking: bool) -> bool {
    at_pos && !sneaking
}

/// Compute the vertical climb velocity for a player on scaffolding.
///
/// - `jump = true`  → [`CLIMB_UP_VELOCITY`]  (ascend)
/// - `sneak = true` → [`CLIMB_DOWN_VELOCITY`] (descend)
/// - otherwise       → `0.0` (stationary, hovering on the scaffold)
///
/// `jump` takes precedence if both are set.
pub fn scaffolding_climb_velocity(jump: bool, sneak: bool) -> f32 {
    if jump {
        CLIMB_UP_VELOCITY
    } else if sneak {
        CLIMB_DOWN_VELOCITY
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Placement rules
// ---------------------------------------------------------------------------

/// Whether scaffolding may be placed at a candidate position.
///
/// Vanilla rules: a scaffolding block is supported if either an adjacent
/// scaffolding block provides lateral support, or a solid block exists below.
pub fn scaffolding_can_be_placed_on(neighbor_is_scaffolding: bool, has_solid_below: bool) -> bool {
    neighbor_is_scaffolding || has_solid_below
}

// ---------------------------------------------------------------------------
// Player position check
// ---------------------------------------------------------------------------

/// Whether a player's feet are positioned on the top of a scaffolding block.
///
/// Players stand on top when their Y coordinate is at-or-above the block's top.
/// Matches with a small tolerance to account for floating-point drift.
pub fn is_player_on_scaffolding(player_y: f32, scaffolding_top_y: f32) -> bool {
    player_y >= scaffolding_top_y - 0.01 && player_y <= scaffolding_top_y + 1.0
}

// ---------------------------------------------------------------------------
// Collapse chain
// ---------------------------------------------------------------------------

/// Flood-fill the horizontally connected scaffolding starting at `start`.
///
/// Explores the ±X and ±Z neighbors (horizontal only) and collects every
/// scaffolding block within Manhattan distance `max_distance` from the start.
/// When the base scaffolding is broken, every returned position collapses.
///
/// `is_scaffolding` is a callback describing the world: `(x, y, z) -> bool`.
/// The start block is always included if it is scaffolding.
pub fn scaffolding_collapse_chain(
    start: (i32, i32, i32),
    is_scaffolding: &impl Fn(i32, i32, i32) -> bool,
    max_distance: u8,
) -> Vec<(i32, i32, i32)> {
    if !is_scaffolding(start.0, start.1, start.2) {
        return Vec::new();
    }

    let max = max_distance as i32;
    let mut visited: Vec<(i32, i32, i32)> = Vec::new();
    let mut frontier: Vec<(i32, i32, i32, i32)> = vec![(start.0, start.1, start.2, 0)];

    while let Some((x, y, z, dist)) = frontier.pop() {
        if visited.contains(&(x, y, z)) {
            continue;
        }
        visited.push((x, y, z));

        if dist >= max {
            continue;
        }

        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x + dx;
            let nz = z + dz;
            if visited.contains(&(nx, y, nz)) {
                continue;
            }
            if is_scaffolding(nx, y, nz) {
                frontier.push((nx, y, nz, dist + 1));
            }
        }
    }

    visited
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffolding_state_defaults() {
        let s = ScaffoldingState::new();
        assert!(!s.on_scaffolding);
        assert_eq!(s.climb_speed, 0.0);
        assert_eq!(s, ScaffoldingState::default());
    }

    #[test]
    fn can_climb_requires_position_and_not_sneaking() {
        assert!(can_climb_scaffolding(true, false));
        assert!(!can_climb_scaffolding(true, true));
        assert!(!can_climb_scaffolding(false, false));
        assert!(!can_climb_scaffolding(false, true));
    }

    #[test]
    fn climb_velocity_jump_ascends() {
        assert_eq!(scaffolding_climb_velocity(true, false), CLIMB_UP_VELOCITY);
        assert!((scaffolding_climb_velocity(true, false) - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn climb_velocity_sneak_descends() {
        assert_eq!(scaffolding_climb_velocity(false, true), CLIMB_DOWN_VELOCITY);
        assert!((scaffolding_climb_velocity(false, true) + 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn climb_velocity_idle_is_zero() {
        assert_eq!(scaffolding_climb_velocity(false, false), 0.0);
    }

    #[test]
    fn climb_velocity_jump_takes_precedence_over_sneak() {
        // When both inputs are active, jump wins.
        assert_eq!(scaffolding_climb_velocity(true, true), CLIMB_UP_VELOCITY);
    }

    #[test]
    fn placement_requires_support() {
        assert!(scaffolding_can_be_placed_on(true, false));
        assert!(scaffolding_can_be_placed_on(false, true));
        assert!(scaffolding_can_be_placed_on(true, true));
        assert!(!scaffolding_can_be_placed_on(false, false));
    }

    #[test]
    fn player_on_scaffolding_within_range() {
        // Player standing at the top of a scaffolding block at y=10.
        assert!(is_player_on_scaffolding(10.0, 10.0));
        assert!(is_player_on_scaffolding(10.5, 10.0));
        assert!(is_player_on_scaffolding(11.0, 10.0));
    }

    #[test]
    fn player_not_on_scaffolding_when_outside_range() {
        assert!(!is_player_on_scaffolding(9.0, 10.0));
        assert!(!is_player_on_scaffolding(12.0, 10.0));
    }

    #[test]
    fn collapse_chain_returns_empty_when_start_not_scaffolding() {
        let result = scaffolding_collapse_chain((0, 0, 0), &|_, _, _| false, MAX_HORIZONTAL_DISTANCE);
        assert!(result.is_empty());
    }

    #[test]
    fn collapse_chain_single_block() {
        let is_scaf = |x: i32, y: i32, z: i32| x == 0 && y == 0 && z == 0;
        let result = scaffolding_collapse_chain((0, 0, 0), &is_scaf, MAX_HORIZONTAL_DISTANCE);
        assert_eq!(result, vec![(0, 0, 0)]);
    }

    #[test]
    fn collapse_chain_propagates_horizontally() {
        // Horizontal row of 4 scaffolding at y=0: x = 0, 1, 2, 3.
        let is_scaf =
            |x: i32, y: i32, z: i32| y == 0 && z == 0 && (0..=3).contains(&x);
        let result = scaffolding_collapse_chain((0, 0, 0), &is_scaf, MAX_HORIZONTAL_DISTANCE);
        assert_eq!(result.len(), 4);
        for x in 0..=3 {
            assert!(result.contains(&(x, 0, 0)), "missing ({x}, 0, 0)");
        }
    }

    #[test]
    fn collapse_chain_respects_max_distance() {
        // 10 blocks in a row; max distance of 2 should only return 3 blocks
        // (start + 2 neighbors).
        let is_scaf = |x: i32, y: i32, z: i32| y == 0 && z == 0 && (0..10).contains(&x);
        let result = scaffolding_collapse_chain((0, 0, 0), &is_scaf, 2);
        assert_eq!(result.len(), 3);
        for x in 0..=2 {
            assert!(result.contains(&(x, 0, 0)));
        }
    }

    #[test]
    fn collapse_chain_does_not_propagate_vertically() {
        // Vertical column at x=0, z=0, y=0..3. Only the start block at y=0 should
        // be included because the flood-fill is horizontal-only.
        let is_scaf = |x: i32, y: i32, z: i32| x == 0 && z == 0 && (0..3).contains(&y);
        let result = scaffolding_collapse_chain((0, 0, 0), &is_scaf, MAX_HORIZONTAL_DISTANCE);
        assert_eq!(result, vec![(0, 0, 0)]);
    }

    #[test]
    fn collapse_chain_branches_in_all_horizontal_directions() {
        // Plus sign at y=0: center (0,0,0) plus (±1, 0, 0) and (0, 0, ±1).
        let is_scaf = |x: i32, y: i32, z: i32| {
            y == 0
                && ((x == 0 && z.abs() <= 1) || (z == 0 && x.abs() <= 1))
        };
        let result = scaffolding_collapse_chain((0, 0, 0), &is_scaf, MAX_HORIZONTAL_DISTANCE);
        assert_eq!(result.len(), 5);
        for pos in &[(0, 0, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)] {
            assert!(result.contains(pos), "missing {pos:?}");
        }
    }

    #[test]
    fn max_horizontal_distance_is_six() {
        assert_eq!(MAX_HORIZONTAL_DISTANCE, 6);
    }
}
