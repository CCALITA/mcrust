use crate::chunk::Chunk;
use mc_core::block::BlockId;
use mc_core::pos::CHUNK_SIZE;
use noise::Fbm;
use noise::MultiFractal;
use noise::NoiseFn;
use noise::Perlin;

/// Sea level constant (y=63, matching vanilla Minecraft).
const SEA_LEVEL: i32 = 63;

/// Scale factor for the primary continent/terrain shape noise.
const CONTINENT_SCALE: f64 = 0.005;

/// Scale factor for terrain detail noise.
const DETAIL_SCALE: f64 = 0.02;

/// Base terrain height around which noise is centered.
const BASE_HEIGHT: f64 = 70.0;

/// Amplitude for the large-scale continent noise.
const CONTINENT_AMPLITUDE: f64 = 40.0;

/// Amplitude for the detail noise layer.
const DETAIL_AMPLITUDE: f64 = 12.0;

/// Minimum terrain height (ocean floor).
const MIN_HEIGHT: i32 = 40;

/// Maximum terrain height (mountain peaks).
const MAX_HEIGHT: i32 = 128;

/// Number of dirt layers below the surface.
const DIRT_DEPTH: i32 = 4;

/// Noise-based terrain generator that produces varied landscapes with
/// mountains, plains, valleys, and oceans.
pub struct NoiseTerrainGen {
    /// Large-scale continent shape noise (low frequency).
    continent_noise: Fbm<Perlin>,
    /// Detail noise for local terrain variation (higher frequency).
    detail_noise: Fbm<Perlin>,
}

impl NoiseTerrainGen {
    /// Creates a new noise terrain generator with the given seed.
    ///
    /// Two fBm noise layers are configured:
    /// - `continent_noise`: 6 octaves, low frequency for broad terrain shape
    /// - `detail_noise`: 4 octaves, higher frequency for local detail
    pub fn new(seed: u64) -> Self {
        let base_seed = seed as u32;

        let continent_noise = Fbm::<Perlin>::new(base_seed)
            .set_octaves(6)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let detail_noise = Fbm::<Perlin>::new(base_seed.wrapping_add(1000))
            .set_octaves(4)
            .set_frequency(1.0)
            .set_persistence(0.45)
            .set_lacunarity(2.0);

        Self {
            continent_noise,
            detail_noise,
        }
    }

    /// Returns the terrain surface height at the given world coordinates.
    ///
    /// Uses two noise layers:
    /// - A low-frequency continent layer for broad terrain shape
    /// - A higher-frequency detail layer for local variation
    ///
    /// The result is clamped to `[MIN_HEIGHT, MAX_HEIGHT]`.
    pub fn height_at(&self, world_x: i32, world_z: i32) -> i32 {
        let x = world_x as f64;
        let z = world_z as f64;

        let continent_value = self
            .continent_noise
            .get([x * CONTINENT_SCALE, z * CONTINENT_SCALE]);
        let detail_value = self.detail_noise.get([x * DETAIL_SCALE, z * DETAIL_SCALE]);

        let height =
            BASE_HEIGHT + continent_value * CONTINENT_AMPLITUDE + detail_value * DETAIL_AMPLITUDE;

        (height as i32).clamp(MIN_HEIGHT, MAX_HEIGHT)
    }

