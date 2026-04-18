use mc_core::block::BlockId;
use mc_core::pos::{CHUNK_SIZE, WORLD_BOTTOM, WORLD_TOP};

use crate::chunk::Chunk;

/// Cardinal direction for corridor placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorridorDirection {
    /// Extends along +X
    East,
    /// Extends along -X
    West,
    /// Extends along +Z
    South,
    /// Extends along -Z
    North,
}

/// Unique identifier for each structure type, used in the deterministic hash
/// to produce independent placement decisions per structure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructureType {
    Dungeon = 0,
    Village = 1,
    Mineshaft = 2,
}

/// Deterministic pseudo-random number generator using splitmix64-style hashing.
/// Mirrors the `PosRng` used by `OreGenerator` for consistency across the codebase.
struct PosRng {
    state: u64,
}

impl PosRng {
    fn from_seed(seed: u64) -> Self {
        let mut rng = Self { state: seed };
        // Diffuse bits with a few warm-up rounds.
        for _ in 0..4 {
            rng.next();
        }
        rng
    }

    fn next(&mut self) -> u64 {
        let mut z = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        self.state = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Returns a value in `[0, bound)`.
    fn next_bounded(&mut self, bound: u32) -> u32 {
        (self.next() % bound as u64) as u32
    }

    /// Returns a value in `[min, max]` (inclusive).
    fn next_range_inclusive(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let range = (max - min + 1) as u32;
        min + self.next_bounded(range) as i32
    }

    /// Returns `true` with the given probability (0.0 = never, 1.0 = always).
    fn next_bool(&mut self, probability: f64) -> bool {
        let val = (self.next() >> 11) as f64 / ((1u64 << 53) as f64);
        val < probability
    }
}

/// Deterministic hash combining seed with chunk coordinates and a structure tag.
/// Used to decide whether a chunk hosts a particular structure type.
fn structure_hash(seed: u64, cx: i32, cz: i32, structure_type: StructureType) -> u64 {
    let mut h = seed;
    h = h
        .wrapping_add(cx as u64)
        .wrapping_mul(6_364_136_223_846_793_005);
    h = h
        .wrapping_add(cz as u64)
        .wrapping_mul(6_364_136_223_846_793_005);
    h = h
        .wrapping_add(structure_type as u64)
        .wrapping_mul(6_364_136_223_846_793_005);
    // Extra mixing round.
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h
}

/// Returns `true` if the chunk at `(cx, cz)` should contain the given structure.
///
/// Rates:
/// - Dungeon: ~25% of chunks
/// - Village: ~2% of chunks
/// - Mineshaft: ~10% of chunks
fn should_generate_structure(cx: i32, cz: i32, seed: u64, structure_type: StructureType) -> bool {
    let h = structure_hash(seed, cx, cz, structure_type);
    match structure_type {
        StructureType::Dungeon => h.is_multiple_of(4),
        StructureType::Village => h.is_multiple_of(50),
        StructureType::Mineshaft => h.is_multiple_of(10),
    }
}

/// Generator that places dungeons, village houses, and mineshaft corridors into
/// chunks using deterministic seeded placement.
pub struct StructureGenerator {
    seed: u64,
}

impl StructureGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Determine which structures belong in the chunk and place them.
    pub fn generate_structures(&self, chunk: &mut Chunk, cx: i32, cz: i32) {
        if should_generate_structure(cx, cz, self.seed, StructureType::Dungeon) {
            let mut rng =
                PosRng::from_seed(structure_hash(self.seed, cx, cz, StructureType::Dungeon));
            let x = rng.next_range_inclusive(1, CHUNK_SIZE - 8) as usize;
            let y = rng.next_range_inclusive(WORLD_BOTTOM + 5, 49);
            let z = rng.next_range_inclusive(1, CHUNK_SIZE - 8) as usize;
            place_dungeon(chunk, x, y, z, &mut rng);
        }

        if should_generate_structure(cx, cz, self.seed, StructureType::Village) {
            let mut rng =
                PosRng::from_seed(structure_hash(self.seed, cx, cz, StructureType::Village));
            let x = rng.next_range_inclusive(1, CHUNK_SIZE - 6) as usize;
            let z = rng.next_range_inclusive(1, CHUNK_SIZE - 6) as usize;
            // Find a suitable surface y by scanning down from a reasonable height.
            let surface_y = find_surface(chunk, x, z);
            if let Some(sy) = surface_y {
                place_house(chunk, x, sy + 1, z);
            }
        }

        if should_generate_structure(cx, cz, self.seed, StructureType::Mineshaft) {
            let mut rng =
                PosRng::from_seed(structure_hash(self.seed, cx, cz, StructureType::Mineshaft));
            let x = rng.next_range_inclusive(2, CHUNK_SIZE - 4) as usize;
            let y = rng.next_range_inclusive(10, 40);
            let z = rng.next_range_inclusive(2, CHUNK_SIZE - 4) as usize;
            let length = rng.next_range_inclusive(8, 14) as usize;
            let dir = match rng.next_bounded(4) {
                0 => CorridorDirection::North,
                1 => CorridorDirection::South,
                2 => CorridorDirection::East,
                _ => CorridorDirection::West,
            };
            place_corridor(chunk, x, y, z, length, dir);
        }
    }
}

