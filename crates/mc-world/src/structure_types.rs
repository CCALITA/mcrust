use mc_core::block::BlockId;
use mc_core::pos::{CHUNK_SIZE, WORLD_BOTTOM, WORLD_TOP};

use crate::chunk::Chunk;
use crate::structures::{CorridorDirection, PosRng};

/// Place a dungeon room underground.
///
/// Dimensions: `width` x `depth` x 4 (height), randomly chosen between 5-7.
/// Walls: `Cobblestone`/`MossyCobblestone`, floor: `Cobblestone`,
/// interior: `Air`, center: `Chest` (spawner placeholder).
pub(crate) fn place_dungeon(chunk: &mut Chunk, x: usize, y: i32, z: usize, rng: &mut PosRng) {
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
                if by < WORLD_BOTTOM || by >= WORLD_TOP {
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

/// Place a simple 5x4x5 wooden village house.
/// Walls/floor/roof: `OakPlanks`, door: 1x2 air on south side (+Z),
/// interior: `Torch` on north wall, `Chest` on floor.
pub(crate) fn place_house(chunk: &mut Chunk, x: usize, y: i32, z: usize) {
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
                if by < WORLD_BOTTOM || by >= WORLD_TOP {
                    continue;
                }

                let is_wall_x = dx == 0 || dx == width - 1;
                let is_wall_z = dz == 0 || dz == depth - 1;
                let is_floor = dy == 0;
                let is_roof = dy == height - 1;

                let block = if is_floor || is_roof {
                    BlockId::OakPlanks
                } else if is_wall_x || is_wall_z {
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

/// Place a straight mineshaft corridor.
/// 3-wide, 3-tall tunnel with `OakPlanks` floor, `OakLog` supports every
/// 4 blocks, `OakLog` beams across the top, `Torch` on each support.
pub(crate) fn place_corridor(
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

    // Perpendicular offset (rotate 90 degrees).
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

            if bx < 0 || bx >= CHUNK_SIZE || bz < 0 || bz >= CHUNK_SIZE {
                continue;
            }
            let ux = bx as usize;
            let uz = bz as usize;

            // Floor
            if y >= WORLD_BOTTOM && y < WORLD_TOP {
                chunk.set_block(ux, y, uz, BlockId::OakPlanks);
            }

            // Air interior (y+1 and y+2)
            for dy in 1..=2 {
                let by = y + dy;
                if by >= WORLD_BOTTOM && by < WORLD_TOP {
                    chunk.set_block(ux, by, uz, BlockId::Air);
                }
            }

            // Ceiling level (y+3) -- air by default, beams placed below.
            let ceiling_y = y + 3;
            if ceiling_y >= WORLD_BOTTOM && ceiling_y < WORLD_TOP {
                chunk.set_block(ux, ceiling_y, uz, BlockId::Air);
            }
        }

        // Supports every 4 blocks.
        if i % 4 == 0 {
            for side in [-1i32, 1] {
                let sx = base_x + px * side;
                let sz = base_z + pz * side;

                if sx < 0 || sx >= CHUNK_SIZE || sz < 0 || sz >= CHUNK_SIZE {
                    continue;
                }
                let ux = sx as usize;
                let uz = sz as usize;

                // Vertical support posts (y+1 and y+2).
                for dy in 1..=2 {
                    let by = y + dy;
                    if by >= WORLD_BOTTOM && by < WORLD_TOP {
                        chunk.set_block(ux, by, uz, BlockId::OakLog);
                    }
                }

                // Torch on top of the support (y+3).
                let torch_y = y + 3;
                if torch_y >= WORLD_BOTTOM && torch_y < WORLD_TOP {
                    chunk.set_block(ux, torch_y, uz, BlockId::Torch);
                }
            }

            // Horizontal beam across the top (y+3) on the center column.
            let beam_y = y + 3;
            if base_x >= 0
                && base_x < CHUNK_SIZE
                && base_z >= 0
                && base_z < CHUNK_SIZE
                && beam_y >= WORLD_BOTTOM
                && beam_y < WORLD_TOP
            {
                chunk.set_block(base_x as usize, beam_y, base_z as usize, BlockId::OakLog);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::pos::{CHUNK_SIZE, WORLD_BOTTOM};

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
    fn dungeon_has_cobblestone_walls_and_air_inside() {
        let mut chunk = make_terrain_chunk(64);
        let mut rng = PosRng::from_seed(42);
        place_dungeon(&mut chunk, 4, 20, 4, &mut rng);

        assert_eq!(chunk.get_block(4, 20, 4), BlockId::Cobblestone);
        let interior = chunk.get_block(6, 21, 6);
        assert_eq!(interior, BlockId::Air, "Interior should be air, got {interior:?}");
    }

    #[test]
    fn dungeon_has_chest_in_center() {
        let mut chunk = make_terrain_chunk(64);
        let mut rng = PosRng::from_seed(42);

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

        let mut found_mossy = false;
        for dx in 0..width {
            for dz in 0..depth {
                for dy in 1..4i32 {
                    let is_wall = dx == 0 || dx == width - 1 || dz == 0 || dz == depth - 1;
                    let is_ceiling = dy == 3;
                    if (is_wall || is_ceiling)
                        && chunk.get_block(2 + dx, 15 + dy, 2 + dz) == BlockId::MossyCobblestone
                    {
                        found_mossy = true;
                    }
                }
            }
        }
        assert!(found_mossy, "Expected at least one MossyCobblestone on dungeon walls");
    }

    #[test]
    fn house_has_oak_planks_walls() {
        let mut chunk = Chunk::new();
        let base_y = 64;
        place_house(&mut chunk, 4, base_y, 4);

        for dx in 0..5usize {
            let block = chunk.get_block(4 + dx, base_y + 1, 4);
            assert_eq!(block, BlockId::OakPlanks, "North wall at dx={dx}: {block:?}");
        }
        let interior = chunk.get_block(6, base_y + 1, 6);
        assert_eq!(interior, BlockId::Air, "Interior should be air, got {interior:?}");
    }

    #[test]
    fn house_has_door_opening() {
        let mut chunk = Chunk::new();
        let base_y = 64;
        place_house(&mut chunk, 4, base_y, 4);

        let door_x = 4 + 5 / 2;
        let door_z = 4 + 4;
        assert_eq!(chunk.get_block(door_x, base_y + 1, door_z), BlockId::Air);
        assert_eq!(chunk.get_block(door_x, base_y + 2, door_z), BlockId::Air);
    }

    #[test]
    fn house_has_torch_and_chest() {
        let mut chunk = Chunk::new();
        let base_y = 64;
        place_house(&mut chunk, 4, base_y, 4);

        assert_eq!(chunk.get_block(4 + 5 / 2, base_y + 2, 5), BlockId::Torch);
        assert_eq!(chunk.get_block(5, base_y + 1, 5), BlockId::Chest);
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
                    block, BlockId::OakPlanks,
                    "Floor at ({}, {}) should be OakPlanks, got {block:?}", 4 + dx, 4 + dz
                );
            }
        }
    }

    #[test]
    fn corridor_has_planks_floor() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 5, y, 5, 8, CorridorDirection::East);

        for i in 0..8 {
            let block = chunk.get_block(5 + i, y, 5);
            assert_eq!(block, BlockId::OakPlanks, "Floor at x={}: {block:?}", 5 + i);
        }
    }

    #[test]
    fn corridor_has_air_interior() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 5, y, 5, 8, CorridorDirection::East);

        for i in 0..8 {
            for dy in 1..=2 {
                let block = chunk.get_block(5 + i, y + dy, 5);
                assert_eq!(block, BlockId::Air, "Interior at x={}, dy={dy}: {block:?}", 5 + i);
            }
        }
    }

    #[test]
    fn corridor_has_oak_log_supports() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 4, y, 5, 12, CorridorDirection::East);

        for support_i in [0, 4, 8] {
            let sx = 4 + support_i;
            for sz in [4usize, 6] {
                for dy in 1..=2 {
                    let block = chunk.get_block(sx, y + dy, sz);
                    assert_eq!(
                        block, BlockId::OakLog,
                        "Support at ({sx}, y+{dy}, {sz}): {block:?}"
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

        for support_i in [0, 4, 8] {
            let sx = 4 + support_i;
            for sz in [4usize, 6] {
                let block = chunk.get_block(sx, y + 3, sz);
                assert_eq!(block, BlockId::Torch, "Torch at ({sx}, y+3, {sz}): {block:?}");
            }
        }
    }

    #[test]
    fn corridor_south_direction_extends_along_z() {
        let mut chunk = make_terrain_chunk(64);
        let y = 20;
        place_corridor(&mut chunk, 5, y, 2, 6, CorridorDirection::South);

        for i in 0..6 {
            let block = chunk.get_block(5, y, 2 + i);
            assert_eq!(block, BlockId::OakPlanks, "Floor at z={}: {block:?}", 2 + i);
        }
    }
}
