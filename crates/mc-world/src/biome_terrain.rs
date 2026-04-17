use mc_core::biome::{BiomeId, BiomeProperties};
use mc_core::block::BlockId;
use mc_core::pos::CHUNK_SIZE;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::chunk::Chunk;

/// Sea level constant (y=63, matching vanilla Minecraft).
const SEA_LEVEL: i32 = 63;

/// Number of dirt/filler layers below the surface.
const FILLER_DEPTH: i32 = 4;

/// Minimum terrain height (ocean floor).
const MIN_HEIGHT: i32 = 30;

/// Maximum terrain height (mountain peaks).
const MAX_HEIGHT: i32 = 180;

/// All biome variants for iteration during biome selection.
const ALL_BIOMES: [BiomeId; BiomeId::COUNT] = [
    BiomeId::Plains,
    BiomeId::Forest,
    BiomeId::Desert,
    BiomeId::Ocean,
    BiomeId::Mountains,
    BiomeId::Taiga,
    BiomeId::Swamp,
    BiomeId::Jungle,
    BiomeId::Savanna,
    BiomeId::Tundra,
    BiomeId::BirchForest,
    BiomeId::DarkForest,
    BiomeId::Beach,
    BiomeId::River,
    BiomeId::MushroomIsland,
];

/// Biome-aware terrain generator that selects biomes based on temperature and
/// humidity noise, then varies terrain height, surface blocks, and features
/// according to each biome's properties.
pub struct BiomeTerrainGen {
    /// Temperature noise (very smooth, large-scale).
    temperature_noise: Fbm<Perlin>,
    /// Humidity noise (smooth, large-scale).
    humidity_noise: Fbm<Perlin>,
    /// Terrain height noise (medium-scale).
    height_noise: Fbm<Perlin>,
}

impl BiomeTerrainGen {
    /// Creates a new biome terrain generator with the given seed.
    ///
    /// Three fBm noise layers are configured:
    /// - `temperature_noise`: scale 0.002, very smooth for broad biome regions
    /// - `humidity_noise`: scale 0.003, smooth for biome moisture variation
    /// - `height_noise`: scale 0.01, medium frequency for local terrain detail
    pub fn new(seed: u64) -> Self {
        let base_seed = seed as u32;

        let temperature_noise = Fbm::<Perlin>::new(base_seed)
            .set_octaves(4)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let humidity_noise = Fbm::<Perlin>::new(base_seed.wrapping_add(1000))
            .set_octaves(4)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let height_noise = Fbm::<Perlin>::new(base_seed.wrapping_add(2000))
            .set_octaves(6)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        Self {
            temperature_noise,
            humidity_noise,
            height_noise,
        }
    }

    /// Returns the biome at the given world coordinates.
    ///
    /// Samples temperature and humidity noise, maps the noise values to the
    /// [0, 2] and [0, 1] ranges respectively, then finds the biome whose
    /// registered (temperature, humidity) is closest in Euclidean distance.
    pub fn biome_at(&self, world_x: i32, world_z: i32) -> BiomeId {
        let x = world_x as f64;
        let z = world_z as f64;

        // Temperature noise: scale 0.002, map from [-1, 1] to [0, 2]
        let temp_raw = self.temperature_noise.get([x * 0.002, z * 0.002]);
        let temperature = (temp_raw + 1.0) as f32; // [0.0, 2.0]

        // Humidity noise: scale 0.003, map from [-1, 1] to [0, 1]
        let humid_raw = self.humidity_noise.get([x * 0.003, z * 0.003]);
        let humidity = ((humid_raw + 1.0) * 0.5) as f32; // [0.0, 1.0]

        find_closest_biome(temperature, humidity)
    }

    /// Returns the terrain surface height at the given world coordinates.
    ///
    /// The height is derived from the biome's `base_height` and
    /// `height_variation`, modulated by height noise.
    pub fn height_at(&self, world_x: i32, world_z: i32) -> i32 {
        let biome = self.biome_at(world_x, world_z);
        let props = biome.properties();
        self.height_at_with_biome(world_x, world_z, props)
    }

