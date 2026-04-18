use mc_core::block::BlockId;
use mc_core::pos::{CHUNK_SIZE, WORLD_BOTTOM, WORLD_TOP};

use crate::chunk::Chunk;

/// Configuration for a single ore type's distribution in the world.
#[derive(Debug, Clone)]
pub struct OreConfig {
    pub block: BlockId,
    pub min_y: i32,
    pub max_y: i32,
    pub vein_size: u32,
    pub veins_per_chunk: u32,
}

/// Returns the default Minecraft-like ore distribution table.
pub fn default_ore_configs() -> Vec<OreConfig> {
    vec![
        OreConfig {
            block: BlockId::CoalOre,
            min_y: 0,
            max_y: 128,
            vein_size: 17,
            veins_per_chunk: 20,
        },
        OreConfig {
            block: BlockId::IronOre,
            min_y: -64,
            max_y: 72,
            vein_size: 9,
            veins_per_chunk: 20,
        },
        OreConfig {
            block: BlockId::GoldOre,
            min_y: -64,
            max_y: 32,
            vein_size: 9,
            veins_per_chunk: 2,
        },
        OreConfig {
            block: BlockId::DiamondOre,
            min_y: -64,
            max_y: 16,
            vein_size: 8,
            veins_per_chunk: 1,
        },
    ]
}

/// Deterministic pseudo-random number generator seeded from position data.
/// Uses a splitmix64-style hash for reproducibility without external crates.
struct PosRng {
    state: u64,
}

impl PosRng {
    fn new(seed: u64, cx: i32, cz: i32, ore_index: u32, vein_index: u32) -> Self {
        let mut s = seed;
        s = s
            .wrapping_add(cx as u64)
            .wrapping_mul(6_364_136_223_846_793_005);
        s = s
            .wrapping_add(cz as u64)
            .wrapping_mul(6_364_136_223_846_793_005);
        s = s
            .wrapping_add(ore_index as u64)
            .wrapping_mul(6_364_136_223_846_793_005);
        s = s
            .wrapping_add(vein_index as u64)
            .wrapping_mul(6_364_136_223_846_793_005);
        // Run a few rounds to diffuse the bits
        let mut rng = Self { state: s };
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

    /// Returns a value in `[min, max)`.
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let range = (max - min) as u32;
        min + self.next_bounded(range) as i32
    }

    /// Returns a float in `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

/// Generator that places ore veins into chunks deterministically.
pub struct OreGenerator {
    seed: u64,
    configs: Vec<OreConfig>,
}

impl OreGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            configs: default_ore_configs(),
        }
    }

    /// Create an `OreGenerator` with custom ore configurations.
    pub fn with_configs(seed: u64, configs: Vec<OreConfig>) -> Self {
        Self { seed, configs }
    }

    /// Place ore veins into the given chunk at chunk coordinates `(cx, cz)`.
    ///
    /// Only replaces `BlockId::Stone` blocks — other blocks are left untouched.
    pub fn generate_ores(&self, chunk: &mut Chunk, cx: i32, cz: i32) {
        for (ore_index, config) in self.configs.iter().enumerate() {
            let clamped_min_y = config.min_y.max(WORLD_BOTTOM);
            let clamped_max_y = config.max_y.min(WORLD_TOP);
            if clamped_min_y >= clamped_max_y {
                continue;
            }

            for vein_index in 0..config.veins_per_chunk {
                let mut rng = PosRng::new(self.seed, cx, cz, ore_index as u32, vein_index);

                let center_x = rng.next_bounded(CHUNK_SIZE as u32) as i32;
                let center_y = rng.next_range(clamped_min_y, clamped_max_y);
                let center_z = rng.next_bounded(CHUNK_SIZE as u32) as i32;

                place_vein(
                    chunk,
                    &mut rng,
                    config.block,
                    center_x,
                    center_y,
                    center_z,
                    config.vein_size,
                );
            }
        }
    }
}

