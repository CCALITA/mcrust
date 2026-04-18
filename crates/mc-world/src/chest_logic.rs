//! Chest interaction logic — double-chest detection, open/close state,
//! lid animation, blockage checks, and trapped-chest redstone signal.

/// Cardinal neighbor offsets: north, south, east, west.
const CARDINAL_OFFSETS: [(i32, i32, i32); 4] = [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)];

/// Checks the four cardinal neighbors of `pos` for an adjacent chest,
/// returning the first neighboring position that satisfies `is_chest`.
///
/// In Minecraft, a double chest forms when two chests are placed adjacent
/// along a cardinal axis (no diagonal, no vertical).
pub fn detect_double_chest(
    pos: (i32, i32, i32),
    is_chest: &dyn Fn(i32, i32, i32) -> bool,
) -> Option<(i32, i32, i32)> {
    for (dx, dy, dz) in CARDINAL_OFFSETS {
        let nx = pos.0 + dx;
        let ny = pos.1 + dy;
        let nz = pos.2 + dz;
        if is_chest(nx, ny, nz) {
            return Some((nx, ny, nz));
        }
    }
    None
}

/// Runtime state of a chest's open/close lid animation.
#[derive(Debug, Clone, PartialEq)]
pub struct ChestOpenState {
    /// Number of players currently viewing this chest.
    pub viewers: u8,
    /// Lid animation progress in `[0.0, 1.0]`.
    /// `0.0` = fully closed, `1.0` = fully open.
    pub open_progress: f32,
}

impl ChestOpenState {
    /// Create a new, closed chest state with zero viewers.
    pub fn new() -> Self {
        Self {
            viewers: 0,
            open_progress: 0.0,
        }
    }
}

impl Default for ChestOpenState {
    fn default() -> Self {
        Self::new()
    }
}

/// Increment the viewer count (a player opened the chest).
/// Returns a new state with the updated viewer count.
pub fn open_chest(state: &ChestOpenState) -> ChestOpenState {
    ChestOpenState {
        viewers: state.viewers.saturating_add(1),
        open_progress: state.open_progress,
    }
}

/// Decrement the viewer count (a player closed the chest).
/// Returns a new state with the updated viewer count (floors at 0).
pub fn close_chest(state: &ChestOpenState) -> ChestOpenState {
    ChestOpenState {
        viewers: state.viewers.saturating_sub(1),
        open_progress: state.open_progress,
    }
}

/// Animation speed per second — the lid takes 0.5 s to fully open or close.
const ANIMATION_SPEED: f32 = 2.0;

/// Advance the lid animation toward its target:
/// - `1.0` when `viewers > 0` (opening)
/// - `0.0` when `viewers == 0` (closing)
///
/// Returns a new state with the updated `open_progress`, clamped to `[0.0, 1.0]`.
pub fn tick_animation(state: &ChestOpenState, dt: f32) -> ChestOpenState {
    let target = if state.viewers > 0 { 1.0 } else { 0.0 };
    let delta = ANIMATION_SPEED * dt;
    let new_progress = if state.open_progress < target {
        (state.open_progress + delta).min(target)
    } else {
        (state.open_progress - delta).max(target)
    };
    ChestOpenState {
        viewers: state.viewers,
        open_progress: new_progress,
    }
}

/// Returns `true` if the chest at `pos` is blocked from opening
/// (i.e. there is a solid block directly above it).
pub fn is_chest_blocked(_pos: (i32, i32, i32), is_solid_above: bool) -> bool {
    is_solid_above
}