    /// Generates a full chunk at chunk coordinates `(cx, cz)`.
    ///
    /// Block placement rules:
    /// - Bedrock at y = -64
    /// - Stone from y = -63 up to (surface_height - DIRT_DEPTH)
    /// - Dirt from (surface_height - DIRT_DEPTH + 1) to (surface_height - 1)
    /// - Surface block at surface_height:
    ///   - Grass if above sea level
    ///   - Sand if at sea level
    ///   - Gravel if below sea level
    /// - Water fills from sea level down to (surface_height + 1) for underwater columns
    pub fn generate(&self, cx: i32, cz: i32) -> Chunk {
        let mut chunk = Chunk::new();
        let base_x = cx * CHUNK_SIZE;
        let base_z = cz * CHUNK_SIZE;

        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                let world_x = base_x + local_x as i32;
                let world_z = base_z + local_z as i32;
                let surface_height = self.height_at(world_x, world_z);

                // Bedrock at y=-64
                chunk.set_block(local_x, -64, local_z, BlockId::Bedrock);

                // Stone from -63 up to (surface_height - DIRT_DEPTH)
                let stone_top = surface_height - DIRT_DEPTH;
                for y in -63..=stone_top {
                    chunk.set_block(local_x, y, local_z, BlockId::Stone);
                }

                // Dirt from (stone_top + 1) to (surface_height - 1)
                for y in (stone_top + 1)..surface_height {
                    chunk.set_block(local_x, y, local_z, BlockId::Dirt);
                }

                // Surface block depends on height relative to sea level
                let surface_block = if surface_height > SEA_LEVEL {
                    BlockId::GrassBlock
                } else if surface_height == SEA_LEVEL {
                    BlockId::Sand
                } else {
                    BlockId::Gravel
                };
                chunk.set_block(local_x, surface_height, local_z, surface_block);

                // Water fills from sea level down to just above the terrain
                if surface_height < SEA_LEVEL {
                    for y in (surface_height + 1)..=SEA_LEVEL {
                        chunk.set_block(local_x, y, local_z, BlockId::Water);
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
    fn height_at_returns_values_in_valid_range() {
        let terrain = NoiseTerrainGen::new(42);
        for x in -100..100 {
            for z in -100..100 {
                let h = terrain.height_at(x, z);
                assert!(
                    h >= MIN_HEIGHT && h <= MAX_HEIGHT,
                    "height_at({x}, {z}) = {h} is outside [{MIN_HEIGHT}, {MAX_HEIGHT}]"
                );
            }
        }
    }

    #[test]
    fn terrain_is_not_flat() {
        let terrain = NoiseTerrainGen::new(42);
        let mut heights = std::collections::HashSet::new();
        // Sample a wide area to ensure terrain varies
        for x in (-200..200).step_by(10) {
            for z in (-200..200).step_by(10) {
                heights.insert(terrain.height_at(x, z));
            }
        }
        // With noise-based generation over a 400x400 area, we should see
        // many distinct height values (not a flat world).
        assert!(
            heights.len() > 10,
            "expected varied terrain, got only {} distinct heights: {:?}",
            heights.len(),
            heights
        );
    }

    #[test]
    fn generate_places_bedrock_at_bottom() {
        let terrain = NoiseTerrainGen::new(42);
        let chunk = terrain.generate(0, 0);
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                assert_eq!(
                    chunk.get_block(x, -64, z),
                    BlockId::Bedrock,
                    "expected bedrock at ({x}, -64, {z})"
                );
            }
        }
    }

    #[test]
    fn generate_fills_water_below_sea_level() {
        let terrain = NoiseTerrainGen::new(12345);
        // Generate several chunks to find an underwater column
        let positions = [
            (0, 0),
            (1, 0),
            (0, 1),
            (-1, -1),
            (5, 5),
            (10, 10),
            (-10, -10),
        ];
        let mut found_water = false;

        for (cx, cz) in positions {
            let chunk = terrain.generate(cx, cz);
            let base_x = cx * CHUNK_SIZE;
            let base_z = cz * CHUNK_SIZE;

            for local_x in 0..CHUNK_SIZE as usize {
                for local_z in 0..CHUNK_SIZE as usize {
                    let world_x = base_x + local_x as i32;
                    let world_z = base_z + local_z as i32;
                    let surface = terrain.height_at(world_x, world_z);

                    if surface < SEA_LEVEL {
                        // Water should be present from surface+1 to sea level
                        for y in (surface + 1)..=SEA_LEVEL {
                            assert_eq!(
                                chunk.get_block(local_x, y, local_z),
                                BlockId::Water,
                                "expected water at ({local_x}, {y}, {local_z}) in chunk ({cx}, {cz})"
                            );
                        }
                        found_water = true;
                    }
                }
            }
        }

        assert!(
            found_water,
            "expected to find at least one underwater column across sampled chunks"
        );
    }

    #[test]
    fn generate_places_correct_surface_blocks() {
        let terrain = NoiseTerrainGen::new(42);
        let chunk = terrain.generate(0, 0);

        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                let surface = terrain.height_at(local_x as i32, local_z as i32);
                let block = chunk.get_block(local_x, surface, local_z);

                if surface > SEA_LEVEL {
                    assert_eq!(
                        block,
                        BlockId::GrassBlock,
                        "expected grass at ({local_x}, {surface}, {local_z})"
                    );
                } else if surface == SEA_LEVEL {
                    assert_eq!(
                        block,
                        BlockId::Sand,
                        "expected sand at ({local_x}, {surface}, {local_z})"
                    );
                } else {
                    assert_eq!(
                        block,
                        BlockId::Gravel,
                        "expected gravel at ({local_x}, {surface}, {local_z})"
                    );
                }
            }
        }
    }

    #[test]
    fn different_seeds_produce_different_terrain() {
        let gen_a = NoiseTerrainGen::new(1);
        let gen_b = NoiseTerrainGen::new(999);

        let mut differ = false;
        for x in 0..50 {
            for z in 0..50 {
                if gen_a.height_at(x, z) != gen_b.height_at(x, z) {
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

    #[test]
    fn generate_chunks_at_different_positions_vary() {
        let terrain = NoiseTerrainGen::new(42);
        let chunk_a = terrain.generate(0, 0);
        let chunk_b = terrain.generate(10, 10);

        // Compare a sample of blocks to confirm the two chunks differ
        let mut differ = false;
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in 50..80 {
                    if chunk_a.get_block(x, y, z) != chunk_b.get_block(x, y, z) {
                        differ = true;
                        break;
                    }
                }
                if differ {
                    break;
                }
            }
            if differ {
                break;
            }
        }

        assert!(
            differ,
            "chunks at (0,0) and (10,10) should have different terrain"
        );
    }
}