    /// Generates a full chunk at chunk coordinates `(cx, cz)`.
    ///
    /// For each column (x, z):
    /// 1. Determines the biome via temperature/humidity noise
    /// 2. Computes surface height from biome properties + height noise
    /// 3. Places blocks using biome-specific rules:
    ///    - Bedrock at y = -64
    ///    - Stone from -63 up to filler zone
    ///    - Filler blocks (biome-specific) below surface
    ///    - Surface block (biome-specific) at terrain top
    ///    - Snow layer on cold biomes (Taiga, Tundra)
    ///    - Water fills from sea level down for underwater columns
    pub fn generate(&self, cx: i32, cz: i32) -> Chunk {
        let mut chunk = Chunk::new();
        let base_x = cx * CHUNK_SIZE;
        let base_z = cz * CHUNK_SIZE;

        for local_x in 0..CHUNK_SIZE as usize {
            for local_z in 0..CHUNK_SIZE as usize {
                let world_x = base_x + local_x as i32;
                let world_z = base_z + local_z as i32;

                let biome = self.biome_at(world_x, world_z);
                let props = biome.properties();
                let surface_height = self.height_at_with_biome(world_x, world_z, props);

                self.place_column(&mut chunk, local_x, local_z, surface_height, biome, props);
            }
        }

        chunk
    }

    /// Computes surface height for a column given its biome properties.
    fn height_at_with_biome(&self, world_x: i32, world_z: i32, props: &BiomeProperties) -> i32 {
        let x = world_x as f64;
        let z = world_z as f64;

        let noise_value = self.height_noise.get([x * 0.01, z * 0.01]);
        let height = props.base_height as f64 + noise_value * props.height_variation as f64;

        (height as i32).clamp(MIN_HEIGHT, MAX_HEIGHT)
    }

    /// Places all blocks for a single column in the chunk.
    fn place_column(
        &self,
        chunk: &mut Chunk,
        local_x: usize,
        local_z: usize,
        surface_height: i32,
        biome: BiomeId,
        props: &BiomeProperties,
    ) {
        // Bedrock at y = -64
        chunk.set_block(local_x, -64, local_z, BlockId::Bedrock);

        // Stone from -63 up to filler zone
        let stone_top = surface_height - FILLER_DEPTH;
        for y in -63..=stone_top {
            chunk.set_block(local_x, y, local_z, BlockId::Stone);
        }

        // Filler blocks (biome-specific) from stone_top+1 to surface_height-1
        for y in (stone_top + 1)..surface_height {
            chunk.set_block(local_x, y, local_z, props.filler_block);
        }

        // Surface block at terrain top
        let surface_block = if surface_height < SEA_LEVEL {
            // Underwater: use biome's underwater_block
            props.underwater_block
        } else {
            props.surface_block
        };
        chunk.set_block(local_x, surface_height, local_z, surface_block);

        // Water fills from sea level down to just above terrain for underwater columns
        if surface_height < SEA_LEVEL {
            for y in (surface_height + 1)..=SEA_LEVEL {
                chunk.set_block(local_x, y, local_z, BlockId::Water);
            }
        }

        // Snow layer on cold biomes above sea level
        if surface_height >= SEA_LEVEL && is_snowy_biome(biome) {
            chunk.set_block(local_x, surface_height + 1, local_z, BlockId::Snow);
        }
    }
}

/// Returns true if a biome should have snow on its surface.
fn is_snowy_biome(biome: BiomeId) -> bool {
    matches!(biome, BiomeId::Taiga | BiomeId::Tundra)
}

