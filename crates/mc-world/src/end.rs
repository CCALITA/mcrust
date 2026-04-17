use mc_core::block::BlockId;
use mc_core::pos::CHUNK_SIZE;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::chunk::Chunk;

/// Platform base height for the main island.
const PLATFORM_Y: i32 = 64;

/// Approximate radius (in blocks) of the main End island around world origin.
const MAIN_ISLAND_RADIUS: f64 = 40.0;

/// Chunk radius threshold for the main island region.
/// Chunks whose center is within this many chunks of the origin belong to the
/// main island zone.
const MAIN_ISLAND_CHUNK_RADIUS: f64 = 3.0;

/// Chunk radius beyond which outer floating islands may appear.
const OUTER_ISLAND_MIN_CHUNK_RADIUS: f64 = 8.0;

/// Number of obsidian pillars arranged in a ring around the origin.
const PILLAR_COUNT: usize = 10;

/// Radius of the pillar ring (in blocks from world origin).
const PILLAR_RING_RADIUS: f64 = 45.0;

/// Half-width of each pillar (pillars are 3x3, so half-width is 1).
const PILLAR_HALF_WIDTH: i32 = 1;

/// Minimum pillar height (blocks above PLATFORM_Y).
const PILLAR_MIN_HEIGHT: i32 = 20;

/// Maximum pillar height (blocks above PLATFORM_Y).
const PILLAR_MAX_HEIGHT: i32 = 50;

/// Scale for the 2D noise that modulates the main island edge.
const ISLAND_EDGE_NOISE_SCALE: f64 = 0.08;

/// How much the edge noise modulates the radius (in blocks).
const ISLAND_EDGE_AMPLITUDE: f64 = 8.0;

/// Scale for the 3D noise used to generate outer floating islands.
const OUTER_ISLAND_NOISE_SCALE: f64 = 0.04;

/// Threshold above which 3D noise produces solid EndStone for outer islands.
const OUTER_ISLAND_THRESHOLD: f64 = 0.55;

/// Minimum Y for outer floating islands.
const OUTER_ISLAND_Y_MIN: i32 = 55;

/// Maximum Y for outer floating islands.
const OUTER_ISLAND_Y_MAX: i32 = 75;

/// Terrain generator for the End dimension.
///
/// Produces:
/// - A main EndStone island at y=64 around world origin
/// - 10 obsidian pillars in a ring around the origin
/// - Sparse floating EndStone islands in outer regions
/// - Void (air) everywhere else
pub struct EndTerrainGen {
    /// 2D noise for modulating the main island edge shape.
    edge_noise: Fbm<Perlin>,
    /// 3D noise for generating outer floating islands.
    island_noise: Fbm<Perlin>,
    /// Seed used for deterministic pillar height variation.
    seed: u64,
}

impl EndTerrainGen {
    /// Creates a new End terrain generator with the given seed.
    pub fn new(seed: u64) -> Self {
        let base_seed = seed as u32;

        let edge_noise = Fbm::<Perlin>::new(base_seed.wrapping_add(5000))
            .set_octaves(4)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let island_noise = Fbm::<Perlin>::new(base_seed.wrapping_add(6000))
            .set_octaves(4)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        Self {
            edge_noise,
            island_noise,
            seed,
        }
    }

    /// Generates a chunk at chunk coordinates `(cx, cz)`.
    ///
    /// The chunk is filled with air by default (void). Solid blocks are placed
    /// only for the main island, obsidian pillars, or outer floating islands.
    pub fn generate(&self, cx: i32, cz: i32) -> Chunk {
        let mut chunk = Chunk::new();
        let base_x = cx * CHUNK_SIZE;
        let base_z = cz * CHUNK_SIZE;

        // Determine which region this chunk falls into based on its center
        // distance from the origin (in chunk units).
        let chunk_center_x = base_x as f64 + (CHUNK_SIZE as f64 / 2.0);
        let chunk_center_z = base_z as f64 + (CHUNK_SIZE as f64 / 2.0);
        let chunk_dist = (chunk_center_x * chunk_center_x + chunk_center_z * chunk_center_z).sqrt()
            / CHUNK_SIZE as f64;

        if chunk_dist <= MAIN_ISLAND_CHUNK_RADIUS {
            self.generate_main_island(&mut chunk, base_x, base_z);
        }

        // Pillars can appear in chunks near the main island ring
        self.generate_pillars(&mut chunk, cx, cz, base_x, base_z);

        if chunk_dist >= OUTER_ISLAND_MIN_CHUNK_RADIUS {
            self.generate_outer_islands(&mut chunk, base_x, base_z);
        }

        chunk
    }

