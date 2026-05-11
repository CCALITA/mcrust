/// Banner layer rendering: pattern UV mapping, dye color lookup,
/// and model geometry constants for wall and standing banners.

/// Maximum number of pattern layers a banner can hold.
pub const MAX_BANNER_LAYERS: usize = 6;

/// A single pattern layer applied to a banner.
pub struct BannerLayer {
    /// Pattern index (0–25 matching Minecraft banner patterns).
    pub pattern: u8,
    /// Dye color index (0–15).
    pub color: u8,
}

/// A banner with a base color and up to [`MAX_BANNER_LAYERS`] pattern layers.
pub struct Banner {
    /// Dye color index for the base cloth (0–15).
    pub base_color: u8,
    /// Ordered pattern layers from bottom to top.
    pub layers: Vec<BannerLayer>,
}

/// Returns the UV origin `(u, v)` on the banner pattern atlas for the given
/// pattern index. Patterns are arranged in a 4-column grid with each cell
/// spanning 0.25 × 0.25 in UV space.
pub fn banner_pattern_uv(pattern: u8) -> (f32, f32) {
    let col = (pattern % 4) as f32;
    let row = (pattern / 4) as f32;
    (col * 0.25, row * 0.25)
}

/// Maps a Minecraft dye color index (0–15) to an RGB triplet.
pub fn banner_color_rgb(color: u8) -> [f32; 3] {
    match color {
        0 => [0.10, 0.10, 0.10],  // black
        1 => [0.15, 0.09, 0.03],  // brown
        2 => [0.60, 0.10, 0.10],  // red
        3 => [0.90, 0.45, 0.10],  // orange
        4 => [0.90, 0.90, 0.15],  // yellow
        5 => [0.30, 0.60, 0.10],  // lime
        6 => [0.10, 0.50, 0.10],  // green
        7 => [0.10, 0.60, 0.60],  // cyan
        8 => [0.20, 0.40, 0.80],  // light blue
        9 => [0.10, 0.10, 0.70],  // blue
        10 => [0.50, 0.25, 0.80], // purple
        11 => [0.70, 0.30, 0.70], // magenta
        12 => [0.85, 0.55, 0.65], // pink
        13 => [0.60, 0.60, 0.60], // light gray
        14 => [0.35, 0.35, 0.35], // gray
        15 => [0.95, 0.95, 0.95], // white
        _ => [1.0, 1.0, 1.0],
    }
}

/// Returns the height of a standing banner model in blocks.
pub fn banner_model_height() -> f32 {
    2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_uv_first() {
        let (u, v) = banner_pattern_uv(0);
        assert!((u - 0.0).abs() < 1e-6);
        assert!((v - 0.0).abs() < 1e-6);
    }

    #[test]
    fn pattern_uv_wraps_columns() {
        let (u, v) = banner_pattern_uv(5);
        // pattern 5 => col=1, row=1
        assert!((u - 0.25).abs() < 1e-6);
        assert!((v - 0.25).abs() < 1e-6);
    }

    #[test]
    fn color_rgb_known_values() {
        let black = banner_color_rgb(0);
        assert!((black[0] - 0.10).abs() < 1e-6);
        let white = banner_color_rgb(15);
        assert!((white[0] - 0.95).abs() < 1e-6);
    }

    #[test]
    fn color_rgb_out_of_range_returns_white() {
        let c = banner_color_rgb(255);
        assert_eq!(c, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn model_height_is_two() {
        assert!((banner_model_height() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn max_layers_is_six() {
        assert_eq!(MAX_BANNER_LAYERS, 6);
    }

    #[test]
    fn banner_respects_max_layers() {
        let banner = Banner {
            base_color: 15,
            layers: (0..MAX_BANNER_LAYERS)
                .map(|i| BannerLayer {
                    pattern: i as u8,
                    color: i as u8,
                })
                .collect(),
        };
        assert_eq!(banner.layers.len(), MAX_BANNER_LAYERS);
    }
}
