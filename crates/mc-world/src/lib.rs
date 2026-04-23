//! Voxel world: chunk storage, terrain generation, lighting, and world simulation.
//!
//! Handles [`Chunk`] management, cave/ore/structure generation, redstone circuits,
//! weather, farming, fluids, fire, explosions, pistons, and save/load persistence.

pub mod async_chunks;
pub mod beacon;
pub mod biome_terrain;
pub mod block_entity;
pub mod block_update;
pub mod bucket;
pub mod campfire;
pub mod caves;
pub mod chest_logic;
pub mod chunk;
pub mod chunk_manager;
pub mod climbable;
pub mod composter;
pub mod container;
pub mod end;
pub mod explosion;
pub mod farming;
pub mod fire;
pub mod fluid;
pub mod gravity_block;
pub mod hopper_logic;
pub mod lectern;
pub mod lighting;
pub mod map_data;
pub mod nether;
pub mod noise_terrain;
pub mod noteblock;
pub mod ores;
pub mod piston;
pub mod portal_link;
pub mod rails;
pub mod redstone;
pub mod redstone_components;
pub mod save;
pub mod sensors;
pub mod sign;
pub mod spawn;
pub mod structure_types;
pub mod structures;
pub mod terrain;
pub mod tick_scheduler;
pub mod tnt;
pub mod trees;
pub mod weather;
pub mod world_border;

pub use async_chunks::{
    AsyncChunkLoader, ChunkGenConfig, ChunkLoadRequest, generate_chunk_standalone,
};
pub use beacon::{
    BeaconEffect, BeaconState, available_effects, beacon_range, is_beacon_base_block, is_in_range,
    scan_pyramid,
};
pub use biome_terrain::BiomeTerrainGen;
pub use block_entity::{
    BlockEntity, BlockEntityManager, BlockEntityType, BrewingStandData, ChestData, FurnaceData,
    HopperData,
};
pub use block_update::BlockUpdateQueue;
pub use bucket::{
    BucketContents, BucketResult, milk_effects, use_bucket_on_block, use_bucket_on_entity,
};
pub use campfire::{CampfireState, campfire_damage, cooked_item, smoke_height};
pub use caves::CaveCarver;
pub use chest_logic::{
    ChestOpenState, close_chest, detect_double_chest, is_chest_blocked, open_chest, tick_animation,
    trapped_chest_signal,
};
pub use chunk::{Chunk, Section};
pub use chunk_manager::ChunkManager;
pub use climbable::{
    CLIMBING_SPEED, SCAFFOLDING_MAX_DISTANCE, can_place_scaffolding, is_climbable,
    scaffolding_distance, should_scaffolding_fall,
};
pub use composter::{
    BONE_MEAL_ID, CompostResult, compost_chance, harvest as composter_harvest, try_compost,
};
pub use container::{
    ChestContainer, Container, DispenserContainer, DoubleChestContainer, HopperContainer,
    SlotContent, add_to_container, find_slot_for_item, transfer_item,
};
pub use end::EndTerrainGen;
pub use explosion::{
    CHARGED_CREEPER_POWER, CREEPER_POWER, ExplosionResult, TNT_POWER, apply_explosion,
    block_resistance, calculate_entity_damage, calculate_explosion,
};
pub use farming::{CropState, CropType, can_plant_on, harvest, is_hydrated, tick_crop};
pub use fire::{
    BurningEntity, FireAction, FireState, burn_chance, flammability, is_flammable, on_fire_tick,
    tick_fire,
};
pub use fluid::{FluidWorld, process_water_update};
pub use gravity_block::{
    FallingBlock, FallingBlockAction, check_should_fall, is_gravity_block, on_block_update,
    tick_falling,
};
pub use hopper_logic::{
    HOPPER_COOLDOWN, HopperDirection, HopperTransfer, hopper_pull, hopper_push,
    hopper_should_transfer,
};
pub use lectern::{BookData, LecternState, lectern_redstone};
pub use lighting::{LightMap, max_light, propagate_block_light, propagate_sky_light};
pub use map_data::{
    MAP_SIZE, MapData, block_to_map_color, generate_map, map_color, map_color_to_rgb,
};
pub use nether::{DimensionId, NetherTerrainGen};
pub use noise_terrain::NoiseTerrainGen;
pub use noteblock::{
    Instrument, JukeboxState, NoteBlockState, eject_disc, insert_disc, instrument_from_block,
    is_playing, play_note, tune,
};
pub use ores::OreGenerator;
pub use piston::{
    PUSH_LIMIT, PistonState, can_push_block, extend_piston, push_line, retract, retract_piston,
};
pub use portal_link::{
    PortalLink, create_portal_frame, find_nearest_portal, nether_coordinate_scale,
    overworld_coordinate_scale, search_radius,
};
pub use rails::{
    RailNetwork, RailShape, RailType, detector_rail_signal, determine_rail_shape, is_rail,
    powered_rail_effect, rail_type_from_id,
};
pub use redstone::{RedstoneCircuit, RedstoneWorld, is_power_source, propagate_redstone};
pub use redstone_components::{
    PistonAction, hopper_tick_rate, lamp_state, noteblock_pitch, piston_can_push, piston_push_limit,
};
pub use save::{
    ChunkSave, SaveError, SectionSave, WorldSave, chunk_to_save, load_chunk, load_world,
    save_chunk, save_to_chunk, save_world,
};
pub use sensors::{
    DaylightDetector, ObserverState, daylight_signal, observer_check, observer_tick,
};
pub use sign::{SignColor, SignData, format_sign_text};
pub use spawn::{BedResult, SpawnManager, SpawnPoint};
pub use structures::StructureGenerator;
pub use terrain::FlatWorldGen;
pub use tick_scheduler::{ScheduledEvent, ScheduledTick, TickScheduler, TickType};
pub use tnt::{TntAction, TntEntity, activate_tnt, chain_activation, tick_tnt};
pub use trees::{birch_tree, oak_tree, place_trees, place_vegetation, spruce_tree};
pub use weather::{WeatherState, WeatherSystem};
pub use world_border::WorldBorder;
