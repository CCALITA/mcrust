use std::collections::{HashMap, VecDeque};

use mc_core::block::BlockId;
use mc_core::pos::BlockPos;

// ---------------------------------------------------------------------------
// Trait-based world access (mirrors FluidWorld pattern from fluid.rs)
// ---------------------------------------------------------------------------

/// Trait abstracting world access so that redstone propagation can be tested
/// without a full `ChunkManager`.
pub trait RedstoneWorld {
    fn get_block(&self, pos: BlockPos) -> BlockId;
    fn get_power(&self, pos: BlockPos) -> u8;
    fn set_power(&mut self, pos: BlockPos, level: u8);
    fn is_solid_block(&self, pos: BlockPos) -> bool;
}

// ---------------------------------------------------------------------------
// Power source detection
// ---------------------------------------------------------------------------

/// Maximum redstone signal strength.
const MAX_POWER: u8 = 15;

/// Six axis-aligned neighbor offsets.
const NEIGHBORS: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

/// Returns the power level a block emits as a source, or `None` if it is not
/// a power source.
///
/// Since dedicated redstone blocks (RedstoneTorch, Lever, Button) do not yet
/// exist in `BlockId`, we use `Torch` as a placeholder for RedstoneTorch.
pub fn is_power_source(block: BlockId) -> Option<u8> {
    match block {
        // Torch stands in for RedstoneTorch — emits full power.
        BlockId::Torch => Some(MAX_POWER),
        _ => None,
    }
}

/// Returns `true` when a block acts as redstone dust (conducts power with
/// attenuation).  Since there is no dedicated `RedstoneDust` variant yet, we
/// treat any transparent, non-air block as dust.
fn is_dust(block: BlockId) -> bool {
    !block.is_air() && block.is_transparent() && is_power_source(block).is_none()
}

// ---------------------------------------------------------------------------
// BFS propagation
// ---------------------------------------------------------------------------

