//! Debug visualization colors for rendering overlays.

/// Returns a highlight color for chunk borders.
/// Yellow if on a border, transparent gray otherwise.
pub fn chunk_border_color(on_border: bool) -> [f32; 3] {
    if on_border {
        [1.0, 1.0, 0.0]
    } else {
        [0.5, 0.5, 0.5]
    }
}

/// Maps a light level (0–15) to a color gradient from red (0) to green (15).
pub fn light_level_color(level: u8) -> [f32; 3] {
    let t = (level.min(15) as f32) / 15.0;
    [1.0 - t, t, 0.0]
}

/// Returns a unique debug color for each biome ID using a hash-based palette.
pub fn biome_debug_color(biome_id: u8) -> [f32; 3] {
    // Golden-ratio-based hue distribution for visually distinct colors
    let hue = ((biome_id as f32) * 0.618_034) % 1.0;
    hsv_to_rgb(hue, 0.7, 0.9)
}

/// Maps an ambient occlusion value (0.0 = full occlusion, 1.0 = none) to a
/// grayscale color from black to white.
pub fn ao_debug_color(ao: f32) -> [f32; 3] {
    let v = ao.clamp(0.0, 1.0);
    [v, v, v]
}

/// Maps a normalized depth value (0.0 = near, 1.0 = far) from red to blue.
pub fn depth_debug_color(depth: f32) -> [f32; 3] {
    let t = depth.clamp(0.0, 1.0);
    [1.0 - t, 0.0, t]
}

/// Converts HSV (all in 0.0–1.0) to RGB.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_border_on() {
        assert_eq!(chunk_border_color(true), [1.0, 1.0, 0.0]);
    }

    #[test]
    fn chunk_border_off() {
        assert_eq!(chunk_border_color(false), [0.5, 0.5, 0.5]);
    }

    #[test]
    fn light_level_zero_is_red() {
        let c = light_level_color(0);
        assert_eq!(c, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn light_level_max_is_green() {
        let c = light_level_color(15);
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn light_level_clamps_above_15() {
        let c = light_level_color(255);
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn light_level_mid() {
        let c = light_level_color(7);
        assert!(c[0] > 0.4 && c[0] < 0.6);
        assert!(c[1] > 0.4 && c[1] < 0.6);
    }

    #[test]
    fn biome_colors_are_distinct() {
        let c0 = biome_debug_color(0);
        let c1 = biome_debug_color(1);
        let c2 = biome_debug_color(2);
        assert_ne!(c0, c1);
        assert_ne!(c1, c2);
        assert_ne!(c0, c2);
    }

    #[test]
    fn biome_colors_in_range() {
        for id in 0..=255 {
            let c = biome_debug_color(id);
            for ch in &c {
                assert!(*ch >= 0.0 && *ch <= 1.0, "biome {id}: {c:?}");
            }
        }
    }

    #[test]
    fn ao_full_occlusion_is_black() {
        assert_eq!(ao_debug_color(0.0), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn ao_no_occlusion_is_white() {
        assert_eq!(ao_debug_color(1.0), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn ao_clamps() {
        assert_eq!(ao_debug_color(-1.0), [0.0, 0.0, 0.0]);
        assert_eq!(ao_debug_color(2.0), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn depth_near_is_red() {
        assert_eq!(depth_debug_color(0.0), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn depth_far_is_blue() {
        assert_eq!(depth_debug_color(1.0), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn depth_clamps() {
        assert_eq!(depth_debug_color(-0.5), [1.0, 0.0, 0.0]);
        assert_eq!(depth_debug_color(1.5), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn depth_mid() {
        let c = depth_debug_color(0.5);
        assert!((c[0] - 0.5).abs() < 0.01);
        assert!((c[2] - 0.5).abs() < 0.01);
    }
}
