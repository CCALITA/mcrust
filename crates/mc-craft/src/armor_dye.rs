//! Leather armor dyeing, washing, and color utilities.

use crate::item_ids::{
    ITEM_LEATHER_BOOTS, ITEM_LEATHER_CHESTPLATE, ITEM_LEATHER_HELMET, ITEM_LEATHER_LEGGINGS,
};

/// Default undyed leather color (brownish).
const DEFAULT_LEATHER: [u8; 3] = [160, 101, 64];

/// RGB values for the 16 Minecraft dye colors (indexed 0..=15).
const DYE_COLORS: [[u8; 3]; 16] = [
    [25, 25, 25],       // 0  Black
    [153, 51, 51],      // 1  Red
    [102, 127, 51],     // 2  Green
    [102, 76, 51],      // 3  Brown
    [51, 76, 178],      // 4  Blue
    [127, 63, 178],     // 5  Purple
    [76, 127, 153],     // 6  Cyan
    [153, 153, 153],    // 7  Light Gray
    [76, 76, 76],       // 8  Gray
    [242, 127, 165],    // 9  Pink
    [127, 204, 25],     // 10 Lime
    [229, 229, 51],     // 11 Yellow
    [102, 153, 216],    // 12 Light Blue
    [178, 76, 216],     // 13 Magenta
    [229, 178, 51],     // 14 Orange
    [255, 255, 255],    // 15 White
];

/// Returns the RGB triple for a dye index (0..=15).
///
/// Indices outside the valid range wrap via modulo 16.
pub fn dye_rgb(dye: u8) -> [u8; 3] {
    DYE_COLORS[(dye % 16) as usize]
}

/// Returns the default (undyed) leather armor color.
pub fn default_leather_color() -> [u8; 3] {
    DEFAULT_LEATHER
}

/// Averages the current armor color with a dye color, producing a new color.
pub fn dye_leather_armor(current: [u8; 3], dye: u8) -> [u8; 3] {
    let dye_color = dye_rgb(dye);
    [
        ((current[0] as u16 + dye_color[0] as u16) / 2) as u8,
        ((current[1] as u16 + dye_color[1] as u16) / 2) as u8,
        ((current[2] as u16 + dye_color[2] as u16) / 2) as u8,
    ]
}

/// Washes armor in a cauldron, resetting to the default leather color.
pub fn wash_armor_in_cauldron() -> [u8; 3] {
    DEFAULT_LEATHER
}

/// Returns `true` if the given item ID corresponds to a dyeable leather armor piece.
pub fn can_dye_item(item_id: u16) -> bool {
    matches!(
        item_id,
        ITEM_LEATHER_HELMET | ITEM_LEATHER_CHESTPLATE | ITEM_LEATHER_LEGGINGS | ITEM_LEATHER_BOOTS
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_color_is_leather_brown() {
        assert_eq!(default_leather_color(), [160, 101, 64]);
    }

    #[test]
    fn dye_rgb_returns_correct_colors() {
        assert_eq!(dye_rgb(0), [25, 25, 25]); // Black
        assert_eq!(dye_rgb(15), [255, 255, 255]); // White
        assert_eq!(dye_rgb(11), [229, 229, 51]); // Yellow
    }

    #[test]
    fn dye_rgb_wraps_on_overflow() {
        assert_eq!(dye_rgb(16), dye_rgb(0));
        assert_eq!(dye_rgb(17), dye_rgb(1));
    }

    #[test]
    fn dye_leather_armor_averages_colors() {
        let result = dye_leather_armor([160, 101, 64], 1); // Red
        // (160+153)/2=156, (101+51)/2=76, (64+51)/2=57
        assert_eq!(result, [156, 76, 57]);
    }

    #[test]
    fn dye_leather_armor_with_white() {
        let result = dye_leather_armor([100, 100, 100], 15);
        // (100+255)/2=177, (100+255)/2=177, (100+255)/2=177
        assert_eq!(result, [177, 177, 177]);
    }

    #[test]
    fn dye_leather_armor_with_black() {
        let result = dye_leather_armor([200, 200, 200], 0);
        // (200+25)/2=112, same for all
        assert_eq!(result, [112, 112, 112]);
    }

    #[test]
    fn wash_resets_to_default() {
        assert_eq!(wash_armor_in_cauldron(), default_leather_color());
    }

    #[test]
    fn can_dye_leather_armor_pieces() {
        assert!(can_dye_item(300)); // helmet
        assert!(can_dye_item(301)); // chestplate
        assert!(can_dye_item(302)); // leggings
        assert!(can_dye_item(303)); // boots
    }

    #[test]
    fn cannot_dye_non_leather_armor() {
        assert!(!can_dye_item(310)); // iron helmet
        assert!(!can_dye_item(0));
        assert!(!can_dye_item(999));
    }

    #[test]
    fn sequential_dyeing_blends() {
        let color = default_leather_color();
        let once = dye_leather_armor(color, 1);
        let twice = dye_leather_armor(once, 1);
        // Each application moves closer to the dye color
        assert_ne!(once, twice);
    }
}