/// Computes the redstone signal strength emitted by a trapped chest.
/// The signal is `min(15, viewers)`.
pub fn trapped_chest_signal(viewers: u8) -> u8 {
    viewers.min(15)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- detect_double_chest --------------------------------------------------

    #[test]
    fn detects_neighbor_to_the_east() {
        let pos = (0, 0, 0);
        let neighbor = detect_double_chest(pos, &|x, y, z| (x, y, z) == (1, 0, 0));
        assert_eq!(neighbor, Some((1, 0, 0)));
    }

    #[test]
    fn detects_neighbor_to_the_west() {
        let pos = (0, 0, 0);
        let neighbor = detect_double_chest(pos, &|x, y, z| (x, y, z) == (-1, 0, 0));
        assert_eq!(neighbor, Some((-1, 0, 0)));
    }

    #[test]
    fn detects_neighbor_to_the_south() {
        let pos = (0, 0, 0);
        let neighbor = detect_double_chest(pos, &|x, y, z| (x, y, z) == (0, 0, 1));
        assert_eq!(neighbor, Some((0, 0, 1)));
    }

    #[test]
    fn detects_neighbor_to_the_north() {
        let pos = (0, 0, 0);
        let neighbor = detect_double_chest(pos, &|x, y, z| (x, y, z) == (0, 0, -1));
        assert_eq!(neighbor, Some((0, 0, -1)));
    }

    #[test]
    fn returns_none_when_no_adjacent_chest() {
        let pos = (5, 10, 5);
        let neighbor = detect_double_chest(pos, &|_, _, _| false);
        assert_eq!(neighbor, None);
    }

    #[test]
    fn ignores_diagonal_neighbors() {
        let pos = (0, 0, 0);
        // Only diagonal positions have chests
        let neighbor = detect_double_chest(pos, &|x, _y, z| x.abs() == 1 && z.abs() == 1);
        assert_eq!(neighbor, None);
    }

    #[test]
    fn ignores_vertical_neighbors() {
        let pos = (0, 0, 0);
        let neighbor = detect_double_chest(pos, &|x, y, z| (x, y, z) == (0, 1, 0));
        assert_eq!(neighbor, None);
    }

    #[test]
    fn returns_first_matching_cardinal_neighbor() {
        let pos = (0, 0, 0);
        // Two adjacent chests — should return the first one found in iteration order
        let neighbor = detect_double_chest(pos, &|x, y, z| {
            (x, y, z) == (1, 0, 0) || (x, y, z) == (-1, 0, 0)
        });
        assert_eq!(neighbor, Some((1, 0, 0)));
    }

    // -- is_chest_blocked -----------------------------------------------------

    #[test]
    fn blocked_when_solid_above() {
        assert!(is_chest_blocked((0, 0, 0), true));
    }

    #[test]
    fn not_blocked_when_air_above() {
        assert!(!is_chest_blocked((0, 0, 0), false));
    }

    // -- trapped_chest_signal -------------------------------------------------

    #[test]
    fn signal_zero_with_no_viewers() {
        assert_eq!(trapped_chest_signal(0), 0);
    }

    #[test]
    fn signal_equals_viewers_up_to_15() {
        for v in 0..=15 {
            assert_eq!(trapped_chest_signal(v), v);
        }
    }

    #[test]
    fn signal_clamped_to_15() {
        assert_eq!(trapped_chest_signal(16), 15);
        assert_eq!(trapped_chest_signal(255), 15);
    }

    // -- open / close ---------------------------------------------------------

    #[test]
    fn open_chest_increments_viewers() {
        let state = ChestOpenState::new();
        let opened = open_chest(&state);
        assert_eq!(opened.viewers, 1);
    }

    #[test]
    fn close_chest_decrements_viewers() {
        let state = ChestOpenState {
            viewers: 2,
            open_progress: 0.5,
        };
        let closed = close_chest(&state);
        assert_eq!(closed.viewers, 1);
    }

    #[test]
    fn close_chest_does_not_underflow() {
        let state = ChestOpenState::new();
        let closed = close_chest(&state);
        assert_eq!(closed.viewers, 0);
    }

    #[test]
    fn open_chest_does_not_overflow() {
        let state = ChestOpenState {
            viewers: 255,
            open_progress: 1.0,
        };
        let opened = open_chest(&state);
        assert_eq!(opened.viewers, 255);
    }

    // -- tick_animation -------------------------------------------------------

    #[test]
    fn animation_opens_when_viewers_present() {
        let state = ChestOpenState {
            viewers: 1,
            open_progress: 0.0,
        };
        let next = tick_animation(&state, 0.1);
        assert!(
            next.open_progress > 0.0,
            "progress should increase toward 1.0"
        );
    }

    #[test]
    fn animation_closes_when_no_viewers() {
        let state = ChestOpenState {
            viewers: 0,
            open_progress: 0.8,
        };
        let next = tick_animation(&state, 0.1);
        assert!(
            next.open_progress < 0.8,
            "progress should decrease toward 0.0"
        );
    }

    #[test]
    fn animation_clamps_to_one() {
        let state = ChestOpenState {
            viewers: 1,
            open_progress: 0.95,
        };
        let next = tick_animation(&state, 10.0); // large dt
        assert!(
            (next.open_progress - 1.0).abs() < f32::EPSILON,
            "progress should clamp to 1.0"
        );
    }

    #[test]
    fn animation_clamps_to_zero() {
        let state = ChestOpenState {
            viewers: 0,
            open_progress: 0.05,
        };
        let next = tick_animation(&state, 10.0); // large dt
        assert!(
            next.open_progress.abs() < f32::EPSILON,
            "progress should clamp to 0.0"
        );
    }

    #[test]
    fn animation_stays_at_target_when_already_there() {
        let open = ChestOpenState {
            viewers: 1,
            open_progress: 1.0,
        };
        let next = tick_animation(&open, 0.5);
        assert!(
            (next.open_progress - 1.0).abs() < f32::EPSILON,
            "should stay at 1.0"
        );

        let closed = ChestOpenState {
            viewers: 0,
            open_progress: 0.0,
        };
        let next = tick_animation(&closed, 0.5);
        assert!(
            next.open_progress.abs() < f32::EPSILON,
            "should stay at 0.0"
        );
    }

    #[test]
    fn animation_preserves_viewer_count() {
        let state = ChestOpenState {
            viewers: 3,
            open_progress: 0.5,
        };
        let next = tick_animation(&state, 0.1);
        assert_eq!(next.viewers, 3);
    }
}
