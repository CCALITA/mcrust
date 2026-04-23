//! Ambient occlusion calculation for block face vertices.
//!
//! Implements the standard Minecraft AO algorithm where each vertex of a block
//! face is darkened based on how many neighboring blocks occlude it.

/// Ambient occlusion intensity level for a single vertex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AoLevel {
    /// No occlusion — fully lit.
    None,
    /// Light occlusion — one neighbor.
    Light,
    /// Medium occlusion — two neighbors.
    Medium,
    /// Full occlusion — surrounded on both sides (or three neighbors).
    Full,
}

impl AoLevel {
    /// Returns the brightness multiplier for this AO level.
    ///
    /// Values range from 1.0 (no occlusion) to 0.4 (full occlusion).
    pub fn brightness(&self) -> f32 {
        match self {
            AoLevel::None => 1.0,
            AoLevel::Light => 0.8,
            AoLevel::Medium => 0.6,
            AoLevel::Full => 0.4,
        }
    }
}

/// Calculates the AO level for a single vertex using the standard Minecraft algorithm.
///
/// The two `side` parameters represent blocks adjacent to the vertex along the face edges,
/// and `corner` represents the block diagonally adjacent to the vertex.
///
/// If both sides are solid, the vertex is fully occluded regardless of the corner.
/// Otherwise, the occlusion level equals the count of solid neighbors.
pub fn calculate_vertex_ao(side1: bool, side2: bool, corner: bool) -> AoLevel {
    if side1 && side2 {
        return AoLevel::Full;
    }
    let count = side1 as u8 + side2 as u8 + corner as u8;
    match count {
        0 => AoLevel::None,
        1 => AoLevel::Light,
        2 => AoLevel::Medium,
        3 => AoLevel::Full,
        _ => AoLevel::None,
    }
}

