use mc_core::block::BlockId;
use mc_core::pos::BlockPos;

/// Trait abstracting world access so that `FluidSystem` can be tested without
/// a full `ChunkManager`.
pub trait FluidWorld {
    fn get_block(&self, pos: BlockPos) -> BlockId;
    fn set_block(&mut self, pos: BlockPos, block: BlockId);
    fn schedule_update(&mut self, pos: BlockPos, delay: u32);
}

/// Water flow delay in ticks.  Minecraft uses 5 ticks for water in the
/// overworld; we match that default.
const WATER_FLOW_DELAY: u32 = 5;

/// Horizontal neighbor offsets (north, south, east, west).
const HORIZONTAL_OFFSETS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Process a water block update at `pos`.
///
/// Rules (simplified):
/// 1. If the block at `pos` is not Water, do nothing (block may have been
///    removed between scheduling and execution).
/// 2. If the block directly below is Air, place Water there (gravity).
/// 3. Otherwise, spread horizontally to each Air neighbor.
///
/// Every newly placed Water block gets a scheduled update so the flow
/// continues in subsequent ticks.
pub fn process_water_update(world: &mut impl FluidWorld, pos: BlockPos) {
    if world.get_block(pos) != BlockId::Water {
        return;
    }

    let below = BlockPos::new(pos.x, pos.y - 1, pos.z);

    if world.get_block(below) == BlockId::Air {
        world.set_block(below, BlockId::Water);
        world.schedule_update(below, WATER_FLOW_DELAY);
        return;
    }

    // Block below is not air — spread horizontally.
    for (dx, dz) in HORIZONTAL_OFFSETS {
        let neighbor = BlockPos::new(pos.x + dx, pos.y, pos.z + dz);
        if world.get_block(neighbor) == BlockId::Air {
            world.set_block(neighbor, BlockId::Water);
            world.schedule_update(neighbor, WATER_FLOW_DELAY);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A minimal in-memory world for testing fluid flow.
    struct TestWorld {
        blocks: HashMap<BlockPos, BlockId>,
        scheduled: Vec<(BlockPos, u32)>,
    }

    impl TestWorld {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                scheduled: Vec::new(),
            }
        }

        /// Place a block in the test world.
        fn place(&mut self, pos: BlockPos, block: BlockId) {
            self.blocks.insert(pos, block);
        }
    }

    impl FluidWorld for TestWorld {
        fn get_block(&self, pos: BlockPos) -> BlockId {
            self.blocks.get(&pos).copied().unwrap_or(BlockId::Air)
        }

        fn set_block(&mut self, pos: BlockPos, block: BlockId) {
            self.blocks.insert(pos, block);
        }

        fn schedule_update(&mut self, pos: BlockPos, delay: u32) {
            self.scheduled.push((pos, delay));
        }
    }

    #[test]
    fn water_flows_down_into_air() {
        let mut world = TestWorld::new();
        let water_pos = BlockPos::new(5, 10, 5);
        world.place(water_pos, BlockId::Water);

        process_water_update(&mut world, water_pos);

        let below = BlockPos::new(5, 9, 5);
        assert_eq!(world.get_block(below), BlockId::Water);
        // The newly placed block should be scheduled.
        assert!(world.scheduled.iter().any(|(p, _)| *p == below));
    }

    #[test]
    fn water_on_solid_spreads_horizontally() {
        let mut world = TestWorld::new();
        let water_pos = BlockPos::new(5, 10, 5);
        world.place(water_pos, BlockId::Water);
        // Solid block below prevents downward flow.
        let below = BlockPos::new(5, 9, 5);
        world.place(below, BlockId::Stone);

        process_water_update(&mut world, water_pos);

        // All four horizontal neighbors should now be water.
        let neighbors = [
            BlockPos::new(6, 10, 5),
            BlockPos::new(4, 10, 5),
            BlockPos::new(5, 10, 6),
            BlockPos::new(5, 10, 4),
        ];
        for n in &neighbors {
            assert_eq!(
                world.get_block(*n),
                BlockId::Water,
                "neighbor {:?} should be water",
                n
            );
        }
        // Each new water block should be scheduled.
        assert_eq!(world.scheduled.len(), 4);
    }

    #[test]
    fn water_does_not_flow_through_solid() {
        let mut world = TestWorld::new();
        let water_pos = BlockPos::new(5, 10, 5);
        world.place(water_pos, BlockId::Water);

        // Block below and all four neighbors are solid.
        world.place(BlockPos::new(5, 9, 5), BlockId::Stone);
        world.place(BlockPos::new(6, 10, 5), BlockId::Stone);
        world.place(BlockPos::new(4, 10, 5), BlockId::Stone);
        world.place(BlockPos::new(5, 10, 6), BlockId::Stone);
        world.place(BlockPos::new(5, 10, 4), BlockId::Stone);

        process_water_update(&mut world, water_pos);

        // Nothing should have changed — no new water blocks.
        assert!(world.scheduled.is_empty());
        // The only water is the original.
        let water_count = world
            .blocks
            .values()
            .filter(|b| **b == BlockId::Water)
            .count();
        assert_eq!(water_count, 1);
    }

    #[test]
    fn noop_when_source_block_is_not_water() {
        let mut world = TestWorld::new();
        let pos = BlockPos::new(5, 10, 5);
        // No water placed at pos — it is Air.

        process_water_update(&mut world, pos);

        // Nothing should have been scheduled or placed.
        assert!(world.scheduled.is_empty());
        assert!(world.blocks.is_empty());
    }

    #[test]
    fn water_prefers_downward_over_horizontal() {
        let mut world = TestWorld::new();
        let water_pos = BlockPos::new(5, 10, 5);
        world.place(water_pos, BlockId::Water);
        // Air below — downward should take priority, no horizontal spread.

        process_water_update(&mut world, water_pos);

        let below = BlockPos::new(5, 9, 5);
        assert_eq!(world.get_block(below), BlockId::Water);

        // Horizontal neighbors should remain air.
        let neighbors = [
            BlockPos::new(6, 10, 5),
            BlockPos::new(4, 10, 5),
            BlockPos::new(5, 10, 6),
            BlockPos::new(5, 10, 4),
        ];
        for n in &neighbors {
            assert_eq!(world.get_block(*n), BlockId::Air);
        }

        // Only one scheduled update (the block below).
        assert_eq!(world.scheduled.len(), 1);
    }

    #[test]
    fn water_does_not_replace_existing_water() {
        let mut world = TestWorld::new();
        let water_pos = BlockPos::new(5, 10, 5);
        world.place(water_pos, BlockId::Water);

        // Water already exists below.
        let below = BlockPos::new(5, 9, 5);
        world.place(below, BlockId::Water);

        process_water_update(&mut world, water_pos);

        // Below is still water (unchanged) and no new downward schedule
        // because it was already water. Horizontal neighbors that are air
        // should not be affected because the below block is not Air but Water
        // — our logic returns early only if below is Air.
        // Actually: below is Water (not Air), so the function falls through
        // to horizontal spread.
        let neighbors = [
            BlockPos::new(6, 10, 5),
            BlockPos::new(4, 10, 5),
            BlockPos::new(5, 10, 6),
            BlockPos::new(5, 10, 4),
        ];
        for n in &neighbors {
            assert_eq!(world.get_block(*n), BlockId::Water);
        }
        assert_eq!(world.scheduled.len(), 4);
    }

    #[test]
    fn partial_horizontal_spread() {
        let mut world = TestWorld::new();
        let water_pos = BlockPos::new(5, 10, 5);
        world.place(water_pos, BlockId::Water);
        world.place(BlockPos::new(5, 9, 5), BlockId::Stone); // solid below

        // Block two of the four horizontal neighbors.
        world.place(BlockPos::new(6, 10, 5), BlockId::Stone);
        world.place(BlockPos::new(5, 10, 6), BlockId::Stone);

        process_water_update(&mut world, water_pos);

        // Only the two unblocked neighbors should become water.
        assert_eq!(world.get_block(BlockPos::new(4, 10, 5)), BlockId::Water);
        assert_eq!(world.get_block(BlockPos::new(5, 10, 4)), BlockId::Water);
        assert_eq!(world.get_block(BlockPos::new(6, 10, 5)), BlockId::Stone);
        assert_eq!(world.get_block(BlockPos::new(5, 10, 6)), BlockId::Stone);

        assert_eq!(world.scheduled.len(), 2);
    }
}
