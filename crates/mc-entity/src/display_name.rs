//! Entity display name colors and formatting.

/// The 16 Minecraft color variants for entity names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameColor {
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Aqua,
    Gold,
    Gray,
    DarkRed,
    DarkGreen,
    DarkBlue,
    DarkAqua,
    DarkGray,
    Purple,
    LightPurple,
    Black,
}

/// A styled display name for an entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayName {
    pub text: String,
    pub color: NameColor,
    pub bold: bool,
    pub italic: bool,
}

/// Returns the RGB color values (0.0–1.0) for a given `NameColor`.
pub fn name_color_rgb(color: NameColor) -> [f32; 3] {
    match color {
        NameColor::White => [1.0, 1.0, 1.0],
        NameColor::Red => [1.0, 0.333, 0.333],
        NameColor::Green => [0.333, 1.0, 0.333],
        NameColor::Blue => [0.333, 0.333, 1.0],
        NameColor::Yellow => [1.0, 1.0, 0.333],
        NameColor::Aqua => [0.333, 1.0, 1.0],
        NameColor::Gold => [1.0, 0.667, 0.0],
        NameColor::Gray => [0.667, 0.667, 0.667],
        NameColor::DarkRed => [0.667, 0.0, 0.0],
        NameColor::DarkGreen => [0.0, 0.667, 0.0],
        NameColor::DarkBlue => [0.0, 0.0, 0.667],
        NameColor::DarkAqua => [0.0, 0.667, 0.667],
        NameColor::DarkGray => [0.333, 0.333, 0.333],
        NameColor::Purple => [0.667, 0.0, 0.667],
        NameColor::LightPurple => [1.0, 0.333, 1.0],
        NameColor::Black => [0.0, 0.0, 0.0],
    }
}

/// Formats a `DisplayName` into a string with Minecraft-style formatting codes.
pub fn format_display_name(name: &DisplayName) -> String {
    let color_code = match name.color {
        NameColor::White => 'f',
        NameColor::Red => 'c',
        NameColor::Green => 'a',
        NameColor::Blue => '9',
        NameColor::Yellow => 'e',
        NameColor::Aqua => 'b',
        NameColor::Gold => '6',
        NameColor::Gray => '7',
        NameColor::DarkRed => '4',
        NameColor::DarkGreen => '2',
        NameColor::DarkBlue => '1',
        NameColor::DarkAqua => '3',
        NameColor::DarkGray => '8',
        NameColor::Purple => '5',
        NameColor::LightPurple => 'd',
        NameColor::Black => '0',
    };

    let mut result = format!("\u{00A7}{color_code}");
    if name.bold {
        result.push_str("\u{00A7}l");
    }
    if name.italic {
        result.push_str("\u{00A7}o");
    }
    result.push_str(&name.text);
    result
}

/// Parses a Minecraft color code character into a `NameColor`.
pub fn parse_color_code(code: char) -> Option<NameColor> {
    match code {
        '0' => Some(NameColor::Black),
        '1' => Some(NameColor::DarkBlue),
        '2' => Some(NameColor::DarkGreen),
        '3' => Some(NameColor::DarkAqua),
        '4' => Some(NameColor::DarkRed),
        '5' => Some(NameColor::Purple),
        '6' => Some(NameColor::Gold),
        '7' => Some(NameColor::Gray),
        '8' => Some(NameColor::DarkGray),
        '9' => Some(NameColor::Blue),
        'a' => Some(NameColor::Green),
        'b' => Some(NameColor::Aqua),
        'c' => Some(NameColor::Red),
        'd' => Some(NameColor::LightPurple),
        'e' => Some(NameColor::Yellow),
        'f' => Some(NameColor::White),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_color_rgb_returns_white() {
        let rgb = name_color_rgb(NameColor::White);
        assert_eq!(rgb, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn name_color_rgb_returns_black() {
        let rgb = name_color_rgb(NameColor::Black);
        assert_eq!(rgb, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn format_display_name_plain() {
        let name = DisplayName {
            text: "Steve".to_string(),
            color: NameColor::White,
            bold: false,
            italic: false,
        };
        assert_eq!(format_display_name(&name), "\u{00A7}fSteve");
    }

    #[test]
    fn format_display_name_bold_italic() {
        let name = DisplayName {
            text: "Hero".to_string(),
            color: NameColor::Gold,
            bold: true,
            italic: true,
        };
        let formatted = format_display_name(&name);
        assert!(formatted.contains("\u{00A7}6"));
        assert!(formatted.contains("\u{00A7}l"));
        assert!(formatted.contains("\u{00A7}o"));
        assert!(formatted.ends_with("Hero"));
    }

    #[test]
    fn parse_color_code_valid() {
        assert_eq!(parse_color_code('0'), Some(NameColor::Black));
        assert_eq!(parse_color_code('a'), Some(NameColor::Green));
        assert_eq!(parse_color_code('f'), Some(NameColor::White));
    }

    #[test]
    fn parse_color_code_invalid() {
        assert_eq!(parse_color_code('z'), None);
        assert_eq!(parse_color_code('g'), None);
    }

    #[test]
    fn roundtrip_color_code() {
        let name = DisplayName {
            text: "Test".to_string(),
            color: NameColor::Red,
            bold: false,
            italic: false,
        };
        let formatted = format_display_name(&name);
        // The color code for Red is 'c'
        assert_eq!(parse_color_code('c'), Some(NameColor::Red));
        assert!(formatted.starts_with("\u{00A7}c"));
    }

    #[test]
    fn all_16_colors_have_rgb() {
        let colors = [
            NameColor::White, NameColor::Red, NameColor::Green, NameColor::Blue,
            NameColor::Yellow, NameColor::Aqua, NameColor::Gold, NameColor::Gray,
            NameColor::DarkRed, NameColor::DarkGreen, NameColor::DarkBlue,
            NameColor::DarkAqua, NameColor::DarkGray, NameColor::Purple,
            NameColor::LightPurple, NameColor::Black,
        ];
        for color in colors {
            let rgb = name_color_rgb(color);
            for channel in &rgb {
                assert!(*channel >= 0.0 && *channel <= 1.0);
            }
        }
    }

    #[test]
    fn all_16_color_codes_parse() {
        let codes = ['0', '1', '2', '3', '4', '5', '6', '7',
                     '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
        for code in codes {
            assert!(parse_color_code(code).is_some(), "code '{code}' should parse");
        }
    }
}
