//! Mipmap level calculation and LOD utilities.

/// Returns the number of mipmap levels for a texture of the given dimensions.
/// Result includes the base level (level 0).
pub fn mipmap_levels(width: u32, height: u32) -> u32 {
    if width == 0 || height == 0 {
        return 0;
    }
    let max_dim = width.max(height);
    max_dim.ilog2() + 1
}

/// Returns the (width, height) of a mipmap at the given level.
/// Each level halves both dimensions, with a minimum of 1.
pub fn mipmap_size(base_w: u32, base_h: u32, level: u32) -> (u32, u32) {
    let w = (base_w >> level).max(1);
    let h = (base_h >> level).max(1);
    (w, h)
}

/// Returns a LOD bias in [0.0, max] based on distance.
/// Closer distances yield lower bias (sharper textures).
pub fn lod_bias_for_distance(distance: f32, max: f32) -> f32 {
    if distance <= 0.0 || max <= 0.0 {
        return 0.0;
    }
    // Logarithmic curve: increases quickly at short range, flattens at long range
    let bias = (1.0 + distance).ln() / (1.0 + 128.0_f32).ln() * max;
    bias.clamp(0.0, max)
}

/// Returns the recommended anisotropic filtering level (1..=16) based on distance.
/// Nearby surfaces get higher anisotropy for sharper textures at oblique angles.
pub fn anisotropic_level(distance: f32) -> u8 {
    if distance <= 8.0 {
        16
    } else if distance <= 32.0 {
        8
    } else if distance <= 64.0 {
        4
    } else if distance <= 128.0 {
        2
    } else {
        1
    }
}

/// Returns whether mipmapping should be used at the given distance.
/// Very close surfaces do not need mipmapping.
pub fn should_use_mipmap(distance: f32) -> bool {
    distance > 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mipmap_levels_power_of_two() {
        assert_eq!(mipmap_levels(256, 256), 9);
        assert_eq!(mipmap_levels(1, 1), 1);
        assert_eq!(mipmap_levels(1024, 1024), 11);
    }

    #[test]
    fn mipmap_levels_non_power_of_two() {
        assert_eq!(mipmap_levels(300, 200), 9); // max=300, ilog2(300)=8, +1=9
        assert_eq!(mipmap_levels(1, 512), 10);
    }

    #[test]
    fn mipmap_levels_zero_dimensions() {
        assert_eq!(mipmap_levels(0, 256), 0);
        assert_eq!(mipmap_levels(256, 0), 0);
        assert_eq!(mipmap_levels(0, 0), 0);
    }

    #[test]
    fn mipmap_size_at_levels() {
        assert_eq!(mipmap_size(256, 128, 0), (256, 128));
        assert_eq!(mipmap_size(256, 128, 1), (128, 64));
        assert_eq!(mipmap_size(256, 128, 7), (2, 1));
        assert_eq!(mipmap_size(256, 128, 8), (1, 1));
        // Beyond max level, clamps to 1x1
        assert_eq!(mipmap_size(256, 128, 20), (1, 1));
    }

    #[test]
    fn lod_bias_zero_at_zero_distance() {
        assert_eq!(lod_bias_for_distance(0.0, 4.0), 0.0);
        assert_eq!(lod_bias_for_distance(-5.0, 4.0), 0.0);
    }

    #[test]
    fn lod_bias_clamped_to_max() {
        let bias = lod_bias_for_distance(1000.0, 2.0);
        assert!(bias <= 2.0);
    }

    #[test]
    fn lod_bias_increases_with_distance() {
        let near = lod_bias_for_distance(5.0, 4.0);
        let far = lod_bias_for_distance(50.0, 4.0);
        assert!(far > near);
    }

    #[test]
    fn lod_bias_zero_max() {
        assert_eq!(lod_bias_for_distance(10.0, 0.0), 0.0);
    }

    #[test]
    fn anisotropic_level_near() {
        assert_eq!(anisotropic_level(1.0), 16);
        assert_eq!(anisotropic_level(8.0), 16);
    }

    #[test]
    fn anisotropic_level_mid() {
        assert_eq!(anisotropic_level(20.0), 8);
        assert_eq!(anisotropic_level(50.0), 4);
    }

    #[test]
    fn anisotropic_level_far() {
        assert_eq!(anisotropic_level(100.0), 2);
        assert_eq!(anisotropic_level(200.0), 1);
    }

    #[test]
    fn should_use_mipmap_close() {
        assert!(!should_use_mipmap(0.5));
        assert!(!should_use_mipmap(2.0));
    }

    #[test]
    fn should_use_mipmap_far() {
        assert!(should_use_mipmap(3.0));
        assert!(should_use_mipmap(100.0));
    }
}
