//! Snow layer rendering: geometry and biome support.

/// Maximum number of snow layers per block.
pub const MAX_SNOW_LAYERS: u8 = 8;

/// A snow layer block with position and layer count.
pub struct SnowLayer {
    pub layers: u8,
    pub pos: [f32; 3],
}

/// Returns the height of snow given the number of layers (0.0 to 1.0).
pub fn snow_layer_height(layers: u8) -> f32 {
    layers as f32 / 8.0
}

/// Returns an AABB collision box [min_x, min_y, min_z, max_x, max_y, max_z] for the snow layers.
pub fn snow_collision_box(layers: u8) -> [f32; 6] {
    let height = snow_layer_height(layers);
    [0.0, 0.0, 0.0, 1.0, height, 1.0]
}

/// Returns the 4 top-face vertices at the snow layer height.
pub fn snow_top_vertices(pos: [f32; 3], layers: u8) -> Vec<[f32; 3]> {
    let height = snow_layer_height(layers);
    let y = pos[1] + height;
    vec![
        [pos[0], y, pos[2]],
        [pos[0] + 1.0, y, pos[2]],
        [pos[0] + 1.0, y, pos[2] + 1.0],
        [pos[0], y, pos[2] + 1.0],
    ]
}

/// Returns whether the given biome supports snow accumulation.
pub fn biome_supports_snow(biome_id: u8) -> bool {
    // Cold biomes: snowy_plains(12), ice_spikes(13), snowy_taiga(30),
    // frozen_river(11), snowy_beach(26), frozen_ocean(10), taiga(5)
    matches!(biome_id, 5 | 10 | 11 | 12 | 13 | 26 | 30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snow_layer_height() {
        assert_eq!(snow_layer_height(0), 0.0);
        assert_eq!(snow_layer_height(4), 0.5);
        assert_eq!(snow_layer_height(8), 1.0);
    }

    #[test]
    fn test_snow_collision_box() {
        let bbox = snow_collision_box(4);
        assert_eq!(bbox, [0.0, 0.0, 0.0, 1.0, 0.5, 1.0]);
    }

    #[test]
    fn test_snow_top_vertices() {
        let verts = snow_top_vertices([1.0, 2.0, 3.0], 8);
        assert_eq!(verts.len(), 4);
        assert_eq!(verts[0], [1.0, 3.0, 3.0]);
        assert_eq!(verts[1], [2.0, 3.0, 3.0]);
        assert_eq!(verts[2], [2.0, 3.0, 4.0]);
        assert_eq!(verts[3], [1.0, 3.0, 4.0]);
    }

    #[test]
    fn test_biome_supports_snow() {
        assert!(biome_supports_snow(12));
        assert!(biome_supports_snow(30));
        assert!(!biome_supports_snow(1));
        assert!(!biome_supports_snow(0));
    }

    #[test]
    fn test_snow_layer_struct() {
        let layer = SnowLayer {
            layers: 3,
            pos: [0.0, 64.0, 0.0],
        };
        assert_eq!(layer.layers, 3);
        assert_eq!(layer.pos, [0.0, 64.0, 0.0]);
    }
}
