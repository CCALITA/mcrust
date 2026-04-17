use std::collections::VecDeque;

use mc_core::pos::{CHUNK_SIZE, WORLD_BOTTOM, WORLD_TOP};

const WORLD_HEIGHT: i32 = WORLD_TOP - WORLD_BOTTOM;
const VOLUME: usize = (CHUNK_SIZE as usize) * (WORLD_HEIGHT as usize) * (CHUNK_SIZE as usize);

/// Stores per-block light levels for an entire chunk column.
///
/// Each entry holds a value in 0..=15. Two parallel arrays track block-emitted
/// light and sky light independently.
pub struct LightMap {
    block_light: Vec<u8>,
    sky_light: Vec<u8>,
}

/// Six axis-aligned neighbours (dx, dy, dz).
const NEIGHBOURS: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

#[inline]
fn index(x: usize, y: usize, z: usize) -> usize {
    y * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) + z * (CHUNK_SIZE as usize) + x
}

impl LightMap {
    fn new() -> Self {
        Self {
            block_light: vec![0; VOLUME],
            sky_light: vec![0; VOLUME],
        }
    }

    /// Returns `(block_light, sky_light)` for a position local to the chunk.
    ///
    /// `x` and `z` are in `0..CHUNK_SIZE`, `world_y` is an absolute Y coordinate
    /// (`WORLD_BOTTOM..WORLD_TOP`).
    pub fn get_light(&self, x: usize, world_y: i32, z: usize) -> (u8, u8) {
        let y = (world_y - WORLD_BOTTOM) as usize;
        let idx = index(x, y, z);
        (self.block_light[idx], self.sky_light[idx])
    }
}

/// Compute the effective light level from block light, sky light, and time of day.
///
/// `time_of_day` ranges from `0.0` (midnight) to `1.0` (next midnight), with
/// noon at `0.5`.  The daylight factor follows a simple cosine curve clamped
/// to `[0, 1]`.
pub fn max_light(block: u8, sky: u8, time_of_day: f32) -> u8 {
    // Map time_of_day to a daylight factor.
    // noon (0.5) -> factor 1.0, midnight (0.0 / 1.0) -> factor 0.0
    let angle = (time_of_day - 0.5) * 2.0 * std::f32::consts::PI;
    let daylight_factor = (angle.cos() * 0.5 + 0.5).clamp(0.0, 1.0);

    let effective_sky = (sky as f32 * daylight_factor).round() as u8;
    block.max(effective_sky)
}

/// Propagate block-emitted light through a chunk using BFS flood-fill.
///
/// Every block whose `light_emission > 0` acts as a source.  Light decreases
/// by 1 for each step through a transparent (non-opaque) block.  Opaque blocks
/// stop propagation entirely.
pub fn propagate_block_light(chunk: &crate::Chunk) -> LightMap {
    let mut map = LightMap::new();
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();

    let height = WORLD_HEIGHT as usize;
    let cs = CHUNK_SIZE as usize;

    // Seed every light-emitting block.
    for y in 0..height {
        for z in 0..cs {
            for x in 0..cs {
                let world_y = y as i32 + WORLD_BOTTOM;
                let block = chunk.get_block(x, world_y, z);
                let emission = block.properties().light_emission;
                if emission > 0 {
                    let idx = index(x, y, z);
                    map.block_light[idx] = emission;
                    queue.push_back((x, y, z));
                }
            }
        }
    }

    // BFS flood-fill.
    while let Some((x, y, z)) = queue.pop_front() {
        let current = map.block_light[index(x, y, z)];
        if current <= 1 {
            continue;
        }
        let new_level = current - 1;

        for (dx, dy, dz) in NEIGHBOURS {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;

            if nx < 0
                || nx >= CHUNK_SIZE
                || ny < 0
                || ny >= WORLD_HEIGHT
                || nz < 0
                || nz >= CHUNK_SIZE
            {
                continue;
            }

            let (ux, uy, uz) = (nx as usize, ny as usize, nz as usize);
            let world_y = ny + WORLD_BOTTOM;
            let neighbour = chunk.get_block(ux, world_y, uz);

            // Opaque blocks block light.
            if !neighbour.is_transparent() {
                continue;
            }

            let nidx = index(ux, uy, uz);
            if map.block_light[nidx] < new_level {
                map.block_light[nidx] = new_level;
                queue.push_back((ux, uy, uz));
            }
        }
    }

    map
}

