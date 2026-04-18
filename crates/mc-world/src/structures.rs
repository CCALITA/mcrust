use mc_core::pos::{CHUNK_SIZE, WORLD_BOTTOM, WORLD_TOP};

use crate::chunk::Chunk;
use crate::structure_types::{place_corridor, place_dungeon, place_house};

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
pub(crate) struct PosRng {
    state: u64,
}

impl PosRng {
    pub(crate) fn from_seed(seed: u64) -> Self {
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
    pub(crate) fn next_bounded(&mut self, bound: u32) -> u32 {
        (self.next() % bound as u64) as u32
    }

    /// Returns a value in `[min, max]` (inclusive).
    pub(crate) fn next_range_inclusive(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let range = (max - min + 1) as u32;
        min + self.next_bounded(range) as i32
    }

    /// Returns `true` with the given probability (0.0 = never, 1.0 = always).
    pub(crate) fn next_bool(&mut self, probability: f64) -> bool {
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
        StructureType::Dungeon => h % 4 == 0,
        StructureType::Village => h % 50 == 0,
        StructureType::Mineshaft => h % 10 == 0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::block::BlockId;
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
        assert!(
            count > 60 && count < 140,
            "Expected ~100 mineshafts out of {total}, got {count}"
        );
    }

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

        assert!(total_cobble > 0, "Expected some cobblestone from dungeon generation");
        assert!(total_planks > 0, "Expected some oak planks from house/corridor generation");
        assert!(total_chest > 0, "Expected at least one chest from structure generation");
    }
}
