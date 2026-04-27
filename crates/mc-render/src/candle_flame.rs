//! Candle flame particle effects for 1–4 candles per block.

/// Maximum number of candles that can be placed on a single block.
pub const MAX_CANDLES: u8 = 4;

/// Represents the flame state for a candle block.
#[derive(Debug, Clone)]
pub struct CandleFlame {
    pub pos: [f32; 3],
    pub flicker_phase: f32,
    pub count: u8,
}

/// Returns flame positions on top of the candle block for 1–4 candles.
///
/// Positions are offset from `base` to spread candles across the block top.
pub fn candle_flame_positions(base: [f32; 3], count: u8) -> Vec<[f32; 3]> {
    let count = count.clamp(1, MAX_CANDLES);
    let offsets: &[[f32; 2]] = match count {
        1 => &[[0.0, 0.0]],
        2 => &[[-0.1, 0.0], [0.1, 0.0]],
        3 => &[[-0.1, -0.1], [0.1, -0.1], [0.0, 0.1]],
        4 => &[[-0.1, -0.1], [0.1, -0.1], [-0.1, 0.1], [0.1, 0.1]],
        _ => unreachable!(),
    };
    offsets
        .iter()
        .map(|[dx, dz]| [base[0] + dx, base[1] + 0.5, base[2] + dz])
        .collect()
}

/// Computes a flicker intensity oscillating between 0.8 and 1.0.
pub fn flame_flicker(phase: f32, time: f32) -> f32 {
    0.9 + 0.1 * (phase + time * 6.0).sin()
}

/// Returns the light level for the given candle count (3 per candle, max 12).
pub fn candle_light_level(count: u8) -> u8 {
    (count.clamp(1, MAX_CANDLES) as u16 * 3).min(12) as u8
}

/// Returns an RGB color tint for the given dye index (0–15).
///
/// Indices follow the standard Minecraft dye color order.
pub fn candle_color_tint(dye: u8) -> [f32; 3] {
    match dye {
        0 => [1.0, 1.0, 1.0],       // white
        1 => [1.0, 0.5, 0.0],       // orange
        2 => [0.8, 0.3, 0.8],       // magenta
        3 => [0.4, 0.6, 1.0],       // light blue
        4 => [1.0, 1.0, 0.2],       // yellow
        5 => [0.4, 1.0, 0.2],       // lime
        6 => [1.0, 0.6, 0.7],       // pink
        7 => [0.3, 0.3, 0.3],       // gray
        8 => [0.6, 0.6, 0.6],       // light gray
        9 => [0.2, 0.6, 0.6],       // cyan
        10 => [0.5, 0.2, 0.8],      // purple
        11 => [0.2, 0.2, 0.8],      // blue
        12 => [0.4, 0.25, 0.1],     // brown
        13 => [0.2, 0.4, 0.1],      // green
        14 => [0.8, 0.2, 0.2],      // red
        15 => [0.05, 0.05, 0.05],   // black
        _ => [1.0, 1.0, 1.0],       // default to white
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_count_matches_candle_count() {
        for count in 1..=MAX_CANDLES {
            let positions = candle_flame_positions([0.0, 0.0, 0.0], count);
            assert_eq!(positions.len(), count as usize);
        }
    }

    #[test]
    fn positions_clamp_to_valid_range() {
        assert_eq!(candle_flame_positions([0.0, 0.0, 0.0], 0).len(), 1);
        assert_eq!(candle_flame_positions([0.0, 0.0, 0.0], 5).len(), 4);
    }

    #[test]
    fn positions_y_offset_above_base() {
        let base = [1.0, 2.0, 3.0];
        for pos in candle_flame_positions(base, 2) {
            assert_eq!(pos[1], 2.5);
        }
    }

    #[test]
    fn flicker_stays_in_range() {
        for t in 0..100 {
            let time = t as f32 * 0.1;
            let v = flame_flicker(0.0, time);
            assert!(v >= 0.8 - f32::EPSILON && v <= 1.0 + f32::EPSILON,
                "flicker {v} out of range at time {time}");
        }
    }

    #[test]
    fn flicker_with_phase_offset() {
        let a = flame_flicker(0.0, 1.0);
        let b = flame_flicker(1.0, 1.0);
        assert!((a - b).abs() > f32::EPSILON, "phase should affect output");
    }

    #[test]
    fn light_level_scales_with_count() {
        assert_eq!(candle_light_level(1), 3);
        assert_eq!(candle_light_level(2), 6);
        assert_eq!(candle_light_level(3), 9);
        assert_eq!(candle_light_level(4), 12);
    }

    #[test]
    fn light_level_clamps() {
        assert_eq!(candle_light_level(0), 3);
        assert_eq!(candle_light_level(10), 12);
    }

    #[test]
    fn color_tint_returns_valid_colors() {
        for dye in 0..=15 {
            let [r, g, b] = candle_color_tint(dye);
            assert!((0.0..=1.0).contains(&r));
            assert!((0.0..=1.0).contains(&g));
            assert!((0.0..=1.0).contains(&b));
        }
    }

    #[test]
    fn color_tint_unknown_dye_defaults_to_white() {
        assert_eq!(candle_color_tint(16), [1.0, 1.0, 1.0]);
        assert_eq!(candle_color_tint(255), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn candle_flame_struct_fields() {
        let flame = CandleFlame {
            pos: [1.0, 2.0, 3.0],
            flicker_phase: 0.5,
            count: 3,
        };
        assert_eq!(flame.pos, [1.0, 2.0, 3.0]);
        assert_eq!(flame.flicker_phase, 0.5);
        assert_eq!(flame.count, 3);
    }

    #[test]
    fn max_candles_is_four() {
        assert_eq!(MAX_CANDLES, 4);
    }
}