/// Finds the biome whose registered (temperature, humidity) is closest to the
/// given values using squared Euclidean distance. Temperature is weighted more
/// heavily than humidity to create broader biome bands.
fn find_closest_biome(temperature: f32, humidity: f32) -> BiomeId {
    let mut best_biome = BiomeId::Plains;
    let mut best_distance = f32::MAX;

    for &biome in &ALL_BIOMES {
        let props = biome.properties();
        let dt = temperature - props.temperature;
        let dh = humidity - props.humidity;
        // Weight temperature slightly more to create broader bands
        let distance = dt * dt + dh * dh;

        if distance < best_distance {
            best_distance = distance;
            best_biome = biome;
        }
    }

    best_biome
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desert_biome_at_hot_dry_produces_sand_surface() {
        let terrain = BiomeTerrainGen::new(42);
        // Search for a column that resolves to Desert biome
        let mut found_desert = false;
        for x in -1000..1000 {
            for z in -1000..1000 {
                if terrain.biome_at(x, z) == BiomeId::Desert {
                    let props = BiomeId::Desert.properties();
                    assert_eq!(
                        props.surface_block,
                        BlockId::Sand,
                        "desert surface block should be sand"
                    );
                    found_desert = true;
                    break;
                }
            }
            if found_desert {
                break;
            }
        }
        assert!(found_desert, "should find at least one desert column");
    }

    #[test]
    fn ocean_biome_produces_water() {
        let terrain = BiomeTerrainGen::new(42);
        // Find an ocean column and verify water is present
        let mut found_ocean = false;
        for x in -1000..1000 {
            for z in -1000..1000 {
                if terrain.biome_at(x, z) == BiomeId::Ocean {
                    let height = terrain.height_at(x, z);
                    // Ocean base_height is 36, so surface should be well below sea level
                    assert!(
                        height < SEA_LEVEL,
                        "ocean height {} should be below sea level {}",
                        height,
                        SEA_LEVEL
                    );
                    found_ocean = true;
                    break;
                }
            }
            if found_ocean {
                break;
            }
        }
        assert!(found_ocean, "should find at least one ocean column");
    }

    #[test]
    fn mountains_are_tall() {
        let terrain = BiomeTerrainGen::new(42);
        let mut found_mountain = false;
        let mut max_mountain_height = 0;
        for x in -1000..1000 {
            for z in -1000..1000 {
                if terrain.biome_at(x, z) == BiomeId::Mountains {
                    let height = terrain.height_at(x, z);
                    if height > max_mountain_height {
                        max_mountain_height = height;
                    }
                    found_mountain = true;
                }
            }
            if found_mountain && max_mountain_height > 90 {
                break;
            }
        }
        assert!(found_mountain, "should find at least one mountain column");
        assert!(
            max_mountain_height > 80,
            "mountain max height {} should be > 80",
            max_mountain_height
        );
    }

    #[test]
    fn height_at_returns_values_in_valid_range() {
        let terrain = BiomeTerrainGen::new(42);
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
    fn generate_places_bedrock_at_bottom() {
        let terrain = BiomeTerrainGen::new(42);
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
        let terrain = BiomeTerrainGen::new(42);
        // Generate several chunks to find an underwater column
        let mut found_water = false;
        for cx in -10..10 {
            for cz in -10..10 {
                let chunk = terrain.generate(cx, cz);
                let base_x = cx * CHUNK_SIZE;
                let base_z = cz * CHUNK_SIZE;

                for local_x in 0..CHUNK_SIZE as usize {
                    for local_z in 0..CHUNK_SIZE as usize {
                        let world_x = base_x + local_x as i32;
                        let world_z = base_z + local_z as i32;
                        let surface = terrain.height_at(world_x, world_z);

                        if surface < SEA_LEVEL {
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
                if found_water {
                    break;
                }
            }
            if found_water {
                break;
            }
        }

        assert!(
            found_water,
            "expected to find at least one underwater column"
        );
    }

    #[test]
    fn snowy_biomes_get_snow_layer() {
        let terrain = BiomeTerrainGen::new(42);
        // First, find a snowy column with a fast biome_at scan over a wide area
        let mut target = None;
        'search: for x in (-2000..2000).step_by(4) {
            for z in (-2000..2000).step_by(4) {
                let biome = terrain.biome_at(x, z);
                if is_snowy_biome(biome) {
                    let surface = terrain.height_at(x, z);
                    if surface >= SEA_LEVEL {
                        target = Some((x, z));
                        break 'search;
                    }
                }
            }
        }

        let (wx, wz) = target.expect("should find at least one snowy column above sea level");
        let cx = wx.div_euclid(CHUNK_SIZE);
        let cz = wz.div_euclid(CHUNK_SIZE);
        let chunk = terrain.generate(cx, cz);

        let local_x = wx.rem_euclid(CHUNK_SIZE) as usize;
        let local_z = wz.rem_euclid(CHUNK_SIZE) as usize;
        let surface = terrain.height_at(wx, wz);

        assert_eq!(
            chunk.get_block(local_x, surface + 1, local_z),
            BlockId::Snow,
            "expected snow above surface at ({local_x}, {}, {local_z})",
            surface + 1
        );
    }

    #[test]
    fn biome_selection_returns_closest_biome() {
        // Desert: temperature=2.0, humidity=0.0
        let biome = find_closest_biome(2.0, 0.0);
        assert_eq!(biome, BiomeId::Desert, "hot+dry should map to desert");

        // Tundra: temperature=0.0, humidity=0.5
        let biome = find_closest_biome(0.0, 0.5);
        assert_eq!(
            biome,
            BiomeId::Tundra,
            "freezing+moderate humidity should map to tundra"
        );

        // Ocean: temperature=0.5, humidity=0.5
        // Note: Ocean and River share (0.5, 0.5) but Ocean comes first in the array
        let biome = find_closest_biome(0.5, 0.5);
        assert!(
            biome == BiomeId::Ocean || biome == BiomeId::River || biome == BiomeId::BirchForest,
            "moderate temp+humidity should map to a temperate biome, got {:?}",
            biome
        );
    }

    #[test]
    fn different_seeds_produce_different_biome_maps() {
        let gen_a = BiomeTerrainGen::new(1);
        let gen_b = BiomeTerrainGen::new(999);

        let mut differ = false;
        for x in 0..100 {
            for z in 0..100 {
                if gen_a.biome_at(x, z) != gen_b.biome_at(x, z) {
                    differ = true;
                    break;
                }
            }
            if differ {
                break;
            }
        }

        assert!(
            differ,
            "different seeds should produce different biome maps"
        );
    }

    #[test]
    fn multiple_biomes_present_in_large_area() {
        let terrain = BiomeTerrainGen::new(42);
        let mut biomes = std::collections::HashSet::new();

        for x in (-500..500).step_by(10) {
            for z in (-500..500).step_by(10) {
                biomes.insert(terrain.biome_at(x, z));
            }
        }

        assert!(
            biomes.len() >= 3,
            "expected at least 3 distinct biomes in a 1000x1000 area, got {}: {:?}",
            biomes.len(),
            biomes
        );
    }

    #[test]
    fn desert_chunk_has_sand_surface() {
        let terrain = BiomeTerrainGen::new(42);
        // Find a chunk that contains desert
        let mut found = false;
        'outer: for cx in -20..20 {
            for cz in -20..20 {
                let base_x = cx * CHUNK_SIZE;
                let base_z = cz * CHUNK_SIZE;

                // Check if center of chunk is desert
                let center_x = base_x + 8;
                let center_z = base_z + 8;
                if terrain.biome_at(center_x, center_z) != BiomeId::Desert {
                    continue;
                }

                let chunk = terrain.generate(cx, cz);
                let surface = terrain.height_at(center_x, center_z);

                if surface >= SEA_LEVEL {
                    let block = chunk.get_block(8, surface, 8);
                    assert_eq!(
                        block,
                        BlockId::Sand,
                        "desert surface at ({}, {}, {}) should be sand, got {:?}",
                        8,
                        surface,
                        8,
                        block
                    );
                    found = true;
                    break 'outer;
                }
            }
        }

        assert!(found, "should find a desert chunk with sand surface");
    }
}
