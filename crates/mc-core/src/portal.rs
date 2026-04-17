use crate::block::BlockId;
use crate::pos::BlockPos;

// ---------------------------------------------------------------------------
// DimensionId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionId {
    Overworld,
    Nether,
    End,
}

// ---------------------------------------------------------------------------
// Axis (portal orientation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Z,
}

// ---------------------------------------------------------------------------
// PortalFrame
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalFrame {
    pub min: BlockPos,
    pub max: BlockPos,
    pub dimension_from: DimensionId,
    pub dimension_to: DimensionId,
}

// ---------------------------------------------------------------------------
// PortalTransition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalTransition {
    pub source_dim: DimensionId,
    pub target_dim: DimensionId,
    pub source_pos: BlockPos,
    pub target_pos: BlockPos,
}

// ---------------------------------------------------------------------------
// NetherPortal
// ---------------------------------------------------------------------------

pub struct NetherPortal;

impl NetherPortal {
    pub const FRAME_WIDTH: i32 = 4;
    pub const FRAME_HEIGHT: i32 = 5;

    /// Validate a 4-wide x 5-tall Nether portal frame.
    ///
    /// `bottom_left` is the bottom-left corner of the frame (the lowest,
    /// smallest-coordinate corner block).  `axis` selects whether the portal
    /// extends along the X axis or the Z axis.
    ///
    /// A valid frame has:
    /// - Obsidian on the two vertical pillars (columns 0 and 3, rows 0..5)
    /// - Obsidian on the bottom row (row 0, columns 0..4) and top row (row 4)
    /// - Corners (the four intersections of pillars and floor/ceiling) may be
    ///   *any* block (Minecraft allows non-obsidian corners)
    /// - The inner 2x3 area (columns 1..3, rows 1..4) must be Air or portal
    ///   blocks (we treat only Air as acceptable since we don't have a
    ///   dedicated Portal block yet).
    ///
    /// Returns `Some(PortalFrame)` on success, `None` if invalid.
    pub fn validate_frame(
        get_block: &dyn Fn(BlockPos) -> BlockId,
        bottom_left: BlockPos,
        axis: Axis,
    ) -> Option<PortalFrame> {
        let (dx, dz) = match axis {
            Axis::X => (1, 0),
            Axis::Z => (0, 1),
        };

        // Check the frame blocks (obsidian), skipping corners.
        for col in 0..Self::FRAME_WIDTH {
            for row in 0..Self::FRAME_HEIGHT {
                let pos = BlockPos::new(
                    bottom_left.x + col * dx,
                    bottom_left.y + row,
                    bottom_left.z + col * dz,
                );

                let is_corner = (col == 0 || col == Self::FRAME_WIDTH - 1)
                    && (row == 0 || row == Self::FRAME_HEIGHT - 1);

                let is_inner = col > 0
                    && col < Self::FRAME_WIDTH - 1
                    && row > 0
                    && row < Self::FRAME_HEIGHT - 1;

                let block = get_block(pos);

                if is_corner {
                    // Corners can be anything -- skip validation.
                    continue;
                } else if is_inner {
                    // Inner blocks must be Air.
                    if !block.is_air() {
                        return None;
                    }
                } else {
                    // Frame edge (non-corner) must be Obsidian.
                    if block != BlockId::Obsidian {
                        return None;
                    }
                }
            }
        }

        let max = BlockPos::new(
            bottom_left.x + (Self::FRAME_WIDTH - 1) * dx,
            bottom_left.y + Self::FRAME_HEIGHT - 1,
            bottom_left.z + (Self::FRAME_WIDTH - 1) * dz,
        );

        Some(PortalFrame {
            min: bottom_left,
            max,
            dimension_from: DimensionId::Overworld,
            dimension_to: DimensionId::Nether,
        })
    }

    /// Convert an Overworld position to the corresponding Nether position.
    /// Divides x and z by 8, keeps y unchanged.
    pub fn overworld_to_nether(pos: BlockPos) -> BlockPos {
        BlockPos::new(pos.x / 8, pos.y, pos.z / 8)
    }

    /// Convert a Nether position to the corresponding Overworld position.
    /// Multiplies x and z by 8, keeps y unchanged.
    pub fn nether_to_overworld(pos: BlockPos) -> BlockPos {
        BlockPos::new(pos.x * 8, pos.y, pos.z * 8)
    }
}

// ---------------------------------------------------------------------------
// EndPortal
// ---------------------------------------------------------------------------

pub struct EndPortal;

impl EndPortal {
    pub const FRAME_SIZE: i32 = 3;

