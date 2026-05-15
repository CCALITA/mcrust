//! Texture atlas UV coordinate helpers for blocks and items.

/// Number of tiles per row in the texture atlas.
pub const ATLAS_TILES_PER_ROW: u32 = 16;

/// UV size of a single tile in the atlas (1.0 / 16.0).
pub const ATLAS_TILE_UV_SIZE: f32 = 1.0 / ATLAS_TILES_PER_ROW as f32;

/// UV coordinates for a region within the texture atlas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasUV {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

/// Returns the UV coordinates for a block face in the texture atlas.
///
/// The `block_id` selects the base tile index, and `face` offsets it
/// (e.g. top, bottom, sides may use different tiles). The tile index
/// is mapped to a row and column in a 16x16 grid.
pub fn block_uv(block_id: u16, face: u8) -> AtlasUV {
    let tile_index = block_id as u32 * 6 + face as u32;
    let col = tile_index % ATLAS_TILES_PER_ROW;
    let row = tile_index / ATLAS_TILES_PER_ROW;

    AtlasUV {
        u0: col as f32 * ATLAS_TILE_UV_SIZE,
        v0: row as f32 * ATLAS_TILE_UV_SIZE,
        u1: (col + 1) as f32 * ATLAS_TILE_UV_SIZE,
        v1: (row + 1) as f32 * ATLAS_TILE_UV_SIZE,
    }
}

/// Returns the UV coordinates for an item in the texture atlas.
///
/// Items use a single tile per item, indexed directly by `item_id`.
pub fn item_uv(item_id: u16) -> AtlasUV {
    let col = item_id as u32 % ATLAS_TILES_PER_ROW;
    let row = item_id as u32 / ATLAS_TILES_PER_ROW;

    AtlasUV {
        u0: col as f32 * ATLAS_TILE_UV_SIZE,
        v0: row as f32 * ATLAS_TILE_UV_SIZE,
        u1: (col + 1) as f32 * ATLAS_TILE_UV_SIZE,
        v1: (row + 1) as f32 * ATLAS_TILE_UV_SIZE,
    }
}

/// Shrinks a UV region inward by `padding` on each edge to avoid texture bleeding.
pub fn uv_with_padding(uv: AtlasUV, padding: f32) -> AtlasUV {
    AtlasUV {
        u0: uv.u0 + padding,
        v0: uv.v0 + padding,
        u1: uv.u1 - padding,
        v1: uv.v1 - padding,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_correct() {
        assert_eq!(ATLAS_TILES_PER_ROW, 16);
        assert!((ATLAS_TILE_UV_SIZE - 0.0625).abs() < f32::EPSILON);
    }

    #[test]
    fn block_uv_first_tile_face_0() {
        let uv = block_uv(0, 0);
        assert!((uv.u0 - 0.0).abs() < f32::EPSILON);
        assert!((uv.v0 - 0.0).abs() < f32::EPSILON);
        assert!((uv.u1 - ATLAS_TILE_UV_SIZE).abs() < f32::EPSILON);
        assert!((uv.v1 - ATLAS_TILE_UV_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn block_uv_wraps_to_next_row() {
        // block_id=0, face=0 -> tile 0 -> (0,0)
        // tile 16 should be at row 1, col 0
        // block_id=2, face=4 -> tile 2*6+4=16 -> row 1, col 0
        let uv = block_uv(2, 4);
        assert!((uv.u0 - 0.0).abs() < f32::EPSILON);
        assert!((uv.v0 - ATLAS_TILE_UV_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn item_uv_first_item() {
        let uv = item_uv(0);
        assert!((uv.u0 - 0.0).abs() < f32::EPSILON);
        assert!((uv.v0 - 0.0).abs() < f32::EPSILON);
        assert!((uv.u1 - ATLAS_TILE_UV_SIZE).abs() < f32::EPSILON);
        assert!((uv.v1 - ATLAS_TILE_UV_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn item_uv_second_row() {
        let uv = item_uv(16);
        assert!((uv.u0 - 0.0).abs() < f32::EPSILON);
        assert!((uv.v0 - ATLAS_TILE_UV_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn item_uv_last_in_first_row() {
        let uv = item_uv(15);
        assert!((uv.u0 - 15.0 * ATLAS_TILE_UV_SIZE).abs() < f32::EPSILON);
        assert!((uv.v0 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn uv_with_padding_shrinks_region() {
        let uv = block_uv(0, 0);
        let padded = uv_with_padding(uv, 0.001);
        assert!(padded.u0 > uv.u0);
        assert!(padded.v0 > uv.v0);
        assert!(padded.u1 < uv.u1);
        assert!(padded.v1 < uv.v1);
    }

    #[test]
    fn uv_with_zero_padding_is_identity() {
        let uv = item_uv(5);
        let padded = uv_with_padding(uv, 0.0);
        assert_eq!(uv, padded);
    }
}
