//! Music transition crossfade.
//!
//! Provides a small state machine that fades the volume of a currently playing
//! track down while fading a new track up over a fixed duration. The audio
//! backend is expected to read [`current_volume`] and [`next_volume`] each
//! frame and apply them to the corresponding track sources.

/// State of an in-progress music crossfade.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossfadeState {
    /// The track currently audible (fading out when [`next_track`] is set).
    pub current_track: Option<u16>,
    /// The track being faded in. `None` when no crossfade is active.
    pub next_track: Option<u16>,
    /// Progress of the fade in the range `[0.0, 1.0]`.
    pub fade_progress: f32,
    /// Total duration of the fade in seconds.
    pub fade_duration: f32,
}

impl CrossfadeState {
    /// Creates an idle crossfade state with no active tracks.
    pub fn new() -> Self {
        Self {
            current_track: None,
            next_track: None,
            fade_progress: 0.0,
            fade_duration: default_fade_duration(),
        }
    }
}

impl Default for CrossfadeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Default crossfade duration in seconds.
pub fn default_fade_duration() -> f32 {
    3.0
}

/// Begin crossfading to `next_track` over `duration` seconds.
///
/// Resets `fade_progress` to `0.0`. The `current_track` is left untouched so
/// the backend continues playing it at decreasing volume until the fade
/// completes via [`tick_crossfade`].
pub fn start_crossfade(state: &mut CrossfadeState, next_track: u16, duration: f32) {
    state.next_track = Some(next_track);
    state.fade_progress = 0.0;
    state.fade_duration = duration;
}

/// Advance the crossfade by `dt` seconds.
///
/// When `fade_progress` reaches `1.0`, `current_track` is replaced with
/// `next_track`, `next_track` is cleared, and `fade_progress` is reset to
/// `0.0`.
pub fn tick_crossfade(state: &mut CrossfadeState, dt: f32) {
    if state.next_track.is_none() {
        return;
    }
    if state.fade_duration <= 0.0 {
        // Degenerate duration: snap to completion.
        state.current_track = state.next_track;
        state.next_track = None;
        state.fade_progress = 0.0;
        return;
    }
    state.fade_progress += dt / state.fade_duration;
    if state.fade_progress >= 1.0 {
        state.current_track = state.next_track;
        state.next_track = None;
        state.fade_progress = 0.0;
    }
}

/// Volume to apply to the currently playing track.
///
/// Returns `1.0 - fade_progress` while crossfading, otherwise `1.0`.
pub fn current_volume(state: &CrossfadeState) -> f32 {
    if is_crossfading(state) {
        1.0 - state.fade_progress
    } else {
        1.0
    }
}

/// Volume to apply to the incoming (next) track.
///
/// Returns `fade_progress` while crossfading, otherwise `0.0`.
pub fn next_volume(state: &CrossfadeState) -> f32 {
    if is_crossfading(state) {
        state.fade_progress
    } else {
        0.0
    }
}

/// Returns true when a crossfade is currently in progress.
pub fn is_crossfading(state: &CrossfadeState) -> bool {
    state.next_track.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_idle() {
        let state = CrossfadeState::new();
        assert_eq!(state.current_track, None);
        assert_eq!(state.next_track, None);
        assert_eq!(state.fade_progress, 0.0);
        assert_eq!(state.fade_duration, default_fade_duration());
        assert!(!is_crossfading(&state));
    }

    #[test]
    fn default_fade_duration_is_three_seconds() {
        assert_eq!(default_fade_duration(), 3.0);
    }

    #[test]
    fn start_crossfade_sets_next_track_and_resets_progress() {
        let mut state = CrossfadeState::new();
        state.fade_progress = 0.7;
        start_crossfade(&mut state, 42, 5.0);
        assert_eq!(state.next_track, Some(42));
        assert_eq!(state.fade_progress, 0.0);
        assert_eq!(state.fade_duration, 5.0);
        assert!(is_crossfading(&state));
    }

    #[test]
    fn tick_crossfade_advances_progress() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(1);
        start_crossfade(&mut state, 2, 4.0);
        tick_crossfade(&mut state, 1.0);
        assert!((state.fade_progress - 0.25).abs() < f32::EPSILON);
        assert_eq!(state.current_track, Some(1));
        assert_eq!(state.next_track, Some(2));
    }

    #[test]
    fn tick_crossfade_completes_swap_at_full_progress() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(1);
        start_crossfade(&mut state, 2, 2.0);
        tick_crossfade(&mut state, 2.0);
        assert_eq!(state.current_track, Some(2));
        assert_eq!(state.next_track, None);
        assert_eq!(state.fade_progress, 0.0);
        assert!(!is_crossfading(&state));
    }

    #[test]
    fn tick_crossfade_no_op_when_idle() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(7);
        tick_crossfade(&mut state, 10.0);
        assert_eq!(state.current_track, Some(7));
        assert_eq!(state.fade_progress, 0.0);
    }

    #[test]
    fn current_volume_reduces_during_fade() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(1);
        start_crossfade(&mut state, 2, 4.0);
        tick_crossfade(&mut state, 1.0);
        assert!((current_volume(&state) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn current_volume_is_full_when_idle() {
        let state = CrossfadeState::new();
        assert_eq!(current_volume(&state), 1.0);
    }

    #[test]
    fn next_volume_increases_during_fade() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(1);
        start_crossfade(&mut state, 2, 4.0);
        tick_crossfade(&mut state, 1.0);
        assert!((next_volume(&state) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn next_volume_is_zero_when_idle() {
        let state = CrossfadeState::new();
        assert_eq!(next_volume(&state), 0.0);
    }

    #[test]
    fn volumes_sum_to_one_during_fade() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(1);
        start_crossfade(&mut state, 2, 10.0);
        for _ in 0..5 {
            tick_crossfade(&mut state, 1.0);
            let total = current_volume(&state) + next_volume(&state);
            assert!((total - 1.0).abs() < 1e-5, "volumes should sum to 1, got {total}");
        }
    }

    #[test]
    fn zero_duration_snaps_to_completion() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(1);
        start_crossfade(&mut state, 9, 0.0);
        tick_crossfade(&mut state, 0.016);
        assert_eq!(state.current_track, Some(9));
        assert_eq!(state.next_track, None);
        assert!(!is_crossfading(&state));
    }

    #[test]
    fn multiple_consecutive_crossfades() {
        let mut state = CrossfadeState::new();
        state.current_track = Some(1);

        start_crossfade(&mut state, 2, 1.0);
        tick_crossfade(&mut state, 1.0);
        assert_eq!(state.current_track, Some(2));

        start_crossfade(&mut state, 3, 1.0);
        tick_crossfade(&mut state, 1.0);
        assert_eq!(state.current_track, Some(3));
        assert!(!is_crossfading(&state));
    }
}
