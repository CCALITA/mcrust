//! First-person hand swing animation state.
//!
//! Models the attack swing arc (0.3s duration) and the idle walk bob applied
//! to the held item in first-person view.

/// Total duration of a full swing, in seconds.
pub const SWING_DURATION: f32 = 0.3;

/// Peak pitch rotation (radians) reached at the middle of a swing.
pub const SWING_PEAK_PITCH: f32 = -1.2;

/// Peak vertical displacement reached at the middle of a swing.
pub const SWING_PEAK_Y: f32 = 0.15;

/// Horizontal (X) pull-back at the start / end of a swing.
pub const SWING_PEAK_X: f32 = 0.1;

/// Amplitude of walking bob on the Y axis.
pub const WALK_BOB_AMPLITUDE_Y: f32 = 0.03;

/// Amplitude of walking bob on the X axis.
pub const WALK_BOB_AMPLITUDE_X: f32 = 0.02;

/// Angular frequency (rad/s) of the walking bob.
pub const WALK_BOB_FREQUENCY: f32 = 6.0;

/// State of a first-person hand swing animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandSwingState {
    /// Normalized animation progress in `[0.0, 1.0]`.
    pub progress: f32,
    /// Whether a swing is currently playing.
    pub active: bool,
    /// Item being swung (numeric id).
    pub item_id: u16,
}

impl HandSwingState {
    /// Create a fresh, inactive swing state.
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            active: false,
            item_id: 0,
        }
    }
}

impl Default for HandSwingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Begin a swing for the given item. Resets progress to 0 and marks the state active.
pub fn start_swing(state: &mut HandSwingState, item_id: u16) {
    state.progress = 0.0;
    state.active = true;
    state.item_id = item_id;
}

/// Advance the swing by `dt` seconds. Deactivates once `progress` reaches 1.0.
pub fn tick_swing(state: &mut HandSwingState, dt: f32) {
    if !state.active {
        return;
    }
    state.progress += dt / SWING_DURATION;
    if state.progress >= 1.0 {
        state.progress = 1.0;
        state.active = false;
    }
}

/// Positional offset of the held item during a swing.
///
/// Produces an attack arc peaking at `progress == 0.5`:
/// - Y follows a sin-based arc reaching `SWING_PEAK_Y` at the midpoint.
/// - X pulls back (negative) at the start and forward (positive) at the end.
pub fn swing_offset(progress: f32) -> [f32; 3] {
    let p = progress.clamp(0.0, 1.0);
    let y = SWING_PEAK_Y * (p * std::f32::consts::PI).sin();
    // Cosine gives +1 at p=0, -1 at p=1. Negate so we get pull-back then forward push.
    let x = -SWING_PEAK_X * (p * std::f32::consts::PI).cos();
    [x, y, 0.0]
}

/// Pitch rotation (radians) of the held item during a swing.
///
/// Starts and ends at 0, reaching `SWING_PEAK_PITCH` at the midpoint.
pub fn swing_rotation(progress: f32) -> f32 {
    let p = progress.clamp(0.0, 1.0);
    SWING_PEAK_PITCH * (p * std::f32::consts::PI).sin()
}

