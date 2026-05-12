// ---------------------------------------------------------------------------
// Ominous bottle & bad omen mechanics
// ---------------------------------------------------------------------------

/// Maximum bad omen level a player can have.
const BAD_OMEN_MAX: u8 = 5;

/// Default duration in ticks for the bad omen effect (100 minutes = 120000 ticks
/// at 20 tps).
const DEFAULT_DURATION_TICKS: u32 = 120_000;

/// Duration in ticks added per ominous bottle level (1 minute 40 seconds = 2000
/// ticks at 20 tps).
const TICKS_PER_LEVEL: u32 = 2_000;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the bad omen / ominous effect on a player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OminousState {
    pub bad_omen_level: u8,
    pub duration_ticks: u32,
}

impl OminousState {
    /// Create a new `OminousState` with no active bad omen.
    pub fn new() -> Self {
        Self {
            bad_omen_level: 0,
            duration_ticks: 0,
        }
    }
}

impl Default for OminousState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Apply an ominous bottle to the player, granting or stacking bad omen.
///
/// `level` is the bottle's potency (1..=5). The bad omen level is clamped to
/// [`bad_omen_max_level`]. Duration is reset based on the resulting level.
pub fn apply_ominous_bottle(state: &mut OminousState, level: u8) {
    let clamped = level.min(BAD_OMEN_MAX);
    let new_level = (state.bad_omen_level.saturating_add(clamped)).min(BAD_OMEN_MAX);
    state.bad_omen_level = new_level;
    state.duration_ticks = DEFAULT_DURATION_TICKS + TICKS_PER_LEVEL * u32::from(new_level);
}

/// Returns `true` if a raid should trigger when a player with bad omen enters
/// a village boundary.
pub fn trigger_raid_on_village_enter(has_bad_omen: bool) -> bool {
    has_bad_omen
}

/// Returns `true` if an ominous trial spawner should activate when a player
/// with bad omen is nearby.
pub fn ominous_trial_activation(has_bad_omen: bool) -> bool {
    has_bad_omen
}

/// The maximum level bad omen can reach.
pub fn bad_omen_max_level() -> u8 {
    BAD_OMEN_MAX
}