/// Place a roughly spherical cluster of ore blocks centered at
/// `(center_x, center_y, center_z)` in local chunk coordinates.
///
/// The radius is derived from `vein_size`. Blocks are placed with a
/// probability that decreases with distance from the center, and only
/// `Stone` blocks are replaced.
fn place_vein(
    chunk: &mut Chunk,
    rng: &mut PosRng,
    block: BlockId,
    center_x: i32,
    center_y: i32,
    center_z: i32,
    vein_size: u32,
) {
    // Approximate radius so the expected number of placed blocks is close to vein_size.
    // Volume of sphere = (4/3)*pi*r^3; we want roughly vein_size blocks inside,
    // accounting for the probability falloff. A radius of cbrt(vein_size) works well.
    let radius = (vein_size as f64).cbrt().ceil() as i32;

    let radius_f = radius as f64;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                let dist_sq = (dx * dx + dy * dy + dz * dz) as f64;
                let max_dist_sq = radius_f * radius_f;
                if dist_sq > max_dist_sq {
                    continue;
                }

                // Probability decreases linearly with distance from center.
                let probability = 1.0 - (dist_sq / max_dist_sq).sqrt();
                if rng.next_f64() >= probability {
                    continue;
                }

                let bx = center_x + dx;
                let by = center_y + dy;
                let bz = center_z + dz;

                // Stay within chunk boundaries (0..CHUNK_SIZE) horizontally.
                if !(0..CHUNK_SIZE).contains(&bx) || !(0..CHUNK_SIZE).contains(&bz) {
                    continue;
                }
                // Stay within world height.
                if !(WORLD_BOTTOM..WORLD_TOP).contains(&by) {
                    continue;
                }

                let ux = bx as usize;
                let uz = bz as usize;

                if chunk.get_block(ux, by, uz) == BlockId::Stone {
                    chunk.set_block(ux, by, uz, block);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::pos::CHUNK_SIZE;

    /// Fill an entire chunk with stone, as the terrain generator would.
    fn stone_filled_chunk() -> Chunk {
        let mut chunk = Chunk::new();
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in WORLD_BOTTOM..WORLD_TOP {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
            }
        }
        chunk
    }

    /// Count all blocks of a given type in a chunk within the specified Y range.
    fn count_block_in_range(chunk: &Chunk, block: BlockId, min_y: i32, max_y: i32) -> u32 {
        let mut count = 0u32;
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in min_y..max_y {
                    if chunk.get_block(x, y, z) == block {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    #[test]
    fn generates_coal_ore_at_valid_heights() {
        let mut chunk = stone_filled_chunk();
        let ore_gen = OreGenerator::new(42);
        ore_gen.generate_ores(&mut chunk, 0, 0);

        let coal_count = count_block_in_range(&chunk, BlockId::CoalOre, 0, 128);
        assert!(coal_count > 0, "Expected coal ore between y=0..128");

        // Coal should NOT appear below y=0 (its configured min_y).
        let coal_below = count_block_in_range(&chunk, BlockId::CoalOre, WORLD_BOTTOM, 0);
        assert_eq!(
            coal_below, 0,
            "Coal ore should not appear below y=0, found {coal_below}"
        );
    }

    #[test]
    fn generates_diamond_only_at_low_y() {
        let mut chunk = stone_filled_chunk();
        let ore_gen = OreGenerator::new(42);
        ore_gen.generate_ores(&mut chunk, 0, 0);

        let diamond_valid = count_block_in_range(&chunk, BlockId::DiamondOre, -64, 16);
        assert!(diamond_valid > 0, "Expected diamond ore between y=-64..16");

        // Diamond should NOT appear at y >= 16.
        let diamond_above = count_block_in_range(&chunk, BlockId::DiamondOre, 16, WORLD_TOP);
        assert_eq!(
            diamond_above, 0,
            "Diamond ore should not appear at y>=16, found {diamond_above}"
        );
    }

    #[test]
    fn ore_count_is_reasonable() {
        let mut chunk = stone_filled_chunk();
        let ore_gen = OreGenerator::new(12345);
        ore_gen.generate_ores(&mut chunk, 5, 7);

        let total_ore = count_block_in_range(&chunk, BlockId::CoalOre, WORLD_BOTTOM, WORLD_TOP)
            + count_block_in_range(&chunk, BlockId::IronOre, WORLD_BOTTOM, WORLD_TOP)
            + count_block_in_range(&chunk, BlockId::GoldOre, WORLD_BOTTOM, WORLD_TOP)
            + count_block_in_range(&chunk, BlockId::DiamondOre, WORLD_BOTTOM, WORLD_TOP);

        // With default configs: coal 20 veins, iron 20 veins, gold 2, diamond 1 = 43 veins.
        // Each vein places a handful of blocks. Expect at least a dozen total and
        // no more than a few thousand (chunk is 16x384x16 = 98304 blocks).
        assert!(
            total_ore >= 10,
            "Expected at least 10 ore blocks total, got {total_ore}"
        );
        assert!(total_ore < 5000, "Ore count suspiciously high: {total_ore}");
    }

    #[test]
    fn generation_is_deterministic() {
        let ore_gen = OreGenerator::new(99);

        let mut chunk_a = stone_filled_chunk();
        ore_gen.generate_ores(&mut chunk_a, 3, 4);

        let mut chunk_b = stone_filled_chunk();
        ore_gen.generate_ores(&mut chunk_b, 3, 4);

        // Both runs with the same seed and coordinates must produce identical results.
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
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
    fn only_replaces_stone() {
        let mut chunk = Chunk::new(); // all Air
        // Place some stone only in a small area
        for x in 0..4 {
            for z in 0..4 {
                for y in 0..16 {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
            }
        }
        // Fill the rest with dirt so we can verify it stays
        for x in 4..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in WORLD_BOTTOM..WORLD_TOP {
                    if chunk.get_block(x, y, z) == BlockId::Air {
                        chunk.set_block(x, y, z, BlockId::Dirt);
                    }
                }
            }
        }

        let ore_gen = OreGenerator::new(42);
        ore_gen.generate_ores(&mut chunk, 0, 0);

        // Dirt blocks should never be replaced.
        for x in 4..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in WORLD_BOTTOM..WORLD_TOP {
                    let b = chunk.get_block(x, y, z);
                    assert!(
                        b == BlockId::Dirt || b == BlockId::Stone,
                        "Non-stone block at ({x}, {y}, {z}) was replaced: {b:?}"
                    );
                }
            }
        }
    }
}
