//! Texture atlas coordinate computation for block faces.
//!
//! Given a block ID and face index, computes the UV coordinates within the
//! texture atlas. The atlas is a square `ATLAS_SIZE x ATLAS_SIZE` pixel sheet
//! tiled with `TILE_SIZE x TILE_SIZE` pixel sprites.

/// Side length of the square texture atlas in pixels.
pub const ATLAS_SIZE: u32 = 256;

/// Side length of a single tile in pixels.
pub const TILE_SIZE: u32 = 16;

/// UV rectangle within the texture atlas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasCoords {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

/// Number of tiles that fit along one row of the atlas.
pub const fn tiles_per_row() -> u32 {
    ATLAS_SIZE / TILE_SIZE
}

/// Number of unique face offsets used per block.
/// Each block has 6 faces, but we collapse opposite pairs to 3 texture slots
/// (top/bottom, front/back, left/right) for a simple face-offset mapping.
const FACE_OFFSETS: u8 = 6;

/// Compute the atlas UV coordinates for a given block face.
///
/// `block_id` — numeric block identifier (0-based).
/// `face` — face index in `0..6` (top, bottom, north, south, east, west).
///
/// The tile index is `block_id * FACE_OFFSETS + face`, laid out left-to-right,
/// top-to-bottom in the atlas.
pub fn block_texture_coords(block_id: u16, face: u8) -> AtlasCoords {
    let face_offset = (face % FACE_OFFSETS) as u32;
    let index = block_id as u32 * FACE_OFFSETS as u32 + face_offset;
    let tpr = tiles_per_row();

    let col = index % tpr;
    let row = index / tpr;

    let atlas_f = ATLAS_SIZE as f32;
    let tile_f = TILE_SIZE as f32;

    AtlasCoords {
        u_min: (col as f32 * tile_f) / atlas_f,
        v_min: (row as f32 * tile_f) / atlas_f,
        u_max: ((col + 1) as f32 * tile_f) / atlas_f,
        v_max: ((row + 1) as f32 * tile_f) / atlas_f,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_per_row_is_16() {
        assert_eq!(tiles_per_row(), 16);
    }

    #[test]
    fn first_block_first_face_is_top_left() {
        let coords = block_texture_coords(0, 0);
        assert!((coords.u_min - 0.0).abs() < f32::EPSILON);
        assert!((coords.v_min - 0.0).abs() < f32::EPSILON);
        assert!((coords.u_max - 1.0 / 16.0).abs() < f32::EPSILON);
        assert!((coords.v_max - 1.0 / 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn second_face_of_first_block() {
        let coords = block_texture_coords(0, 1);
        // tile index = 1 → col=1, row=0
        assert!((coords.u_min - 1.0 / 16.0).abs() < f32::EPSILON);
        assert!((coords.v_min - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn face_wraps_at_face_offsets() {
        // face=6 should wrap to face=0
        let wrapped = block_texture_coords(0, 6);
        let original = block_texture_coords(0, 0);
        assert_eq!(wrapped, original);
    }

    #[test]
    fn second_block_starts_after_six_tiles() {
        // block_id=1, face=0 → index=6 → col=6, row=0
        let coords = block_texture_coords(1, 0);
        assert!((coords.u_min - 6.0 / 16.0).abs() < f32::EPSILON);
        assert!((coords.v_min - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tile_wraps_to_next_row() {
        // block_id=2, face=4 → index=16 → col=0, row=1
        let coords = block_texture_coords(2, 4);
        assert!((coords.u_min - 0.0).abs() < f32::EPSILON);
        assert!((coords.v_min - 1.0 / 16.0).abs() < f32::EPSILON);
        assert!((coords.u_max - 1.0 / 16.0).abs() < f32::EPSILON);
        assert!((coords.v_max - 2.0 / 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn coords_are_normalized_zero_to_one() {
        for block_id in 0..10_u16 {
            for face in 0..6_u8 {
                let c = block_texture_coords(block_id, face);
                assert!(c.u_min >= 0.0 && c.u_min <= 1.0, "u_min out of range");
                assert!(c.v_min >= 0.0, "v_min out of range");
                assert!(c.u_max >= 0.0 && c.u_max <= 1.0, "u_max out of range");
                assert!(c.u_max > c.u_min, "u_max must be > u_min");
                assert!(c.v_max > c.v_min, "v_max must be > v_min");
            }
        }
    }

    #[test]
    fn tile_size_evenly_divides_atlas() {
        assert_eq!(ATLAS_SIZE % TILE_SIZE, 0);
    }
}
