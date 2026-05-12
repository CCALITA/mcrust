//! Post-processing color grading for different game environments.

/// Color grading parameters for post-processing adjustments.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorGrading {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub gamma: f32,
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
        }
    }
}

/// Apply color grading to an RGB color (each channel in 0.0..1.0 range).
pub fn apply_color_grading(color: [f32; 3], grading: &ColorGrading) -> [f32; 3] {
    let mut result = [0.0f32; 3];
    for i in 0..3 {
        // Gamma correction
        let mut c = color[i].powf(1.0 / grading.gamma);
        // Brightness
        c *= grading.brightness;
        // Contrast (around midpoint 0.5)
        c = (c - 0.5) * grading.contrast + 0.5;
        result[i] = c;
    }
    // Saturation (desaturate toward luminance)
    let luminance = 0.2126 * result[0] + 0.7152 * result[1] + 0.0722 * result[2];
    for i in 0..3 {
        result[i] = luminance + (result[i] - luminance) * grading.saturation;
        result[i] = result[i].clamp(0.0, 1.0);
    }
    result
}

/// Color grading preset for night vision effect.
pub fn night_vision_grading() -> ColorGrading {
    ColorGrading {
        brightness: 1.5,
        contrast: 0.8,
        saturation: 0.3,
        gamma: 0.7,
    }
}

/// Color grading preset for underwater environment.
pub fn underwater_grading() -> ColorGrading {
    ColorGrading {
        brightness: 0.8,
        contrast: 0.9,
        saturation: 1.2,
        gamma: 1.1,
    }
}

/// Color grading preset for the Nether dimension.
pub fn nether_grading() -> ColorGrading {
    ColorGrading {
        brightness: 0.9,
        contrast: 1.3,
        saturation: 1.4,
        gamma: 0.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grading_has_neutral_values() {
        let g = ColorGrading::default();
        assert_eq!(g.brightness, 1.0);
        assert_eq!(g.contrast, 1.0);
        assert_eq!(g.saturation, 1.0);
        assert_eq!(g.gamma, 1.0);
    }

    #[test]
    fn default_grading_preserves_color() {
        let color = [0.5, 0.5, 0.5];
        let result = apply_color_grading(color, &ColorGrading::default());
        for i in 0..3 {
            assert!((result[i] - color[i]).abs() < 1e-5, "channel {i} changed");
        }
    }

    #[test]
    fn output_is_clamped_to_valid_range() {
        let bright = ColorGrading {
            brightness: 10.0,
            ..ColorGrading::default()
        };
        let result = apply_color_grading([0.8, 0.9, 1.0], &bright);
        for i in 0..3 {
            assert!(result[i] >= 0.0 && result[i] <= 1.0, "channel {i} out of range: {}", result[i]);
        }
    }

    #[test]
    fn brightness_increases_values() {
        let grading = ColorGrading {
            brightness: 1.5,
            ..ColorGrading::default()
        };
        let base = [0.3, 0.3, 0.3];
        let result = apply_color_grading(base, &grading);
        for i in 0..3 {
            assert!(result[i] > base[i], "channel {i} not brighter");
        }
    }

    #[test]
    fn saturation_zero_produces_grayscale() {
        let grading = ColorGrading {
            saturation: 0.0,
            ..ColorGrading::default()
        };
        let result = apply_color_grading([0.8, 0.2, 0.5], &grading);
        assert!((result[0] - result[1]).abs() < 1e-5);
        assert!((result[1] - result[2]).abs() < 1e-5);
    }

    #[test]
    fn night_vision_preset_values() {
        let g = night_vision_grading();
        assert!(g.brightness > 1.0);
        assert!(g.saturation < 1.0);
    }

    #[test]
    fn underwater_preset_values() {
        let g = underwater_grading();
        assert!(g.brightness < 1.0);
        assert!(g.saturation > 1.0);
    }

    #[test]
    fn nether_preset_values() {
        let g = nether_grading();
        assert!(g.contrast > 1.0);
        assert!(g.saturation > 1.0);
    }

    #[test]
    fn black_stays_dark_with_default() {
        let result = apply_color_grading([0.0, 0.0, 0.0], &ColorGrading::default());
        // After contrast: (0 - 0.5)*1.0 + 0.5 = 0.0 (clamped)
        // Brightness 1.0 * 0.0 = 0.0, then contrast maps to -0.5+0.5=0.0... wait
        // Actually: gamma(0^1)=0, brightness=0*1=0, contrast=(0-0.5)*1+0.5=0.0
        // luminance=0, saturation=0+0*1=0
        for i in 0..3 {
            assert!((result[i]).abs() < 1e-5);
        }
    }
}
