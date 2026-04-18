use mc_core::block::BlockId;

/// State of an observer block that detects block changes in front of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverState {
    /// Direction the observer is facing (0..5 for the six cardinal directions).
    pub facing: u8,
    /// Whether the observer output is currently powered.
    pub powered: bool,
    /// Remaining cooldown ticks before the observer can emit again.
    pub cooldown: u8,
}

impl ObserverState {
    /// Creates a new observer facing the given direction.
    pub fn new(facing: u8) -> Self {
        Self {
            facing,
            powered: false,
            cooldown: 0,
        }
    }
}

/// Returns `true` if the block in front of the observer has changed.
///
/// Compares the old and new block ids; any difference triggers detection.
pub fn observer_check(old: BlockId, new: BlockId) -> bool {
    old != new
}

/// Advances the observer by one tick and returns the current power output (0 or 15).
///
/// Pulse behaviour:
/// - When `powered` transitions to `true`, the observer emits power level 15 for
///   exactly 1 tick, then enters a 2-tick cooldown.
/// - During cooldown the output is 0 and the observer ignores further changes.
pub fn observer_tick(state: ObserverState) -> (ObserverState, u8) {
    if state.cooldown > 0 {
        let next = ObserverState {
            cooldown: state.cooldown - 1,
            powered: false,
            ..state
        };
        return (next, 0);
    }

    if state.powered {
        // Emit 1-tick pulse then enter 2-tick cooldown.
        let next = ObserverState {
            powered: false,
            cooldown: 2,
            ..state
        };
        return (next, 15);
    }

    (state, 0)
}

/// Configuration for a daylight detector block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaylightDetector {
    /// When `true` the detector outputs the inverted signal.
    pub inverted: bool,
}

/// Returns the redstone signal strength (0..=15) for a daylight detector
/// given the current `time` (0.0 = sunrise, 0.5 = noon, 1.0 = next sunrise)
/// and whether the detector is inverted.
///
/// Normal mode follows a sine curve: 15 at noon, 0 at midnight.
/// Inverted mode outputs `15 - normal`.
pub fn daylight_signal(time: f32, inverted: bool) -> u8 {
    // Map time to a sine wave: sin(time * pi) gives 0 at 0/1 and 1 at 0.5.
    let raw = (time * std::f32::consts::PI).sin().max(0.0);
    let normal = (raw * 15.0).round() as u8;

    if inverted { 15 - normal } else { normal }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Observer: block change detection ----

    #[test]
    fn observer_detects_block_change() {
        assert!(observer_check(BlockId::Stone, BlockId::Air));
        assert!(observer_check(BlockId::Dirt, BlockId::GrassBlock));
    }

    #[test]
    fn observer_ignores_same_block() {
        assert!(!observer_check(BlockId::Stone, BlockId::Stone));
        assert!(!observer_check(BlockId::Air, BlockId::Air));
    }

    // ---- Observer: pulse duration ----

    #[test]
    fn observer_emits_one_tick_pulse() {
        let state = ObserverState {
            facing: 0,
            powered: true,
            cooldown: 0,
        };

        // Tick 1: pulse fires (power=15), enters cooldown.
        let (state, power) = observer_tick(state);
        assert_eq!(power, 15);
        assert!(!state.powered);
        assert_eq!(state.cooldown, 2);

        // Tick 2: cooldown, power=0.
        let (state, power) = observer_tick(state);
        assert_eq!(power, 0);
        assert_eq!(state.cooldown, 1);

        // Tick 3: cooldown ends, power=0.
        let (state, power) = observer_tick(state);
        assert_eq!(power, 0);
        assert_eq!(state.cooldown, 0);

        // Tick 4: idle, power=0.
        let (_, power) = observer_tick(state);
        assert_eq!(power, 0);
    }

    #[test]
    fn observer_idle_emits_no_power() {
        let state = ObserverState::new(3);
        let (_, power) = observer_tick(state);
        assert_eq!(power, 0);
    }

    #[test]
    fn observer_in_cooldown_ignores_trigger() {
        let state = ObserverState {
            facing: 0,
            powered: true,
            cooldown: 1,
        };
        // Cooldown takes priority over powered flag.
        let (_, power) = observer_tick(state);
        assert_eq!(power, 0);
    }

    // ---- Daylight detector: signal curve at key times ----

    #[test]
    fn daylight_signal_at_noon_is_max() {
        assert_eq!(daylight_signal(0.5, false), 15);
    }

    #[test]
    fn daylight_signal_at_midnight_is_zero() {
        // Midnight maps to time=0.0 and time=1.0.
        assert_eq!(daylight_signal(0.0, false), 0);
        assert_eq!(daylight_signal(1.0, false), 0);
    }

    #[test]
    fn daylight_signal_at_quarter_day() {
        // sin(0.25 * pi) ≈ 0.707 → 0.707 * 15 ≈ 10.6 → rounds to 11
        let signal = daylight_signal(0.25, false);
        assert_eq!(signal, 11);
    }

    #[test]
    fn daylight_signal_inverted_at_noon() {
        // Normal=15 → inverted=0
        assert_eq!(daylight_signal(0.5, true), 0);
    }

    #[test]
    fn daylight_signal_inverted_at_midnight() {
        // Normal=0 → inverted=15
        assert_eq!(daylight_signal(0.0, true), 15);
        assert_eq!(daylight_signal(1.0, true), 15);
    }

    #[test]
    fn daylight_signal_inverted_at_quarter_day() {
        let normal = daylight_signal(0.25, false);
        let inv = daylight_signal(0.25, true);
        assert_eq!(normal + inv, 15);
    }

    #[test]
    fn daylight_signal_symmetric_around_noon() {
        let before = daylight_signal(0.3, false);
        let after = daylight_signal(0.7, false);
        assert_eq!(before, after);
    }

    #[test]
    fn daylight_signal_decreases_toward_dusk() {
        // At time 0.75 (dusk) the signal is lower than at noon but still positive.
        let dusk = daylight_signal(0.75, false);
        assert!(dusk > 0 && dusk < 15);
    }

    #[test]
    fn daylight_signal_range_is_bounded() {
        // Signal should always be 0..=15 for any time in [0, 1].
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let s = daylight_signal(t, false);
            assert!(s <= 15, "signal {} at time {} exceeds 15", s, t);
        }
    }
}