/// Computes AO brightness values for the four corners of a block face.
///
/// - `face`: face index — 0=top(+Y), 1=bottom(-Y), 2=north(-Z), 3=south(+Z), 4=east(+X), 5=west(-X)
/// - `bx`, `by`, `bz`: block position
/// - `is_solid`: closure returning whether the block at a given position is solid
///
/// Returns `[f32; 4]` brightness values for the four face vertices.
pub fn ao_for_face(
    face: u8,
    bx: i32,
    by: i32,
    bz: i32,
    is_solid: &impl Fn(i32, i32, i32) -> bool,
) -> [f32; 4] {
    // Each face has 4 vertices. For each vertex we need to check the two edge
    // neighbors and one corner neighbor on the plane one step away from the face.
    //
    // The neighbor offsets are defined relative to the face normal direction.
    // For the top face (+Y), neighbor checks are at y+1 in the XZ plane around the block.

    let neighbors: [[(i32, i32, i32); 3]; 4] = match face {
        // Top face (y+1): vertices at corners of the top surface
        0 => {
            let ny = by + 1;
            [
                // vertex 0 — (-x, -z) corner
                [(bx - 1, ny, bz), (bx, ny, bz - 1), (bx - 1, ny, bz - 1)],
                // vertex 1 — (+x, -z) corner
                [(bx + 1, ny, bz), (bx, ny, bz - 1), (bx + 1, ny, bz - 1)],
                // vertex 2 — (+x, +z) corner
                [(bx + 1, ny, bz), (bx, ny, bz + 1), (bx + 1, ny, bz + 1)],
                // vertex 3 — (-x, +z) corner
                [(bx - 1, ny, bz), (bx, ny, bz + 1), (bx - 1, ny, bz + 1)],
            ]
        }
        // Bottom face (y-1)
        1 => {
            let ny = by - 1;
            [
                [(bx - 1, ny, bz), (bx, ny, bz - 1), (bx - 1, ny, bz - 1)],
                [(bx + 1, ny, bz), (bx, ny, bz - 1), (bx + 1, ny, bz - 1)],
                [(bx + 1, ny, bz), (bx, ny, bz + 1), (bx + 1, ny, bz + 1)],
                [(bx - 1, ny, bz), (bx, ny, bz + 1), (bx - 1, ny, bz + 1)],
            ]
        }
        // North face (-Z): vertices in XY plane at z-1
        2 => {
            let nz = bz - 1;
            [
                [(bx - 1, by, nz), (bx, by + 1, nz), (bx - 1, by + 1, nz)],
                [(bx + 1, by, nz), (bx, by + 1, nz), (bx + 1, by + 1, nz)],
                [(bx + 1, by, nz), (bx, by - 1, nz), (bx + 1, by - 1, nz)],
                [(bx - 1, by, nz), (bx, by - 1, nz), (bx - 1, by - 1, nz)],
            ]
        }
        // South face (+Z): vertices in XY plane at z+1
        3 => {
            let nz = bz + 1;
            [
                [(bx - 1, by, nz), (bx, by + 1, nz), (bx - 1, by + 1, nz)],
                [(bx + 1, by, nz), (bx, by + 1, nz), (bx + 1, by + 1, nz)],
                [(bx + 1, by, nz), (bx, by - 1, nz), (bx + 1, by - 1, nz)],
                [(bx - 1, by, nz), (bx, by - 1, nz), (bx - 1, by - 1, nz)],
            ]
        }
        // East face (+X): vertices in YZ plane at x+1
        4 => {
            let nx = bx + 1;
            [
                [(nx, by, bz - 1), (nx, by + 1, bz), (nx, by + 1, bz - 1)],
                [(nx, by, bz + 1), (nx, by + 1, bz), (nx, by + 1, bz + 1)],
                [(nx, by, bz + 1), (nx, by - 1, bz), (nx, by - 1, bz + 1)],
                [(nx, by, bz - 1), (nx, by - 1, bz), (nx, by - 1, bz - 1)],
            ]
        }
        // West face (-X): vertices in YZ plane at x-1
        5 => {
            let nx = bx - 1;
            [
                [(nx, by, bz - 1), (nx, by + 1, bz), (nx, by + 1, bz - 1)],
                [(nx, by, bz + 1), (nx, by + 1, bz), (nx, by + 1, bz + 1)],
                [(nx, by, bz + 1), (nx, by - 1, bz), (nx, by - 1, bz + 1)],
                [(nx, by, bz - 1), (nx, by - 1, bz), (nx, by - 1, bz - 1)],
            ]
        }
        _ => {
            return [1.0; 4];
        }
    };

    let mut ao = [0.0_f32; 4];
    for (i, [side1_pos, side2_pos, corner_pos]) in neighbors.iter().enumerate() {
        let s1 = is_solid(side1_pos.0, side1_pos.1, side1_pos.2);
        let s2 = is_solid(side2_pos.0, side2_pos.1, side2_pos.2);
        let c = is_solid(corner_pos.0, corner_pos.1, corner_pos.2);
        ao[i] = calculate_vertex_ao(s1, s2, c).brightness();
    }
    ao
}

