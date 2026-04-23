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

/// Maximum character length for a single sign line.
pub const fn max_line_length() -> usize {
    15
}

/// Map a dye color name to its palette index (0..=15).
///
/// Accepts lowercase Minecraft dye names (e.g. `"red"`, `"light_blue"`).
/// Returns `0` (black) for unrecognised names.
pub fn dye_color_index(color_name: &str) -> u8 {
    match color_name {
        "black" => 0,
        "dark_blue" => 1,
        "dark_green" => 2,
        "dark_cyan" => 3,
        "dark_red" => 4,
        "purple" => 5,
        "gold" => 6,
        "gray" => 7,
        "dark_gray" => 8,
        "blue" => 9,
        "green" => 10,
        "cyan" => 11,
        "red" => 12,
        "pink" => 13,
        "yellow" => 14,
        "white" => 15,
        _ => 0,
    }
}

/// RGB colour value for a dye palette index (0..=15).
///
/// Returns black (`[0.0, 0.0, 0.0]`) for out-of-range indices.
pub fn color_rgb(color_index: u8) -> [f32; 3] {
    match color_index {
        0 => [0.0, 0.0, 0.0],           // black
        1 => [0.0, 0.0, 0.67],          // dark_blue
        2 => [0.0, 0.67, 0.0],          // dark_green
        3 => [0.0, 0.67, 0.67],         // dark_cyan
        4 => [0.67, 0.0, 0.0],          // dark_red
        5 => [0.67, 0.0, 0.67],         // purple
        6 => [1.0, 0.67, 0.0],          // gold
        7 => [0.67, 0.67, 0.67],        // gray
        8 => [0.33, 0.33, 0.33],        // dark_gray
        9 => [0.33, 0.33, 1.0],         // blue
        10 => [0.33, 1.0, 0.33],        // green
        11 => [0.33, 1.0, 1.0],         // cyan
        12 => [1.0, 0.33, 0.33],        // red
        13 => [1.0, 0.67, 0.8],         // pink
        14 => [1.0, 1.0, 0.33],         // yellow
        15 => [1.0, 1.0, 1.0],          // white
        _ => [0.0, 0.0, 0.0],
    }
}

/// Convert a [`SignColor`] enum variant to its palette index.
fn sign_color_to_index(color: SignColor) -> u8 {
    match color {
        SignColor::Black => 0,
        SignColor::DarkBlue => 1,
        SignColor::DarkGreen => 2,
        SignColor::DarkCyan => 3,
        SignColor::DarkRed => 4,
        SignColor::Purple => 5,
        SignColor::Gold => 6,
        SignColor::Gray => 7,
        SignColor::DarkGray => 8,
        SignColor::Blue => 9,
        SignColor::Green => 10,
        SignColor::Cyan => 11,
        SignColor::Red => 12,
        SignColor::Pink => 13,
        SignColor::Yellow => 14,
        SignColor::White => 15,
    }
}

