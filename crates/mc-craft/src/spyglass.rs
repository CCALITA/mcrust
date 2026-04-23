//! Spyglass zoom item mechanics.
//!
//! Provides spyglass activation/deactivation, smooth zoom transitions,
//! FOV adjustment, vignette overlay alpha, and movement slowdown.

/// Maximum zoom level when the spyglass is fully active.
const MAX_ZOOM: f32 = 10.0;

/// Resting zoom level when the spyglass is inactive.
const MIN_ZOOM: f32 = 1.0;

/// Speed of the zoom transition in units per second.
const TRANSITION_SPEED: f32 = 8.0;

/// Maximum vignette alpha when the spyglass is at full zoom.
const MAX_VIGNETTE_ALPHA: f32 = 0.8;

/// Movement speed multiplier when the spyglass is active.
const ACTIVE_SLOWDOWN: f32 = 0.2;

/// Tracks spyglass activation and zoom state.
#[derive(Debug, Clone)]
pub struct SpyglassState {
    /// Whether the player is currently using the spyglass.
    pub active: bool,
    /// Current zoom multiplier (1.0 = no zoom, 10.0 = max zoom).
    pub zoom_level: f32,
    /// Normalized transition progress (0.0 = fully inactive, 1.0 = fully active).
    pub zoom_transition: f32,
}

impl SpyglassState {
    /// Create a new inactive spyglass state with no zoom.
    pub fn new() -> Self {
        Self {
            active: false,
            zoom_level: MIN_ZOOM,
            zoom_transition: 0.0,
        }
    }

    /// Begin using the spyglass.
    pub fn start_using(&mut self) {
        self.active = true;
    }

    /// Stop using the spyglass.
    pub fn stop_using(&mut self) {
        self.active = false;
    }

    /// Advance the zoom transition by `dt` seconds.
    ///
    /// When active, `zoom_level` lerps toward [`MAX_ZOOM`].
    /// When inactive, `zoom_level` lerps toward [`MIN_ZOOM`].
    /// Transition speed is [`TRANSITION_SPEED`] units per second.
    pub fn tick(&mut self, dt: f32) {
        let target = if self.active { MAX_ZOOM } else { MIN_ZOOM };
        let diff = target - self.zoom_level;
        let step = TRANSITION_SPEED * dt;

        if diff.abs() <= step {
            self.zoom_level = target;
        } else {
            self.zoom_level += step * diff.signum();
        }

        // Keep zoom_transition normalized: 0.0 at MIN_ZOOM, 1.0 at MAX_ZOOM.
        self.zoom_transition =
            ((self.zoom_level - MIN_ZOOM) / (MAX_ZOOM - MIN_ZOOM)).clamp(0.0, 1.0);
    }
}

/// Apply zoom to a base field-of-view angle.
///
/// Returns `base_fov / zoom_level`.
pub fn apply_zoom_fov(base_fov: f32, zoom_level: f32) -> f32 {
    base_fov / zoom_level
}

/// Compute the vignette overlay alpha for a given zoom level.
///
/// Returns 0.0 when zoom is 1.0 (no zoom) and [`MAX_VIGNETTE_ALPHA`] when
/// zoom is at [`MAX_ZOOM`].
pub fn vignette_alpha(zoom_level: f32) -> f32 {
    let t = ((zoom_level - MIN_ZOOM) / (MAX_ZOOM - MIN_ZOOM)).clamp(0.0, 1.0);
    t * MAX_VIGNETTE_ALPHA
}