/// Propagate sky light downward through a chunk.
///
/// Sky light starts at 15 at the very top.  It travels straight down through
/// transparent blocks *without* decreasing.  Once it hits a solid/opaque block,
/// it stops going down at that column.  Any block that received sky light also
/// spreads it horizontally (and vertically) with a -1 decrease per step, using
/// the same BFS approach as block light.
pub fn propagate_sky_light(chunk: &crate::Chunk) -> LightMap {
    let mut map = LightMap::new();
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();

    let height = WORLD_HEIGHT as usize;
    let cs = CHUNK_SIZE as usize;

    // Phase 1: vertical downcast -- light 15 falls straight down.
    for z in 0..cs {
        for x in 0..cs {
            for y in (0..height).rev() {
                let world_y = y as i32 + WORLD_BOTTOM;
                let block = chunk.get_block(x, world_y, z);
                if block.is_transparent() {
                    let idx = index(x, y, z);
                    map.sky_light[idx] = 15;
                    queue.push_back((x, y, z));
                } else {
                    // Solid/opaque block: stop vertical propagation for this column.
                    break;
                }
            }
        }
    }

    // Phase 2: BFS horizontal (and further vertical) spread with -1 per step.
    while let Some((x, y, z)) = queue.pop_front() {
        let current = map.sky_light[index(x, y, z)];
        if current <= 1 {
            continue;
        }
        let new_level = current - 1;

        for (dx, dy, dz) in NEIGHBOURS {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;

            if nx < 0
                || nx >= CHUNK_SIZE
                || ny < 0
                || ny >= WORLD_HEIGHT
                || nz < 0
                || nz >= CHUNK_SIZE
            {
                continue;
            }

            let (ux, uy, uz) = (nx as usize, ny as usize, nz as usize);
            let world_y = ny + WORLD_BOTTOM;
            let neighbour = chunk.get_block(ux, world_y, uz);

            if !neighbour.is_transparent() {
                continue;
            }

            let nidx = index(ux, uy, uz);
            if map.sky_light[nidx] < new_level {
                map.sky_light[nidx] = new_level;
                queue.push_back((ux, uy, uz));
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::block::BlockId;
    use mc_core::pos::WORLD_BOTTOM;

    /// Helper: build a chunk that is entirely air.
    fn air_chunk() -> crate::Chunk {
        crate::Chunk::new()
    }

    // ------------------------------------------------------------------
    // Block light tests
    // ------------------------------------------------------------------

    #[test]
    fn torch_lights_surrounding_blocks() {
        let mut chunk = air_chunk();
        // Place a torch at the center of the chunk at y=0 (world y = WORLD_BOTTOM + 64 = 0).
        let cy = 0i32; // world y
        chunk.set_block(8, cy, 8, BlockId::Torch);

        let map = propagate_block_light(&chunk);

        // The torch itself should be at emission level 14.
        let (bl, _) = map.get_light(8, cy, 8);
        assert_eq!(bl, 14, "torch block light should be 14");

        // One block away in each cardinal direction should be 13.
        assert_eq!(map.get_light(9, cy, 8).0, 13);
        assert_eq!(map.get_light(7, cy, 8).0, 13);
        assert_eq!(map.get_light(8, cy, 9).0, 13);
        assert_eq!(map.get_light(8, cy, 7).0, 13);
        assert_eq!(map.get_light(8, cy + 1, 8).0, 13);
        assert_eq!(map.get_light(8, cy - 1, 8).0, 13);

        // Two blocks away should be 12.
        assert_eq!(map.get_light(10, cy, 8).0, 12);

        // 14 blocks away should be 0 (14 - 14 = 0, clamped).
        assert_eq!(map.get_light(0, cy, 8).0, 6); // distance = 8 => 14 - 8 = 6
    }

    #[test]
    fn glowstone_emits_level_15() {
        let mut chunk = air_chunk();
        let cy = 0i32;
        chunk.set_block(8, cy, 8, BlockId::Glowstone);

        let map = propagate_block_light(&chunk);

        assert_eq!(map.get_light(8, cy, 8).0, 15);
        assert_eq!(map.get_light(9, cy, 8).0, 14);
    }

    #[test]
    fn light_blocked_by_stone() {
        let mut chunk = air_chunk();
        let cy = 0i32;
        chunk.set_block(8, cy, 8, BlockId::Torch); // emission 14
        // Place a wall of stone one block east.
        chunk.set_block(9, cy, 8, BlockId::Stone);

        let map = propagate_block_light(&chunk);

        // Stone itself should receive no light (it is opaque).
        assert_eq!(map.get_light(9, cy, 8).0, 0);
        // Block behind the stone should only get light from indirect paths,
        // not the direct line through the stone.
        let behind = map.get_light(10, cy, 8).0;
        // Without the stone the block at (10,cy,8) would get 14 - 2 = 12.
        // With the stone blocking the direct path the light must arrive via
        // a detour of length >= 4 (around the single stone block), so the
        // level is at most 14 - 4 = 10.
        assert!(
            behind <= 10,
            "light behind stone should be attenuated, got {behind}"
        );
    }

    #[test]
    fn no_emission_means_no_block_light() {
        let chunk = air_chunk();
        let map = propagate_block_light(&chunk);

        // Spot-check several positions -- everything should be dark.
        assert_eq!(map.get_light(0, 0, 0).0, 0);
        assert_eq!(map.get_light(8, 100, 8).0, 0);
    }

    // ------------------------------------------------------------------
    // Sky light tests
    // ------------------------------------------------------------------

    #[test]
    fn sky_light_is_15_on_surface() {
        let chunk = air_chunk();
        let map = propagate_sky_light(&chunk);

        // In an all-air chunk, every block should receive sky light 15.
        assert_eq!(map.get_light(0, 0, 0).1, 15);
        assert_eq!(map.get_light(8, WORLD_BOTTOM, 8).1, 15);
        assert_eq!(map.get_light(15, WORLD_TOP - 1, 15).1, 15);
    }

    #[test]
    fn sky_light_blocked_by_stone_ceiling() {
        let mut chunk = air_chunk();
        let ceiling_y = 100i32;
        // Build a full 16x16 stone ceiling at y=100.
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, ceiling_y, z, BlockId::Stone);
            }
        }

        let map = propagate_sky_light(&chunk);

        // Above the ceiling: still 15.
        assert_eq!(map.get_light(8, ceiling_y + 1, 8).1, 15);
        // The ceiling block itself is opaque -- sky light should be 0.
        assert_eq!(map.get_light(8, ceiling_y, 8).1, 0);
        // One block below the ceiling: no direct path, can only receive via
        // horizontal spread, but the ceiling is wall-to-wall so no path at all.
        assert_eq!(map.get_light(8, ceiling_y - 1, 8).1, 0);
    }

    #[test]
    fn sky_light_spreads_under_partial_roof() {
        let mut chunk = air_chunk();
        let roof_y = 100i32;
        // Place stone everywhere at y=100 *except* column (0, z=0).
        for x in 0..16usize {
            for z in 0..16usize {
                if x == 0 && z == 0 {
                    continue; // leave a gap
                }
                chunk.set_block(x, roof_y, z, BlockId::Stone);
            }
        }

        let map = propagate_sky_light(&chunk);

        // The gap column (0,0) should still have sky light 15 below the roof.
        assert_eq!(map.get_light(0, roof_y - 1, 0).1, 15);
        // One block over from the gap should be 14 (spread horizontally -1).
        assert_eq!(map.get_light(1, roof_y - 1, 0).1, 14);
        // Two blocks from gap should be 13.
        assert_eq!(map.get_light(2, roof_y - 1, 0).1, 13);
    }

    // ------------------------------------------------------------------
    // max_light tests
    // ------------------------------------------------------------------

    #[test]
    fn max_light_at_noon_uses_full_sky() {
        // noon = 0.5
        let result = max_light(5, 15, 0.5);
        assert_eq!(result, 15, "at noon sky light should dominate");
    }

    #[test]
    fn max_light_at_midnight_uses_block_light() {
        // midnight = 0.0
        let result = max_light(10, 15, 0.0);
        // Daylight factor at midnight is ~0 so effective sky ~ 0.
        assert_eq!(result, 10);
    }

    #[test]
    fn max_light_picks_higher_value() {
        // Block light exceeds dimmed sky.
        let result = max_light(12, 8, 0.25);
        assert!(result >= 12);
    }
}
