//! Damage indicator (red flash) overlay system.
//!
//! When the player takes damage, the screen flashes red and a directional
//! indicator shows the source direction. [`DamageIndicator`] tracks the flash
//! state and fades it out over time, while the free functions compute overlay
//! colors, vignette rectangles, and directional indicator positions.

/// Tracks the current state of the damage flash overlay.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageIndicator {
    /// Current red overlay opacity (0.0 = invisible, up to 0.5).
    pub red_alpha: f32,
    /// Yaw direction the damage came from (radians).
    pub hurt_direction: f32,
    /// Remaining time for the flash effect (seconds).
    pub timer: f32,
}

impl DamageIndicator {
    /// Create a new indicator with no active flash (all zeros).
    pub fn new() -> Self {
        Self {
            red_alpha: 0.0,
            hurt_direction: 0.0,
            timer: 0.0,
        }
    }

    /// Trigger a damage flash.
    ///
    /// - `damage` controls intensity: `red_alpha` is set to `(damage / 20.0).min(0.5)`.
    /// - `hurt_yaw` records the direction the damage came from.
    /// - The flash lasts 0.5 seconds before fully fading out.
    pub fn trigger(&mut self, damage: f32, hurt_yaw: f32) {
        self.red_alpha = (damage / 20.0).min(0.5);
        self.hurt_direction = hurt_yaw;
        self.timer = 0.5;
    }

    /// Advance the indicator by `dt` seconds, linearly fading `red_alpha` to zero.
    pub fn tick(&mut self, dt: f32) {
        if self.timer <= 0.0 {
            return;
        }

        let prev_timer = self.timer;
        self.timer = (self.timer - dt).max(0.0);

        if self.timer <= 0.0 {
            self.red_alpha = 0.0;
        } else {
            // Linear fade: scale current alpha by remaining-time ratio.
            self.red_alpha *= self.timer / prev_timer;
        }
    }

    /// Returns `true` if the flash effect is still active.
    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }
}