    /// Places the main EndStone island platform centered at world origin.
    ///
    /// The island is a circular disc at `PLATFORM_Y` with a noise-modulated
    /// edge to give it an organic shape. A few layers of depth are added below
    /// the surface.
    fn generate_main_island(&self, chunk: &mut Chunk, base_x: i32, base_z: i32) {
        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                let world_x = base_x + local_x as i32;
                let world_z = base_z + local_z as i32;

                let dist = ((world_x as f64).powi(2) + (world_z as f64).powi(2)).sqrt();

                // Modulate the effective radius with noise for organic edges
                let edge_mod = self.edge_noise.get([
                    world_x as f64 * ISLAND_EDGE_NOISE_SCALE,
                    world_z as f64 * ISLAND_EDGE_NOISE_SCALE,
                ]);
                let effective_radius = MAIN_ISLAND_RADIUS + edge_mod * ISLAND_EDGE_AMPLITUDE;

                if dist <= effective_radius {
                    // Place a disc of EndStone with some depth
                    // Depth tapers near the edge for a natural bowl shape
                    let edge_factor = 1.0 - (dist / effective_radius).min(1.0);
                    let depth = (edge_factor * 5.0).ceil() as i32;

                    for y in (PLATFORM_Y - depth)..=PLATFORM_Y {
                        chunk.set_block(local_x, y, local_z, BlockId::EndStone);
                    }
                }
            }
        }
    }

    /// Places obsidian pillars if their positions fall within this chunk.
    ///
    /// Pillars are 3x3 Obsidian columns at fixed angular positions around
    /// the origin, with deterministic heights derived from the seed.
    fn generate_pillars(&self, chunk: &mut Chunk, cx: i32, cz: i32, base_x: i32, base_z: i32) {
        for i in 0..PILLAR_COUNT {
            let angle = (i as f64 / PILLAR_COUNT as f64) * std::f64::consts::TAU;
            let pillar_x = (PILLAR_RING_RADIUS * angle.cos()).round() as i32;
            let pillar_z = (PILLAR_RING_RADIUS * angle.sin()).round() as i32;

            // Check if any part of this 3x3 pillar overlaps this chunk
            let pillar_min_x = pillar_x - PILLAR_HALF_WIDTH;
            let pillar_max_x = pillar_x + PILLAR_HALF_WIDTH;
            let pillar_min_z = pillar_z - PILLAR_HALF_WIDTH;
            let pillar_max_z = pillar_z + PILLAR_HALF_WIDTH;

            let chunk_min_x = cx * CHUNK_SIZE;
            let chunk_max_x = chunk_min_x + CHUNK_SIZE - 1;
            let chunk_min_z = cz * CHUNK_SIZE;
            let chunk_max_z = chunk_min_z + CHUNK_SIZE - 1;

            // Axis-aligned overlap test
            if pillar_max_x < chunk_min_x
                || pillar_min_x > chunk_max_x
                || pillar_max_z < chunk_min_z
                || pillar_min_z > chunk_max_z
            {
                continue;
            }

            // Deterministic height for this pillar based on seed and index
            let pillar_height = pillar_height_for_index(self.seed, i);

            // Place only the blocks within this chunk
            for px in pillar_min_x..=pillar_max_x {
                for pz in pillar_min_z..=pillar_max_z {
                    if px < chunk_min_x || px > chunk_max_x || pz < chunk_min_z || pz > chunk_max_z
                    {
                        continue;
                    }
                    let local_x = (px - base_x) as usize;
                    let local_z = (pz - base_z) as usize;

                    for y in PLATFORM_Y..=(PLATFORM_Y + pillar_height) {
                        chunk.set_block(local_x, y, local_z, BlockId::Obsidian);
                    }
                }
            }
        }
    }

    /// Generates sparse floating EndStone islands in outer regions using 3D noise.
    fn generate_outer_islands(&self, chunk: &mut Chunk, base_x: i32, base_z: i32) {
        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                let world_x = base_x + local_x as i32;
                let world_z = base_z + local_z as i32;

                for y in OUTER_ISLAND_Y_MIN..=OUTER_ISLAND_Y_MAX {
                    let nx = world_x as f64 * OUTER_ISLAND_NOISE_SCALE;
                    let ny = y as f64 * OUTER_ISLAND_NOISE_SCALE;
                    let nz = world_z as f64 * OUTER_ISLAND_NOISE_SCALE;

                    let value = self.island_noise.get([nx, ny, nz]);

                    if value > OUTER_ISLAND_THRESHOLD {
                        chunk.set_block(local_x, y, local_z, BlockId::EndStone);
                    }
                }
            }
        }
    }
}

