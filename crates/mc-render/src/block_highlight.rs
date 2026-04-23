/// Block highlight wireframe for rendering a selection outline around the targeted block.
///
/// Generates line-list vertices for a slightly oversized cube (scale 1.002) to avoid
/// z-fighting with the block faces.

/// Returns the wireframe highlight color: black at 40% alpha.
pub fn highlight_color() -> [f32; 4] {
    [0.0, 0.0, 0.0, 0.4]
}

/// Generates wireframe vertices for all 12 edges of a block highlight cube.
///
/// The cube extends from `(x - 0.001, y - 0.001, z - 0.001)` to
/// `(x + 1.001, y + 1.001, z + 1.001)` — slightly larger than 1x1x1 to prevent
/// z-fighting. Returns 24 vertices (12 edges x 2 endpoints) suitable for a line list.
pub fn generate_highlight_vertices(block_x: i32, block_y: i32, block_z: i32) -> Vec<[f32; 3]> {
    let x0 = block_x as f32 - 0.001;
    let y0 = block_y as f32 - 0.001;
    let z0 = block_z as f32 - 0.001;
    let x1 = block_x as f32 + 1.001;
    let y1 = block_y as f32 + 1.001;
    let z1 = block_z as f32 + 1.001;

    vec![
        // Bottom face edges (y = y0)
        [x0, y0, z0], [x1, y0, z0],
        [x1, y0, z0], [x1, y0, z1],
        [x1, y0, z1], [x0, y0, z1],
        [x0, y0, z1], [x0, y0, z0],
        // Top face edges (y = y1)
        [x0, y1, z0], [x1, y1, z0],
        [x1, y1, z0], [x1, y1, z1],
        [x1, y1, z1], [x0, y1, z1],
        [x0, y1, z1], [x0, y1, z0],
        // Vertical edges connecting bottom to top
        [x0, y0, z0], [x0, y1, z0],
        [x1, y0, z0], [x1, y1, z0],
        [x1, y0, z1], [x1, y1, z1],
        [x0, y0, z1], [x0, y1, z1],
    ]
}

/// Generates wireframe vertices for the 4 edges of a single block face.
///
/// Returns 8 vertices (4 edges x 2 endpoints) suitable for a line list.
///
/// Face indices: 0 = top, 1 = bottom, 2 = north (-Z), 3 = south (+Z),
/// 4 = east (+X), 5 = west (-X).
pub fn highlight_face_vertices(block_x: i32, block_y: i32, block_z: i32, face: u8) -> Vec<[f32; 3]> {
    let x0 = block_x as f32 - 0.001;
    let y0 = block_y as f32 - 0.001;
    let z0 = block_z as f32 - 0.001;
    let x1 = block_x as f32 + 1.001;
    let y1 = block_y as f32 + 1.001;
    let z1 = block_z as f32 + 1.001;

    match face {
        // Top (y = y1)
        0 => vec![
            [x0, y1, z0], [x1, y1, z0],
            [x1, y1, z0], [x1, y1, z1],
            [x1, y1, z1], [x0, y1, z1],
            [x0, y1, z1], [x0, y1, z0],
        ],
        // Bottom (y = y0)
        1 => vec![
            [x0, y0, z0], [x1, y0, z0],
            [x1, y0, z0], [x1, y0, z1],
            [x1, y0, z1], [x0, y0, z1],
            [x0, y0, z1], [x0, y0, z0],
        ],
        // North (-Z, z = z0)
        2 => vec![
            [x0, y0, z0], [x1, y0, z0],
            [x1, y0, z0], [x1, y1, z0],
            [x1, y1, z0], [x0, y1, z0],
            [x0, y1, z0], [x0, y0, z0],
        ],
        // South (+Z, z = z1)
        3 => vec![
            [x0, y0, z1], [x1, y0, z1],
            [x1, y0, z1], [x1, y1, z1],
            [x1, y1, z1], [x0, y1, z1],
            [x0, y1, z1], [x0, y0, z1],
        ],
        // East (+X, x = x1)
        4 => vec![
            [x1, y0, z0], [x1, y0, z1],
            [x1, y0, z1], [x1, y1, z1],
            [x1, y1, z1], [x1, y1, z0],
            [x1, y1, z0], [x1, y0, z0],
        ],
        // West (-X, x = x0)
        5 => vec![
            [x0, y0, z0], [x0, y0, z1],
            [x0, y0, z1], [x0, y1, z1],
            [x0, y1, z1], [x0, y1, z0],
            [x0, y1, z0], [x0, y0, z0],
        ],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_color_is_black_40_percent_alpha() {
        let color = highlight_color();
        assert_eq!(color, [0.0, 0.0, 0.0, 0.4]);
    }

    #[test]
    fn full_highlight_has_24_vertices() {
        let verts = generate_highlight_vertices(0, 0, 0);
        assert_eq!(verts.len(), 24);
    }

    #[test]
    fn face_highlight_has_8_vertices() {
        for face in 0..6 {
            let verts = highlight_face_vertices(0, 0, 0, face);
            assert_eq!(verts.len(), 8, "face {face} should have 8 vertices");
        }
    }

    #[test]
    fn invalid_face_returns_empty() {
        let verts = highlight_face_vertices(0, 0, 0, 6);
        assert!(verts.is_empty());
        let verts = highlight_face_vertices(0, 0, 0, 255);
        assert!(verts.is_empty());
    }

    #[test]
    fn vertices_within_expected_bounds_at_origin() {
        let verts = generate_highlight_vertices(0, 0, 0);
        for v in &verts {
            assert!(v[0] >= -0.002 && v[0] <= 1.002, "x out of bounds: {}", v[0]);
            assert!(v[1] >= -0.002 && v[1] <= 1.002, "y out of bounds: {}", v[1]);
            assert!(v[2] >= -0.002 && v[2] <= 1.002, "z out of bounds: {}", v[2]);
        }
    }

    #[test]
    fn vertices_offset_for_different_block_position() {
        let verts = generate_highlight_vertices(10, -5, 20);
        for v in &verts {
            assert!(v[0] >= 9.998 && v[0] <= 11.002, "x out of bounds: {}", v[0]);
            assert!(v[1] >= -5.002 && v[1] <= -3.998, "y out of bounds: {}", v[1]);
            assert!(v[2] >= 19.998 && v[2] <= 21.002, "z out of bounds: {}", v[2]);
        }
    }

    #[test]
    fn face_vertices_offset_for_different_block_position() {
        let verts = highlight_face_vertices(3, 7, -2, 0); // top face
        assert_eq!(verts.len(), 8);
        for v in &verts {
            assert!(v[0] >= 2.998 && v[0] <= 4.002, "x out of bounds: {}", v[0]);
            // Top face: all y should be at y1 = 7 + 1.001 = 8.001
            assert!((v[1] - 8.001).abs() < 0.0001, "y should be ~8.001: {}", v[1]);
            assert!(v[2] >= -2.002 && v[2] <= -0.998, "z out of bounds: {}", v[2]);
        }
    }
}
