//! Credits screen scroller.
//!
//! Provides [`CreditsState`] for tracking scroll position and helpers for
//! computing visible lines and total height of the credits roll.

/// Lines displayed in the credits screen.
///
/// Empty strings represent blank spacer lines between sections.
pub const CREDITS_LINES: &[&str] = &[
    "",
    "MCRust",
    "",
    "A Minecraft Clone Written in Rust",
    "",
    "Programming",
    "Claude Code Agent Fleet",
    "",
    "Rendering Engine",
    "wgpu + Metal/Vulkan/DX12",
    "",
    "Mathematics",
    "glam",
    "",
    "Window Management",
    "winit",
    "",
    "GPU Data",
    "bytemuck",
    "",
    "Terrain Noise",
    "noise-rs",
    "",
    "World Generation",
    "Procedural terrain, caves, ores, structures",
    "",
    "Physics",
    "AABB collision, raycasting, fluid simulation",
    "",
    "Entity System",
    "Component-based with AI, combat, spawning",
    "",
    "Crafting",
    "77+ recipes, enchanting, brewing, smithing",
    "",
    "Audio",
    "Spatial sound, music, ambient",
    "",
    "Networking",
    "Binary packet protocol, chat commands",
    "",
    "Special Thanks",
    "The Rust community",
    "Mojang for creating Minecraft",
    "",
    "Built with love and unsafe { transmute(coffee) }",
    "",
    "Thank you for playing!",
    "",
    "",
];

/// Scroll state for the credits screen.
#[derive(Debug, Clone)]
pub struct CreditsState {
    /// Current vertical scroll offset in pixels.
    pub scroll_y: f32,
    /// Scroll speed in pixels per second.
    pub speed: f32,
    /// Whether the credits have scrolled past all lines.
    pub finished: bool,
}

impl CreditsState {
    /// Create a new credits state with default speed (30.0 pixels/sec).
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            speed: 30.0,
            finished: false,
        }
    }

    /// Advance the scroll position by `dt` seconds.
    ///
    /// Sets `finished` to `true` once `scroll_y` exceeds the total height
    /// of all credit lines (computed with a default line height of 20.0).
    pub fn tick(&mut self, dt: f32) {
        if self.finished {
            return;
        }
        self.scroll_y += self.speed * dt;
        let height = total_height(20.0);
        if self.scroll_y >= height {
            self.finished = true;
        }
    }
}

/// Compute the total pixel height of the credits.
///
/// Equal to `CREDITS_LINES.len() * line_h`.
pub fn total_height(line_h: f32) -> f32 {
    CREDITS_LINES.len() as f32 * line_h
}

/// Return the indices and y-positions of lines currently visible on screen.
///
/// Each entry is `(line_index, y_position)` where `y_position` is the
/// vertical offset at which the line should be drawn. Only lines whose
/// y-position falls within `0.0..screen_h` are included.
pub fn visible_lines(state: &CreditsState, screen_h: f32, line_h: f32) -> Vec<(usize, f32)> {
    let mut result = Vec::new();
    for (i, _line) in CREDITS_LINES.iter().enumerate() {
        let y = i as f32 * line_h - state.scroll_y;
        if y + line_h > 0.0 && y < screen_h {
            result.push((i, y));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // CREDITS_LINES constant
    // ------------------------------------------------------------------

    #[test]
    fn credits_has_at_least_50_lines() {
        assert!(
            CREDITS_LINES.len() >= 50,
            "expected >= 50 lines, got {}",
            CREDITS_LINES.len()
        );
    }

    #[test]
    fn credits_contains_title() {
        assert!(CREDITS_LINES.contains(&"MCRust"));
    }

    #[test]
    fn credits_contains_special_thanks() {
        assert!(CREDITS_LINES.contains(&"Special Thanks"));
    }

    // ------------------------------------------------------------------
    // CreditsState::new
    // ------------------------------------------------------------------

    #[test]
    fn new_state_starts_at_zero() {
        let state = CreditsState::new();
        assert!((state.scroll_y - 0.0).abs() < f32::EPSILON);
        assert!((state.speed - 30.0).abs() < f32::EPSILON);
        assert!(!state.finished);
    }

    // ------------------------------------------------------------------
    // CreditsState::tick
    // ------------------------------------------------------------------

    #[test]
    fn tick_advances_scroll() {
        let mut state = CreditsState::new();
        state.tick(1.0);
        assert!((state.scroll_y - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_accumulates_over_multiple_calls() {
        let mut state = CreditsState::new();
        state.tick(0.5);
        state.tick(0.5);
        assert!((state.scroll_y - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_does_not_advance_when_finished() {
        let mut state = CreditsState::new();
        state.finished = true;
        state.tick(1.0);
        assert!((state.scroll_y - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_sets_finished_when_past_all_lines() {
        let mut state = CreditsState::new();
        // With default line_h of 20.0, total height = lines * 20.
        let height = total_height(20.0);
        // Jump to just before the end.
        state.scroll_y = height - 1.0;
        state.tick(1.0); // +30.0, well past end
        assert!(state.finished);
    }

    #[test]
    fn tick_not_finished_before_end() {
        let mut state = CreditsState::new();
        state.tick(0.1);
        assert!(!state.finished);
    }

    // ------------------------------------------------------------------
    // total_height
    // ------------------------------------------------------------------

    #[test]
    fn total_height_is_lines_times_line_h() {
        let h = total_height(20.0);
        let expected = CREDITS_LINES.len() as f32 * 20.0;
        assert!((h - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn total_height_with_different_line_h() {
        let h = total_height(15.0);
        let expected = CREDITS_LINES.len() as f32 * 15.0;
        assert!((h - expected).abs() < f32::EPSILON);
    }

    // ------------------------------------------------------------------
    // visible_lines
    // ------------------------------------------------------------------

    #[test]
    fn visible_lines_at_start() {
        let state = CreditsState::new();
        let visible = visible_lines(&state, 100.0, 20.0);
        // screen_h=100, line_h=20 => 5 lines visible (indices 0..4)
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0].0, 0);
        assert!((visible[0].1 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn visible_lines_after_scroll() {
        let mut state = CreditsState::new();
        state.scroll_y = 40.0; // skip first 2 full lines
        let visible = visible_lines(&state, 100.0, 20.0);
        // Line 1 has y = 1*20 - 40 = -20, bottom at 0 => not visible
        // Line 2 has y = 2*20 - 40 = 0 => visible
        assert_eq!(visible[0].0, 2);
        assert!((visible[0].1 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn visible_lines_empty_when_scrolled_past() {
        let mut state = CreditsState::new();
        state.scroll_y = total_height(20.0) + 100.0;
        let visible = visible_lines(&state, 100.0, 20.0);
        assert!(visible.is_empty());
    }

    #[test]
    fn visible_lines_includes_partial_top() {
        let mut state = CreditsState::new();
        state.scroll_y = 10.0; // line 0 partially visible (top half clipped)
        let visible = visible_lines(&state, 100.0, 20.0);
        // Line 0: y = 0 - 10 = -10, bottom at 10 > 0 => visible
        assert_eq!(visible[0].0, 0);
        assert!((visible[0].1 - (-10.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn visible_lines_includes_partial_bottom() {
        let state = CreditsState::new();
        // screen_h = 90 => line at y=80 (index 4) partially visible
        let visible = visible_lines(&state, 90.0, 20.0);
        let last = visible.last().unwrap();
        assert_eq!(last.0, 4);
        assert!((last.1 - 80.0).abs() < f32::EPSILON);
    }
}
