use mc_core::block::BlockId;
use mc_core::pos::{CHUNK_SIZE, WORLD_TOP};

use crate::chunk::Chunk;

/// Deterministic hash for placement decisions.
/// Combines seed with block coordinates using a simple mixing function.
fn hash(seed: u64, x: i32, z: i32) -> u64 {
    let mut h = seed;
    h = h.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    h ^= x as u64;
    h = h.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    h ^= z as u64;
    h = h.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    h
}

/// Returns a deterministic pseudo-random value in `[min, max]` (inclusive).
fn hash_range(seed: u64, x: i32, z: i32, min: i32, max: i32) -> i32 {
    let h = hash(seed, x, z);
    let range = (max - min + 1) as u64;
    min + (h % range) as i32
}

/// Place a classic oak tree at the given local (x, world_y, z) position.
///
/// * Trunk: 4-6 blocks of `OakLog`
/// * Leaves: blob of `OakLeaves` around the top of the trunk (radius ~2)
/// * Dirt placed below the trunk base
pub fn oak_tree(chunk: &mut Chunk, x: usize, y: i32, z: usize, seed: u64) {
    let height = hash_range(seed, x as i32, z as i32, 4, 6);

    // Set block below trunk to Dirt
    chunk.set_block(x, y - 1, z, BlockId::Dirt);

    // Trunk
    for dy in 0..height {
        chunk.set_block(x, y + dy, z, BlockId::OakLog);
    }

    // Leaves — blob around the top portion of the trunk
    let top = y + height - 1;
    let leaf_bottom = top - 2;
    let leaf_top = top + 1;

    for ly in leaf_bottom..=leaf_top {
        let radius = if ly >= top { 1 } else { 2 };
        place_leaf_layer(chunk, x, ly, z, radius);
    }
}

/// Place a birch-style tree (uses OakLog/OakLeaves since BirchLog is unavailable).
///
/// * Trunk: 5-7 blocks
/// * Leaves: narrower canopy (radius 1-2)
pub fn birch_tree(chunk: &mut Chunk, x: usize, y: i32, z: usize, seed: u64) {
    let height = hash_range(seed, x as i32, z as i32, 5, 7);

    chunk.set_block(x, y - 1, z, BlockId::Dirt);

    for dy in 0..height {
        chunk.set_block(x, y + dy, z, BlockId::OakLog);
    }

    let top = y + height - 1;
    let leaf_bottom = top - 1;
    let leaf_top = top + 1;

    for ly in leaf_bottom..=leaf_top {
        let radius = if ly >= top { 1 } else { 2 };
        place_leaf_layer(chunk, x, ly, z, radius);
    }
}

/// Place a spruce-style tree with a conical leaf shape.
///
/// * Trunk: 7-10 blocks
/// * Leaves: descending rings that widen toward the bottom
pub fn spruce_tree(chunk: &mut Chunk, x: usize, y: i32, z: usize, seed: u64) {
    let height = hash_range(seed, x as i32, z as i32, 7, 10);

    chunk.set_block(x, y - 1, z, BlockId::Dirt);

    for dy in 0..height {
        chunk.set_block(x, y + dy, z, BlockId::OakLog);
    }

    // Conical leaves: top is narrow, wider toward bottom of canopy
    let top = y + height - 1;
    let leaf_start = top - (height / 2);

    for ly in leaf_start..=top + 1 {
        let distance_from_top = (top + 1 - ly).max(0);
        let radius = match distance_from_top {
            0 => 0, // tip: just the center block
            1 => 1,
            2 => 2,
            _ => 3.min(distance_from_top),
        };
        place_leaf_layer(chunk, x, ly, z, radius);
    }
}

/// Place a square-ish layer of leaves at the given y level, centered on (x, z).
/// Does not replace solid blocks (except Air).
fn place_leaf_layer(chunk: &mut Chunk, cx: usize, y: i32, cz: usize, radius: i32) {
    if !(-64..WORLD_TOP).contains(&y) {
        return;
    }

    let chunk_max = CHUNK_SIZE - 1;

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            // Skip corners for a rounder shape when radius >= 2
            if radius >= 2 && dx.abs() == radius && dz.abs() == radius {
                continue;
            }

            let bx = cx as i32 + dx;
            let bz = cz as i32 + dz;

            if bx < 0 || bx > chunk_max || bz < 0 || bz > chunk_max {
                continue;
            }

            let existing = chunk.get_block(bx as usize, y, bz as usize);
            if !existing.is_solid() {
                chunk.set_block(bx as usize, y, bz as usize, BlockId::OakLeaves);
            }
        }
    }
}

