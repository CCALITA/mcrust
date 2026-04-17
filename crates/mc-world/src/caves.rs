use mc_core::block::BlockId;
use mc_core::pos::{CHUNK_SIZE, WORLD_BOTTOM};
use noise::{NoiseFn, Perlin};

use crate::chunk::Chunk;

/// Sea level constant used to restrict cave carving.
const SEA_LEVEL: i32 = 63;

/// Maximum Y for cheese cave carving (exclusive).
const CHEESE_MAX_Y: i32 = 50;

/// Y range for spaghetti caves.
const SPAGHETTI_MIN_Y: i32 = -60;
const SPAGHETTI_MAX_Y: i32 = 40;

/// Noise scale for cheese caves (large caverns).
const CHEESE_SCALE: f64 = 0.03;

/// Noise threshold above which cheese caves carve.
const CHEESE_THRESHOLD: f64 = 0.6;

/// Noise scale for spaghetti caves (narrow tunnels).
const SPAGHETTI_SCALE: f64 = 0.06;

/// Half-width of the spaghetti tunnel "near zero" band.
const SPAGHETTI_RADIUS: f64 = 0.05;

/// Offset applied to the second spaghetti noise field so the two fields
/// sample different regions of the noise space.
const SPAGHETTI_OFFSET: f64 = 1000.0;

/// Carves caves into chunks using 3D noise.
///
/// Two styles are generated:
/// - **Cheese caves**: large open caverns below `CHEESE_MAX_Y`.
/// - **Spaghetti caves**: winding tunnels between `SPAGHETTI_MIN_Y` and `SPAGHETTI_MAX_Y`.
///
/// Rules:
/// - Bedrock at `WORLD_BOTTOM` is never carved.
/// - No carving at or above `SEA_LEVEL`.
/// - Blocks carved below `SEA_LEVEL` that are adjacent to an existing air block
///   are filled with Water instead of Air (simplified aquifer).
pub struct CaveCarver {
    cheese_noise: Perlin,
    spaghetti_noise_a: Perlin,
    spaghetti_noise_b: Perlin,
}

impl CaveCarver {
    /// Create a new `CaveCarver` seeded with `seed`.
    pub fn new(seed: u64) -> Self {
        // Perlin::new takes a u32, so we derive three distinct seeds.
        let base = seed as u32;
        Self {
            cheese_noise: Perlin::new(base),
            spaghetti_noise_a: Perlin::new(base.wrapping_add(1)),
            spaghetti_noise_b: Perlin::new(base.wrapping_add(2)),
        }
    }

    /// Carve caves into `chunk` located at chunk coordinates (`cx`, `cz`).
    pub fn carve(&self, chunk: &mut Chunk, cx: i32, cz: i32) {
        let base_x = cx * CHUNK_SIZE;
        let base_z = cz * CHUNK_SIZE;

        // Determine the Y range we actually need to iterate.
        // Cheese: WORLD_BOTTOM+1 .. CHEESE_MAX_Y
        // Spaghetti: SPAGHETTI_MIN_Y .. SPAGHETTI_MAX_Y
        // Combined min is WORLD_BOTTOM+1, combined max is max(CHEESE_MAX_Y, SPAGHETTI_MAX_Y).
        let y_min = WORLD_BOTTOM + 1; // never carve bedrock layer
        let y_max = if CHEESE_MAX_Y > SPAGHETTI_MAX_Y {
            CHEESE_MAX_Y
        } else {
            SPAGHETTI_MAX_Y
        };

        for lx in 0..CHUNK_SIZE as usize {
            let wx = (base_x + lx as i32) as f64;
            for lz in 0..CHUNK_SIZE as usize {
                let wz = (base_z + lz as i32) as f64;
                for wy in y_min..y_max {
                    // Never carve at or above sea level.
                    if wy >= SEA_LEVEL {
                        continue;
                    }

                    let current = chunk.get_block(lx, wy, lz);
                    // Only carve solid blocks.
                    if !current.is_solid() {
                        continue;
                    }
                    // Never carve bedrock.
                    if current == BlockId::Bedrock {
                        continue;
                    }

                    let should_carve = self.should_carve_cheese(wx, wy, wz)
                        || self.should_carve_spaghetti(wx, wy, wz);

                    if should_carve {
                        let replacement =
                            if wy < SEA_LEVEL && self.has_adjacent_air(chunk, lx, wy, lz) {
                                BlockId::Water
                            } else {
                                BlockId::Air
                            };
                        chunk.set_block(lx, wy, lz, replacement);
                    }
                }
            }
        }
    }

