//! Death screen overlay: red fade-in, "You Died" text, score, respawn and title menu buttons.

/// State for the death screen overlay shown when the player dies.
pub struct DeathScreen {
    /// Whether the death screen is currently displayed.
    pub visible: bool,
    /// Elapsed time in seconds since the death screen appeared.
    pub timer: f32,
    /// Message describing the cause of death (e.g. "Slain by Zombie").
    pub death_message: String,
    /// The player's score at the time of death.
    pub score: u32,
}

impl DeathScreen {
    /// Create a new death screen with the given message and score.
    /// Starts visible with timer at zero.
    pub fn new(message: impl Into<String>, score: u32) -> Self {
        Self {
            visible: true,
            timer: 0.0,
            death_message: message.into(),
            score,
        }
    }
}

/// Advance the death screen timer by `dt` seconds.
pub fn tick_death_screen(screen: &mut DeathScreen, dt: f32) {
    screen.timer += dt;
}

/// Compute the red overlay alpha for the death screen.
///
/// Linearly interpolates from 0.0 to 0.6 over the first second,
/// then clamps at 0.6.
pub fn death_overlay_alpha(timer: f32) -> f32 {
    let max_alpha = 0.6;
    let fade_duration = 1.0;
    (timer / fade_duration).min(1.0) * max_alpha
}

/// Return the `(x, y, width, height)` rectangle for the "Respawn" button,
/// centered horizontally at 60% of screen height.
pub fn respawn_button_rect(screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
    let btn_w = 200.0;
    let btn_h = 40.0;
    let x = (screen_w - btn_w) / 2.0;
    let y = screen_h * 0.6;
    (x, y, btn_w, btn_h)
}

/// Return the `(x, y, width, height)` rectangle for the "Title Menu" button,
/// centered horizontally below the respawn button.
pub fn title_menu_button_rect(screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
    let btn_w = 200.0;
    let btn_h = 40.0;
    let x = (screen_w - btn_w) / 2.0;
    let spacing = 10.0;
    let respawn_y = screen_h * 0.6;
    let y = respawn_y + btn_h + spacing;
    (x, y, btn_w, btn_h)
}

/// Return the `(x, y)` position for the centered "You Died" heading.
///
/// Placed at roughly 30% of screen height, centered horizontally
/// (assuming a text width of ~200 pixels for the heading).
pub fn you_died_text_position(screen_w: f32, screen_h: f32) -> (f32, f32) {
    let text_w = 200.0;
    let x = (screen_w - text_w) / 2.0;
    let y = screen_h * 0.3;
    (x, y)
}

/// Format the score for display on the death screen.
pub fn score_text(score: u32) -> String {
    format!("Score: {score}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_death_screen_is_visible_with_zero_timer() {
        let screen = DeathScreen::new("Slain by Zombie", 42);
        assert!(screen.visible);
        assert!((screen.timer - 0.0).abs() < f32::EPSILON);
        assert_eq!(screen.death_message, "Slain by Zombie");
        assert_eq!(screen.score, 42);
    }

    #[test]
    fn tick_advances_timer() {
        let mut screen = DeathScreen::new("Fell from a high place", 10);
        tick_death_screen(&mut screen, 0.5);
        assert!((screen.timer - 0.5).abs() < f32::EPSILON);
        tick_death_screen(&mut screen, 0.3);
        assert!((screen.timer - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_alpha_fades_in_over_one_second() {
        assert!((death_overlay_alpha(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((death_overlay_alpha(0.5) - 0.3).abs() < f32::EPSILON);
        assert!((death_overlay_alpha(1.0) - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_alpha_clamps_at_max() {
        assert!((death_overlay_alpha(2.0) - 0.6).abs() < f32::EPSILON);
        assert!((death_overlay_alpha(100.0) - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn respawn_button_is_centered_at_sixty_percent() {
        let (x, y, w, h) = respawn_button_rect(800.0, 600.0);
        assert!((x - 300.0).abs() < f32::EPSILON); // (800 - 200) / 2
        assert!((y - 360.0).abs() < f32::EPSILON); // 600 * 0.6
        assert!((w - 200.0).abs() < f32::EPSILON);
        assert!((h - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn title_menu_button_is_below_respawn() {
        let (_, respawn_y, _, respawn_h) = respawn_button_rect(800.0, 600.0);
        let (x, y, w, h) = title_menu_button_rect(800.0, 600.0);
        assert!((x - 300.0).abs() < f32::EPSILON);
        assert!((y - (respawn_y + respawn_h + 10.0)).abs() < f32::EPSILON);
        assert!((w - 200.0).abs() < f32::EPSILON);
        assert!((h - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn you_died_text_centered_at_thirty_percent() {
        let (x, y) = you_died_text_position(800.0, 600.0);
        assert!((x - 300.0).abs() < f32::EPSILON); // (800 - 200) / 2
        assert!((y - 180.0).abs() < f32::EPSILON); // 600 * 0.3
    }

    #[test]
    fn score_text_formats_correctly() {
        assert_eq!(score_text(0), "Score: 0");
        assert_eq!(score_text(42), "Score: 42");
        assert_eq!(score_text(999_999), "Score: 999999");
    }
}
