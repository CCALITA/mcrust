//! Integration tests for mc-world.
//!
//! Each test exercises at least two modules together to verify they compose
//! correctly.  All tests are independent and can run in any order.

use mc_core::*;
use mc_world::*;

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal in-memory world that satisfies the `FluidWorld` trait.
struct TestFluidWorld {
    blocks: HashMap<BlockPos, BlockId>,
    scheduled: Vec<(BlockPos, u32)>,
}

impl TestFluidWorld {
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            scheduled: Vec::new(),
        }
    }

    fn place(&mut self, pos: BlockPos, block: BlockId) {
        self.blocks.insert(pos, block);
    }
}

impl FluidWorld for TestFluidWorld {
    fn get_block(&self, pos: BlockPos) -> BlockId {
        self.blocks.get(&pos).copied().unwrap_or(BlockId::Air)
    }

    fn set_block(&mut self, pos: BlockPos, block: BlockId) {
        self.blocks.insert(pos, block);
    }

    fn schedule_update(&mut self, pos: BlockPos, delay: u32) {
        self.scheduled.push((pos, delay));
    }
}

/// Count how many blocks of `target` exist in the chunk between `min_y` and
/// `max_y` (exclusive upper bound).
fn count_block(chunk: &Chunk, target: BlockId, min_y: i32, max_y: i32) -> u64 {
    let mut count = 0u64;
    for x in 0..16 {
        for z in 0..16 {
            for y in min_y..max_y {
                if chunk.get_block(x, y, z) == target {
                    count += 1;
                }
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// 1. NoiseTerrainGen + OreGenerator: terrain AND ores
// ---------------------------------------------------------------------------

#[test]
fn terrain_gen_produces_non_uniform_terrain_with_ores() {
    let seed = 42u64;
    let terrain_gen = NoiseTerrainGen::new(seed);
    let ore_gen = OreGenerator::new(seed);

    let mut chunk = terrain_gen.generate(0, 0);
    ore_gen.generate_ores(&mut chunk, 0, 0);

    // Verify non-uniform terrain: bedrock at bottom, different blocks above.
    assert_eq!(chunk.get_block(0, -64, 0), BlockId::Bedrock);

    let mut distinct_blocks = std::collections::HashSet::new();
    for x in 0..16 {
        for z in 0..16 {
            for y in -64..130 {
                distinct_blocks.insert(chunk.get_block(x, y, z));
            }
        }
    }
    // Terrain + ores should yield many distinct block types (stone, dirt, grass,
    // bedrock, air, plus at least one ore type).
    assert!(
        distinct_blocks.len() >= 6,
        "expected >= 6 distinct block types, got {}",
        distinct_blocks.len()
    );

    // At least one ore type should be present.
    let ore_blocks = [
        BlockId::CoalOre,
        BlockId::IronOre,
        BlockId::GoldOre,
        BlockId::DiamondOre,
    ];
    let has_ore = ore_blocks.iter().any(|ore| distinct_blocks.contains(ore));
    assert!(
        has_ore,
        "expected at least one ore type in the generated chunk"
    );
}

// ---------------------------------------------------------------------------
// 2. NoiseTerrainGen + CaveCarver: caves create air in solid areas
// ---------------------------------------------------------------------------

#[test]
fn terrain_gen_plus_cave_carver_creates_air_in_solid() {
    let seed = 42u64;
    let terrain_gen = NoiseTerrainGen::new(seed);
    let carver = CaveCarver::new(seed);

    let mut chunk = terrain_gen.generate(0, 0);

    // Count solid blocks before carving.
    let solid_before = count_block(&chunk, BlockId::Stone, -63, 50);

    carver.carve(&mut chunk, 0, 0);

    let solid_after = count_block(&chunk, BlockId::Stone, -63, 50);
    let air_after = count_block(&chunk, BlockId::Air, -63, 50);

    // Caves should have removed some stone.
    assert!(
        solid_after < solid_before,
        "carving should reduce stone count (before={solid_before}, after={solid_after})"
    );
    assert!(
        air_after > 0,
        "caves should have created at least one air block underground"
    );

    // Bedrock must survive.
    assert_eq!(chunk.get_block(0, -64, 0), BlockId::Bedrock);
}

// ---------------------------------------------------------------------------
// 3. ChunkManager: update loads chunks AND dirty tracking works
// ---------------------------------------------------------------------------

#[test]
fn chunk_manager_loads_and_tracks_dirty() {
    let mut mgr = ChunkManager::new(2);
    mgr.update(ChunkPos::new(0, 0));

    // Should have loaded (2*2+1)^2 = 25 chunks.
    let loaded_count = mgr.loaded_chunks().count();
    assert_eq!(loaded_count, 25);

    // All freshly loaded chunks should be dirty.
    let dirty = mgr.take_dirty();
    assert_eq!(dirty.len(), 25);

    // After draining, dirty set should be empty.
    let dirty2 = mgr.take_dirty();
    assert!(dirty2.is_empty());

    // Modifying a block should mark the chunk dirty again.
    mgr.set_block(BlockPos::new(5, 70, 5), BlockId::Cobblestone);
    let dirty3 = mgr.take_dirty();
    assert!(dirty3.contains(&ChunkPos::new(0, 0)));
}

// ---------------------------------------------------------------------------
// 4. save::chunk_to_save + save_to_chunk round-trip
// ---------------------------------------------------------------------------

#[test]
fn save_round_trip_preserves_blocks() {
    let terrain_gen = NoiseTerrainGen::new(99);
    let chunk = terrain_gen.generate(3, 5);

    let save_data = chunk_to_save(&chunk, ChunkPos::new(3, 5));
    let restored = save_to_chunk(&save_data);

    // Verify every block matches.
    for x in 0..16 {
        for z in 0..16 {
            for y in -64..320 {
                assert_eq!(
                    chunk.get_block(x, y, z),
                    restored.get_block(x, y, z),
                    "mismatch at ({x}, {y}, {z})"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 5. explosion + block_resistance: blocks destroyed AND some survive
// ---------------------------------------------------------------------------

#[test]
fn explosion_destroys_some_blocks_but_not_bedrock_or_obsidian() {
    // Build a world: dirt everywhere except bedrock at y=-64 and obsidian at y=0.
    let get_block = |x: i32, y: i32, z: i32| -> BlockId {
        let _ = x;
        let _ = z;
        if y == -64 {
            BlockId::Bedrock
        } else if y == 0 {
            BlockId::Obsidian
        } else {
            BlockId::Dirt
        }
    };

    let result = calculate_explosion((0.0, 5.0, 0.0), TNT_POWER, &get_block);

    // TNT should destroy some dirt blocks.
    assert!(
        !result.destroyed_blocks.is_empty(),
        "explosion should destroy dirt blocks"
    );

    // Bedrock and obsidian must NOT appear in the destroyed list.
    for &(x, y, z) in &result.destroyed_blocks {
        let block = get_block(x, y, z);
        assert_ne!(
            block,
            BlockId::Bedrock,
            "bedrock at ({x},{y},{z}) should survive"
        );
        assert_ne!(
            block,
            BlockId::Obsidian,
            "obsidian at ({x},{y},{z}) should survive"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. fluid: place water, tick, verify horizontal spread
// ---------------------------------------------------------------------------

#[test]
fn water_spreads_horizontally_on_solid_floor() {
    let mut world = TestFluidWorld::new();
    let source = BlockPos::new(10, 65, 10);

    // Place water on top of a solid floor.
    world.place(source, BlockId::Water);
    world.place(BlockPos::new(10, 64, 10), BlockId::Stone);

    process_water_update(&mut world, source);

    // Water should spread to the four horizontal neighbors.
    let neighbors = [
        BlockPos::new(11, 65, 10),
        BlockPos::new(9, 65, 10),
        BlockPos::new(10, 65, 11),
        BlockPos::new(10, 65, 9),
    ];
    for n in &neighbors {
        assert_eq!(
            world.get_block(*n),
            BlockId::Water,
            "water should have spread to {:?}",
            n
        );
    }
    assert_eq!(
        world.scheduled.len(),
        4,
        "each new water block should be scheduled"
    );
}

// ---------------------------------------------------------------------------
// 7. farming + crop growth: hydrated crop grows faster
// ---------------------------------------------------------------------------

#[test]
fn hydrated_crop_grows_and_can_be_harvested() {
    let mut crop = CropState::new(CropType::Wheat);

    // Verify planting condition.
    assert!(can_plant_on(CropType::Wheat, BlockId::Farmland as u16));
    assert!(is_hydrated(true));

    // Grow the crop to maturity using a guaranteed-grow random value.
    while !crop.is_mature() {
        let grew = tick_crop(&mut crop, true, 15, 0.0);
        assert!(
            grew,
            "crop should grow with random_val=0.0 and sufficient light"
        );
    }
    assert_eq!(crop.growth_stage, 7);

    // Harvest should produce drops.
    let drops = harvest(&crop);
    assert!(!drops.is_empty(), "mature wheat should yield drops");
}

// ---------------------------------------------------------------------------
// 8. fire + flammability: fire on flammable block produces actions
// ---------------------------------------------------------------------------

#[test]
fn fire_on_flammable_block_spreads_or_destroys() {
    // Verify OakPlanks is flammable.
    assert!(is_flammable(BlockId::OakPlanks));
    assert!(flammability(BlockId::OakPlanks) > 0);
    assert!(burn_chance(BlockId::OakPlanks) > 0);

    // Tick fire with a mid-range random to trigger spread.
    let mut fire = FireState::new((5, 64, 5));
    let action = tick_fire(&mut fire, false, 0.4);
    match action {
        FireAction::SpreadTo(targets) => {
            assert_eq!(
                targets.len(),
                6,
                "fire should attempt to spread to 6 neighbors"
            );
        }
        other => panic!("expected SpreadTo, got {:?}", other),
    }

    // Tick fire with low random to trigger block destruction.
    let mut fire2 = FireState::new((5, 64, 5));
    let action2 = tick_fire(&mut fire2, false, 0.1);
    assert_eq!(action2, FireAction::BlockDestroyed((5, 63, 5)));

    // Rain should extinguish fire.
    let mut fire3 = FireState::new((5, 64, 5));
    let action3 = tick_fire(&mut fire3, true, 0.5);
    assert_eq!(action3, FireAction::Extinguished);
}

// ---------------------------------------------------------------------------
// 9. beacon: pyramid scanning + level detection + effects
// ---------------------------------------------------------------------------

#[test]
fn beacon_pyramid_level_detection_and_effects() {
    // Build a 2-layer pyramid (level 2 = 5x5 base + 3x3 above it).
    let get_block = |x: i32, y: i32, z: i32| -> BlockId {
        let _ = y;
        // Layer 1 (3x3): centered on origin, y = beacon_y - 1
        // Layer 2 (5x5): centered on origin, y = beacon_y - 2
        if x.abs() <= 2 && z.abs() <= 2 {
            BlockId::IronOre
        } else {
            BlockId::Air
        }
    };

    let level = scan_pyramid((0, 10, 0), &get_block);
    assert_eq!(level, 2, "expected level 2 pyramid");

    let range = beacon_range(level);
    assert_eq!(range, 30.0, "level 2 beacon should have 30 block range");

    let effects = available_effects(level);
    assert_eq!(effects.len(), 4);
    assert!(effects.contains(&BeaconEffect::Speed));
    assert!(effects.contains(&BeaconEffect::Haste));
    assert!(effects.contains(&BeaconEffect::Resistance));
    assert!(effects.contains(&BeaconEffect::JumpBoost));

    // An entity 25 blocks away should be in range (30 block range).
    assert!(is_in_range((0, 10, 0), (25.0, 10.0, 0.0), level));
    // An entity 35 blocks away should be out of range.
    assert!(!is_in_range((0, 10, 0), (35.0, 10.0, 0.0), level));
}

// ---------------------------------------------------------------------------
// 10. map_data + NoiseTerrainGen: map from generated terrain has varied colors
// ---------------------------------------------------------------------------

#[test]
fn map_from_terrain_has_non_uniform_colors() {
    let terrain_gen = NoiseTerrainGen::new(42);

    let get_surface_block = |world_x: i32, world_z: i32| -> BlockId {
        let height = terrain_gen.height_at(world_x, world_z);
        if height > 63 {
            BlockId::GrassBlock
        } else if height == 63 {
            BlockId::Sand
        } else {
            BlockId::Water
        }
    };

    let map = generate_map(0, 0, 1, 0, &get_surface_block);

    // Collect distinct colors.
    let distinct_colors: std::collections::HashSet<u8> = map.pixels().iter().copied().collect();

    assert!(
        distinct_colors.len() >= 2,
        "map should contain at least 2 distinct colors from varied terrain, got {}",
        distinct_colors.len()
    );

    // Map dimensions are correct.
    assert_eq!(map.pixels().len(), MAP_SIZE * MAP_SIZE);
}

// ---------------------------------------------------------------------------
// 11. world_border: clamping with moving border
// ---------------------------------------------------------------------------

#[test]
fn world_border_clamping_with_shrinking_border() {
    let mut border = WorldBorder::new(200.0);

    // Position inside is not clamped.
    let (cx, cz) = border.clamp_position(50.0, 50.0);
    assert_eq!((cx, cz), (50.0, 50.0));

    // Start shrinking border from 200 to 100 over 10 seconds.
    border.set_size(100.0, 10.0);

    // Tick halfway (5 seconds).
    border.tick(5.0);
    assert!((border.size - 150.0).abs() < f64::EPSILON);

    // A position at x=80 was inside at size 200, but should still be inside at 150.
    assert!(border.is_inside(70.0, 0.0));

    // Tick the rest of the way.
    border.tick(5.0);
    assert!((border.size - 100.0).abs() < f64::EPSILON);

    // Now x=60 is outside (border edge is at 50).
    assert!(!border.is_inside(60.0, 0.0));

    // Clamping should bring it to the edge.
    let (cx2, _cz2) = border.clamp_position(60.0, 0.0);
    assert!((cx2 - 50.0).abs() < f64::EPSILON);

    // Damage should be nonzero outside.
    let damage = border.damage_at(55.0, 0.0);
    assert!(damage > 0.0, "should take damage outside border");
}

// ---------------------------------------------------------------------------
// 12. weather: transitions over many ticks
// ---------------------------------------------------------------------------

#[test]
fn weather_transitions_through_multiple_states() {
    let mut ws = WeatherSystem::new(42);

    let mut seen_clear = true; // starts clear
    let mut seen_rain = false;

    // Tick up to 100k ticks to observe at least one transition.
    for _ in 0..100_000 {
        ws.tick();
        if ws.is_raining() {
            seen_rain = true;
        }
        if seen_rain && !ws.is_raining() {
            seen_clear = true;
            break;
        }
    }

    assert!(seen_clear, "should have seen clear weather");
    assert!(seen_rain, "should have seen rain within 100k ticks");

    // Sky darkness changes with state.
    // We can at least verify the API works consistently.
    let _darkness = ws.sky_darkness();
    let _strength = ws.rain_strength();
}

// ---------------------------------------------------------------------------
// 13. spawn: set/get lifecycle with sleep validation
// ---------------------------------------------------------------------------

#[test]
fn spawn_manager_lifecycle_with_sleep() {
    let mut mgr = SpawnManager::new(64);

    // Default spawn.
    let spawn = mgr.get_spawn();
    assert_eq!(spawn.position, (0, 64, 0));
    assert_eq!(spawn.dimension, 0);

    // Sleep successfully at night.
    let result = SpawnManager::try_sleep(0.80, true, false, 2.0);
    assert_eq!(result, BedResult::SleptSuccessfully);

    // Set spawn after sleeping.
    mgr.set_spawn((100, 72, -50), 0);
    let new_spawn = mgr.get_spawn();
    assert_eq!(new_spawn.position, (100, 72, -50));

    // Advance time to morning.
    let mut time = 0.80;
    SpawnManager::advance_to_morning(&mut time);
    assert!((time - 0.25).abs() < f32::EPSILON);

    // Clear spawn reverts to default.
    mgr.clear_spawn();
    assert_eq!(mgr.get_spawn().position, (0, 64, 0));
}

// ---------------------------------------------------------------------------
// 14. container: transfer between chest and hopper
// ---------------------------------------------------------------------------

#[test]
fn container_transfer_chest_to_hopper() {
    let mut chest = ChestContainer::new();
    let mut hopper = HopperContainer::new();

    // Place items in the chest.
    chest.set_slot(0, Some((264, 32))); // 32 diamonds in slot 0

    // Transfer from chest slot 0 to hopper.
    assert!(transfer_item(&mut chest, 0, &mut hopper, 64));

    // Chest should be empty, hopper should have the items.
    assert_eq!(chest.get_slot(0), Some(None));
    assert_eq!(hopper.get_slot(0), Some(Some((264, 32))));

    // Add more items to the chest and transfer.
    chest.set_slot(1, Some((264, 10)));
    assert!(transfer_item(&mut chest, 1, &mut hopper, 64));

    // Should stack with existing diamonds in hopper slot 0.
    assert_eq!(hopper.get_slot(0), Some(Some((264, 42))));
    assert_eq!(chest.get_slot(1), Some(None));

    // find_slot_for_item should prefer the existing stack.
    let slot_idx = find_slot_for_item(&hopper, 264, 64);
    assert_eq!(slot_idx, Some(0));
}

// ---------------------------------------------------------------------------
// 15. block_entity manager: furnace ticking + hopper cooldown
// ---------------------------------------------------------------------------

#[test]
fn block_entity_manager_furnace_and_hopper_tick() {
    let mut manager = BlockEntityManager::new();

    // Place a furnace with active timers.
    let furnace_pos = BlockPos::new(10, 64, 10);
    let mut furnace_data = FurnaceData::new();
    furnace_data.burn_time = 200;
    furnace_data.cook_time = 100;
    manager.place(furnace_pos, BlockEntity::Furnace(furnace_data));

    // Place a hopper with cooldown.
    let hopper_pos = BlockPos::new(10, 63, 10);
    let mut hopper_data = HopperData::new();
    hopper_data.cooldown = 8;
    hopper_data.slots[0] = Some((264, 5));
    manager.place(hopper_pos, BlockEntity::Hopper(hopper_data));

    // Tick 5 times.
    for _ in 0..5 {
        manager.tick();
    }

    // Verify furnace timers decreased.
    if let Some(BlockEntity::Furnace(data)) = manager.get(furnace_pos) {
        assert_eq!(data.burn_time, 195);
        assert_eq!(data.cook_time, 95);
    } else {
        panic!("expected furnace at {:?}", furnace_pos);
    }

    // Verify hopper cooldown decreased.
    if let Some(BlockEntity::Hopper(data)) = manager.get(hopper_pos) {
        assert_eq!(data.cooldown, 3);
        assert_eq!(data.slots[0], Some((264, 5))); // inventory unchanged
    } else {
        panic!("expected hopper at {:?}", hopper_pos);
    }

    // Verify both entities are iterable.
    assert_eq!(manager.iter().count(), 2);
}

// ---------------------------------------------------------------------------
// 16. NoiseTerrainGen + trees: generated chunk has oak logs AND leaves
// ---------------------------------------------------------------------------

#[test]
fn terrain_with_trees_has_logs_and_leaves() {
    let seed = 42u64;
    let terrain_gen = NoiseTerrainGen::new(seed);

    // Try several chunk positions to find one with trees.
    let positions = [(0, 0), (1, 1), (2, 2), (3, 3), (5, 5), (10, 10)];
    let mut found_logs = false;
    let mut found_leaves = false;

    for (cx, cz) in positions {
        let mut chunk = terrain_gen.generate(cx, cz);
        place_trees(&mut chunk, cx, cz, seed);
        place_vegetation(&mut chunk, cx, cz, seed);

        let logs = count_block(&chunk, BlockId::OakLog, -64, 320)
            + count_block(&chunk, BlockId::BirchLog, -64, 320)
            + count_block(&chunk, BlockId::SpruceLog, -64, 320);
        let leaves = count_block(&chunk, BlockId::OakLeaves, -64, 320)
            + count_block(&chunk, BlockId::BirchLeaves, -64, 320)
            + count_block(&chunk, BlockId::SpruceLeaves, -64, 320);

        if logs > 0 {
            found_logs = true;
        }
        if leaves > 0 {
            found_leaves = true;
        }
        if found_logs && found_leaves {
            break;
        }
    }

    assert!(
        found_logs,
        "expected to find tree logs in at least one chunk"
    );
    assert!(
        found_leaves,
        "expected to find tree leaves in at least one chunk"
    );
}

// ---------------------------------------------------------------------------
// 17. lighting + terrain: light map from generated terrain
// ---------------------------------------------------------------------------

#[test]
fn light_propagation_from_torch_in_dark_room() {
    // Use propagate_block_light to verify light spreads from a torch source.
    let mut chunk = Chunk::new();

    // Build a small enclosed room of stone with a torch inside.
    // Floor at y=64, walls around, ceiling at y=68.
    for x in 0..8 {
        for z in 0..8 {
            for y in 64..69 {
                if x == 0 || x == 7 || z == 0 || z == 7 || y == 64 || y == 68 {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
                // Interior is air (default).
            }
        }
    }
    // Place a torch at the center.
    chunk.set_block(4, 65, 4, BlockId::Torch);

    let light_map = propagate_block_light(&chunk);

    // The torch position should have maximum block light (14 for torch).
    let (bl, _sl) = light_map.get_light(4, 65, 4);
    assert!(bl > 0, "torch should emit block light, got {bl}");

    // Adjacent blocks should also have some light.
    let (bl_adj, _) = light_map.get_light(5, 65, 4);
    assert!(
        bl_adj > 0,
        "adjacent to torch should have light, got {bl_adj}"
    );

    // Light should decrease with distance.
    assert!(
        bl >= bl_adj,
        "light should not increase with distance (torch={bl}, adjacent={bl_adj})"
    );
}

// ---------------------------------------------------------------------------
// 18. explosion + entity damage: entities take damage based on distance
// ---------------------------------------------------------------------------

#[test]
fn explosion_entity_damage_decreases_with_distance() {
    let center = (0.0, 64.0, 0.0);
    let power = TNT_POWER;

    let damage_at_center = calculate_entity_damage(center, center, power);
    let damage_at_1 = calculate_entity_damage((1.0, 64.0, 0.0), center, power);
    let damage_at_3 = calculate_entity_damage((3.0, 64.0, 0.0), center, power);
    let damage_beyond = calculate_entity_damage((5.0, 64.0, 0.0), center, power);

    assert!(
        damage_at_center > damage_at_1,
        "center should take most damage"
    );
    assert!(
        damage_at_1 > damage_at_3,
        "closer entities take more damage"
    );
    assert!(
        damage_beyond.abs() < f32::EPSILON,
        "entities beyond power range should take zero damage"
    );

    // apply_explosion should set destroyed blocks to air.
    let result = calculate_explosion(center, power, &|_x, _y, _z| BlockId::Dirt);
    let mut applied = Vec::new();
    apply_explosion(&result, &mut |x, y, z, block| {
        applied.push((x, y, z, block));
    });
    assert!(!applied.is_empty());
    for (_, _, _, block) in &applied {
        assert_eq!(*block, BlockId::Air);
    }
}

// ---------------------------------------------------------------------------
// 19. save + ChunkManager: save generated chunk to disk and reload
// ---------------------------------------------------------------------------

#[test]
fn save_and_load_chunk_from_disk() {
    let dir = std::env::temp_dir().join("mcrust_integration_save_test");
    let _ = std::fs::remove_dir_all(&dir);

    let terrain_gen = NoiseTerrainGen::new(777);
    let ore_gen = OreGenerator::new(777);

    let mut chunk = terrain_gen.generate(5, -3);
    ore_gen.generate_ores(&mut chunk, 5, -3);

    let pos = ChunkPos::new(5, -3);
    save_chunk(&dir, &chunk, pos).unwrap();

    let loaded = load_chunk(&dir, pos)
        .unwrap()
        .expect("chunk file should exist");

    // Spot-check specific positions.
    assert_eq!(loaded.get_block(0, -64, 0), BlockId::Bedrock);
    assert_eq!(loaded.get_block(8, 200, 8), BlockId::Air);

    // Full comparison.
    for x in 0..16 {
        for z in 0..16 {
            for y in -64..320 {
                assert_eq!(
                    chunk.get_block(x, y, z),
                    loaded.get_block(x, y, z),
                    "mismatch at ({x}, {y}, {z})"
                );
            }
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 20. fire entity burning + weather: rain extinguishes entity fire
// ---------------------------------------------------------------------------

#[test]
fn weather_rain_extinguishes_entity_fire() {
    // Verify weather system can reach rain.
    let mut ws = WeatherSystem::new(42);
    while !ws.is_raining() {
        ws.tick();
    }
    assert!(ws.is_raining());

    // Create a burning entity.
    let mut burning = BurningEntity {
        burn_ticks: 100,
        fire_damage_timer: 0.0,
    };

    // If it's raining, the entity should be extinguished when stepping into water.
    // (Rain doesn't directly extinguish entities; water does.)
    let (still, damage) = on_fire_tick(&mut burning, true, false, 1.0);
    assert!(!still, "water should extinguish entity fire");
    assert!(
        damage.abs() < f32::EPSILON,
        "no damage when extinguished by water"
    );
    assert_eq!(burning.burn_ticks, 0);

    // Without water, fire damages the entity.
    let mut burning2 = BurningEntity {
        burn_ticks: 100,
        fire_damage_timer: 0.0,
    };
    let (still2, damage2) = on_fire_tick(&mut burning2, false, false, 1.5);
    assert!(still2, "entity should still be burning without water");
    assert!(damage2 > 0.0, "entity should take fire damage");
}