    /// Validate a 3x3 End portal frame on the ground.
    ///
    /// Checks that a ring of `StoneBricks` (placeholder for End portal frame
    /// blocks) exists around `center` at the same Y level.  The center block
    /// itself must be Air.
    pub fn validate_end_frame(get_block: &dyn Fn(BlockPos) -> BlockId, center: BlockPos) -> bool {
        // Check that the center is Air.
        if !get_block(center).is_air() {
            return false;
        }

        // Check the 8 surrounding blocks for StoneBricks.
        for dx in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let pos = BlockPos::new(center.x + dx, center.y, center.z + dz);
                if get_block(pos) != BlockId::StoneBricks {
                    return false;
                }
            }
        }

        true
    }

    /// The fixed spawn position in The End dimension.
    pub fn end_spawn_pos() -> BlockPos {
        BlockPos::new(0, 64, 0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Coordinate mapping tests ------------------------------------------

    #[test]
    fn overworld_to_nether_divides_xz_by_8() {
        let ow = BlockPos::new(80, 65, -160);
        let nether = NetherPortal::overworld_to_nether(ow);
        assert_eq!(nether, BlockPos::new(10, 65, -20));
    }

    #[test]
    fn nether_to_overworld_multiplies_xz_by_8() {
        let nether = BlockPos::new(10, 65, -20);
        let ow = NetherPortal::nether_to_overworld(nether);
        assert_eq!(ow, BlockPos::new(80, 65, -160));
    }

    #[test]
    fn coordinate_round_trip() {
        let original = BlockPos::new(800, 70, 400);
        let nether = NetherPortal::overworld_to_nether(original);
        let back = NetherPortal::nether_to_overworld(nether);
        assert_eq!(back, original);
    }

    #[test]
    fn overworld_to_nether_at_origin() {
        let pos = BlockPos::new(0, 64, 0);
        let nether = NetherPortal::overworld_to_nether(pos);
        assert_eq!(nether, BlockPos::new(0, 64, 0));
    }

    #[test]
    fn overworld_to_nether_small_values() {
        // Values smaller than 8 should map to 0 via integer division.
        let pos = BlockPos::new(7, 100, 3);
        let nether = NetherPortal::overworld_to_nether(pos);
        assert_eq!(nether, BlockPos::new(0, 100, 0));
    }

    // -- Nether frame validation tests -------------------------------------

    fn make_nether_frame_x() -> Vec<(BlockPos, BlockId)> {
        // Build a 4x5 frame along the X axis at origin.
        let mut blocks = Vec::new();
        for col in 0..4 {
            for row in 0..5 {
                let pos = BlockPos::new(col, row, 0);
                let is_corner = (col == 0 || col == 3) && (row == 0 || row == 4);
                let is_inner = col > 0 && col < 3 && row > 0 && row < 4;

                let block = if is_corner {
                    BlockId::Obsidian // corners can be anything; use obsidian
                } else if is_inner {
                    BlockId::Air
                } else {
                    BlockId::Obsidian
                };
                blocks.push((pos, block));
            }
        }
        blocks
    }

    fn lookup(blocks: &[(BlockPos, BlockId)]) -> impl Fn(BlockPos) -> BlockId + '_ {
        move |pos| {
            blocks
                .iter()
                .find(|(p, _)| *p == pos)
                .map(|(_, b)| *b)
                .unwrap_or(BlockId::Air)
        }
    }

    #[test]
    fn valid_nether_frame_x_axis() {
        let blocks = make_nether_frame_x();
        let get = lookup(&blocks);
        let frame = NetherPortal::validate_frame(&get, BlockPos::new(0, 0, 0), Axis::X);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.min, BlockPos::new(0, 0, 0));
        assert_eq!(frame.max, BlockPos::new(3, 4, 0));
        assert_eq!(frame.dimension_from, DimensionId::Overworld);
        assert_eq!(frame.dimension_to, DimensionId::Nether);
    }

    #[test]
    fn valid_nether_frame_z_axis() {
        // Build a frame along the Z axis.
        let mut blocks = Vec::new();
        for col in 0..4 {
            for row in 0..5 {
                let pos = BlockPos::new(0, row, col);
                let is_corner = (col == 0 || col == 3) && (row == 0 || row == 4);
                let is_inner = col > 0 && col < 3 && row > 0 && row < 4;
                let block = if is_corner {
                    BlockId::Obsidian
                } else if is_inner {
                    BlockId::Air
                } else {
                    BlockId::Obsidian
                };
                blocks.push((pos, block));
            }
        }
        let get = lookup(&blocks);
        let frame = NetherPortal::validate_frame(&get, BlockPos::new(0, 0, 0), Axis::Z);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.min, BlockPos::new(0, 0, 0));
        assert_eq!(frame.max, BlockPos::new(0, 4, 3));
    }

    #[test]
    fn invalid_nether_frame_missing_obsidian() {
        let mut blocks = make_nether_frame_x();
        // Replace one pillar block with stone.
        if let Some(entry) = blocks
            .iter_mut()
            .find(|(p, _)| *p == BlockPos::new(0, 2, 0))
        {
            entry.1 = BlockId::Stone;
        }
        let get = lookup(&blocks);
        let frame = NetherPortal::validate_frame(&get, BlockPos::new(0, 0, 0), Axis::X);
        assert!(frame.is_none());
    }

    #[test]
    fn invalid_nether_frame_blocked_interior() {
        let mut blocks = make_nether_frame_x();
        // Place a solid block in the interior.
        if let Some(entry) = blocks
            .iter_mut()
            .find(|(p, _)| *p == BlockPos::new(1, 2, 0))
        {
            entry.1 = BlockId::Stone;
        }
        let get = lookup(&blocks);
        let frame = NetherPortal::validate_frame(&get, BlockPos::new(0, 0, 0), Axis::X);
        assert!(frame.is_none());
    }

    #[test]
    fn nether_frame_corners_can_be_any_block() {
        let mut blocks = make_nether_frame_x();
        // Replace corners with non-obsidian blocks.
        let corners = [
            BlockPos::new(0, 0, 0),
            BlockPos::new(3, 0, 0),
            BlockPos::new(0, 4, 0),
            BlockPos::new(3, 4, 0),
        ];
        for corner in &corners {
            if let Some(entry) = blocks.iter_mut().find(|(p, _)| p == corner) {
                entry.1 = BlockId::Dirt;
            }
        }
        let get = lookup(&blocks);
        let frame = NetherPortal::validate_frame(&get, BlockPos::new(0, 0, 0), Axis::X);
        assert!(frame.is_some(), "corners may be any block");
    }

    // -- End portal validation tests ---------------------------------------

    fn make_end_frame(center: BlockPos) -> Vec<(BlockPos, BlockId)> {
        let mut blocks = Vec::new();
        for dx in -1..=1 {
            for dz in -1..=1 {
                let pos = BlockPos::new(center.x + dx, center.y, center.z + dz);
                let block = if dx == 0 && dz == 0 {
                    BlockId::Air
                } else {
                    BlockId::StoneBricks
                };
                blocks.push((pos, block));
            }
        }
        blocks
    }

    #[test]
    fn valid_end_frame() {
        let center = BlockPos::new(10, 5, 10);
        let blocks = make_end_frame(center);
        let get = lookup(&blocks);
        assert!(EndPortal::validate_end_frame(&get, center));
    }

    #[test]
    fn invalid_end_frame_missing_stone_bricks() {
        let center = BlockPos::new(10, 5, 10);
        let mut blocks = make_end_frame(center);
        // Replace one surrounding block with dirt.
        if let Some(entry) = blocks
            .iter_mut()
            .find(|(p, _)| *p == BlockPos::new(11, 5, 10))
        {
            entry.1 = BlockId::Dirt;
        }
        let get = lookup(&blocks);
        assert!(!EndPortal::validate_end_frame(&get, center));
    }

    #[test]
    fn invalid_end_frame_center_not_air() {
        let center = BlockPos::new(10, 5, 10);
        let mut blocks = make_end_frame(center);
        // Block the center.
        if let Some(entry) = blocks.iter_mut().find(|(p, _)| *p == center) {
            entry.1 = BlockId::Stone;
        }
        let get = lookup(&blocks);
        assert!(!EndPortal::validate_end_frame(&get, center));
    }

    #[test]
    fn end_spawn_pos_is_fixed() {
        let pos = EndPortal::end_spawn_pos();
        assert_eq!(pos, BlockPos::new(0, 64, 0));
    }

    // -- PortalTransition --------------------------------------------------

    #[test]
    fn portal_transition_struct() {
        let t = PortalTransition {
            source_dim: DimensionId::Overworld,
            target_dim: DimensionId::Nether,
            source_pos: BlockPos::new(80, 65, 160),
            target_pos: BlockPos::new(10, 65, 20),
        };
        assert_eq!(t.source_dim, DimensionId::Overworld);
        assert_eq!(t.target_dim, DimensionId::Nether);
    }

    // -- DimensionId -------------------------------------------------------

    #[test]
    fn dimension_id_equality() {
        assert_eq!(DimensionId::Overworld, DimensionId::Overworld);
        assert_ne!(DimensionId::Overworld, DimensionId::Nether);
        assert_ne!(DimensionId::Nether, DimensionId::End);
    }
}
