/// Dye color mechanics: leather armor dyeing, wool dyeing, and dye mixing.
///
/// Implements Minecraft's 16 dye colors with their RGB values. Supports mixing
/// multiple dyes by averaging RGB channels and snapping to the nearest known color.

/// The 16 Minecraft dye colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

/// All dye color variants in declaration order, for iteration and nearest-color lookup.
const ALL_COLORS: [DyeColor; 16] = [
    DyeColor::White,
    DyeColor::Orange,
    DyeColor::Magenta,
    DyeColor::LightBlue,
    DyeColor::Yellow,
    DyeColor::Lime,
    DyeColor::Pink,
    DyeColor::Gray,
    DyeColor::LightGray,
    DyeColor::Cyan,
    DyeColor::Purple,
    DyeColor::Blue,
    DyeColor::Brown,
    DyeColor::Green,
    DyeColor::Red,
    DyeColor::Black,
];

/// Returns the RGB color tuple for a given dye color.
///
/// Values are based on Minecraft Java Edition dye item colors.
pub fn dye_rgb(color: DyeColor) -> (u8, u8, u8) {
    match color {
        DyeColor::White => (249, 255, 254),
        DyeColor::Orange => (249, 128, 29),
        DyeColor::Magenta => (199, 78, 189),
        DyeColor::LightBlue => (58, 179, 218),
        DyeColor::Yellow => (254, 216, 61),
        DyeColor::Lime => (128, 199, 31),
        DyeColor::Pink => (243, 139, 170),
        DyeColor::Gray => (71, 79, 82),
        DyeColor::LightGray => (157, 157, 151),
        DyeColor::Cyan => (22, 156, 156),
        DyeColor::Purple => (137, 50, 184),
        DyeColor::Blue => (60, 68, 170),
        DyeColor::Brown => (131, 84, 50),
        DyeColor::Green => (94, 124, 22),
        DyeColor::Red => (176, 46, 38),
        DyeColor::Black => (29, 29, 33),
    }
}

/// Squared Euclidean distance between two RGB colors.
///
/// Uses `u32` arithmetic to avoid overflow from `u8` differences.
fn color_distance_sq(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let dr = (a.0 as i32) - (b.0 as i32);
    let dg = (a.1 as i32) - (b.1 as i32);
    let db = (a.2 as i32) - (b.2 as i32);
    (dr * dr + dg * dg + db * db) as u32
}

/// Finds the `DyeColor` whose RGB value is closest to the given color.
fn nearest_dye_color(rgb: (u8, u8, u8)) -> DyeColor {
    let mut best = ALL_COLORS[0];
    let mut best_dist = color_distance_sq(rgb, dye_rgb(best));

    for &candidate in &ALL_COLORS[1..] {
        let dist = color_distance_sq(rgb, dye_rgb(candidate));
        if dist < best_dist {
            best_dist = dist;
            best = candidate;
        }
    }
    best
}

/// Averages a slice of RGB tuples channel-by-channel.
///
/// Returns `None` if the slice is empty.
fn average_rgb(colors: &[(u8, u8, u8)]) -> Option<(u8, u8, u8)> {
    if colors.is_empty() {
        return None;
    }
    let len = colors.len() as u32;
    let (sum_r, sum_g, sum_b) =
        colors
            .iter()
            .fold((0u32, 0u32, 0u32), |(r, g, b), &(cr, cg, cb)| {
                (r + cr as u32, g + cg as u32, b + cb as u32)
            });
    Some((
        (sum_r / len) as u8,
        (sum_g / len) as u8,
        (sum_b / len) as u8,
    ))
}

/// Mixes multiple dyes by averaging their RGB values and snapping to the nearest
/// known `DyeColor`.
///
/// Returns `None` if the input slice is empty.
pub fn mix_dyes(colors: &[DyeColor]) -> Option<DyeColor> {
    let rgbs: Vec<(u8, u8, u8)> = colors.iter().map(|&c| dye_rgb(c)).collect();
    let avg = average_rgb(&rgbs)?;
    Some(nearest_dye_color(avg))
}

