//! HUD visibility toggle (F1) and cinematic camera support.
//!
//! Mirrors vanilla Minecraft's F1 behavior:
//! - F1 hides every per-frame HUD layer except the persistent chat log and the
//!   player list (Tab) overlay, since both are still useful while taking
//!   screenshots.
//! - Cinematic camera applies low-pass smoothing on yaw input and a slight zoom
//!   factor.

/// Smoothing rate (units per second) used when cinematic camera is enabled.
const CINEMATIC_SMOOTHING_RATE: f32 = 5.0;

/// Zoom multiplier applied while cinematic camera is enabled.
const CINEMATIC_ZOOM: f32 = 0.7;

/// Toggleable visibility flags for the HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudVisibility {
    /// True when the entire HUD is hidden (F1 pressed).
    pub hidden: bool,
    /// True when cinematic camera smoothing is active.
    pub cinematic_camera: bool,
    /// True when the held item / hand should be hidden.
    pub hide_hand: bool,
    /// True when subtitles should be suppressed.
    pub hide_subtitles: bool,
}

impl Default for HudVisibility {
    fn default() -> Self {
        Self {
            hidden: false,
            cinematic_camera: false,
            hide_hand: false,
            hide_subtitles: false,
        }
    }
}

impl HudVisibility {
    /// Flip the master `hidden` flag (called when F1 is pressed).
    pub fn toggle(&mut self) {
        self.hidden = !self.hidden;
    }
}

/// Individual HUD layers that participate in visibility filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HudLayer {
    Crosshair,
    Hotbar,
    HealthHunger,
    XpBar,
    Hand,
    Subtitles,
    ChatLog,
    Boss,
    Scoreboard,
    Tab,
}

/// Decide whether `layer` should be rendered for the given visibility state.
///
/// Rules:
/// - `ChatLog` and `Tab` always render (they remain visible even when F1 hides
///   the rest of the HUD).
/// - Everything else is suppressed when `vis.hidden` is true.
/// - `Hand` is additionally suppressed by `vis.hide_hand`.
/// - `Subtitles` are additionally suppressed by `vis.hide_subtitles`.
pub fn should_render_layer(layer: HudLayer, vis: &HudVisibility) -> bool {
    match layer {
        HudLayer::ChatLog | HudLayer::Tab => true,
        HudLayer::Hand => !vis.hidden && !vis.hide_hand,
        HudLayer::Subtitles => !vis.hidden && !vis.hide_subtitles,
        HudLayer::Crosshair
        | HudLayer::Hotbar
        | HudLayer::HealthHunger
        | HudLayer::XpBar
        | HudLayer::Boss
        | HudLayer::Scoreboard => !vis.hidden,
    }
}

/// Smooth `current_yaw` toward `input_yaw` at [`CINEMATIC_SMOOTHING_RATE`] per
/// second. The result is clamped to never overshoot the input.
pub fn cinematic_smoothing(input_yaw: f32, current_yaw: f32, dt: f32) -> f32 {
    let delta = input_yaw - current_yaw;
    let step = CINEMATIC_SMOOTHING_RATE * dt.max(0.0);
    // Clamp so we don't overshoot when dt is large.
    let factor = step.min(1.0);
    current_yaw + delta * factor
}