/// Idle walk bob applied to the held item.
///
/// Returns `[0, 0, 0]` when `walking` is false. When walking, produces a
/// figure-eight bob on X/Y driven by `time` (seconds).
pub fn walk_bob_offset(time: f32, walking: bool) -> [f32; 3] {
    if !walking {
        return [0.0, 0.0, 0.0];
    }
    let phase = time * WALK_BOB_FREQUENCY;
    let x = WALK_BOB_AMPLITUDE_X * phase.sin();
    // Vertical bob runs at double frequency (two bobs per stride).
    let y = WALK_BOB_AMPLITUDE_Y * (2.0 * phase).sin().abs();
    [x, y, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected {} ~= {}", a, b);
    }

    #[test]
    fn new_state_is_inactive() {
        let s = HandSwingState::new();
        assert!(!s.active);
        assert_eq!(s.progress, 0.0);
        assert_eq!(s.item_id, 0);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(HandSwingState::default(), HandSwingState::new());
    }

    #[test]
    fn start_swing_activates_state() {
        let mut s = HandSwingState::new();
        s.progress = 0.5; // simulate leftover state
        start_swing(&mut s, 42);
        assert!(s.active);
        assert_eq!(s.progress, 0.0);
        assert_eq!(s.item_id, 42);
    }

    #[test]
    fn tick_noop_when_inactive() {
        let mut s = HandSwingState::new();
        tick_swing(&mut s, 1.0);
        assert!(!s.active);
        assert_eq!(s.progress, 0.0);
    }

    #[test]
    fn tick_advances_progress() {
        let mut s = HandSwingState::new();
        start_swing(&mut s, 1);
        tick_swing(&mut s, SWING_DURATION / 2.0);
        approx(s.progress, 0.5);
        assert!(s.active);
    }

    #[test]
    fn tick_deactivates_at_end() {
        let mut s = HandSwingState::new();
        start_swing(&mut s, 7);
        tick_swing(&mut s, SWING_DURATION);
        assert!(!s.active);
        assert_eq!(s.progress, 1.0);
    }

    #[test]
    fn tick_clamps_overshoot() {
        let mut s = HandSwingState::new();
        start_swing(&mut s, 7);
        tick_swing(&mut s, SWING_DURATION * 10.0);
        assert!(!s.active);
        assert_eq!(s.progress, 1.0);
    }

    #[test]
    fn swing_offset_zero_at_boundaries_y() {
        approx(swing_offset(0.0)[1], 0.0);
        approx(swing_offset(1.0)[1], 0.0);
    }

    #[test]
    fn swing_offset_peaks_at_midpoint() {
        let [_, y, _] = swing_offset(0.5);
        approx(y, SWING_PEAK_Y);
    }

    #[test]
    fn swing_offset_z_always_zero() {
        for i in 0..=10 {
            let p = i as f32 / 10.0;
            assert_eq!(swing_offset(p)[2], 0.0);
        }
    }

    #[test]
    fn swing_offset_clamps_out_of_range() {
        // Negative and >1 values should clamp, not NaN.
        let low = swing_offset(-1.0);
        let high = swing_offset(2.0);
        approx(low[1], 0.0);
        approx(high[1], 0.0);
    }

    #[test]
    fn swing_rotation_zero_at_boundaries() {
        approx(swing_rotation(0.0), 0.0);
        approx(swing_rotation(1.0), 0.0);
    }

    #[test]
    fn swing_rotation_peak_at_midpoint() {
        approx(swing_rotation(0.5), SWING_PEAK_PITCH);
    }

    #[test]
    fn swing_rotation_monotonic_first_half() {
        let a = swing_rotation(0.1);
        let b = swing_rotation(0.3);
        let c = swing_rotation(0.5);
        assert!(a > b && b > c, "pitch should decrease toward peak: {a} {b} {c}");
    }

    #[test]
    fn walk_bob_zero_when_not_walking() {
        assert_eq!(walk_bob_offset(1.23, false), [0.0, 0.0, 0.0]);
        assert_eq!(walk_bob_offset(99.0, false), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn walk_bob_nonzero_when_walking() {
        // At t such that phase = pi/2, sin(phase) = 1 -> x at max amplitude.
        let t = std::f32::consts::FRAC_PI_2 / WALK_BOB_FREQUENCY;
        let [x, y, z] = walk_bob_offset(t, true);
        approx(x, WALK_BOB_AMPLITUDE_X);
        assert!(y >= 0.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn walk_bob_y_non_negative() {
        // |sin| is always >= 0.
        for i in 0..100 {
            let t = i as f32 * 0.05;
            assert!(walk_bob_offset(t, true)[1] >= 0.0);
        }
    }
}