/// Scatter trees on grass blocks within the chunk.
///
/// Uses deterministic hashing so the same seed + chunk coordinates always
/// produce the same tree placement. Averages roughly 3 trees per chunk.
/// Avoids chunk edges (x=0,15 and z=0,15) to prevent cross-chunk issues.
pub fn place_trees(chunk: &mut Chunk, cx: i32, cz: i32, seed: u64) {
    let chunk_size = CHUNK_SIZE as usize;

    for x in 2..chunk_size - 2 {
        for z in 2..chunk_size - 2 {
            let world_x = cx * CHUNK_SIZE + x as i32;
            let world_z = cz * CHUNK_SIZE + z as i32;
            let h = hash(seed, world_x, world_z);

            // ~3 trees per chunk: 3 / (12*12) ≈ 2.1% chance per valid position
            if !h.is_multiple_of(48) {
                continue;
            }

            // Find the topmost grass block by scanning downward
            let surface_y = match find_surface_grass(chunk, x, z) {
                Some(y) => y,
                None => continue,
            };

            // Ensure enough vertical space (at least 12 blocks of air above)
            let tree_base = surface_y + 1;
            let has_space = (1..=12).all(|dy| {
                chunk.get_block(x, tree_base + dy - 1, z) == BlockId::Air
            });
            if !has_space {
                continue;
            }

            // Pick tree type based on hash
            let tree_type = h % 3;
            match tree_type {
                0 => oak_tree(chunk, x, tree_base, z, seed.wrapping_add(h)),
                1 => birch_tree(chunk, x, tree_base, z, seed.wrapping_add(h)),
                _ => spruce_tree(chunk, x, tree_base, z, seed.wrapping_add(h)),
            }
        }
    }
}

/// Scatter vegetation (flowers) on exposed grass blocks.
///
/// Uses `BlockId::Torch` as a placeholder for flowers since dedicated flower
/// blocks are not available. Approximately 10% of exposed grass blocks receive
/// vegetation.
pub fn place_vegetation(chunk: &mut Chunk, cx: i32, cz: i32, seed: u64) {
    let chunk_size = CHUNK_SIZE as usize;
    let veg_seed = seed.wrapping_add(0xDEAD_BEEF);

    for x in 0..chunk_size {
        for z in 0..chunk_size {
            let world_x = cx * CHUNK_SIZE + x as i32;
            let world_z = cz * CHUNK_SIZE + z as i32;
            let h = hash(veg_seed, world_x, world_z);

            // ~10% chance
            if !h.is_multiple_of(10) {
                continue;
            }

            let surface_y = match find_surface_grass(chunk, x, z) {
                Some(y) => y,
                None => continue,
            };

            // Place vegetation one block above the grass if that block is air
            let above = surface_y + 1;
            if chunk.get_block(x, above, z) == BlockId::Air {
                chunk.set_block(x, above, z, BlockId::Torch);
            }
        }
    }
}