    /// Returns `true` if the cheese noise at this world position exceeds the threshold.
    fn should_carve_cheese(&self, wx: f64, wy: i32, wz: f64) -> bool {
        if wy >= CHEESE_MAX_Y {
            return false;
        }
        let val = self.cheese_noise.get([
            wx * CHEESE_SCALE,
            wy as f64 * CHEESE_SCALE,
            wz * CHEESE_SCALE,
        ]);
        val > CHEESE_THRESHOLD
    }

    /// Returns `true` if both spaghetti noise fields are near zero at this world position.
    fn should_carve_spaghetti(&self, wx: f64, wy: i32, wz: f64) -> bool {
        if !(SPAGHETTI_MIN_Y..SPAGHETTI_MAX_Y).contains(&wy) {
            return false;
        }
        let sx = wx * SPAGHETTI_SCALE;
        let sy = wy as f64 * SPAGHETTI_SCALE;
        let sz = wz * SPAGHETTI_SCALE;

        let a = self.spaghetti_noise_a.get([sx, sy, sz]);
        let b = self.spaghetti_noise_b.get([
            sx + SPAGHETTI_OFFSET,
            sy + SPAGHETTI_OFFSET,
            sz + SPAGHETTI_OFFSET,
        ]);

        a.abs() < SPAGHETTI_RADIUS && b.abs() < SPAGHETTI_RADIUS
    }

