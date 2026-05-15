//! Block face tint colors based on block type and biome.

/// Returns true if the given block ID requires biome-dependent tinting.
pub fn needs_tint(block_id: u16) -> bool {
    matches!(block_id, 2 | 18 | 106 | 111 | 161)
}

/// Grass tint color for a given biome ID.
pub fn grass_tint_for_biome(biome: u8) -> [f32; 3] {
    match biome {
        0 => [0.56, 0.74, 0.35],  // plains
        1 => [0.75, 0.72, 0.42],  // desert
        2 => [0.42, 0.64, 0.23],  // forest
        3 => [0.50, 0.70, 0.30],  // taiga
        4 => [0.41, 0.71, 0.31],  // swamp
        5 => [0.53, 0.76, 0.33],  // river
        6 => [0.74, 0.72, 0.42],  // nether (same as desert, unused)
        7 => [0.56, 0.74, 0.35],  // the end (fallback)
        8 => [0.68, 0.72, 0.44],  // snowy tundra
        9 => [0.48, 0.68, 0.28],  // mushroom island
        10 => [0.53, 0.76, 0.38], // jungle
        11 => [0.55, 0.75, 0.35], // beach
        12 => [0.62, 0.69, 0.30], // savanna
        13 => [0.49, 0.66, 0.27], // dark forest
        14 => [0.44, 0.63, 0.25], // birch forest
        15 => [0.55, 0.73, 0.35], // meadow
        _ => [0.56, 0.74, 0.35],  // default (plains)
    }
}

/// Foliage (leaves/vines) tint color for a given biome ID.
pub fn foliage_tint_for_biome(biome: u8) -> [f32; 3] {
    match biome {
        0 => [0.47, 0.65, 0.24],  // plains
        1 => [0.68, 0.65, 0.34],  // desert
        2 => [0.35, 0.56, 0.16],  // forest
        3 => [0.42, 0.62, 0.22],  // taiga
        4 => [0.41, 0.60, 0.26],  // swamp
        5 => [0.45, 0.67, 0.24],  // river
        8 => [0.60, 0.64, 0.36],  // snowy tundra
        9 => [0.40, 0.60, 0.20],  // mushroom island
        10 => [0.45, 0.68, 0.30], // jungle
        12 => [0.54, 0.61, 0.22], // savanna
        13 => [0.41, 0.58, 0.19], // dark forest
        14 => [0.36, 0.55, 0.17], // birch forest
        15 => [0.47, 0.65, 0.26], // meadow
        _ => [0.47, 0.65, 0.24],  // default (plains)
    }
}

/// Water tint color for a given biome ID.
pub fn water_tint_for_biome(biome: u8) -> [f32; 3] {
    match biome {
        0 => [0.24, 0.45, 0.87],  // plains
        1 => [0.24, 0.45, 0.62],  // desert (murky)
        2 => [0.24, 0.45, 0.87],  // forest
        3 => [0.22, 0.40, 0.72],  // taiga (darker)
        4 => [0.38, 0.50, 0.42],  // swamp (greenish)
        5 => [0.24, 0.45, 0.87],  // river
        8 => [0.22, 0.38, 0.68],  // snowy tundra (cold blue)
        9 => [0.24, 0.45, 0.87],  // mushroom island
        10 => [0.20, 0.42, 0.62], // jungle (tropical)
        11 => [0.24, 0.50, 0.92], // beach (bright)
        12 => [0.30, 0.48, 0.62], // savanna (warm)
        _ => [0.24, 0.45, 0.87],  // default (plains)
    }
}

/// Returns the tint color for a specific block face.
///
/// For blocks that don't need tinting, returns white `[1.0, 1.0, 1.0]`.
/// For grass blocks (id=2), only the top face (face=2) and side faces get tinted.
/// For leaves, vines, and lily pads, all faces are tinted with foliage color.
pub fn block_face_tint(block_id: u16, face: u8, biome: u8) -> [f32; 3] {
    const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

    match block_id {
        2 => {
            // Grass block: top face (2) gets grass tint, side faces get grass tint,
            // bottom face (3) stays white
            if face == 3 {
                WHITE
            } else {
                grass_tint_for_biome(biome)
            }
        }
        18 | 161 => foliage_tint_for_biome(biome), // leaves
        106 => foliage_tint_for_biome(biome),       // vines
        111 => foliage_tint_for_biome(biome),       // lily pad
        _ => WHITE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_tint_positive() {
        assert!(needs_tint(2));   // grass
        assert!(needs_tint(18));  // leaves
        assert!(needs_tint(161)); // acacia leaves
        assert!(needs_tint(106)); // vines
        assert!(needs_tint(111)); // lily pad
    }

    #[test]
    fn test_needs_tint_negative() {
        assert!(!needs_tint(0));  // air
        assert!(!needs_tint(1));  // stone
        assert!(!needs_tint(4));  // cobblestone
        assert!(!needs_tint(9));  // water
        assert!(!needs_tint(255));
    }

    #[test]
    fn test_grass_tint_plains() {
        let tint = grass_tint_for_biome(0);
        assert_eq!(tint, [0.56, 0.74, 0.35]);
    }

    #[test]
    fn test_grass_tint_desert_differs_from_plains() {
        let plains = grass_tint_for_biome(0);
        let desert = grass_tint_for_biome(1);
        assert_ne!(plains, desert);
    }

    #[test]
    fn test_grass_tint_unknown_biome_returns_default() {
        let default_tint = grass_tint_for_biome(0);
        let unknown = grass_tint_for_biome(200);
        assert_eq!(default_tint, unknown);
    }

    #[test]
    fn test_foliage_tint_forest() {
        let tint = foliage_tint_for_biome(2);
        assert_eq!(tint, [0.35, 0.56, 0.16]);
    }

    #[test]
    fn test_water_tint_swamp_is_greenish() {
        let tint = water_tint_for_biome(4);
        // Swamp water has higher green relative to blue
        assert!(tint[1] > tint[2]);
    }

    #[test]
    fn test_block_face_tint_non_tinted_block() {
        let tint = block_face_tint(1, 0, 0); // stone, any face, any biome
        assert_eq!(tint, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_block_face_tint_grass_top() {
        let tint = block_face_tint(2, 2, 0); // grass, top face, plains
        assert_eq!(tint, grass_tint_for_biome(0));
    }

    #[test]
    fn test_block_face_tint_grass_bottom_is_white() {
        let tint = block_face_tint(2, 3, 0); // grass, bottom face
        assert_eq!(tint, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_block_face_tint_grass_side_is_tinted() {
        let tint = block_face_tint(2, 0, 0); // grass, side face, plains
        assert_eq!(tint, grass_tint_for_biome(0));
    }

    #[test]
    fn test_block_face_tint_leaves_all_faces() {
        for face in 0..6 {
            let tint = block_face_tint(18, face, 2); // oak leaves, forest
            assert_eq!(tint, foliage_tint_for_biome(2));
        }
    }

    #[test]
    fn test_block_face_tint_vines() {
        let tint = block_face_tint(106, 0, 10); // vines, jungle
        assert_eq!(tint, foliage_tint_for_biome(10));
    }

    #[test]
    fn test_block_face_tint_lily_pad() {
        let tint = block_face_tint(111, 2, 4); // lily pad, swamp
        assert_eq!(tint, foliage_tint_for_biome(4));
    }
}
