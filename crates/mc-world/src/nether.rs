use mc_core::block::BlockId;
use mc_core::pos::CHUNK_SIZE;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::chunk::Chunk;

/// Nether floor (y=0).
const NETHER_FLOOR: i32 = 0;

/// Nether ceiling (y=127).
const NETHER_CEILING: i32 = 127;

/// Height at or below which air is replaced by lava (Water placeholder).
const LAVA_LEVEL: i32 = 31;

/// Minimum Y for cave carving (above bedrock floor).
const CAVE_MIN_Y: i32 = 5;

/// Maximum Y for cave carving (below bedrock ceiling).
const CAVE_MAX_Y: i32 = 120;

/// 3D noise threshold above which blocks are carved into caves.
const CAVE_THRESHOLD: f64 = 0.3;

/// Scale for the 3D cave-carving noise.
const CAVE_NOISE_SCALE: f64 = 0.04;

/// Scale for the 2D soul sand patch noise.
const SOUL_SAND_NOISE_SCALE: f64 = 0.05;

/// Threshold above which soul sand patches appear.
const SOUL_SAND_THRESHOLD: f64 = 0.4;

/// Y range for soul sand patches.
const SOUL_SAND_MIN_Y: i32 = 31;
const SOUL_SAND_MAX_Y: i32 = 35;

/// Scale for the 3D glowstone placement noise.
const GLOWSTONE_NOISE_SCALE: f64 = 0.08;

/// Threshold above which glowstone is placed on ceiling surfaces.
const GLOWSTONE_THRESHOLD: f64 = 0.7;

/// Dimension identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionId {
    Overworld,
    Nether,
    End,
}

/// Nether terrain generator that produces cave-like terrain filled with
/// Netherrack, lava lakes, soul sand patches, and glowstone clusters.
pub struct NetherTerrainGen {
    /// 3D noise used to carve large cave openings.
    cave_noise: Fbm<Perlin>,
    /// 2D noise for soul sand patch placement.
    soul_sand_noise: Fbm<Perlin>,
    /// 3D noise for glowstone cluster placement on ceilings.
    glowstone_noise: Fbm<Perlin>,
}

impl NetherTerrainGen {
    /// Creates a new Nether terrain generator with the given seed.
    ///
    /// Three fBm noise layers are configured:
    /// - `cave_noise`: 4 octaves, used for 3D cave carving
    /// - `soul_sand_noise`: 3 octaves, used for 2D soul sand patches
    /// - `glowstone_noise`: 3 octaves, used for 3D glowstone placement
    pub fn new(seed: u64) -> Self {
        let base_seed = seed as u32;

        let cave_noise = Fbm::<Perlin>::new(base_seed)
            .set_octaves(4)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let soul_sand_noise = Fbm::<Perlin>::new(base_seed.wrapping_add(3000))
            .set_octaves(3)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let glowstone_noise = Fbm::<Perlin>::new(base_seed.wrapping_add(5000))
            .set_octaves(3)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        Self {
            cave_noise,
            soul_sand_noise,
            glowstone_noise,
        }
    }