impl Default for DamageIndicator {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the RGBA color for the hurt overlay at the given `alpha`.
///
/// Always pure red `[1.0, 0.0, 0.0, alpha]`.
pub fn hurt_overlay_color(alpha: f32) -> [f32; 4] {
    [1.0, 0.0, 0.0, alpha]
}

/// Return a full-screen vignette rectangle `(x, y, width, height)` for the
/// given screen dimensions.
pub fn hurt_vignette_rect(screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
    (0.0, 0.0, screen_w, screen_h)
}

/// Compute the position `(x, y)` of a directional damage indicator on the
/// screen edge based on the yaw difference between the player and the damage
/// source.
///
/// The indicator is placed on the screen edge at a position determined by
/// `yaw_diff` (radians), where 0 = top center, rotating clockwise.
pub fn directional_indicator_pos(screen_w: f32, screen_h: f32, yaw_diff: f32) -> (f32, f32) {
    let half_w = screen_w / 2.0;
    let half_h = screen_h / 2.0;

    // Unit direction from yaw_diff (0 = up, clockwise positive).
    let dx = yaw_diff.sin();
    let dy = -yaw_diff.cos();

    // Scale to hit the screen edge.
    let scale_x = if dx.abs() > f32::EPSILON {
        half_w / dx.abs()
    } else {
        f32::MAX
    };
    let scale_y = if dy.abs() > f32::EPSILON {
        half_h / dy.abs()
    } else {
        f32::MAX
    };
    let scale = scale_x.min(scale_y);

    let x = half_w + dx * scale;
    let y = half_h + dy * scale;

    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // ── Construction ────────────────────────────────────────────────

    #[test]
    fn new_indicator_is_inactive() {
        let ind = DamageIndicator::new();
        assert_eq!(ind.red_alpha, 0.0);
        assert_eq!(ind.hurt_direction, 0.0);
        assert_eq!(ind.timer, 0.0);
        assert!(!ind.is_active());
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(DamageIndicator::default(), DamageIndicator::new());
    }

    // ── Trigger ─────────────────────────────────────────────────────

    #[test]
    fn trigger_sets_alpha_from_damage() {
        let mut ind = DamageIndicator::new();
        ind.trigger(10.0, 0.0);
        assert!((ind.red_alpha - 0.5).abs() < f32::EPSILON);
        assert!((ind.timer - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn trigger_caps_alpha_at_half() {
        let mut ind = DamageIndicator::new();
        ind.trigger(100.0, 0.0);
        assert!((ind.red_alpha - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn trigger_low_damage_proportional_alpha() {
        let mut ind = DamageIndicator::new();
        ind.trigger(5.0, 1.5);
        assert!((ind.red_alpha - 0.25).abs() < f32::EPSILON);
        assert!((ind.hurt_direction - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn trigger_activates_indicator() {
        let mut ind = DamageIndicator::new();
        ind.trigger(4.0, 0.0);
        assert!(ind.is_active());
    }

    // ── Tick / fade ─────────────────────────────────────────────────

    #[test]
    fn tick_decrements_timer() {
        let mut ind = DamageIndicator::new();
        ind.trigger(10.0, 0.0);
        ind.tick(0.1);
        assert!((ind.timer - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_fades_alpha_toward_zero() {
        let mut ind = DamageIndicator::new();
        ind.trigger(10.0, 0.0);
        let initial_alpha = ind.red_alpha;
        ind.tick(0.25);
        assert!(ind.red_alpha < initial_alpha);
        assert!(ind.red_alpha > 0.0);
    }

    #[test]
    fn tick_past_duration_zeroes_alpha() {
        let mut ind = DamageIndicator::new();
        ind.trigger(10.0, 0.0);
        ind.tick(1.0); // exceeds 0.5s timer
        assert_eq!(ind.red_alpha, 0.0);
        assert!(!ind.is_active());
    }

    #[test]
    fn tick_on_inactive_is_noop() {
        let mut ind = DamageIndicator::new();
        ind.tick(1.0);
        assert_eq!(ind.red_alpha, 0.0);
        assert_eq!(ind.timer, 0.0);
    }

    // ── Color helper ────────────────────────────────────────────────

    #[test]
    fn hurt_color_is_pure_red() {
        let c = hurt_overlay_color(0.3);
        assert_eq!(c[0], 1.0);
        assert_eq!(c[1], 0.0);
        assert_eq!(c[2], 0.0);
        assert!((c[3] - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn hurt_color_zero_alpha() {
        let c = hurt_overlay_color(0.0);
        assert_eq!(c, [1.0, 0.0, 0.0, 0.0]);
    }

    // ── Vignette rect ───────────────────────────────────────────────

    #[test]
    fn vignette_covers_full_screen() {
        let (x, y, w, h) = hurt_vignette_rect(1920.0, 1080.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(w, 1920.0);
        assert_eq!(h, 1080.0);
    }

    // ── Directional indicator position ──────────────────────────────

    #[test]
    fn direction_zero_is_top_center() {
        let (x, y) = directional_indicator_pos(800.0, 600.0, 0.0);
        assert!((x - 400.0).abs() < 1.0);
        assert!(y < 1.0); // top edge
    }

    #[test]
    fn direction_pi_is_bottom_center() {
        let (x, y) = directional_indicator_pos(800.0, 600.0, PI);
        assert!((x - 400.0).abs() < 1.0);
        assert!((y - 600.0).abs() < 1.0); // bottom edge
    }

    #[test]
    fn direction_half_pi_is_right_edge() {
        let (x, y) = directional_indicator_pos(800.0, 600.0, PI / 2.0);
        assert!((x - 800.0).abs() < 1.0); // right edge
        assert!((y - 300.0).abs() < 1.0); // vertical center
    }

    #[test]
    fn direction_negative_half_pi_is_left_edge() {
        let (x, y) = directional_indicator_pos(800.0, 600.0, -PI / 2.0);
        assert!(x < 1.0); // left edge
        assert!((y - 300.0).abs() < 1.0); // vertical center
    }
}