/// Movement speed multiplier while the spyglass is in use.
///
/// Returns [`ACTIVE_SLOWDOWN`] when active, 1.0 otherwise.
pub fn slowdown_factor(active: bool) -> f32 {
    if active {
        ACTIVE_SLOWDOWN
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    #[test]
    fn new_state_is_inactive() {
        let state = SpyglassState::new();
        assert!(!state.active);
        assert!((state.zoom_level - 1.0).abs() < EPSILON);
        assert!((state.zoom_transition - 0.0).abs() < EPSILON);
    }

    #[test]
    fn start_using_sets_active() {
        let mut state = SpyglassState::new();
        state.start_using();
        assert!(state.active);
    }

    #[test]
    fn stop_using_clears_active() {
        let mut state = SpyglassState::new();
        state.start_using();
        state.stop_using();
        assert!(!state.active);
    }

    #[test]
    fn tick_zooms_in_when_active() {
        let mut state = SpyglassState::new();
        state.start_using();
        state.tick(0.1); // step = 8.0 * 0.1 = 0.8
        assert!(
            (state.zoom_level - 1.8).abs() < EPSILON,
            "Expected 1.8, got {}",
            state.zoom_level
        );
    }

    #[test]
    fn tick_zooms_out_when_inactive() {
        let mut state = SpyglassState::new();
        state.start_using();
        // Zoom in for a bit
        state.tick(0.5); // zoom_level = 1.0 + 8.0 * 0.5 = 5.0
        state.stop_using();
        state.tick(0.25); // step = 2.0, zoom_level = 5.0 - 2.0 = 3.0
        assert!(
            (state.zoom_level - 3.0).abs() < EPSILON,
            "Expected 3.0, got {}",
            state.zoom_level
        );
    }

    #[test]
    fn tick_clamps_at_max_zoom() {
        let mut state = SpyglassState::new();
        state.start_using();
        // Large dt to overshoot: 8.0 * 10.0 = 80.0, but diff is only 9.0
        state.tick(10.0);
        assert!(
            (state.zoom_level - MAX_ZOOM).abs() < EPSILON,
            "Expected max zoom {MAX_ZOOM}, got {}",
            state.zoom_level
        );
        assert!(
            (state.zoom_transition - 1.0).abs() < EPSILON,
            "Expected transition 1.0, got {}",
            state.zoom_transition
        );
    }

    #[test]
    fn tick_clamps_at_min_zoom() {
        let mut state = SpyglassState::new();
        state.start_using();
        state.tick(10.0); // fully zoomed in
        state.stop_using();
        state.tick(10.0); // fully zoomed out
        assert!(
            (state.zoom_level - MIN_ZOOM).abs() < EPSILON,
            "Expected min zoom {MIN_ZOOM}, got {}",
            state.zoom_level
        );
        assert!(
            (state.zoom_transition - 0.0).abs() < EPSILON,
            "Expected transition 0.0, got {}",
            state.zoom_transition
        );
    }

    #[test]
    fn zoom_transition_tracks_zoom_level() {
        let mut state = SpyglassState::new();
        state.start_using();
        // zoom_level after tick: 1.0 + 8.0 * 0.5 = 5.0
        // transition: (5.0 - 1.0) / (10.0 - 1.0) = 4.0/9.0
        state.tick(0.5);
        let expected_transition = 4.0 / 9.0;
        assert!(
            (state.zoom_transition - expected_transition).abs() < EPSILON,
            "Expected transition {expected_transition}, got {}",
            state.zoom_transition
        );
    }

    #[test]
    fn apply_zoom_fov_at_no_zoom() {
        let fov = apply_zoom_fov(70.0, 1.0);
        assert!((fov - 70.0).abs() < EPSILON);
    }

    #[test]
    fn apply_zoom_fov_at_max_zoom() {
        let fov = apply_zoom_fov(70.0, 10.0);
        assert!((fov - 7.0).abs() < EPSILON);
    }

    #[test]
    fn apply_zoom_fov_at_half_zoom() {
        let fov = apply_zoom_fov(90.0, 2.0);
        assert!((fov - 45.0).abs() < EPSILON);
    }

    #[test]
    fn vignette_alpha_at_no_zoom() {
        let alpha = vignette_alpha(1.0);
        assert!(alpha.abs() < EPSILON, "Expected 0.0, got {alpha}");
    }

    #[test]
    fn vignette_alpha_at_max_zoom() {
        let alpha = vignette_alpha(10.0);
        assert!(
            (alpha - MAX_VIGNETTE_ALPHA).abs() < EPSILON,
            "Expected {MAX_VIGNETTE_ALPHA}, got {alpha}"
        );
    }

    #[test]
    fn vignette_alpha_scales_linearly() {
        // At zoom 5.5 (midpoint): t = (5.5 - 1.0) / 9.0 = 0.5
        let alpha = vignette_alpha(5.5);
        let expected = 0.5 * MAX_VIGNETTE_ALPHA;
        assert!(
            (alpha - expected).abs() < EPSILON,
            "Expected {expected}, got {alpha}"
        );
    }

    #[test]
    fn slowdown_when_active() {
        let factor = slowdown_factor(true);
        assert!(
            (factor - ACTIVE_SLOWDOWN).abs() < EPSILON,
            "Expected {ACTIVE_SLOWDOWN}, got {factor}"
        );
    }

    #[test]
    fn no_slowdown_when_inactive() {
        let factor = slowdown_factor(false);
        assert!(
            (factor - 1.0).abs() < EPSILON,
            "Expected 1.0, got {factor}"
        );
    }
}