    /// Generates a full Nether chunk at chunk coordinates `(cx, cz)`.
    ///
    /// Generation steps:
    /// 1. Fill y=0 to y=128 with Netherrack
    /// 2. Place bedrock floor at y=0 and bedrock ceiling at y=127
    /// 3. Carve caves using 3D noise between y=5 and y=120
    /// 4. Fill air below lava level (y<=31) with Water (lava placeholder)
    /// 5. Replace Netherrack with SoulSand in scattered patches (y=31-35)
    /// 6. Place Glowstone clusters on ceiling surfaces
    pub fn generate(&self, cx: i32, cz: i32) -> Chunk {
        let mut chunk = Chunk::new();
        let base_x = cx * CHUNK_SIZE;
        let base_z = cz * CHUNK_SIZE;

        // Step 1 & 2: Fill base terrain with Netherrack and bedrock boundaries
        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                // Bedrock floor
                chunk.set_block(local_x, NETHER_FLOOR, local_z, BlockId::Bedrock);
                // Netherrack fill
                for y in (NETHER_FLOOR + 1)..NETHER_CEILING {
                    chunk.set_block(local_x, y, local_z, BlockId::Netherrack);
                }
                // Bedrock ceiling
                chunk.set_block(local_x, NETHER_CEILING, local_z, BlockId::Bedrock);
            }
        }

        // Step 3: Carve caves using 3D noise
        for local_x in 0..CHUNK_SIZE as usize {
            let world_x = (base_x + local_x as i32) as f64;
            for local_z in 0..CHUNK_SIZE as usize {
                let world_z = (base_z + local_z as i32) as f64;
                for y in CAVE_MIN_Y..=CAVE_MAX_Y {
                    let noise_val = self.cave_noise.get([
                        world_x * CAVE_NOISE_SCALE,
                        y as f64 * CAVE_NOISE_SCALE,
                        world_z * CAVE_NOISE_SCALE,
                    ]);
                    if noise_val > CAVE_THRESHOLD {
                        chunk.set_block(local_x, y, local_z, BlockId::Air);
                    }
                }
            }
        }

        // Step 4: Fill air below lava level with Water (lava placeholder)
        // NOTE: Water is used as a placeholder for lava since BlockId::Lava
        // does not exist yet. Replace with Lava when the block type is added.
        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                for y in (NETHER_FLOOR + 1)..=LAVA_LEVEL {
                    if chunk.get_block(local_x, y, local_z) == BlockId::Air {
                        chunk.set_block(local_x, y, local_z, BlockId::Water);
                    }
                }
            }
        }

        // Step 5: Soul sand patches
        for local_x in 0..CHUNK_SIZE as usize {
            let world_x = (base_x + local_x as i32) as f64;
            for local_z in 0..CHUNK_SIZE as usize {
                let world_z = (base_z + local_z as i32) as f64;
                let sand_val = self.soul_sand_noise.get([
                    world_x * SOUL_SAND_NOISE_SCALE,
                    world_z * SOUL_SAND_NOISE_SCALE,
                ]);
                if sand_val > SOUL_SAND_THRESHOLD {
                    for y in SOUL_SAND_MIN_Y..=SOUL_SAND_MAX_Y {
                        if chunk.get_block(local_x, y, local_z) == BlockId::Netherrack {
                            chunk.set_block(local_x, y, local_z, BlockId::SoulSand);
                        }
                    }
                }
            }
        }

        // Step 6: Glowstone clusters on ceiling surfaces
        // A ceiling surface is where there's Netherrack above and Air (or Water)
        // below. We place Glowstone at positions where noise exceeds the threshold
        // and the block is Netherrack with air below it.
        for local_x in 0..CHUNK_SIZE as usize {
            let world_x = (base_x + local_x as i32) as f64;
            for local_z in 0..CHUNK_SIZE as usize {
                let world_z = (base_z + local_z as i32) as f64;
                for y in (CAVE_MIN_Y + 1)..=CAVE_MAX_Y {
                    let block = chunk.get_block(local_x, y, local_z);
                    if block != BlockId::Netherrack {
                        continue;
                    }
                    let below = chunk.get_block(local_x, y - 1, local_z);
                    if below != BlockId::Air && below != BlockId::Water {
                        continue;
                    }
                    let glow_val = self.glowstone_noise.get([
                        world_x * GLOWSTONE_NOISE_SCALE,
                        y as f64 * GLOWSTONE_NOISE_SCALE,
                        world_z * GLOWSTONE_NOISE_SCALE,
                    ]);
                    if glow_val > GLOWSTONE_THRESHOLD {
                        chunk.set_block(local_x, y, local_z, BlockId::Glowstone);
                    }
                }
            }
        }

        chunk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn netherrack_dominates_chunk() {
        let nether = NetherTerrainGen::new(42);
        let chunk = nether.generate(0, 0);

        let mut netherrack_count: u64 = 0;
        let mut total_solid: u64 = 0;

        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in NETHER_FLOOR..=NETHER_CEILING {
                    let block = chunk.get_block(x, y, z);
                    if block.is_solid() {
                        total_solid += 1;
                    }
                    if block == BlockId::Netherrack {
                        netherrack_count += 1;
                    }
                }
            }
        }

        // Netherrack should be the majority of solid blocks
        assert!(
            netherrack_count > total_solid / 2,
            "netherrack ({netherrack_count}) should be more than half of solid blocks ({total_solid})"
        );
    }

    #[test]
    fn caves_exist_between_y5_and_y120() {
        let nether = NetherTerrainGen::new(42);
        let chunk = nether.generate(0, 0);

        let mut air_count: u64 = 0;
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in CAVE_MIN_Y..=CAVE_MAX_Y {
                    let block = chunk.get_block(x, y, z);
                    if block == BlockId::Air || block == BlockId::Water {
                        air_count += 1;
                    }
                }
            }
        }

        assert!(
            air_count > 0,
            "expected cave openings (air or water blocks) between y={CAVE_MIN_Y} and y={CAVE_MAX_Y}"
        );
    }

    #[test]
    fn bedrock_at_floor_and_ceiling() {
        let nether = NetherTerrainGen::new(42);
        let chunk = nether.generate(0, 0);

        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                assert_eq!(
                    chunk.get_block(x, NETHER_FLOOR, z),
                    BlockId::Bedrock,
                    "expected bedrock at floor ({x}, {NETHER_FLOOR}, {z})"
                );
                assert_eq!(
                    chunk.get_block(x, NETHER_CEILING, z),
                    BlockId::Bedrock,
                    "expected bedrock at ceiling ({x}, {NETHER_CEILING}, {z})"
                );
            }
        }
    }

    #[test]
    fn lava_fills_air_below_lava_level() {
        let nether = NetherTerrainGen::new(42);
        let chunk = nether.generate(0, 0);

        // No air should exist at or below lava level (it should be Water or solid)
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in (NETHER_FLOOR + 1)..=LAVA_LEVEL {
                    let block = chunk.get_block(x, y, z);
                    assert_ne!(
                        block,
                        BlockId::Air,
                        "air at ({x}, {y}, {z}) should have been replaced with water (lava placeholder)"
                    );
                }
            }
        }
    }

    #[test]
    fn glowstone_is_present() {
        let nether = NetherTerrainGen::new(42);
        // Check multiple chunks to find glowstone
        let mut found_glowstone = false;
        for cx in -2..=2 {
            for cz in -2..=2 {
                let chunk = nether.generate(cx, cz);
                for x in 0..CHUNK_SIZE as usize {
                    for z in 0..CHUNK_SIZE as usize {
                        for y in CAVE_MIN_Y..=CAVE_MAX_Y {
                            if chunk.get_block(x, y, z) == BlockId::Glowstone {
                                found_glowstone = true;
                            }
                        }
                    }
                }
                if found_glowstone {
                    break;
                }
            }
            if found_glowstone {
                break;
            }
        }
        assert!(found_glowstone, "expected glowstone clusters in nether terrain");
    }

    #[test]
    fn soul_sand_patches_exist() {
        let nether = NetherTerrainGen::new(42);
        let mut found_soul_sand = false;
        for cx in -2..=2 {
            for cz in -2..=2 {
                let chunk = nether.generate(cx, cz);
                for x in 0..CHUNK_SIZE as usize {
                    for z in 0..CHUNK_SIZE as usize {
                        for y in SOUL_SAND_MIN_Y..=SOUL_SAND_MAX_Y {
                            if chunk.get_block(x, y, z) == BlockId::SoulSand {
                                found_soul_sand = true;
                            }
                        }
                    }
                }
                if found_soul_sand {
                    break;
                }
            }
            if found_soul_sand {
                break;
            }
        }
        assert!(found_soul_sand, "expected soul sand patches in nether terrain");
    }

    #[test]
    fn different_seeds_produce_different_nether() {
        let gen_a = NetherTerrainGen::new(1);
        let gen_b = NetherTerrainGen::new(999);
        let chunk_a = gen_a.generate(0, 0);
        let chunk_b = gen_b.generate(0, 0);

        let mut differ = false;
        'outer: for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in CAVE_MIN_Y..=CAVE_MAX_Y {
                    if chunk_a.get_block(x, y, z) != chunk_b.get_block(x, y, z) {
                        differ = true;
                        break 'outer;
                    }
                }
            }
        }
        assert!(differ, "different seeds should produce different nether terrain");
    }

    #[test]
    fn dimension_id_variants() {
        assert_ne!(DimensionId::Overworld, DimensionId::Nether);
        assert_ne!(DimensionId::Nether, DimensionId::End);
        assert_ne!(DimensionId::Overworld, DimensionId::End);
    }
}