/// Scan downward from the world top to find the highest GrassBlock at the
/// given local (x, z) column. Returns `None` if no grass is found.
fn find_surface_grass(chunk: &Chunk, x: usize, z: usize) -> Option<i32> {
    // Scan from a reasonable max height downward
    for y in (0..=WORLD_TOP - 1).rev() {
        let block = chunk.get_block(x, y, z);
        if block == BlockId::GrassBlock {
            return Some(y);
        }
        // If we hit a non-air solid block that isn't grass, no grass above here
        if block.is_solid() {
            return None;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;

    /// Create a flat chunk with grass at the given y level, dirt below, air above.
    fn make_flat_chunk(grass_y: i32) -> Chunk {
        let mut chunk = Chunk::new();
        let size = CHUNK_SIZE as usize;
        for x in 0..size {
            for z in 0..size {
                chunk.set_block(x, grass_y - 1, z, BlockId::Dirt);
                chunk.set_block(x, grass_y, z, BlockId::GrassBlock);
            }
        }
        chunk
    }

    #[test]
    fn oak_tree_places_trunk_and_leaves() {
        let mut chunk = make_flat_chunk(63);
        let x = 8;
        let z = 8;
        let base_y = 64; // one above grass

        oak_tree(&mut chunk, x, base_y, z, 42);

        // Trunk: at least 4 OakLog blocks
        let mut log_count = 0;
        for dy in 0..6 {
            if chunk.get_block(x, base_y + dy, z) == BlockId::OakLog {
                log_count += 1;
            }
        }
        assert!(
            log_count >= 4,
            "Expected at least 4 OakLog blocks, found {log_count}"
        );

        // Block below trunk should be Dirt
        assert_eq!(chunk.get_block(x, base_y - 1, z), BlockId::Dirt);

        // Leaves should exist near the top of the trunk
        let mut leaf_count = 0;
        for dy in 0..10 {
            for dx in -2i32..=2 {
                for dz in -2i32..=2 {
                    let bx = x as i32 + dx;
                    let bz = z as i32 + dz;
                    if bx >= 0 && bx < 16 && bz >= 0 && bz < 16 {
                        if chunk.get_block(bx as usize, base_y + dy, bz as usize)
                            == BlockId::OakLeaves
                        {
                            leaf_count += 1;
                        }
                    }
                }
            }
        }
        assert!(
            leaf_count >= 5,
            "Expected at least 5 OakLeaves blocks, found {leaf_count}"
        );
    }

    #[test]
    fn birch_tree_places_trunk_and_leaves() {
        let mut chunk = make_flat_chunk(63);
        let x = 8;
        let z = 8;
        let base_y = 64;

        birch_tree(&mut chunk, x, base_y, z, 123);

        let mut log_count = 0;
        for dy in 0..8 {
            if chunk.get_block(x, base_y + dy, z) == BlockId::OakLog {
                log_count += 1;
            }
        }
        assert!(
            log_count >= 5,
            "Expected at least 5 OakLog blocks for birch, found {log_count}"
        );
    }

    #[test]
    fn spruce_tree_places_tall_trunk_and_conical_leaves() {
        let mut chunk = make_flat_chunk(63);
        let x = 8;
        let z = 8;
        let base_y = 64;

        spruce_tree(&mut chunk, x, base_y, z, 999);

        let mut log_count = 0;
        for dy in 0..11 {
            if chunk.get_block(x, base_y + dy, z) == BlockId::OakLog {
                log_count += 1;
            }
        }
        assert!(
            log_count >= 7,
            "Expected at least 7 OakLog blocks for spruce, found {log_count}"
        );

        // Leaves should exist
        let mut leaf_count = 0;
        for dy in 0..12 {
            for dx in -3i32..=3 {
                for dz in -3i32..=3 {
                    let bx = x as i32 + dx;
                    let bz = z as i32 + dz;
                    if bx >= 0 && bx < 16 && bz >= 0 && bz < 16 {
                        if chunk.get_block(bx as usize, base_y + dy, bz as usize)
                            == BlockId::OakLeaves
                        {
                            leaf_count += 1;
                        }
                    }
                }
            }
        }
        assert!(
            leaf_count >= 5,
            "Expected at least 5 OakLeaves for spruce, found {leaf_count}"
        );
    }

    #[test]
    fn place_trees_generates_some_trees() {
        let mut chunk = make_flat_chunk(63);
        place_trees(&mut chunk, 0, 0, 12345);

        // Count all OakLog blocks placed above grass level
        let mut log_count = 0;
        let size = CHUNK_SIZE as usize;
        for x in 0..size {
            for z in 0..size {
                for y in 64..128 {
                    if chunk.get_block(x, y, z) == BlockId::OakLog {
                        log_count += 1;
                    }
                }
            }
        }
        assert!(
            log_count > 0,
            "Expected at least some OakLog blocks from place_trees"
        );
    }

    #[test]
    fn place_vegetation_scatters_flowers() {
        let mut chunk = make_flat_chunk(63);
        place_vegetation(&mut chunk, 0, 0, 12345);

        let mut torch_count = 0;
        let size = CHUNK_SIZE as usize;
        for x in 0..size {
            for z in 0..size {
                if chunk.get_block(x, 64, z) == BlockId::Torch {
                    torch_count += 1;
                }
            }
        }
        assert!(
            torch_count > 0,
            "Expected at least some Torch (flower) blocks from place_vegetation"
        );
    }

    #[test]
    fn trees_not_placed_on_non_grass() {
        let mut chunk = Chunk::new();
        let size = CHUNK_SIZE as usize;
        // Fill with stone instead of grass
        for x in 0..size {
            for z in 0..size {
                chunk.set_block(x, 63, z, BlockId::Stone);
            }
        }

        place_trees(&mut chunk, 0, 0, 42);

        // No logs should be placed
        let mut log_count = 0;
        for x in 0..size {
            for z in 0..size {
                for y in 64..128 {
                    if chunk.get_block(x, y, z) == BlockId::OakLog {
                        log_count += 1;
                    }
                }
            }
        }
        assert_eq!(log_count, 0, "No trees should be placed on non-grass blocks");
    }

    #[test]
    fn hash_is_deterministic() {
        let h1 = hash(42, 10, 20);
        let h2 = hash(42, 10, 20);
        assert_eq!(h1, h2, "Hash should be deterministic");

        let h3 = hash(42, 10, 21);
        assert_ne!(h1, h3, "Different inputs should produce different hashes");
    }

    #[test]
    fn oak_tree_does_not_replace_solid_blocks_with_leaves() {
        let mut chunk = make_flat_chunk(63);
        // Place a stone block where a leaf would go
        chunk.set_block(9, 68, 8, BlockId::Stone);

        oak_tree(&mut chunk, 8, 64, 8, 42);

        // The stone block should remain
        assert_eq!(
            chunk.get_block(9, 68, 8),
            BlockId::Stone,
            "Leaves should not replace solid blocks"
        );
    }
}
