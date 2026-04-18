/// Available dye colors for sign text, matching Minecraft's 16 dye palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    Purple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Cyan,
    Red,
    Pink,
    Yellow,
    White,
}

/// Strip control characters (ASCII 0x00..0x1F and 0x7F) from text.
fn strip_control_chars(text: &str) -> String {
    text.chars().filter(|c| !c.is_ascii_control()).collect()
}

/// Format sign text: trim whitespace, truncate to 15 characters, and strip
/// control characters.
pub fn format_sign_text(text: &str) -> String {
    let trimmed = text.trim();
    let stripped = strip_control_chars(trimmed);
    truncate_to_chars(&stripped, 15)
}

/// Truncate a string to at most `max` characters (by Unicode scalar values).
fn truncate_to_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Sign block-entity data: four lines of text with color, glow, and edit state.
///
/// `front` indicates whether this data represents the front face of the sign
/// (Minecraft 1.20+ supports front/back text on signs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignData {
    pub lines: [String; 4],
    pub color: SignColor,
    pub glowing: bool,
    pub editable: bool,
    pub front: bool,
}

impl SignData {
    /// Create a new sign with empty lines, black text, not glowing, editable,
    /// and representing the front face.
    pub fn new() -> Self {
        Self {
            lines: [String::new(), String::new(), String::new(), String::new()],
            color: SignColor::Black,
            glowing: false,
            editable: true,
            front: true,
        }
    }

    /// Set the text of a line (0..=3). The text is formatted via
    /// [`format_sign_text`]: trimmed, control characters stripped, and
    /// truncated to 15 characters. Panics if `idx > 3`. Has no effect if
    /// the sign is locked (not editable).
    pub fn set_line(&mut self, idx: usize, text: &str) {
        assert!(idx < 4, "sign line index must be 0..=3, got {idx}");
        if !self.editable {
            return;
        }
        self.lines[idx] = format_sign_text(text);
    }

    /// Get the text of a line (0..=3). Panics if `idx > 3`.
    pub fn get_line(&self, idx: usize) -> &str {
        assert!(idx < 4, "sign line index must be 0..=3, got {idx}");
        &self.lines[idx]
    }

    /// Change the text color of the sign.
    pub fn set_color(&mut self, color: SignColor) {
        self.color = color;
    }

    /// Enable or disable the glowing-text effect.
    pub fn set_glowing(&mut self, glowing: bool) {
        self.glowing = glowing;
    }

    /// Lock the sign so that [`set_line`](Self::set_line) calls are ignored.
    pub fn lock(&mut self) {
        self.editable = false;
    }

    /// Returns `true` if every line is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|l| l.is_empty())
    }
}

impl Default for SignData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- line limits --

    #[test]
    fn set_line_truncates_to_15_chars() {
        let mut sign = SignData::new();
        sign.set_line(0, "abcdefghijklmnopqrstuvwxyz");
        assert_eq!(sign.get_line(0), "abcdefghijklmno"); // 15 chars
    }

    #[test]
    fn set_line_accepts_short_text() {
        let mut sign = SignData::new();
        sign.set_line(1, "Hi");
        assert_eq!(sign.get_line(1), "Hi");
    }

    #[test]
    fn set_line_trims_whitespace() {
        let mut sign = SignData::new();
        sign.set_line(2, "  padded  ");
        assert_eq!(sign.get_line(2), "padded");
    }

    #[test]
    #[should_panic(expected = "sign line index must be 0..=3")]
    fn set_line_panics_on_out_of_range() {
        let mut sign = SignData::new();
        sign.set_line(4, "boom");
    }

    #[test]
    #[should_panic(expected = "sign line index must be 0..=3")]
    fn get_line_panics_on_out_of_range() {
        let sign = SignData::new();
        let _ = sign.get_line(5);
    }

    // -- color setting --

    #[test]
    fn set_color_changes_color() {
        let mut sign = SignData::new();
        assert_eq!(sign.color, SignColor::Black);
        sign.set_color(SignColor::Red);
        assert_eq!(sign.color, SignColor::Red);
        sign.set_color(SignColor::Gold);
        assert_eq!(sign.color, SignColor::Gold);
    }

    // -- glow toggle --

    #[test]
    fn set_glowing_toggles_glow() {
        let mut sign = SignData::new();
        assert!(!sign.glowing);
        sign.set_glowing(true);
        assert!(sign.glowing);
        sign.set_glowing(false);
        assert!(!sign.glowing);
    }

    // -- lock prevents edit --

    #[test]
    fn lock_prevents_further_edits() {
        let mut sign = SignData::new();
        sign.set_line(0, "editable");
        assert_eq!(sign.get_line(0), "editable");

        sign.lock();
        sign.set_line(0, "overwritten?");
        // Text should remain unchanged after lock
        assert_eq!(sign.get_line(0), "editable");
    }

    // -- format strips control chars --

    #[test]
    fn format_sign_text_strips_control_chars() {
        let input = "he\x00ll\x1Fo\x7F!";
        let result = format_sign_text(input);
        assert_eq!(result, "hello!");
    }

    #[test]
    fn format_sign_text_trims_and_truncates() {
        let result = format_sign_text("  1234567890abcdefghij  ");
        assert_eq!(result, "1234567890abcde"); // 15 chars
    }

    // -- empty check --

    #[test]
    fn is_empty_returns_true_for_blank_sign() {
        let sign = SignData::new();
        assert!(sign.is_empty());
    }

    #[test]
    fn is_empty_returns_false_when_any_line_has_text() {
        let mut sign = SignData::new();
        sign.set_line(2, "hello");
        assert!(!sign.is_empty());
    }

    // -- default values --

    #[test]
    fn new_sign_has_correct_defaults() {
        let sign = SignData::new();
        assert_eq!(sign.color, SignColor::Black);
        assert!(!sign.glowing);
        assert!(sign.editable);
        assert!(sign.front);
        for i in 0..4 {
            assert!(sign.get_line(i).is_empty());
        }
    }
}