/// Return non-empty lines from a sign paired with their RGB colour.
///
/// Each entry is `(line_text, [r, g, b])`. Empty lines are omitted.
pub fn sign_render_lines(sign: &SignData) -> Vec<(String, [f32; 3])> {
    let rgb = color_rgb(sign_color_to_index(sign.color));
    sign.lines
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| (l.clone(), rgb))
        .collect()
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

    /// Set the text of a line (0..=3). The index is clamped to the valid
    /// range. The text is formatted via [`format_sign_text`]: trimmed,
    /// control characters stripped, and truncated to 15 characters. Has no
    /// effect if the sign is locked (not editable).
    pub fn set_line(&mut self, idx: usize, text: &str) {
        let idx = idx.min(3);
        if !self.editable {
            return;
        }
        self.lines[idx] = format_sign_text(text);
    }

    /// Get the text of a line (0..=3). The index is clamped to the valid
    /// range.
    pub fn get_line(&self, idx: usize) -> &str {
        let idx = idx.min(3);
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
    fn set_line_clamps_high_index_to_3() {
        let mut sign = SignData::new();
        sign.set_line(99, "clamped");
        // Should write to line 3 (the maximum valid index)
        assert_eq!(sign.get_line(3), "clamped");
    }

    #[test]
    fn get_line_clamps_high_index_to_3() {
        let mut sign = SignData::new();
        sign.set_line(3, "last");
        assert_eq!(sign.get_line(100), "last");
    }

    // -- all 4 lines --

    #[test]
    fn all_four_lines_independent() {
        let mut sign = SignData::new();
        sign.set_line(0, "line zero");
        sign.set_line(1, "line one");
        sign.set_line(2, "line two");
        sign.set_line(3, "line three");
        assert_eq!(sign.get_line(0), "line zero");
        assert_eq!(sign.get_line(1), "line one");
        assert_eq!(sign.get_line(2), "line two");
        assert_eq!(sign.get_line(3), "line three");
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

    // -- max_line_length --

    #[test]
    fn max_line_length_is_15() {
        assert_eq!(max_line_length(), 15);
    }

    // -- dye_color_index --

    #[test]
    fn dye_color_index_maps_all_16_colors() {
        assert_eq!(dye_color_index("black"), 0);
        assert_eq!(dye_color_index("dark_blue"), 1);
        assert_eq!(dye_color_index("dark_green"), 2);
        assert_eq!(dye_color_index("dark_cyan"), 3);
        assert_eq!(dye_color_index("dark_red"), 4);
        assert_eq!(dye_color_index("purple"), 5);
        assert_eq!(dye_color_index("gold"), 6);
        assert_eq!(dye_color_index("gray"), 7);
        assert_eq!(dye_color_index("dark_gray"), 8);
        assert_eq!(dye_color_index("blue"), 9);
        assert_eq!(dye_color_index("green"), 10);
        assert_eq!(dye_color_index("cyan"), 11);
        assert_eq!(dye_color_index("red"), 12);
        assert_eq!(dye_color_index("pink"), 13);
        assert_eq!(dye_color_index("yellow"), 14);
        assert_eq!(dye_color_index("white"), 15);
    }

    #[test]
    fn dye_color_index_returns_black_for_unknown() {
        assert_eq!(dye_color_index("magenta"), 0);
        assert_eq!(dye_color_index(""), 0);
    }

    // -- color_rgb --

    #[test]
    fn color_rgb_returns_black_for_index_0() {
        assert_eq!(color_rgb(0), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn color_rgb_returns_white_for_index_15() {
        assert_eq!(color_rgb(15), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn color_rgb_returns_black_for_out_of_range() {
        assert_eq!(color_rgb(16), [0.0, 0.0, 0.0]);
        assert_eq!(color_rgb(255), [0.0, 0.0, 0.0]);
    }

    // -- sign_render_lines --

    #[test]
    fn render_lines_skips_empty_lines() {
        let sign = SignData::new();
        let lines = sign_render_lines(&sign);
        assert!(lines.is_empty());
    }

    #[test]
    fn render_lines_returns_non_empty_with_color() {
        let mut sign = SignData::new();
        sign.set_line(0, "Hello");
        sign.set_line(2, "World");
        sign.set_color(SignColor::Red);

        let lines = sign_render_lines(&sign);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "Hello");
        assert_eq!(lines[0].1, color_rgb(12)); // red
        assert_eq!(lines[1].0, "World");
        assert_eq!(lines[1].1, color_rgb(12));
    }

    #[test]
    fn render_lines_returns_all_four_when_filled() {
        let mut sign = SignData::new();
        sign.set_line(0, "A");
        sign.set_line(1, "B");
        sign.set_line(2, "C");
        sign.set_line(3, "D");

        let lines = sign_render_lines(&sign);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].0, "A");
        assert_eq!(lines[3].0, "D");
    }
}