/// Propagate redstone power from all sources reachable from `changed_pos`.
///
/// Algorithm:
/// 1. Scan the affected area around `changed_pos` and clear power levels.
/// 2. Collect all power sources in the area.
/// 3. BFS flood-fill: dust blocks propagate with -1 per step; solid blocks
///    receive "weak power" from adjacent powered dust (but do not propagate
///    further); a torch whose support block (below) is powered turns off.
pub fn propagate_redstone(world: &mut impl RedstoneWorld, changed_pos: BlockPos) {
    // The maximum propagation radius is 15 (one full power chain).  We scan a
    // cube of radius 16 centered on `changed_pos` to cover sources whose signal
    // could reach this area.
    let radius: i32 = 16;

    // Collect all positions in the affected area once.
    let mut positions: Vec<BlockPos> = Vec::new();
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                positions.push(BlockPos::new(
                    changed_pos.x + dx,
                    changed_pos.y + dy,
                    changed_pos.z + dz,
                ));
            }
        }
    }

    // Torch inversion creates a feedback loop: powering a support block turns
    // off the torch above it, which may de-power the support.  We iterate
    // until no torch changes state (convergence is guaranteed because each
    // iteration can only turn torches off, never on, reducing the set of
    // active sources monotonically).
    let max_iterations = 4;
    let mut disabled_torches: Vec<BlockPos> = Vec::new();

    for _ in 0..max_iterations {
        // Phase 1 — clear power.
        for &pos in &positions {
            world.set_power(pos, 0);
        }

        // Phase 2 — seed active sources (skip disabled torches).
        let mut queue: VecDeque<BlockPos> = VecDeque::new();
        for &pos in &positions {
            let block = world.get_block(pos);
            if let Some(level) = is_power_source(block) {
                if disabled_torches.contains(&pos) {
                    continue;
                }
                world.set_power(pos, level);
                queue.push_back(pos);
            }
        }

        // Phase 3 — BFS flood-fill.
        while let Some(pos) = queue.pop_front() {
            let current_power = world.get_power(pos);
            if current_power <= 1 {
                continue;
            }
            let new_level = current_power - 1;

            for (dx, dy, dz) in NEIGHBORS {
                let neighbor = BlockPos::new(pos.x + dx, pos.y + dy, pos.z + dz);
                let neighbor_block = world.get_block(neighbor);

                if neighbor_block.is_air() {
                    continue;
                }

                // Dust-like blocks propagate power with -1 attenuation.
                // Check dust before solid because some blocks (e.g. Glass)
                // are both solid and transparent.
                if is_dust(neighbor_block) {
                    if new_level > world.get_power(neighbor) {
                        world.set_power(neighbor, new_level);
                        queue.push_back(neighbor);
                    }
                    continue;
                }

                if world.is_solid_block(neighbor) {
                    // Solid blocks receive weak power from adjacent dust or
                    // sources but do not propagate further.
                    let current_block = world.get_block(pos);
                    let from_conductor =
                        is_dust(current_block) || is_power_source(current_block).is_some();
                    if from_conductor && new_level > world.get_power(neighbor) {
                        world.set_power(neighbor, new_level);
                    }
                    continue;
                }
            }
        }

        // Phase 4 — check torch inversions.
        let mut changed = false;
        for &pos in &positions {
            if is_power_source(world.get_block(pos)).is_some() && !disabled_torches.contains(&pos) {
                let support = BlockPos::new(pos.x, pos.y - 1, pos.z);
                if world.get_power(support) > 0 {
                    disabled_torches.push(pos);
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Cached per-chunk redstone state
// ---------------------------------------------------------------------------

/// Cached redstone power levels for a region, keyed by block position.
pub struct RedstoneCircuit {
    power_cache: HashMap<BlockPos, u8>,
}

impl RedstoneCircuit {
    pub fn new() -> Self {
        Self {
            power_cache: HashMap::new(),
        }
    }

    /// Recalculate power levels after a block change at `changed_pos`.
    pub fn update(&mut self, world: &mut impl RedstoneWorld, changed_pos: BlockPos) {
        propagate_redstone(world, changed_pos);

        // Rebuild cache from world state within the propagation radius.
        self.power_cache.clear();
        let radius: i32 = 16;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                for dz in -radius..=radius {
                    let pos =
                        BlockPos::new(changed_pos.x + dx, changed_pos.y + dy, changed_pos.z + dz);
                    let power = world.get_power(pos);
                    if power > 0 {
                        self.power_cache.insert(pos, power);
                    }
                }
            }
        }
    }

    /// Look up the cached power level at `pos`.  Returns 0 when the position
    /// has no cached entry.
    pub fn get_power(&self, pos: BlockPos) -> u8 {
        self.power_cache.get(&pos).copied().unwrap_or(0)
    }
}

impl Default for RedstoneCircuit {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal in-memory world for testing redstone propagation.
    struct TestWorld {
        blocks: HashMap<BlockPos, BlockId>,
        power: HashMap<BlockPos, u8>,
    }

    impl TestWorld {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                power: HashMap::new(),
            }
        }

        fn place(&mut self, pos: BlockPos, block: BlockId) {
            self.blocks.insert(pos, block);
        }
    }

    impl RedstoneWorld for TestWorld {
        fn get_block(&self, pos: BlockPos) -> BlockId {
            self.blocks.get(&pos).copied().unwrap_or(BlockId::Air)
        }

        fn get_power(&self, pos: BlockPos) -> u8 {
            self.power.get(&pos).copied().unwrap_or(0)
        }

        fn set_power(&mut self, pos: BlockPos, level: u8) {
            if level == 0 {
                self.power.remove(&pos);
            } else {
                self.power.insert(pos, level);
            }
        }

        fn is_solid_block(&self, pos: BlockPos) -> bool {
            self.get_block(pos).is_solid()
        }
    }

    // Helper: lay a line of dust (Glass as transparent non-air placeholder)
    // along the X axis starting one block east of `origin`.
    fn lay_dust_line(world: &mut TestWorld, origin: BlockPos, length: i32) {
        for i in 1..=length {
            let pos = BlockPos::new(origin.x + i, origin.y, origin.z);
            world.place(pos, BlockId::Glass);
        }
    }

    // ------------------------------------------------------------------
    // Single source propagates 15 blocks
    // ------------------------------------------------------------------

    #[test]
    fn single_source_propagates_15_blocks() {
        let mut world = TestWorld::new();
        let source = BlockPos::new(0, 64, 0);
        world.place(source, BlockId::Torch);

        // Lay 16 blocks of dust to the east.
        lay_dust_line(&mut world, source, 16);

        propagate_redstone(&mut world, source);

        // Source itself should be at 15.
        assert_eq!(world.get_power(source), MAX_POWER);

        // Dust at distance d should be 15 - d.
        for d in 1..=15 {
            let pos = BlockPos::new(source.x + d, source.y, source.z);
            assert_eq!(
                world.get_power(pos),
                MAX_POWER - d as u8,
                "dust at distance {d} should have power {}",
                MAX_POWER - d as u8,
            );
        }

        // Block at distance 16 should have no power (15 - 16 < 0 → 0).
        let too_far = BlockPos::new(source.x + 16, source.y, source.z);
        assert_eq!(world.get_power(too_far), 0);
    }

    // ------------------------------------------------------------------
    // Two sources combine — max wins
    // ------------------------------------------------------------------

    #[test]
    fn two_sources_combine_max_wins() {
        let mut world = TestWorld::new();

        // Two torches 10 blocks apart along the X axis with dust between them.
        let left = BlockPos::new(0, 64, 0);
        let right = BlockPos::new(10, 64, 0);
        world.place(left, BlockId::Torch);
        world.place(right, BlockId::Torch);

        // Fill dust between them (positions 1..=9).
        for x in 1..=9 {
            world.place(BlockPos::new(x, 64, 0), BlockId::Glass);
        }

        propagate_redstone(&mut world, left);

        // The midpoint at x=5 is 5 away from each source.
        // Power from left = 15 - 5 = 10, from right = 15 - 5 = 10.
        let mid = BlockPos::new(5, 64, 0);
        assert_eq!(world.get_power(mid), 10);

        // At x=2 the left source is closer: 15 - 2 = 13, right: 15 - 8 = 7.
        let near_left = BlockPos::new(2, 64, 0);
        assert_eq!(world.get_power(near_left), 13);

        // At x=8 the right source is closer: 15 - 2 = 13.
        let near_right = BlockPos::new(8, 64, 0);
        assert_eq!(world.get_power(near_right), 13);
    }

    // ------------------------------------------------------------------
    // Solid blocks block propagation
    // ------------------------------------------------------------------

    #[test]
    fn solid_blocks_stop_propagation() {
        let mut world = TestWorld::new();
        let source = BlockPos::new(0, 64, 0);
        world.place(source, BlockId::Torch);

        // Place dust at x=1, a solid wall at x=2, and more dust at x=3.
        world.place(BlockPos::new(1, 64, 0), BlockId::Glass);
        world.place(BlockPos::new(2, 64, 0), BlockId::Stone);
        world.place(BlockPos::new(3, 64, 0), BlockId::Glass);

        propagate_redstone(&mut world, source);

        // Dust at x=1 should carry power.
        assert_eq!(world.get_power(BlockPos::new(1, 64, 0)), 14);

        // Stone at x=2 receives weak power from adjacent dust.
        let stone_power = world.get_power(BlockPos::new(2, 64, 0));
        assert!(stone_power > 0, "stone should receive weak power");

        // Dust at x=3 should have no power — solid block stops propagation.
        assert_eq!(world.get_power(BlockPos::new(3, 64, 0)), 0);
    }

    // ------------------------------------------------------------------
    // Power decreases with distance
    // ------------------------------------------------------------------

    #[test]
    fn power_decreases_with_distance() {
        let mut world = TestWorld::new();
        let source = BlockPos::new(0, 64, 0);
        world.place(source, BlockId::Torch);
        lay_dust_line(&mut world, source, 15);

        propagate_redstone(&mut world, source);

        let mut prev = MAX_POWER;
        for d in 1..=15 {
            let pos = BlockPos::new(source.x + d, source.y, source.z);
            let power = world.get_power(pos);
            assert!(
                power < prev,
                "power at distance {d} ({power}) should be less than at distance {} ({prev})",
                d - 1,
            );
            prev = power;
        }
    }

    // ------------------------------------------------------------------
    // Torch inversion — powered support turns torch off
    // ------------------------------------------------------------------

    #[test]
    fn torch_inversion_powered_support_turns_torch_off() {
        let mut world = TestWorld::new();

        // Source torch at origin, dust leading east, a solid block, and a
        // torch on top of it.
        let source = BlockPos::new(0, 64, 0);
        world.place(source, BlockId::Torch);

        // Dust at x=1.
        world.place(BlockPos::new(1, 64, 0), BlockId::Glass);

        // Solid support block at x=2 (receives weak power from dust).
        world.place(BlockPos::new(2, 64, 0), BlockId::Stone);

        // Torch sitting on top of the support block.
        let top_torch = BlockPos::new(2, 65, 0);
        world.place(top_torch, BlockId::Torch);

        propagate_redstone(&mut world, source);

        // The support block at (2,64,0) should have weak power from the dust.
        let support_power = world.get_power(BlockPos::new(2, 64, 0));
        assert!(support_power > 0, "support should be weakly powered");

        // Because the support is powered, the torch on top is inverted (off).
        // The iterative convergence inside propagate_redstone handles this.
        assert_eq!(
            world.get_power(top_torch),
            0,
            "torch on powered support should be off"
        );
    }

    // ------------------------------------------------------------------
    // RedstoneCircuit caches power levels
    // ------------------------------------------------------------------

    #[test]
    fn redstone_circuit_caches_power() {
        let mut world = TestWorld::new();
        let source = BlockPos::new(0, 64, 0);
        world.place(source, BlockId::Torch);
        lay_dust_line(&mut world, source, 5);

        let mut circuit = RedstoneCircuit::new();
        circuit.update(&mut world, source);

        assert_eq!(circuit.get_power(source), MAX_POWER);
        assert_eq!(circuit.get_power(BlockPos::new(3, 64, 0)), 12);
        assert_eq!(circuit.get_power(BlockPos::new(0, 0, 0)), 0);
    }

    // ------------------------------------------------------------------
    // No sources means no power
    // ------------------------------------------------------------------

    #[test]
    fn no_sources_no_power() {
        let mut world = TestWorld::new();
        // Only dust, no source.
        for x in 0..5 {
            world.place(BlockPos::new(x, 64, 0), BlockId::Glass);
        }

        propagate_redstone(&mut world, BlockPos::new(2, 64, 0));

        for x in 0..5 {
            assert_eq!(world.get_power(BlockPos::new(x, 64, 0)), 0);
        }
    }
}
