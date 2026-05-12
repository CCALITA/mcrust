//! Greedy meshing algorithm for combining adjacent block faces into larger quads.
//!
//! Reduces vertex count by merging coplanar, same-block faces first horizontally
//! then vertically.

/// A single merged quad produced by greedy meshing.
#[derive(Debug, Clone, PartialEq)]
pub struct GreedyQuad {
    pub pos: [f32; 3],
    pub width: f32,
    pub height: f32,
    pub face: u8,
    pub block_id: u16,
}

/// Result of greedy meshing a set of faces.
#[derive(Debug, Clone)]
pub struct GreedyMeshResult {
    pub quads: Vec<GreedyQuad>,
    pub vertex_reduction_pct: f32,
}

/// Estimate vertex savings as a percentage.
///
/// `original` is the vertex count without greedy meshing (6 vertices per face),
/// `greedy` is the vertex count after merging (6 vertices per quad).
pub fn estimate_vertex_savings(original: usize, greedy: usize) -> f32 {
    if original == 0 {
        return 0.0;
    }
    (1.0 - greedy as f32 / original as f32) * 100.0
}

/// Perform greedy meshing on a 2D grid of block faces.
///
/// `blocks` is a row-major grid of block IDs (0 = air/no face).
/// `face` is the face direction index stored in each resulting quad.
/// `size_x` is the width (columns), `size_y` is the height (rows).
///
/// Returns merged quads covering all non-zero entries.
pub fn greedy_mesh_face(
    blocks: &[u16],
    face: u8,
    size_x: usize,
    size_y: usize,
) -> Vec<GreedyQuad> {
    assert_eq!(blocks.len(), size_x * size_y, "blocks length must equal size_x * size_y");

    let mut visited = vec![false; size_x * size_y];
    let mut quads = Vec::new();

    for y in 0..size_y {
        for x in 0..size_x {
            let idx = y * size_x + x;
            if visited[idx] || blocks[idx] == 0 {
                continue;
            }

            let block_id = blocks[idx];

            // Expand horizontally
            let mut w = 1usize;
            while x + w < size_x {
                let ni = y * size_x + (x + w);
                if visited[ni] || blocks[ni] != block_id {
                    break;
                }
                w += 1;
            }

            // Expand vertically
            let mut h = 1usize;
            'outer: while y + h < size_y {
                for dx in 0..w {
                    let ni = (y + h) * size_x + (x + dx);
                    if visited[ni] || blocks[ni] != block_id {
                        break 'outer;
                    }
                }
                h += 1;
            }

            // Mark visited
            for dy in 0..h {
                for dx in 0..w {
                    visited[(y + dy) * size_x + (x + dx)] = true;
                }
            }

            quads.push(GreedyQuad {
                pos: [x as f32, y as f32, 0.0],
                width: w as f32,
                height: h as f32,
                face,
                block_id,
            });
        }
    }

    quads
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let blocks = vec![0u16; 16];
        let quads = greedy_mesh_face(&blocks, 0, 4, 4);
        assert!(quads.is_empty());
    }

    #[test]
    fn test_single_block() {
        let mut blocks = vec![0u16; 4];
        blocks[0] = 1;
        let quads = greedy_mesh_face(&blocks, 0, 2, 2);
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].width, 1.0);
        assert_eq!(quads[0].height, 1.0);
        assert_eq!(quads[0].block_id, 1);
    }

    #[test]
    fn test_horizontal_merge() {
        // Row of 4 same blocks
        let blocks = vec![1u16; 4];
        let quads = greedy_mesh_face(&blocks, 2, 4, 1);
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].width, 4.0);
        assert_eq!(quads[0].height, 1.0);
    }

    #[test]
    fn test_vertical_merge() {
        // Column of 4 same blocks (1 wide, 4 tall)
        let blocks = vec![1u16; 4];
        let quads = greedy_mesh_face(&blocks, 3, 1, 4);
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].width, 1.0);
        assert_eq!(quads[0].height, 4.0);
    }

    #[test]
    fn test_full_grid_merge() {
        // 4x4 grid all same block → single quad
        let blocks = vec![5u16; 16];
        let quads = greedy_mesh_face(&blocks, 1, 4, 4);
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].width, 4.0);
        assert_eq!(quads[0].height, 4.0);
        assert_eq!(quads[0].block_id, 5);
    }

    #[test]
    fn test_different_blocks_not_merged() {
        let blocks = vec![1, 2, 3, 4];
        let quads = greedy_mesh_face(&blocks, 0, 4, 1);
        assert_eq!(quads.len(), 4);
    }

    #[test]
    fn test_mixed_with_air() {
        let blocks = vec![1, 0, 1, 1];
        let quads = greedy_mesh_face(&blocks, 0, 2, 2);
        // (0,0)=1, (1,0)=0, (0,1)=1, (1,1)=1
        // Should produce: single at (0,0), and merged (0,1)+(1,1)
        assert_eq!(quads.len(), 2);
    }

    #[test]
    fn test_estimate_vertex_savings_zero() {
        assert_eq!(estimate_vertex_savings(0, 0), 0.0);
    }

    #[test]
    fn test_estimate_vertex_savings_half() {
        let pct = estimate_vertex_savings(100, 50);
        assert!((pct - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_estimate_vertex_savings_full() {
        let pct = estimate_vertex_savings(100, 0);
        assert!((pct - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_greedy_mesh_result() {
        let blocks = vec![1u16; 16];
        let quads = greedy_mesh_face(&blocks, 0, 4, 4);
        let original_verts = 16 * 6;
        let greedy_verts = quads.len() * 6;
        let result = GreedyMeshResult {
            quads,
            vertex_reduction_pct: estimate_vertex_savings(original_verts, greedy_verts),
        };
        assert_eq!(result.quads.len(), 1);
        assert!(result.vertex_reduction_pct > 90.0);
    }
}
