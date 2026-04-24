//! Hanging sign data and types.
//!
//! Defines the ten hanging sign wood types, sign construction,
//! line editing with truncation, and chain length constants.

/// Maximum number of characters per line on a hanging sign.
const MAX_LINE_LENGTH: usize = 15;

/// The ten wood types available for hanging signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HangingSignType {
    Oak,
    Birch,
    Spruce,
    Jungle,
    DarkOak,
    Acacia,
    Cherry,
    Bamboo,
    Crimson,
    Warped,
}

/// All sign types in declaration order, for iteration and counting.
const ALL_SIGN_TYPES: [HangingSignType; 10] = [
    HangingSignType::Oak,
    HangingSignType::Birch,
    HangingSignType::Spruce,
    HangingSignType::Jungle,
    HangingSignType::DarkOak,
    HangingSignType::Acacia,
    HangingSignType::Cherry,
    HangingSignType::Bamboo,
    HangingSignType::Crimson,
    HangingSignType::Warped,
];

impl HangingSignType {
    /// Human-readable name for this sign type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Oak => "Oak",
            Self::Birch => "Birch",
            Self::Spruce => "Spruce",
            Self::Jungle => "Jungle",
            Self::DarkOak => "Dark Oak",
            Self::Acacia => "Acacia",
            Self::Cherry => "Cherry",
            Self::Bamboo => "Bamboo",
            Self::Crimson => "Crimson",
            Self::Warped => "Warped",
        }
    }
}

/// A hanging sign block with four text lines and display properties.
#[derive(Debug, Clone)]
pub struct HangingSign {
    pub sign_type: HangingSignType,
    pub lines: [String; 4],
    pub color: u8,
    pub glowing: bool,
    pub attached_to_wall: bool,
}

impl HangingSign {
    /// Create a new hanging sign with default (blank) lines.
    pub fn new(sign_type: HangingSignType) -> Self {
        Self {
            sign_type,
            lines: [
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            color: 0,
            glowing: false,
            attached_to_wall: false,
        }
    }

    /// Set the text of a line. Index is clamped to 0..=3 and text is
    /// truncated to 15 characters.
    pub fn set_line(&mut self, idx: usize, text: String) {
        let clamped = idx.min(3);
        let truncated = if text.len() > MAX_LINE_LENGTH {
            text[..MAX_LINE_LENGTH].to_string()
        } else {
            text
        };
        self.lines[clamped] = truncated;
    }
}

/// Length of the chain attaching a hanging sign to a block, in blocks.
pub fn hanging_sign_chain_length() -> f32 {
    1.5
}

/// Total number of hanging sign wood types.
pub fn total_sign_types() -> usize {
    ALL_SIGN_TYPES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_types_named() {
        let names: Vec<&str> = ALL_SIGN_TYPES.iter().map(|t| t.name()).collect();
        assert_eq!(
            names,
            vec![
                "Oak", "Birch", "Spruce", "Jungle", "Dark Oak", "Acacia",
                "Cherry", "Bamboo", "Crimson", "Warped",
            ]
        );
    }

    #[test]
    fn total_sign_types_is_ten() {
        assert_eq!(total_sign_types(), 10);
    }

    #[test]
    fn new_sign_has_blank_lines() {
        let sign = HangingSign::new(HangingSignType::Oak);
        for line in &sign.lines {
            assert!(line.is_empty());
        }
    }

    #[test]
    fn new_sign_defaults() {
        let sign = HangingSign::new(HangingSignType::Cherry);
        assert_eq!(sign.sign_type, HangingSignType::Cherry);
        assert_eq!(sign.color, 0);
        assert!(!sign.glowing);
        assert!(!sign.attached_to_wall);
    }

    #[test]
    fn set_line_within_bounds() {
        let mut sign = HangingSign::new(HangingSignType::Birch);
        sign.set_line(0, "Hello".to_string());
        sign.set_line(3, "World".to_string());
        assert_eq!(sign.lines[0], "Hello");
        assert_eq!(sign.lines[3], "World");
    }

    #[test]
    fn set_line_clamps_index() {
        let mut sign = HangingSign::new(HangingSignType::Spruce);
        sign.set_line(99, "Clamped".to_string());
        assert_eq!(sign.lines[3], "Clamped");
    }

    #[test]
    fn set_line_truncates_long_text() {
        let mut sign = HangingSign::new(HangingSignType::Bamboo);
        sign.set_line(1, "This is way too long for a sign".to_string());
        assert_eq!(sign.lines[1].len(), 15);
        assert_eq!(sign.lines[1], "This is way too");
    }

    #[test]
    fn set_line_preserves_short_text() {
        let mut sign = HangingSign::new(HangingSignType::Acacia);
        sign.set_line(2, "Short".to_string());
        assert_eq!(sign.lines[2], "Short");
    }

    #[test]
    fn chain_length_value() {
        assert!((hanging_sign_chain_length() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn sign_type_equality() {
        assert_eq!(HangingSignType::Crimson, HangingSignType::Crimson);
        assert_ne!(HangingSignType::Oak, HangingSignType::Warped);
    }

    #[test]
    fn sign_type_copy() {
        let t = HangingSignType::Jungle;
        let t2 = t;
        assert_eq!(t, t2);
    }
}