/// Returns a deterministic pillar height for the given pillar index and seed.
///
/// Heights range from `PILLAR_MIN_HEIGHT` to `PILLAR_MAX_HEIGHT` inclusive.
fn pillar_height_for_index(seed: u64, index: usize) -> i32 {
    // Simple hash to distribute heights
    let hash = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(index as u64 * 1442695040888963407);
    let range = (PILLAR_MAX_HEIGHT - PILLAR_MIN_HEIGHT + 1) as u64;
    PILLAR_MIN_HEIGHT + (hash % range) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_island_has_end_stone_at_origin() {
        let terrain = EndTerrainGen::new(42);
        let chunk = terrain.generate(0, 0);
        // World (0, 64, 0) is local (0, 64, 0) in chunk (0, 0)
        assert_eq!(
            chunk.get_block(0, PLATFORM_Y, 0),
            BlockId::EndStone,
            "expected EndStone at world (0, 64, 0)"
        );
    }

    #[test]
    fn main_island_has_depth() {
        let terrain = EndTerrainGen::new(42);
        let chunk = terrain.generate(0, 0);
        // Near the center the island should have depth below y=64
        assert_eq!(
            chunk.get_block(0, PLATFORM_Y - 1, 0),
            BlockId::EndStone,
            "expected EndStone depth below platform at origin"
        );
    }

    #[test]
    fn obsidian_pillars_exist() {
        let terrain = EndTerrainGen::new(42);

        // Check that at least one pillar is found by scanning all pillar
        // positions and generating the chunk that contains each pillar center.
        let mut found_obsidian = false;
        for i in 0..PILLAR_COUNT {
            let angle = (i as f64 / PILLAR_COUNT as f64) * std::f64::consts::TAU;
            let pillar_x = (PILLAR_RING_RADIUS * angle.cos()).round() as i32;
            let pillar_z = (PILLAR_RING_RADIUS * angle.sin()).round() as i32;

            let cx = pillar_x.div_euclid(CHUNK_SIZE);
            let cz = pillar_z.div_euclid(CHUNK_SIZE);
            let chunk = terrain.generate(cx, cz);

            let local_x = pillar_x.rem_euclid(CHUNK_SIZE) as usize;
            let local_z = pillar_z.rem_euclid(CHUNK_SIZE) as usize;

            if chunk.get_block(local_x, PLATFORM_Y + 1, local_z) == BlockId::Obsidian {
                found_obsidian = true;
                break;
            }
        }
        assert!(found_obsidian, "expected at least one obsidian pillar");
    }

    #[test]
    fn pillar_heights_vary() {
        let seed = 42u64;
        let heights: Vec<i32> = (0..PILLAR_COUNT)
            .map(|i| pillar_height_for_index(seed, i))
            .collect();

        // All heights should be within range
        for (i, &h) in heights.iter().enumerate() {
            assert!(
                h >= PILLAR_MIN_HEIGHT && h <= PILLAR_MAX_HEIGHT,
                "pillar {i} height {h} out of range [{PILLAR_MIN_HEIGHT}, {PILLAR_MAX_HEIGHT}]"
            );
        }

        // Heights should not all be the same
        let distinct: std::collections::HashSet<i32> = heights.iter().copied().collect();
        assert!(
            distinct.len() > 1,
            "expected varying pillar heights, got {heights:?}"
        );
    }

    #[test]
    fn outer_areas_are_mostly_void() {
        let terrain = EndTerrainGen::new(42);
        // Generate a chunk well outside the main island and outer island range
        // At chunk (100, 100) = world (1600, 1600), far from origin
        let chunk = terrain.generate(100, 100);

        let mut solid_count = 0u32;
        let total_columns = (CHUNK_SIZE * CHUNK_SIZE) as u32;

        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                for y in OUTER_ISLAND_Y_MIN..=OUTER_ISLAND_Y_MAX {
                    if chunk.get_block(local_x, y, local_z) != BlockId::Air {
                        solid_count += 1;
                        break;
                    }
                }
            }
        }

        // The outer islands are sparse; most columns should be void.
        // Allow up to 50% solid columns (generous threshold for noise variation).
        assert!(
            solid_count < total_columns / 2,
            "expected mostly void in outer chunk, but found {solid_count}/{total_columns} solid columns"
        );
    }

    #[test]
    fn void_between_main_and_outer_islands() {
        let terrain = EndTerrainGen::new(42);
        // Chunks in the gap between main island (radius ~3) and outer islands
        // (radius ~8) should be entirely void. Chunk (5, 0) center is at
        // world x=88, which is ~5.5 chunk radii from origin.
        let chunk = terrain.generate(5, 0);

        let mut all_air = true;
        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                for y in -64..320 {
                    if chunk.get_block(local_x, y, local_z) != BlockId::Air {
                        // Pillars may extend into nearby chunks, allow Obsidian
                        if chunk.get_block(local_x, y, local_z) != BlockId::Obsidian {
                            all_air = false;
                        }
                    }
                }
            }
        }
        assert!(
            all_air,
            "expected void (or only obsidian pillars) in gap between main and outer islands"
        );
    }

    #[test]
    fn different_seeds_produce_different_terrain() {
        let gen_a = EndTerrainGen::new(1);
        let gen_b = EndTerrainGen::new(999);

        // Check a chunk at the edge of the main island where noise modulation
        // causes different seeds to produce different block patterns.
        // Chunk (2, 0) starts at world x=32, near the ~40 block radius edge.
        let chunk_a = gen_a.generate(2, 0);
        let chunk_b = gen_b.generate(2, 0);

        let mut differ = false;
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                if chunk_a.get_block(x, PLATFORM_Y, z) != chunk_b.get_block(x, PLATFORM_Y, z) {
                    differ = true;
                    break;
                }
            }
            if differ {
                break;
            }
        }
        assert!(differ, "different seeds should produce different terrain");
    }
}