/// Returns `true` if the quad should be flipped for better triangulation.
///
/// When the AO values on one diagonal are higher than the other, flipping
/// the quad triangulation avoids visual artifacts on the darker diagonal.
pub fn should_flip_quad(ao: [f32; 4]) -> bool {
    ao[0] + ao[2] > ao[1] + ao[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ao_level_brightness_values() {
        assert_eq!(AoLevel::None.brightness(), 1.0);
        assert_eq!(AoLevel::Light.brightness(), 0.8);
        assert_eq!(AoLevel::Medium.brightness(), 0.6);
        assert_eq!(AoLevel::Full.brightness(), 0.4);
    }

    #[test]
    fn vertex_ao_no_neighbors() {
        assert_eq!(calculate_vertex_ao(false, false, false), AoLevel::None);
    }

    #[test]
    fn vertex_ao_one_side() {
        assert_eq!(calculate_vertex_ao(true, false, false), AoLevel::Light);
        assert_eq!(calculate_vertex_ao(false, true, false), AoLevel::Light);
    }

    #[test]
    fn vertex_ao_corner_only() {
        assert_eq!(calculate_vertex_ao(false, false, true), AoLevel::Light);
    }

    #[test]
    fn vertex_ao_one_side_and_corner() {
        assert_eq!(calculate_vertex_ao(true, false, true), AoLevel::Medium);
        assert_eq!(calculate_vertex_ao(false, true, true), AoLevel::Medium);
    }

    #[test]
    fn vertex_ao_two_sides_no_corner() {
        // Both sides solid triggers full occlusion regardless of corner
        assert_eq!(calculate_vertex_ao(true, true, false), AoLevel::Full);
    }

    #[test]
    fn vertex_ao_full_occlusion() {
        assert_eq!(calculate_vertex_ao(true, true, true), AoLevel::Full);
    }

    #[test]
    fn ao_for_face_all_air() {
        let is_solid = |_x: i32, _y: i32, _z: i32| false;
        let ao = ao_for_face(0, 0, 0, 0, &is_solid);
        assert_eq!(ao, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn ao_for_face_all_air_all_faces() {
        let is_solid = |_x: i32, _y: i32, _z: i32| false;
        for face in 0..6 {
            let ao = ao_for_face(face, 5, 10, 3, &is_solid);
            assert_eq!(ao, [1.0, 1.0, 1.0, 1.0], "face {face} should be all 1.0");
        }
    }

    #[test]
    fn ao_for_face_top_full_corner() {
        // Place solid blocks at all 8 neighbors around the block at y+1
        let is_solid = |_x: i32, y: i32, _z: i32| y == 1;
        let ao = ao_for_face(0, 0, 0, 0, &is_solid);
        // Every vertex has both sides and corner solid → Full (0.4)
        assert_eq!(ao, [0.4, 0.4, 0.4, 0.4]);
    }

    #[test]
    fn ao_for_face_top_single_side_neighbor() {
        // Only block at (-1, 1, 0) is solid — affects vertices 0 and 3
        let is_solid = |x: i32, y: i32, z: i32| x == -1 && y == 1 && z == 0;
        let ao = ao_for_face(0, 0, 0, 0, &is_solid);
        // vertex 0: side1=(-1,1,0)=true, side2=(0,1,-1)=false, corner=(-1,1,-1)=false → Light(0.8)
        assert_eq!(ao[0], 0.8);
        // vertex 1: side1=(1,1,0)=false, side2=(0,1,-1)=false, corner=(1,1,-1)=false → None(1.0)
        assert_eq!(ao[1], 1.0);
        // vertex 2: side1=(1,1,0)=false, side2=(0,1,1)=false, corner=(1,1,1)=false → None(1.0)
        assert_eq!(ao[2], 1.0);
        // vertex 3: side1=(-1,1,0)=true, side2=(0,1,1)=false, corner=(-1,1,1)=false → Light(0.8)
        assert_eq!(ao[3], 0.8);
    }

    #[test]
    fn ao_for_face_invalid_face_returns_fully_lit() {
        let is_solid = |_x: i32, _y: i32, _z: i32| true;
        let ao = ao_for_face(6, 0, 0, 0, &is_solid);
        assert_eq!(ao, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn should_flip_quad_detects_diagonal_imbalance() {
        // [0]+[2] = 1.0+1.0 = 2.0 > [1]+[3] = 0.4+0.4 = 0.8
        assert!(should_flip_quad([1.0, 0.4, 1.0, 0.4]));
    }

    #[test]
    fn should_not_flip_quad_when_balanced() {
        assert!(!should_flip_quad([1.0, 1.0, 1.0, 1.0]));
        assert!(!should_flip_quad([0.4, 0.4, 0.4, 0.4]));
    }

    #[test]
    fn should_not_flip_quad_opposite_diagonal() {
        // [0]+[2] = 0.4+0.4 = 0.8 < [1]+[3] = 1.0+1.0 = 2.0
        assert!(!should_flip_quad([0.4, 1.0, 0.4, 1.0]));
    }
}