/// Scan downward to find the topmost solid block at the given local (x, z).
/// Returns `None` if no solid surface is found.
fn find_surface(chunk: &Chunk, x: usize, z: usize) -> Option<i32> {
    for y in (WORLD_BOTTOM..WORLD_TOP).rev() {
        let block = chunk.get_block(x, y, z);
        if block.is_solid() {
            return Some(y);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Dungeon
// ---------------------------------------------------------------------------

/// Place a dungeon room underground.
///
/// Dimensions: `width` x `depth` x 4 (height) where width and depth are
/// randomly chosen between 5 and 7.
///
/// - Walls: random mix of `Cobblestone` and `MossyCobblestone`
/// - Floor: `Cobblestone`
/// - Interior: `Air`
/// - Center: `Chest` (spawner placeholder)
fn place_dungeon(chunk: &mut Chunk, x: usize, y: i32, z: usize, rng: &mut PosRng) {
    let width = rng.next_range_inclusive(5, 7) as usize;
    let depth = rng.next_range_inclusive(5, 7) as usize;
    let height = 4i32;

    for dx in 0..width {
        for dz in 0..depth {
            for dy in 0..height {
                let bx = x + dx;
                let bz = z + dz;
                let by = y + dy;

                if bx >= CHUNK_SIZE as usize || bz >= CHUNK_SIZE as usize {
                    continue;
                }
                if !(WORLD_BOTTOM..WORLD_TOP).contains(&by) {
                    continue;
                }

                let is_wall_x = dx == 0 || dx == width - 1;
                let is_wall_z = dz == 0 || dz == depth - 1;
                let is_floor = dy == 0;
                let is_ceiling = dy == height - 1;

                let block = if is_floor {
                    BlockId::Cobblestone
                } else if is_wall_x || is_wall_z || is_ceiling {
                    if rng.next_bool(0.3) {
                        BlockId::MossyCobblestone
                    } else {
                        BlockId::Cobblestone
                    }
                } else {
                    BlockId::Air
                };

                chunk.set_block(bx, by, bz, block);
            }
        }
    }

    // Place a chest in the center of the room.
    let center_x = x + width / 2;
    let center_z = z + depth / 2;
    let chest_y = y + 1; // one above the floor
    if center_x < CHUNK_SIZE as usize && center_z < CHUNK_SIZE as usize {
        chunk.set_block(center_x, chest_y, center_z, BlockId::Chest);
    }
}

// ---------------------------------------------------------------------------
// Village house
// ---------------------------------------------------------------------------

/// Place a simple 5x4x5 wooden village house.
///
/// - Walls and floor: `OakPlanks`
/// - Flat roof: `OakPlanks`
/// - Door: 1x2 air opening on the south side (+Z)
/// - Interior: `Torch` on the north wall, `Chest` on the floor
pub fn place_house(chunk: &mut Chunk, x: usize, y: i32, z: usize) {
    let width: usize = 5;
    let height: i32 = 4;
    let depth: usize = 5;

    for dx in 0..width {
        for dz in 0..depth {
            for dy in 0..height {
                let bx = x + dx;
                let bz = z + dz;
                let by = y + dy;

                if bx >= CHUNK_SIZE as usize || bz >= CHUNK_SIZE as usize {
                    continue;
                }
                if !(WORLD_BOTTOM..WORLD_TOP).contains(&by) {
                    continue;
                }

                let is_wall_x = dx == 0 || dx == width - 1;
                let is_wall_z = dz == 0 || dz == depth - 1;
                let is_floor = dy == 0;
                let is_roof = dy == height - 1;

                let block = if is_floor || is_roof || is_wall_x || is_wall_z {
                    BlockId::OakPlanks
                } else {
                    BlockId::Air
                };

                chunk.set_block(bx, by, bz, block);
            }
        }
    }

    // Door: 1x2 air opening on the south wall (+Z side), centered.
    let door_x = x + width / 2;
    let door_z = z + depth - 1;
    if door_x < CHUNK_SIZE as usize && door_z < CHUNK_SIZE as usize {
        chunk.set_block(door_x, y + 1, door_z, BlockId::Air);
        chunk.set_block(door_x, y + 2, door_z, BlockId::Air);
    }

    // Torch on the interior north wall.
    let torch_x = x + width / 2;
    let torch_z = z + 1; // one block in from the north wall (dz=0)
    let torch_y = y + 2;
    if torch_x < CHUNK_SIZE as usize && torch_z < CHUNK_SIZE as usize {
        chunk.set_block(torch_x, torch_y, torch_z, BlockId::Torch);
    }

    // Chest on the floor inside.
    let chest_x = x + 1;
    let chest_z = z + 1;
    let chest_y = y + 1;
    if chest_x < CHUNK_SIZE as usize && chest_z < CHUNK_SIZE as usize {
        chunk.set_block(chest_x, chest_y, chest_z, BlockId::Chest);
    }
}

// ---------------------------------------------------------------------------
// Mineshaft corridor
// ---------------------------------------------------------------------------

/// Place a straight mineshaft corridor.
///
/// - 3 wide, 3 tall tunnel
/// - `OakPlanks` floor
/// - `OakLog` vertical supports every 4 blocks on both sides
/// - `OakLog` horizontal beam across the top at each support
/// - `Torch` on top of each support
/// - Air fills the interior
pub fn place_corridor(
    chunk: &mut Chunk,
    x: usize,
    y: i32,
    z: usize,
    length: usize,
    direction: CorridorDirection,
) {
    let (dx_step, dz_step): (i32, i32) = match direction {
        CorridorDirection::East => (1, 0),
        CorridorDirection::West => (-1, 0),
        CorridorDirection::South => (0, 1),
        CorridorDirection::North => (0, -1),
    };

    // The corridor is 3 blocks wide perpendicular to the direction.
    // "perpendicular" offset: rotate the direction 90 degrees.
    let (px, pz): (i32, i32) = match direction {
        CorridorDirection::East | CorridorDirection::West => (0, 1),
        CorridorDirection::South | CorridorDirection::North => (1, 0),
    };

    for i in 0..length as i32 {
        let base_x = x as i32 + dx_step * i;
        let base_z = z as i32 + dz_step * i;

        for w in -1..=1i32 {
            let bx = base_x + px * w;
            let bz = base_z + pz * w;

            if !(0..CHUNK_SIZE).contains(&bx) || !(0..CHUNK_SIZE).contains(&bz) {
                continue;
            }
            let ux = bx as usize;
            let uz = bz as usize;

            // Floor
            if (WORLD_BOTTOM..WORLD_TOP).contains(&y) {
                chunk.set_block(ux, y, uz, BlockId::OakPlanks);
            }

            // Air interior (y+1 and y+2)
            for dy in 1..=2 {
                let by = y + dy;
                if (WORLD_BOTTOM..WORLD_TOP).contains(&by) {
                    chunk.set_block(ux, by, uz, BlockId::Air);
                }
            }

            // Ceiling level (y+3) — air by default, beams placed below.
            let ceiling_y = y + 3;
            if (WORLD_BOTTOM..WORLD_TOP).contains(&ceiling_y) {
                chunk.set_block(ux, ceiling_y, uz, BlockId::Air);
            }
        }

        // Supports every 4 blocks.
        let is_support = i % 4 == 0;
        if is_support {
            for side in [-1i32, 1] {
                let sx = base_x + px * side;
                let sz = base_z + pz * side;

                if !(0..CHUNK_SIZE).contains(&sx) || !(0..CHUNK_SIZE).contains(&sz) {
                    continue;
                }
                let ux = sx as usize;
                let uz = sz as usize;

                // Vertical support posts (y+1 and y+2).
                for dy in 1..=2 {
                    let by = y + dy;
                    if (WORLD_BOTTOM..WORLD_TOP).contains(&by) {
                        chunk.set_block(ux, by, uz, BlockId::OakLog);
                    }
                }

                // Torch on top of the support (y+3, the ceiling level).
                let torch_y = y + 3;
                if (WORLD_BOTTOM..WORLD_TOP).contains(&torch_y) {
                    chunk.set_block(ux, torch_y, uz, BlockId::Torch);
                }
            }

            // Horizontal beam across the top (y+3) on the center column.
            let beam_y = y + 3;
            if (0..CHUNK_SIZE).contains(&base_x)
                && (0..CHUNK_SIZE).contains(&base_z)
                && (WORLD_BOTTOM..WORLD_TOP).contains(&beam_y)
            {
                chunk.set_block(base_x as usize, beam_y, base_z as usize, BlockId::OakLog);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::pos::CHUNK_SIZE;

    /// Create a chunk filled with stone up to a given height, grass on top, air above.
    fn make_terrain_chunk(surface_y: i32) -> Chunk {
        let mut chunk = Chunk::new();
        let size = CHUNK_SIZE as usize;
        for x in 0..size {
            for z in 0..size {
                for y in WORLD_BOTTOM..surface_y {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
                chunk.set_block(x, surface_y, z, BlockId::GrassBlock);
            }
        }
        chunk
    }

    // -----------------------------------------------------------------------
    // Dungeon tests
    // -----------------------------------------------------------------------

    #[test]
    fn dungeon_has_cobblestone_walls_and_air_inside() {
        let mut chunk = make_terrain_chunk(64);
        let mut rng = PosRng::from_seed(42);
        place_dungeon(&mut chunk, 4, 20, 4, &mut rng);

        // The dungeon is at least 5x5x4. Check that the perimeter at floor
        // level (dy=0) is cobblestone and the interior one layer up (dy=1) is air.

        // Floor corners should be Cobblestone.
        assert_eq!(chunk.get_block(4, 20, 4), BlockId::Cobblestone);

        // Interior block (one in from each wall, one above floor) should be Air.
        let interior = chunk.get_block(6, 21, 6);
        assert_eq!(
            interior,
            BlockId::Air,
            "Interior of dungeon should be air, got {interior:?}"
        );
    }

    #[test]
    fn dungeon_has_chest_in_center() {
        let mut chunk = make_terrain_chunk(64);
        let mut rng = PosRng::from_seed(42);

        // Capture width/depth from a separate rng with the same seed to predict center.
        let mut probe = PosRng::from_seed(42);
        let width = probe.next_range_inclusive(5, 7) as usize;
        let depth = probe.next_range_inclusive(5, 7) as usize;

        place_dungeon(&mut chunk, 4, 20, 4, &mut rng);

        let cx = 4 + width / 2;
        let cz = 4 + depth / 2;
        assert_eq!(
            chunk.get_block(cx, 21, cz),
            BlockId::Chest,
            "Expected Chest at dungeon center ({cx}, 21, {cz})"
        );
    }

    #[test]
    fn dungeon_walls_contain_mossy_cobblestone() {
        let mut chunk = make_terrain_chunk(64);
        let mut rng = PosRng::from_seed(12345);

        let mut probe = PosRng::from_seed(12345);
        let width = probe.next_range_inclusive(5, 7) as usize;
        let depth = probe.next_range_inclusive(5, 7) as usize;

        place_dungeon(&mut chunk, 2, 15, 2, &mut rng);

        // Scan walls for at least one MossyCobblestone.
        let mut found_mossy = false;
        for dx in 0..width {
            for dz in 0..depth {
                for dy in 1..4i32 {
                    let is_wall = dx == 0 || dx == width - 1 || dz == 0 || dz == depth - 1;
                    let is_ceiling = dy == 3;
                    if is_wall || is_ceiling {
                        if chunk.get_block(2 + dx, 15 + dy, 2 + dz) == BlockId::MossyCobblestone {
                            found_mossy = true;
                        }
                    }
                }
            }
        }
        assert!(
            found_mossy,
            "Expected at least one MossyCobblestone on dungeon walls"
        );
    }

    // -----------------------------------------------------------------------
    // Village house tests
    // -----------------------------------------------------------------------

    #[test]
    fn house_has_oak_planks_walls() {
        let mut chunk = Chunk::new();
        let base_y = 64;
        place_house(&mut chunk, 4, base_y, 4);

        // North wall (dz=0): all 5 blocks across should be OakPlanks at floor+1 (wall).
        for dx in 0..5usize {
            let block = chunk.get_block(4 + dx, base_y + 1, 4);
            assert_eq!(
                block,
                BlockId::OakPlanks,
                "North wall at dx={dx} should be OakPlanks, got {block:?}"
            );
        }

        // Interior should be air.
        let interior = chunk.get_block(6, base_y + 1, 6);
        assert_eq!(
            interior,
            BlockId::Air,
            "Interior of house should be air, got {interior:?}"
        );
    }

    #[test]
    fn house_has_door_opening() {
        let mut chunk = Chunk::new();
        let base_y = 64;
        place_house(&mut chunk, 4, base_y, 4);

        // Door: centered on south wall (+Z side = z + 4).
        let door_x = 4 + 5 / 2; // = 6
        let door_z = 4 + 4; // = 8

        let lower = chunk.get_block(door_x, base_y + 1, door_z);
        let upper = chunk.get_block(door_x, base_y + 2, door_z);
        assert_eq!(
            lower,
            BlockId::Air,
            "Lower door should be air, got {lower:?}"
        );
        assert_eq!(
            upper,
            BlockId::Air,
            "Upper door should be air, got {upper:?}"
        );
    }

    #[test]
    fn house_has_torch_and_chest() {
        let mut chunk = Chunk::new();
        let base_y = 64;
        place_house(&mut chunk, 4, base_y, 4);

        // Torch on the interior north wall at (x + width/2, y+2, z+1).
        let torch = chunk.get_block(4 + 5 / 2, base_y + 2, 5);
        assert_eq!(torch, BlockId::Torch, "Expected torch, got {torch:?}");

        // Chest on the floor inside at (x+1, y+1, z+1).
        let chest = chunk.get_block(5, base_y + 1, 5);
        assert_eq!(chest, BlockId::Chest, "Expected chest, got {chest:?}");
    }

    #[test]
    fn house_floor_is_oak_planks() {
        let mut chunk = Chunk::new();
        let base_y = 64;
        place_house(&mut chunk, 4, base_y, 4);

        for dx in 0..5usize {
            for dz in 0..5usize {
                let block = chunk.get_block(4 + dx, base_y, 4 + dz);
                assert_eq!(
                    block,
                    BlockId::OakPlanks,
                    "Floor at ({}, {}) should be OakPlanks, got {block:?}",
                    4 + dx,
                    4 + dz
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Mineshaft corridor tests
    // -----------------------------------------------------------------------

    #[test]
    fn corridor_has_planks_floor() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 5, y, 5, 8, CorridorDirection::East);

        // Floor should be OakPlanks along the corridor length, center line.
        for i in 0..8 {
            let block = chunk.get_block(5 + i, y, 5);
            assert_eq!(
                block,
                BlockId::OakPlanks,
                "Floor at x={} should be OakPlanks, got {block:?}",
                5 + i
            );
        }
    }

    #[test]
    fn corridor_has_air_interior() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 5, y, 5, 8, CorridorDirection::East);

        // Interior (y+1, y+2) along the center should be air.
        for i in 0..8 {
            for dy in 1..=2 {
                let block = chunk.get_block(5 + i, y + dy, 5);
                assert_eq!(
                    block,
                    BlockId::Air,
                    "Interior at x={}, dy={dy} should be Air, got {block:?}",
                    5 + i
                );
            }
        }
    }

    #[test]
    fn corridor_has_oak_log_supports() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 4, y, 5, 12, CorridorDirection::East);

        // Supports at i=0 and i=4 and i=8 on the side columns.
        for support_i in [0, 4, 8] {
            let sx = 4 + support_i;
            // Side z=4 and z=6 (perpendicular offset -1 and +1 from center z=5).
            for sz in [4usize, 6] {
                for dy in 1..=2 {
                    let block = chunk.get_block(sx, y + dy, sz);
                    assert_eq!(
                        block,
                        BlockId::OakLog,
                        "Support at ({sx}, y+{dy}, {sz}) should be OakLog, got {block:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn corridor_has_torches_on_supports() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 4, y, 5, 12, CorridorDirection::East);

        // Torches at ceiling level on side supports.
        for support_i in [0, 4, 8] {
            let sx = 4 + support_i;
            for sz in [4usize, 6] {
                let block = chunk.get_block(sx, y + 3, sz);
                assert_eq!(
                    block,
                    BlockId::Torch,
                    "Torch at ({sx}, y+3, {sz}) expected, got {block:?}"
                );
            }
        }
    }

    #[test]
    fn corridor_south_direction_extends_along_z() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 5, y, 2, 6, CorridorDirection::South);

        // Floor should extend along +Z from z=2 to z=7.
        for i in 0..6 {
            let block = chunk.get_block(5, y, 2 + i);
            assert_eq!(
                block,
                BlockId::OakPlanks,
                "Floor at z={} should be OakPlanks, got {block:?}",
                2 + i
            );
        }
    }

    // -----------------------------------------------------------------------
    // should_generate_structure tests
    // -----------------------------------------------------------------------

    #[test]
    fn should_generate_structure_is_deterministic() {
        let a = should_generate_structure(5, 10, 42, StructureType::Dungeon);
        let b = should_generate_structure(5, 10, 42, StructureType::Dungeon);
        assert_eq!(a, b, "Structure decision should be deterministic");
    }

    #[test]
    fn dungeon_rate_is_approximately_25_percent() {
        let seed = 9999;
        let total = 1000;
        let mut count = 0u32;
        for cx in 0..total {
            if should_generate_structure(cx, 0, seed, StructureType::Dungeon) {
                count += 1;
            }
        }
        // 25% of 1000 = 250. Allow generous margin.
        assert!(
            count > 200 && count < 300,
            "Expected ~250 dungeons out of {total}, got {count}"
        );
    }

    #[test]
    fn village_rate_is_approximately_2_percent() {
        let seed = 9999;
        let total = 10_000;
        let mut count = 0u32;
        for cx in 0..total {
            if should_generate_structure(cx, 0, seed, StructureType::Village) {
                count += 1;
            }
        }
        // 2% of 10000 = 200.
        assert!(
            count > 100 && count < 300,
            "Expected ~200 villages out of {total}, got {count}"
        );
    }

    #[test]
    fn mineshaft_rate_is_approximately_10_percent() {
        let seed = 9999;
        let total = 1000;
        let mut count = 0u32;
        for cx in 0..total {
            if should_generate_structure(cx, 0, seed, StructureType::Mineshaft) {
                count += 1;
            }
        }
        // 10% of 1000 = 100.
        assert!(
            count > 60 && count < 140,
            "Expected ~100 mineshafts out of {total}, got {count}"
        );
    }

    // -----------------------------------------------------------------------
    // StructureGenerator integration test
    // -----------------------------------------------------------------------

    #[test]
    fn generate_structures_is_deterministic() {
        let generator = StructureGenerator::new(42);

        let mut chunk_a = make_terrain_chunk(64);
        generator.generate_structures(&mut chunk_a, 0, 0);

        let mut chunk_b = make_terrain_chunk(64);
        generator.generate_structures(&mut chunk_b, 0, 0);

        let size = CHUNK_SIZE as usize;
        for x in 0..size {
            for z in 0..size {
                for y in WORLD_BOTTOM..WORLD_TOP {
                    assert_eq!(
                        chunk_a.get_block(x, y, z),
                        chunk_b.get_block(x, y, z),
                        "Mismatch at ({x}, {y}, {z})"
                    );
                }
            }
        }
    }

    #[test]
    fn generate_structures_places_blocks() {
        let generator = StructureGenerator::new(42);
        let mut chunk = make_terrain_chunk(64);
        generator.generate_structures(&mut chunk, 0, 0);

        // Count structure-related blocks in the chunk.
        let size = CHUNK_SIZE as usize;
        let mut cobble_count = 0u32;
        let mut planks_count = 0u32;
        let mut chest_count = 0u32;
        for x in 0..size {
            for z in 0..size {
                for y in WORLD_BOTTOM..WORLD_TOP {
                    match chunk.get_block(x, y, z) {
                        BlockId::Cobblestone | BlockId::MossyCobblestone => cobble_count += 1,
                        BlockId::OakPlanks => planks_count += 1,
                        BlockId::Chest => chest_count += 1,
                        _ => {}
                    }
                }
            }
        }

        // At least some structure blocks should have been placed across many
        // chunks. Test over several chunk coords to ensure at least one hits.
        let mut total_cobble = cobble_count;
        let mut total_planks = planks_count;
        let mut total_chest = chest_count;

        for c in 1..20 {
            let mut ch = make_terrain_chunk(64);
            generator.generate_structures(&mut ch, c, c);
            for x in 0..size {
                for z in 0..size {
                    for y in WORLD_BOTTOM..WORLD_TOP {
                        match ch.get_block(x, y, z) {
                            BlockId::Cobblestone | BlockId::MossyCobblestone => total_cobble += 1,
                            BlockId::OakPlanks => total_planks += 1,
                            BlockId::Chest => total_chest += 1,
                            _ => {}
                        }
                    }
                }
            }
        }

        assert!(
            total_cobble > 0,
            "Expected some cobblestone from dungeon generation"
        );
        assert!(
            total_planks > 0,
            "Expected some oak planks from house/corridor generation"
        );
        assert!(
            total_chest > 0,
            "Expected at least one chest from structure generation"
        );
    }
}
