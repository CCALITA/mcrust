//! Fence, wall, and fence gate connection logic.
//!
//! Provides [`FenceConnections`] for determining how fences and walls connect
//! to neighboring blocks, [`FenceMaterial`] for wood/nether brick variants,
//! and helper functions for collision heights and gate passability.

use crate::block::BlockId;

// ---------------------------------------------------------------------------
// FenceMaterial
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FenceMaterial {
    Oak,
    Birch,
    Spruce,
    Jungle,
    DarkOak,
    Acacia,
    CrimsonFence,
    WarpedFence,
    NetherBrick,
}

// ---------------------------------------------------------------------------
// FenceConnections
// ---------------------------------------------------------------------------

/// Tracks which of the four horizontal directions a fence or wall connects to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FenceConnections {
    pub north: bool,
    pub south: bool,
    pub east: bool,
    pub west: bool,
}

impl FenceConnections {
    /// No connections in any direction.
    pub fn none() -> Self {
        Self {
            north: false,
            south: false,
            east: false,
            west: false,
        }
    }

    /// Number of active connections (0..=4).
    pub fn count(&self) -> u8 {
        self.north as u8 + self.south as u8 + self.east as u8 + self.west as u8
    }
}

// ---------------------------------------------------------------------------
// Connection logic
// ---------------------------------------------------------------------------

/// Returns `true` if a fence/wall should connect to the block with the given
/// raw ID.
///
/// Fences connect to other fences, walls, fence gates, and any solid full
/// block. Since the current [`BlockId`] enum does not yet include fence, wall,
/// or gate variants, this function checks only for solid blocks via
/// [`BlockId::is_solid`]. As those variants are added this function should be
/// extended to match them explicitly.
pub fn can_fence_connect(neighbor_block_id: u16) -> bool {
    match BlockId::from_raw(neighbor_block_id) {
        Some(block) => block.is_solid(),
        // Unknown IDs are treated as non-connectable.
        None => false,
    }
}

/// Compute wall/fence connections by sampling four horizontal neighbors.
///
/// `bx`, `by`, `bz` are the block coordinates of the fence/wall.
/// `get_block` returns the raw block ID at the given world coordinates.
pub fn wall_connections(
    bx: i32,
    by: i32,
    bz: i32,
    get_block: &impl Fn(i32, i32, i32) -> u16,
) -> FenceConnections {
    FenceConnections {
        north: can_fence_connect(get_block(bx, by, bz - 1)),
        south: can_fence_connect(get_block(bx, by, bz + 1)),
        east: can_fence_connect(get_block(bx + 1, by, bz)),
        west: can_fence_connect(get_block(bx - 1, by, bz)),
    }
}

// ---------------------------------------------------------------------------
// Height helpers
// ---------------------------------------------------------------------------

/// Collision height for fences — entities cannot jump over a 1.5-block fence.
pub fn fence_collision_height() -> f32 {
    1.5
}

/// Visual (render) height of a fence post.
pub fn fence_render_height() -> f32 {
    1.0
}

// ---------------------------------------------------------------------------
// Gate helpers
// ---------------------------------------------------------------------------

/// Returns whether a fence gate is passable (entities can walk through).
pub fn gate_is_passable(open: bool) -> bool {
    open
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_has_zero_connections() {
        let conn = FenceConnections::none();
        assert!(!conn.north);
        assert!(!conn.south);
        assert!(!conn.east);
        assert!(!conn.west);
        assert_eq!(conn.count(), 0);
    }

    #[test]
    fn count_returns_number_of_true_fields() {
        let conn = FenceConnections {
            north: true,
            south: false,
            east: true,
            west: false,
        };
        assert_eq!(conn.count(), 2);
    }

    #[test]
    fn all_connections_count_is_four() {
        let conn = FenceConnections {
            north: true,
            south: true,
            east: true,
            west: true,
        };
        assert_eq!(conn.count(), 4);
    }

    #[test]
    fn solid_block_connects() {
        // Stone is solid — fences should connect.
        assert!(can_fence_connect(BlockId::Stone as u16));
        assert!(can_fence_connect(BlockId::Cobblestone as u16));
        assert!(can_fence_connect(BlockId::OakPlanks as u16));
    }

    #[test]
    fn non_solid_block_does_not_connect() {
        // Air and other non-solid blocks should not connect.
        assert!(!can_fence_connect(BlockId::Air as u16));
        assert!(!can_fence_connect(BlockId::Torch as u16));
        assert!(!can_fence_connect(BlockId::Water as u16));
    }

    #[test]
    fn unknown_block_id_does_not_connect() {
        // An ID beyond the registry should not connect.
        assert!(!can_fence_connect(9999));
    }

    #[test]
    fn wall_connections_no_neighbors() {
        // Surrounded by air (0).
        let get = |_x: i32, _y: i32, _z: i32| -> u16 { BlockId::Air as u16 };
        let conn = wall_connections(0, 0, 0, &get);
        assert_eq!(conn, FenceConnections::none());
        assert_eq!(conn.count(), 0);
    }

    #[test]
    fn wall_connections_all_neighbors() {
        // Surrounded by stone on all four sides.
        let get = |_x: i32, _y: i32, _z: i32| -> u16 { BlockId::Stone as u16 };
        let conn = wall_connections(5, 10, 5, &get);
        assert!(conn.north);
        assert!(conn.south);
        assert!(conn.east);
        assert!(conn.west);
        assert_eq!(conn.count(), 4);
    }

    #[test]
    fn wall_connections_partial_neighbors() {
        // Stone to the north and east, air elsewhere.
        let get = |x: i32, _y: i32, z: i32| -> u16 {
            if (x == 5 && z == 4) || (x == 6 && z == 5) {
                BlockId::Stone as u16
            } else {
                BlockId::Air as u16
            }
        };
        let conn = wall_connections(5, 10, 5, &get);
        assert!(conn.north); // z - 1 = 4
        assert!(!conn.south); // z + 1 = 6
        assert!(conn.east); // x + 1 = 6
        assert!(!conn.west); // x - 1 = 4
        assert_eq!(conn.count(), 2);
    }

    #[test]
    fn fence_collision_height_is_1_5() {
        assert!((fence_collision_height() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fence_render_height_is_1_0() {
        assert!((fence_render_height() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gate_passable_when_open() {
        assert!(gate_is_passable(true));
    }

    #[test]
    fn gate_not_passable_when_closed() {
        assert!(!gate_is_passable(false));
    }

    #[test]
    fn fence_material_variants_are_distinct() {
        let materials = [
            FenceMaterial::Oak,
            FenceMaterial::Birch,
            FenceMaterial::Spruce,
            FenceMaterial::Jungle,
            FenceMaterial::DarkOak,
            FenceMaterial::Acacia,
            FenceMaterial::CrimsonFence,
            FenceMaterial::WarpedFence,
            FenceMaterial::NetherBrick,
        ];
        for (i, a) in materials.iter().enumerate() {
            for (j, b) in materials.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }
}
