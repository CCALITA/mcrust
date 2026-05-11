/// Dedicated observer block logic with mutable tick API and output direction mapping.

/// State of an observer block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverBlockState {
    /// Direction the observer is facing (0..5 for the six cardinal directions).
    pub facing: u8,
    /// Whether the observer output is currently powered.
    pub powered: bool,
    /// Remaining cooldown ticks before the observer can emit again.
    pub cooldown: u32,
}

impl ObserverBlockState {
    /// Creates a new observer facing the given direction, unpowered with no cooldown.
    pub fn new(facing: u8) -> Self {
        Self {
            facing,
            powered: false,
            cooldown: 0,
        }
    }
}

/// Returns `true` if the observed block has changed (old and new differ).
pub fn observer_detect_change(old_block: u16, new_block: u16) -> bool {
    old_block != new_block
}

/// Returns the pulse duration in ticks for an observer (always 2).
pub fn observer_pulse_duration() -> u32 {
    2
}

/// Advances the observer by one tick, returning `true` when the observer emits power
/// this tick.
///
/// When `powered` is set, the observer emits for one tick then enters a cooldown
/// equal to [`observer_pulse_duration`]. During cooldown the observer does not emit.
pub fn tick_observer(state: &mut ObserverBlockState) -> bool {
    if state.cooldown > 0 {
        state.cooldown -= 1;
        state.powered = false;
        return false;
    }

    if state.powered {
        state.powered = false;
        state.cooldown = observer_pulse_duration();
        return true;
    }

    false
}

/// Returns the block-offset direction for the observer's redstone output.
///
/// The output faces opposite to the observer's facing direction:
/// - 0 (down)  → output up    (0, 1, 0)
/// - 1 (up)    → output down  (0, -1, 0)
/// - 2 (north) → output south (0, 0, 1)
/// - 3 (south) → output north (0, 0, -1)
/// - 4 (west)  → output east  (1, 0, 0)
/// - 5 (east)  → output west  (-1, 0, 0)
///
/// Returns `(0, 0, 0)` for invalid facing values.
pub fn observer_output_direction(facing: u8) -> (i32, i32, i32) {
    match facing {
        0 => (0, 1, 0),
        1 => (0, -1, 0),
        2 => (0, 0, 1),
        3 => (0, 0, -1),
        4 => (1, 0, 0),
        5 => (-1, 0, 0),
        _ => (0, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_observer_is_unpowered() {
        let obs = ObserverBlockState::new(2);
        assert_eq!(obs.facing, 2);
        assert!(!obs.powered);
        assert_eq!(obs.cooldown, 0);
    }

    #[test]
    fn detect_change_different_blocks() {
        assert!(observer_detect_change(1, 2));
        assert!(observer_detect_change(0, 100));
    }

    #[test]
    fn detect_change_same_block() {
        assert!(!observer_detect_change(5, 5));
        assert!(!observer_detect_change(0, 0));
    }

    #[test]
    fn pulse_duration_is_two() {
        assert_eq!(observer_pulse_duration(), 2);
    }

    #[test]
    fn tick_emits_then_cooldown() {
        let mut state = ObserverBlockState::new(0);
        state.powered = true;

        // Tick 1: emits power.
        assert!(tick_observer(&mut state));
        assert!(!state.powered);
        assert_eq!(state.cooldown, 2);

        // Tick 2: cooldown, no power.
        assert!(!tick_observer(&mut state));
        assert_eq!(state.cooldown, 1);

        // Tick 3: cooldown ends.
        assert!(!tick_observer(&mut state));
        assert_eq!(state.cooldown, 0);

        // Tick 4: idle.
        assert!(!tick_observer(&mut state));
    }

    #[test]
    fn tick_idle_no_power() {
        let mut state = ObserverBlockState::new(3);
        assert!(!tick_observer(&mut state));
    }

    #[test]
    fn cooldown_ignores_powered_flag() {
        let mut state = ObserverBlockState {
            facing: 0,
            powered: true,
            cooldown: 1,
        };
        assert!(!tick_observer(&mut state));
    }

    #[test]
    fn output_direction_opposite_facing() {
        assert_eq!(observer_output_direction(0), (0, 1, 0));
        assert_eq!(observer_output_direction(1), (0, -1, 0));
        assert_eq!(observer_output_direction(2), (0, 0, 1));
        assert_eq!(observer_output_direction(3), (0, 0, -1));
        assert_eq!(observer_output_direction(4), (1, 0, 0));
        assert_eq!(observer_output_direction(5), (-1, 0, 0));
    }

    #[test]
    fn output_direction_invalid_facing() {
        assert_eq!(observer_output_direction(6), (0, 0, 0));
        assert_eq!(observer_output_direction(255), (0, 0, 0));
    }
}