/// Tick the ominous state by `dt` ticks. Returns `true` when the effect has
/// expired (duration reached zero).
///
/// When the effect expires the bad omen level is also reset to zero.
pub fn tick_ominous(state: &mut OminousState, dt: u32) -> bool {
    if state.bad_omen_level == 0 {
        return true;
    }

    state.duration_ticks = state.duration_ticks.saturating_sub(dt);

    if state.duration_ticks == 0 {
        state.bad_omen_level = 0;
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- OminousState::new ----------------------------------------------------

    #[test]
    fn new_state_has_no_bad_omen() {
        let state = OminousState::new();
        assert_eq!(state.bad_omen_level, 0);
        assert_eq!(state.duration_ticks, 0);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(OminousState::default(), OminousState::new());
    }

    // -- apply_ominous_bottle -------------------------------------------------

    #[test]
    fn apply_bottle_sets_level_and_duration() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 1);
        assert_eq!(state.bad_omen_level, 1);
        assert_eq!(state.duration_ticks, DEFAULT_DURATION_TICKS + TICKS_PER_LEVEL);
    }

    #[test]
    fn apply_bottle_stacks_levels() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 2);
        assert_eq!(state.bad_omen_level, 2);

        apply_ominous_bottle(&mut state, 2);
        assert_eq!(state.bad_omen_level, 4);
        assert_eq!(
            state.duration_ticks,
            DEFAULT_DURATION_TICKS + TICKS_PER_LEVEL * 4
        );
    }

    #[test]
    fn apply_bottle_clamps_at_max_level() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 5);
        assert_eq!(state.bad_omen_level, BAD_OMEN_MAX);

        // Applying more should not exceed max.
        apply_ominous_bottle(&mut state, 3);
        assert_eq!(state.bad_omen_level, BAD_OMEN_MAX);
        assert_eq!(
            state.duration_ticks,
            DEFAULT_DURATION_TICKS + TICKS_PER_LEVEL * u32::from(BAD_OMEN_MAX)
        );
    }

    #[test]
    fn apply_bottle_clamps_input_level() {
        let mut state = OminousState::new();
        // A level above max should be treated as max.
        apply_ominous_bottle(&mut state, 10);
        assert_eq!(state.bad_omen_level, BAD_OMEN_MAX);
    }

    #[test]
    fn apply_bottle_zero_level_is_noop() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 0);
        assert_eq!(state.bad_omen_level, 0);
        assert_eq!(state.duration_ticks, DEFAULT_DURATION_TICKS);
    }

    // -- trigger_raid_on_village_enter ----------------------------------------

    #[test]
    fn raid_triggers_with_bad_omen() {
        assert!(trigger_raid_on_village_enter(true));
    }

    #[test]
    fn raid_does_not_trigger_without_bad_omen() {
        assert!(!trigger_raid_on_village_enter(false));
    }

    // -- ominous_trial_activation ---------------------------------------------

    #[test]
    fn trial_activates_with_bad_omen() {
        assert!(ominous_trial_activation(true));
    }

    #[test]
    fn trial_does_not_activate_without_bad_omen() {
        assert!(!ominous_trial_activation(false));
    }

    // -- bad_omen_max_level ---------------------------------------------------

    #[test]
    fn max_level_is_five() {
        assert_eq!(bad_omen_max_level(), 5);
    }

    // -- tick_ominous ---------------------------------------------------------

    #[test]
    fn tick_decrements_duration() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 1);
        let initial = state.duration_ticks;

        let expired = tick_ominous(&mut state, 100);
        assert!(!expired);
        assert_eq!(state.duration_ticks, initial - 100);
        assert_eq!(state.bad_omen_level, 1);
    }

    #[test]
    fn tick_expires_and_resets_level() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 1);
        let total = state.duration_ticks;

        let expired = tick_ominous(&mut state, total);
        assert!(expired);
        assert_eq!(state.bad_omen_level, 0);
        assert_eq!(state.duration_ticks, 0);
    }

    #[test]
    fn tick_returns_true_when_no_bad_omen() {
        let mut state = OminousState::new();
        assert!(tick_ominous(&mut state, 100));
    }

    #[test]
    fn tick_saturates_at_zero() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 1);

        // Tick by more than the remaining duration.
        let expired = tick_ominous(&mut state, u32::MAX);
        assert!(expired);
        assert_eq!(state.duration_ticks, 0);
        assert_eq!(state.bad_omen_level, 0);
    }

    #[test]
    fn tick_partial_then_expire() {
        let mut state = OminousState::new();
        apply_ominous_bottle(&mut state, 2);
        let total = state.duration_ticks;

        // Tick half.
        let expired = tick_ominous(&mut state, total / 2);
        assert!(!expired);
        assert_eq!(state.bad_omen_level, 2);

        // Tick the rest.
        let expired = tick_ominous(&mut state, total - total / 2);
        assert!(expired);
        assert_eq!(state.bad_omen_level, 0);
    }

    // -- integration ----------------------------------------------------------

    #[test]
    fn full_lifecycle() {
        let mut state = OminousState::new();

        // No bad omen initially.
        assert!(!trigger_raid_on_village_enter(state.bad_omen_level > 0));
        assert!(!ominous_trial_activation(state.bad_omen_level > 0));

        // Drink an ominous bottle.
        apply_ominous_bottle(&mut state, 1);
        assert_eq!(state.bad_omen_level, 1);
        assert!(trigger_raid_on_village_enter(state.bad_omen_level > 0));
        assert!(ominous_trial_activation(state.bad_omen_level > 0));

        // Stack to max.
        for _ in 0..10 {
            apply_ominous_bottle(&mut state, 1);
        }
        assert_eq!(state.bad_omen_level, bad_omen_max_level());

        // Tick until expired.
        let total = state.duration_ticks;
        let expired = tick_ominous(&mut state, total);
        assert!(expired);
        assert_eq!(state.bad_omen_level, 0);
        assert!(!trigger_raid_on_village_enter(state.bad_omen_level > 0));
    }
}
