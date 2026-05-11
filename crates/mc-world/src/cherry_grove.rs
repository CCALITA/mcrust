//! Cherry grove biome data.

/// Surface block ID for cherry grove (grass = 2).
pub fn cherry_grove_surface_block() -> u16 {
    2
}

/// Fog color for the cherry grove biome.
pub fn cherry_grove_fog_color() -> [f32; 3] {
    [0.85, 0.7, 0.8]
}

/// Sky color for the cherry grove biome.
pub fn cherry_grove_sky_color() -> [f32; 3] {
    [0.9, 0.75, 0.85]
}

/// Water color for the cherry grove biome.
pub fn cherry_grove_water_color() -> [f32; 3] {
    [0.24, 0.45, 0.75]
}

/// Rate at which cherry tree leaves emit particles.
pub fn cherry_tree_leaf_particle_rate() -> f32 {
    0.05
}

/// Valid height range (min, max) for cherry grove terrain.
pub fn cherry_grove_height_range() -> (i32, i32) {
    (64, 128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_block() {
        assert_eq!(cherry_grove_surface_block(), 2);
    }

    #[test]
    fn test_fog_color() {
        assert_eq!(cherry_grove_fog_color(), [0.85, 0.7, 0.8]);
    }

    #[test]
    fn test_sky_color() {
        assert_eq!(cherry_grove_sky_color(), [0.9, 0.75, 0.85]);
    }

    #[test]
    fn test_water_color() {
        assert_eq!(cherry_grove_water_color(), [0.24, 0.45, 0.75]);
    }

    #[test]
    fn test_leaf_particle_rate() {
        assert_eq!(cherry_tree_leaf_particle_rate(), 0.05);
    }

    #[test]
    fn test_height_range() {
        assert_eq!(cherry_grove_height_range(), (64, 128));
    }
}