    /// Check whether any of the 6 cardinal neighbours within the chunk is Air.
    /// Neighbours outside the chunk boundary are ignored (conservative: no water fill).
    fn has_adjacent_air(&self, chunk: &Chunk, lx: usize, wy: i32, lz: usize) -> bool {
        let size = CHUNK_SIZE as usize;
        let neighbors: [(i32, i32, i32); 6] = [
            (-1, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ];
        for (dx, dy, dz) in neighbors {
            let nx = lx as i32 + dx;
            let ny = wy + dy;
            let nz = lz as i32 + dz;
            if nx < 0 || nx >= size as i32 || nz < 0 || nz >= size as i32 {
                continue;
            }
            if ny < WORLD_BOTTOM || ny >= WORLD_BOTTOM + (size as i32 * 24) {
                continue;
            }
            if chunk.get_block(nx as usize, ny, nz as usize).is_air() {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Helper: create a chunk filled entirely with stone (except bedrock at WORLD_BOTTOM).
    fn stone_filled_chunk() -> Chunk {
        let mut chunk = Chunk::new();
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                chunk.set_block(x, WORLD_BOTTOM, z, BlockId::Bedrock);
                for y in (WORLD_BOTTOM + 1)..SEA_LEVEL {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
            }
        }
        chunk
    }

    /// After carving a stone-filled chunk, some blocks should become Air.
    #[test]
    fn carving_creates_air_blocks() {
        let mut chunk = stone_filled_chunk();
        let carver = CaveCarver::new(42);
        carver.carve(&mut chunk, 0, 0);

        let mut air_count = 0u64;
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in (WORLD_BOTTOM + 1)..SEA_LEVEL {
                    if chunk.get_block(x, y, z).is_air() {
                        air_count += 1;
                    }
                }
            }
        }
        assert!(
            air_count > 0,
            "Expected at least one air block after carving, but found none"
        );
    }

    /// Bedrock at WORLD_BOTTOM must never be carved.
    #[test]
    fn bedrock_is_never_carved() {
        let mut chunk = stone_filled_chunk();
        let carver = CaveCarver::new(42);
        carver.carve(&mut chunk, 0, 0);

        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                assert_eq!(
                    chunk.get_block(x, WORLD_BOTTOM, z),
                    BlockId::Bedrock,
                    "Bedrock at ({x}, {WORLD_BOTTOM}, {z}) was carved"
                );
            }
        }
    }

    /// Cheese caves must not carve at or above CHEESE_MAX_Y.
    #[test]
    fn cheese_caves_do_not_carve_above_threshold() {
        let mut chunk = Chunk::new();
        // Fill blocks from CHEESE_MAX_Y to SEA_LEVEL-1 with stone.
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in CHEESE_MAX_Y..SEA_LEVEL {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
            }
        }

        let carver = CaveCarver::new(42);

        // Check cheese specifically — we test the private helper.
        // All blocks at CHEESE_MAX_Y or above should NOT be carved by cheese logic.
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in CHEESE_MAX_Y..SEA_LEVEL {
                    let wx = x as f64;
                    let wz = z as f64;
                    assert!(
                        !carver.should_carve_cheese(wx, y, wz),
                        "Cheese carving triggered at y={y} (max is {CHEESE_MAX_Y})"
                    );
                }
            }
        }
    }

    /// No carving happens at or above sea level.
    #[test]
    fn no_carving_at_or_above_sea_level() {
        let mut chunk = Chunk::new();
        // Fill the whole chunk with stone, including above sea level.
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in WORLD_BOTTOM..320 {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
            }
        }

        let carver = CaveCarver::new(42);
        carver.carve(&mut chunk, 0, 0);

        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in SEA_LEVEL..320 {
                    assert_eq!(
                        chunk.get_block(x, y, z),
                        BlockId::Stone,
                        "Block at ({x}, {y}, {z}) was carved above sea level"
                    );
                }
            }
        }
    }

    /// Carving with different seeds produces different results.
    #[test]
    fn different_seeds_produce_different_caves() {
        let mut chunk_a = stone_filled_chunk();
        let mut chunk_b = stone_filled_chunk();

        CaveCarver::new(1).carve(&mut chunk_a, 0, 0);
        CaveCarver::new(999).carve(&mut chunk_b, 0, 0);

        let mut differ = false;
        'outer: for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                for y in (WORLD_BOTTOM + 1)..SEA_LEVEL {
                    if chunk_a.get_block(x, y, z) != chunk_b.get_block(x, y, z) {
                        differ = true;
                        break 'outer;
                    }
                }
            }
        }
        assert!(differ, "Two different seeds produced identical chunks");
    }

    /// Spaghetti caves should not carve outside their Y range.
    #[test]
    fn spaghetti_respects_y_bounds() {
        let carver = CaveCarver::new(42);

        // Test out-of-range Y values for spaghetti.
        for x_off in 0..16 {
            for z_off in 0..16 {
                let wx = x_off as f64;
                let wz = z_off as f64;
                // Below spaghetti range
                assert!(
                    !carver.should_carve_spaghetti(wx, SPAGHETTI_MIN_Y - 1, wz),
                    "Spaghetti carved below min Y"
                );
                // At or above spaghetti max
                assert!(
                    !carver.should_carve_spaghetti(wx, SPAGHETTI_MAX_Y, wz),
                    "Spaghetti carved at max Y"
                );
            }
        }
    }

    /// Verifies CaveCarver::new does not panic for edge-case seeds.
    #[test]
    fn construction_with_edge_seeds() {
        let _ = CaveCarver::new(0);
        let _ = CaveCarver::new(u64::MAX);
    }
}