/// Zoom factor applied to the camera FOV when cinematic camera is on.
pub fn cinematic_zoom_factor() -> f32 {
    CINEMATIC_ZOOM
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_fully_visible() {
        let vis = HudVisibility::default();
        assert!(!vis.hidden);
        assert!(!vis.cinematic_camera);
        assert!(!vis.hide_hand);
        assert!(!vis.hide_subtitles);
    }

    #[test]
    fn toggle_flips_hidden() {
        let mut vis = HudVisibility::default();
        vis.toggle();
        assert!(vis.hidden);
        vis.toggle();
        assert!(!vis.hidden);
    }

    #[test]
    fn toggle_does_not_affect_other_flags() {
        let mut vis = HudVisibility::default();
        vis.cinematic_camera = true;
        vis.hide_hand = true;
        vis.hide_subtitles = true;
        vis.toggle();
        assert!(vis.hidden);
        assert!(vis.cinematic_camera);
        assert!(vis.hide_hand);
        assert!(vis.hide_subtitles);
    }

    #[test]
    fn all_layers_render_by_default() {
        let vis = HudVisibility::default();
        for layer in [
            HudLayer::Crosshair,
            HudLayer::Hotbar,
            HudLayer::HealthHunger,
            HudLayer::XpBar,
            HudLayer::Hand,
            HudLayer::Subtitles,
            HudLayer::ChatLog,
            HudLayer::Boss,
            HudLayer::Scoreboard,
            HudLayer::Tab,
        ] {
            assert!(
                should_render_layer(layer, &vis),
                "layer {layer:?} should render by default"
            );
        }
    }

    #[test]
    fn hidden_keeps_chat_and_tab_visible() {
        let mut vis = HudVisibility::default();
        vis.hidden = true;

        assert!(should_render_layer(HudLayer::ChatLog, &vis));
        assert!(should_render_layer(HudLayer::Tab, &vis));

        for layer in [
            HudLayer::Crosshair,
            HudLayer::Hotbar,
            HudLayer::HealthHunger,
            HudLayer::XpBar,
            HudLayer::Hand,
            HudLayer::Subtitles,
            HudLayer::Boss,
            HudLayer::Scoreboard,
        ] {
            assert!(
                !should_render_layer(layer, &vis),
                "layer {layer:?} should be hidden when vis.hidden is true"
            );
        }
    }

    #[test]
    fn hide_hand_only_suppresses_hand() {
        let mut vis = HudVisibility::default();
        vis.hide_hand = true;
        assert!(!should_render_layer(HudLayer::Hand, &vis));
        assert!(should_render_layer(HudLayer::Crosshair, &vis));
        assert!(should_render_layer(HudLayer::Subtitles, &vis));
    }

    #[test]
    fn hide_subtitles_only_suppresses_subtitles() {
        let mut vis = HudVisibility::default();
        vis.hide_subtitles = true;
        assert!(!should_render_layer(HudLayer::Subtitles, &vis));
        assert!(should_render_layer(HudLayer::Hand, &vis));
        assert!(should_render_layer(HudLayer::Hotbar, &vis));
    }

    #[test]
    fn cinematic_smoothing_lerps_toward_input() {
        // dt = 0.1, factor = 5.0 * 0.1 = 0.5 → halfway.
        let result = cinematic_smoothing(10.0, 0.0, 0.1);
        assert!(
            (result - 5.0).abs() < 1e-5,
            "expected ~5.0, got {result}"
        );
    }

    #[test]
    fn cinematic_smoothing_clamps_at_large_dt() {
        // dt = 1.0 → factor would be 5.0; clamp to 1.0 → reach input exactly.
        let result = cinematic_smoothing(10.0, 0.0, 1.0);
        assert!((result - 10.0).abs() < 1e-5);
    }

    #[test]
    fn cinematic_smoothing_zero_dt_is_identity() {
        let result = cinematic_smoothing(10.0, 3.0, 0.0);
        assert!((result - 3.0).abs() < 1e-5);
    }

    #[test]
    fn cinematic_smoothing_negative_dt_is_identity() {
        let result = cinematic_smoothing(10.0, 3.0, -0.5);
        assert!((result - 3.0).abs() < 1e-5);
    }

    #[test]
    fn cinematic_smoothing_converges_over_time() {
        let input = 100.0_f32;
        let mut current = 0.0_f32;
        for _ in 0..200 {
            current = cinematic_smoothing(input, current, 1.0 / 60.0);
        }
        assert!(
            (current - input).abs() < 0.5,
            "expected convergence near 100.0, got {current}"
        );
    }

    #[test]
    fn cinematic_zoom_is_less_than_one() {
        let z = cinematic_zoom_factor();
        assert!(z > 0.0 && z < 1.0);
        assert!((z - 0.7).abs() < 1e-6);
    }
}