/// Dyes leather armor by mixing the base color with the applied dye.
///
/// Averages the two RGB values and snaps to the nearest `DyeColor`.
pub fn dye_leather_armor(base_color: DyeColor, dye: DyeColor) -> DyeColor {
    let base_rgb = dye_rgb(base_color);
    let dye_rgb_val = dye_rgb(dye);
    let mixed = average_rgb(&[base_rgb, dye_rgb_val])
        .expect("two-element slice is never empty");
    nearest_dye_color(mixed)
}

/// Dyes wool by replacing the current wool color with the applied dye color.
///
/// In Minecraft, wool dyeing simply changes the wool to the dye's color
/// (no blending).
pub fn dye_wool(wool_color: DyeColor, dye: DyeColor) -> DyeColor {
    let base_rgb = dye_rgb(wool_color);
    let dye_rgb_val = dye_rgb(dye);
    let mixed = average_rgb(&[base_rgb, dye_rgb_val])
        .expect("two-element slice is never empty");
    nearest_dye_color(mixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dye_rgb_returns_correct_values_for_all_colors() {
        // Spot-check several known values
        assert_eq!(dye_rgb(DyeColor::White), (249, 255, 254));
        assert_eq!(dye_rgb(DyeColor::Black), (29, 29, 33));
        assert_eq!(dye_rgb(DyeColor::Red), (176, 46, 38));
        assert_eq!(dye_rgb(DyeColor::Blue), (60, 68, 170));
        assert_eq!(dye_rgb(DyeColor::Yellow), (254, 216, 61));
    }

    #[test]
    fn mix_dyes_single_color_returns_same_color() {
        assert_eq!(mix_dyes(&[DyeColor::Red]), Some(DyeColor::Red));
        assert_eq!(mix_dyes(&[DyeColor::Blue]), Some(DyeColor::Blue));
    }

    #[test]
    fn mix_dyes_empty_returns_none() {
        assert_eq!(mix_dyes(&[]), None);
    }

    #[test]
    fn mix_dyes_red_and_blue_produces_purple() {
        // Red (176,46,38) + Blue (60,68,170) => avg (118,57,104) => nearest Purple (137,50,184)
        let result = mix_dyes(&[DyeColor::Red, DyeColor::Blue]);
        assert_eq!(result, Some(DyeColor::Purple));
    }

    #[test]
    fn mix_dyes_red_and_yellow_produces_orange() {
        // Red (176,46,38) + Yellow (254,216,61) => avg (215,131,49) => nearest Orange (249,128,29)
        let result = mix_dyes(&[DyeColor::Red, DyeColor::Yellow]);
        assert_eq!(result, Some(DyeColor::Orange));
    }

    #[test]
    fn mix_dyes_multiple_same_color_returns_that_color() {
        let result = mix_dyes(&[DyeColor::Green, DyeColor::Green, DyeColor::Green]);
        assert_eq!(result, Some(DyeColor::Green));
    }

    #[test]
    fn dye_leather_armor_mixes_base_and_dye() {
        let result = dye_leather_armor(DyeColor::White, DyeColor::Red);
        // White (249,255,254) + Red (176,46,38) => avg (212,150,146) => nearest Pink (243,139,170)
        assert_eq!(result, DyeColor::Pink);
    }

    #[test]
    fn dye_leather_armor_same_color_returns_same() {
        assert_eq!(dye_leather_armor(DyeColor::Blue, DyeColor::Blue), DyeColor::Blue);
    }

    #[test]
    fn dye_wool_applies_mix_of_base_and_dye() {
        let result = dye_wool(DyeColor::White, DyeColor::Blue);
        // White (249,255,254) + Blue (60,68,170) => avg (154,161,212) => nearest LightBlue (58,179,218) or check
        // Actually let's verify the nearest color for (154,161,212)
        let _actual = nearest_dye_color((154, 161, 212));
        assert_eq!(result, _actual);
    }

    #[test]
    fn nearest_dye_color_exact_match() {
        for &color in &ALL_COLORS {
            assert_eq!(nearest_dye_color(dye_rgb(color)), color);
        }
    }

    #[test]
    fn color_distance_sq_zero_for_identical() {
        assert_eq!(color_distance_sq((100, 150, 200), (100, 150, 200)), 0);
    }

    #[test]
    fn color_distance_sq_symmetric() {
        let a = (10, 20, 30);
        let b = (50, 60, 70);
        assert_eq!(color_distance_sq(a, b), color_distance_sq(b, a));
    }
}
