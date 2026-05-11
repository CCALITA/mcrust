//! Chain block model: vertices, collision, connectivity, and light passthrough.

/// Axis along which a chain is oriented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainAxis {
    X,
    Y,
    Z,
}

/// Generates 8 vertices for a narrow 4/16-wide box centered at `pos`, oriented along `axis`.
pub fn generate_chain_vertices(pos: [f32; 3], axis: ChainAxis) -> Vec<[f32; 3]> {
    let half_thick = 2.0 / 16.0; // 4/16 wide → half is 2/16
    let half_len = 8.0 / 16.0; // full block length along axis

    let (dx, dy, dz) = match axis {
        ChainAxis::X => (half_len, half_thick, half_thick),
        ChainAxis::Y => (half_thick, half_len, half_thick),
        ChainAxis::Z => (half_thick, half_thick, half_len),
    };

    let [cx, cy, cz] = pos;
    vec![
        [cx - dx, cy - dy, cz - dz],
        [cx + dx, cy - dy, cz - dz],
        [cx + dx, cy + dy, cz - dz],
        [cx - dx, cy + dy, cz - dz],
        [cx - dx, cy - dy, cz + dz],
        [cx + dx, cy - dy, cz + dz],
        [cx + dx, cy + dy, cz + dz],
        [cx - dx, cy + dy, cz + dz],
    ]
}

/// Returns the AABB collision box `[min_x, min_y, min_z, max_x, max_y, max_z]` for a chain on the given axis.
pub fn chain_collision_box(axis: ChainAxis) -> [f32; 6] {
    let thin = 6.0 / 16.0; // offset from edge for the thin dimensions
    let thick_min = thin;
    let thick_max = 1.0 - thin;

    match axis {
        ChainAxis::X => [0.0, thick_min, thick_min, 1.0, thick_max, thick_max],
        ChainAxis::Y => [thick_min, 0.0, thick_min, thick_max, 1.0, thick_max],
        ChainAxis::Z => [thick_min, thick_min, 0.0, thick_max, thick_max, 1.0],
    }
}

/// Returns whether a chain can connect to the given block id.
/// Chains connect to other chains (id 789) and lanterns (id 790).
pub fn chain_connects_to(block_id: u16) -> bool {
    matches!(block_id, 789 | 790)
}

/// Chains allow light to pass through.
pub fn chain_light_passthrough() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertices_count_is_eight() {
        let verts = generate_chain_vertices([0.0, 0.0, 0.0], ChainAxis::Y);
        assert_eq!(verts.len(), 8);
    }

    #[test]
    fn vertices_symmetric_around_center() {
        let pos = [1.0, 2.0, 3.0];
        let verts = generate_chain_vertices(pos, ChainAxis::Y);
        // First and last vertices should be symmetric in Y around center
        let avg_y: f32 = verts.iter().map(|v| v[1]).sum::<f32>() / 8.0;
        assert!((avg_y - pos[1]).abs() < 1e-6);
    }

    #[test]
    fn collision_box_y_axis() {
        let aabb = chain_collision_box(ChainAxis::Y);
        // Y spans full block
        assert_eq!(aabb[1], 0.0);
        assert_eq!(aabb[4], 1.0);
        // X and Z are narrow
        assert!(aabb[0] > 0.0);
        assert!(aabb[3] < 1.0);
    }

    #[test]
    fn collision_box_x_axis() {
        let aabb = chain_collision_box(ChainAxis::X);
        assert_eq!(aabb[0], 0.0);
        assert_eq!(aabb[3], 1.0);
    }

    #[test]
    fn collision_box_z_axis() {
        let aabb = chain_collision_box(ChainAxis::Z);
        assert_eq!(aabb[2], 0.0);
        assert_eq!(aabb[5], 1.0);
    }

    #[test]
    fn connects_to_chain_and_lantern() {
        assert!(chain_connects_to(789));
        assert!(chain_connects_to(790));
        assert!(!chain_connects_to(1));
    }

    #[test]
    fn light_passthrough_is_true() {
        assert!(chain_light_passthrough());
    }
}
