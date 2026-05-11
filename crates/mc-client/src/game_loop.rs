use std::time::Duration;

use glam::Vec3;
use winit::keyboard::KeyCode;

use mc_core::biome::BiomeId;
use mc_core::block::BlockId;
use mc_core::pos::{BlockPos, ChunkPos};
use mc_physics::collision;
use mc_physics::raycast;
use mc_render::Camera;
use mc_render::Renderer;
use mc_render::crosshair::crosshair_ndc;
use mc_render::fog::fog_for_dimension;
use mc_render::frustum::{self, Frustum};
use mc_render::mesh::{ChunkMesh, NeighborChunks};
use mc_render::sky::DayNightCycle;
use mc_ui::debug_screen::{DebugInfo, direction_name, format_debug_lines, time_to_hhmm};
use mc_ui::hotbar::{HotbarData, hotbar_layout};
use mc_world::ChunkManager;
use mc_world::biome_blend::biome_base_colors;

use crate::player::{
    GRAVITY, JUMP_VELOCITY, REACH_DISTANCE, SNEAK_SPEED, SPRINT_SPEED, WALK_SPEED,
};
use crate::{App, GameState, TICK_DURATION, VOID_DEATH_Y};

/// Frame counter for throttled logging (fog, debug screen).
static FRAME_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Mesh queue processing
// ---------------------------------------------------------------------------

/// Processes up to 16 pending chunk mesh builds per frame.
///
/// Takes the renderer, world, mesh queue, and mesh storage and builds
/// GPU meshes for queued chunk positions.
pub fn process_mesh_queue(
    renderer: &Option<Renderer>,
    world: &ChunkManager,
    mesh_queue: &mut Vec<ChunkPos>,
    chunk_meshes: &mut Vec<ChunkMesh>,
) {
    const MAX_MESHES_PER_FRAME: usize = 16;
    if mesh_queue.is_empty() {
        return;
    }
    if let Some(renderer) = renderer {
        let batch_size = mesh_queue.len().min(MAX_MESHES_PER_FRAME);
        let batch: Vec<ChunkPos> = mesh_queue.drain(..batch_size).collect();

        for pos in &batch {
            if let Some(chunk) = world.get_chunk(*pos) {
                let neighbors = NeighborChunks {
                    east: world.get_chunk(ChunkPos::new(pos.x + 1, pos.z)),
                    west: world.get_chunk(ChunkPos::new(pos.x - 1, pos.z)),
                    south: world.get_chunk(ChunkPos::new(pos.x, pos.z + 1)),
                    north: world.get_chunk(ChunkPos::new(pos.x, pos.z - 1)),
                };
                if let Some(mesh) = ChunkMesh::build(renderer.device(), chunk, *pos, &neighbors) {
                    chunk_meshes.push(mesh);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scene rendering
// ---------------------------------------------------------------------------

/// Renders the scene by performing frustum culling, distance sorting, and
/// issuing draw calls through the renderer.
pub fn render_scene(
    renderer: &Option<Renderer>,
    camera: &Camera,
    sky: &DayNightCycle,
    chunk_meshes: &[ChunkMesh],
    screen_w: f32,
    screen_h: f32,
) {
    // Compute fog settings for the current dimension (overworld, render distance 8)
    let fog = fog_for_dimension(0, 8);
    let frame = FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if frame % 60 == 0 {
        log::debug!("Fog start={:.1} end={:.1}", fog.start, fog.end);
    }

    let vp = camera.view_projection_matrix();
    let frustum = Frustum::from_view_projection(vp);

    // Sort meshes by distance to camera for better rendering and
    // keep only frustum-visible chunks. Cap at 128 draw calls to
    // prevent Metal driver overload on first frame.
    let cam_cx = (camera.position.x / 16.0).floor() as i32;
    let cam_cz = (camera.position.z / 16.0).floor() as i32;

    let mut visible_indices: Vec<usize> = chunk_meshes
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            let (min, max) = frustum::chunk_aabb(m.chunk_pos.x, m.chunk_pos.z);
            frustum.contains_aabb(min, max)
        })
        .map(|(i, _)| i)
        .collect();

    // Sort by distance (nearest first)
    visible_indices.sort_by_key(|&i| {
        let m = &chunk_meshes[i];
        let dx = m.chunk_pos.x - cam_cx;
        let dz = m.chunk_pos.z - cam_cz;
        dx * dx + dz * dz
    });

    // Cap draw calls
    visible_indices.truncate(128);

    // Build a temporary Vec of references for rendering
    let visible_meshes: Vec<&ChunkMesh> =
        visible_indices.iter().map(|&i| &chunk_meshes[i]).collect();

    if let Some(renderer) = renderer {
        match renderer.render_frame_refs(camera, sky, &visible_meshes) {
            Ok(()) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                log::warn!("Surface lost/outdated -- will reconfigure on next resize");
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Out of GPU memory!");
            }
            Err(e) => {
                log::warn!("Render error: {e:?}");
            }
        }
    }

    // Compute crosshair NDC coordinates (will be used for GPU drawing later)
    let _crosshair = crosshair_ndc(screen_w, screen_h);
}

// ---------------------------------------------------------------------------
// Playing-state tick + render sequence
// ---------------------------------------------------------------------------

/// Runs one full Playing-state frame: physics ticks, chunk loading,
/// bridge module updates, HUD sync, rendering, and death checks.
///
/// Returns the new `GameState` if a state transition occurred (e.g. death),
/// or `None` to remain in `Playing`.
pub fn playing_frame(app: &mut App, frame_time: Duration) -> Option<GameState> {
    // Grab cursor on first Playing frame (deferred from state transition
    // to avoid invalidating Metal surface on the transition frame).
    if !app.cursor_grabbed {
        app.grab_cursor();
        // Use max() so a concurrent resize event cannot shrink the window
        app.skip_frames = app.skip_frames.max(3);
        if let Some(window) = app.window.as_ref() {
            window.request_redraw();
        }
        return None;
    }

    // -- Physics ticks --
    app.tick_accumulator += frame_time;
    let tick_dt = TICK_DURATION.as_secs_f32();
    let max_ticks = 4;
    let mut tick_count = 0;
    while app.tick_accumulator >= TICK_DURATION && tick_count < max_ticks {
        app.tick_accumulator -= TICK_DURATION;
        tick(app, tick_dt);
        tick_count += 1;
    }
    if app.tick_accumulator > TICK_DURATION {
        app.tick_accumulator = Duration::ZERO;
    }

    // Sync camera with player
    app.camera.position = app.player.eye_position();
    app.camera.yaw = app.player.yaw;
    app.camera.pitch = app.player.pitch;

    // Update chunk loading
    let player_chunk = ChunkPos::from_block(
        app.player.position.x.floor() as i32,
        app.player.position.z.floor() as i32,
    );
    app.world.update(player_chunk);

    // Rebuild dirty chunk meshes
    let dirty = app.world.take_dirty();
    if !dirty.is_empty() {
        app.chunk_meshes.retain(|m| !dirty.contains(&m.chunk_pos));
        app.mesh_queue.extend(dirty);
    }
    process_mesh_queue(
        &app.renderer,
        &app.world,
        &mut app.mesh_queue,
        &mut app.chunk_meshes,
    );

    // Advance day/night cycle
    let frame_dt = frame_time.as_secs_f32();
    app.sky.advance(frame_dt);

    // --- Bridge module ticks ---
    tick_bridge_modules(app, frame_dt);

    // --- Hotbar layout ---
    // Build HotbarData from inventory state and compute layout for future rendering.
    let mut hotbar_data = HotbarData::new();
    hotbar_data.select(app.hud.selected_slot);
    // Populate selected slot item if present
    if let Some((item_id, count)) = app.inventory.selected_item() {
        hotbar_data.set_slot(
            app.hud.selected_slot,
            mc_ui::hotbar::HotbarSlot {
                item_id,
                count,
                durability: None,
            },
        );
    }
    let _hotbar_layout = hotbar_layout(1280.0, 720.0);

    // --- Biome colors ---
    // Get biome base colors for a placeholder biome (Plains) for future fog tinting.
    let biome_colors = biome_base_colors(BiomeId::Plains);
    let _grass_tinted_fog = biome_colors.grass;

    // Update HUD from bridge modules
    sync_hud(app);

    // Render
    render_scene(&app.renderer, &app.camera, &app.sky, &app.chunk_meshes, 1280.0, 720.0);

    // --- Debug screen ---
    // Build and log debug info every 60 frames when F3 overlay is active.
    if app.hud.show_debug {
        let frame = FRAME_COUNTER.load(std::sync::atomic::Ordering::Relaxed);
        if frame % 60 == 0 {
            let player_chunk = ChunkPos::from_block(
                app.player.position.x.floor() as i32,
                app.player.position.z.floor() as i32,
            );
            let _dir_name = direction_name(app.player.yaw.to_degrees());
            let _time_str = time_to_hhmm(app.sky.time_of_day);
            let debug_info = DebugInfo {
                fps: 60.0,
                x: app.player.position.x,
                y: app.player.position.y,
                z: app.player.position.z,
                chunk_x: player_chunk.x,
                chunk_z: player_chunk.z,
                facing_yaw: app.player.yaw.to_degrees(),
                facing_pitch: app.player.pitch.to_degrees(),
                biome: "plains".to_string(),
                block_light: 15,
                sky_light: 15,
                loaded_chunks: app.world.loaded_chunks().count(),
                entity_count: 0,
                dimension: "overworld".to_string(),
                day: 1,
                time_of_day: app.sky.time_of_day,
                seed: 42,
            };
            let lines = format_debug_lines(&debug_info);
            for line in lines.iter().take(3) {
                log::debug!("F3: {}", line);
            }
        }
    }

    // Check for void death or survival death
    if app.player.position.y < VOID_DEATH_Y {
        app.survival.take_damage(1000.0);
    }
    if app.survival.is_dead() {
        log::info!("Player died!");
        app.release_cursor();
        return Some(GameState::Dead { respawn_timer: 0.0 });
    }

    None
}

// ---------------------------------------------------------------------------
// Bridge module ticks (helpers)
// ---------------------------------------------------------------------------

/// Ticks all bridge modules for one frame.
fn tick_bridge_modules(app: &mut App, frame_dt: f32) {
    // Mob spawning + AI
    app.mob_world
        .tick(app.player.position, app.sky.time_of_day, frame_dt);

    // Weather + world tick scheduler
    app.world_state.tick(frame_dt);

    // Survival (hunger/health) tick
    let is_sprinting = app.keys_held.contains(&KeyCode::ControlLeft);
    app.survival.tick(frame_dt, is_sprinting, 0.0, false);

    // Progression (distance tracking)
    app.progression.on_distance_walked(0.0);

    // Sound system music tick
    let _music_action = app.sounds.tick_music(frame_dt, 0);

    // Auto-save check
    if app.save.tick(frame_dt) {
        let p = app.player.position;
        app.save.save_game(
            (p.x, p.y, p.z),
            app.player.yaw,
            app.player.pitch,
            app.sky.time_of_day,
            42,
        );
        log::info!("Auto-saved");
    }

    // Drain sound events (placeholder for audio playback)
    let _events = app.sounds.drain_sound_events();
}

/// Syncs HUD state from bridge modules.
fn sync_hud(app: &mut App) {
    app.hud.health = app.survival.hud_health();
    app.hud.hunger = app.survival.hud_hunger();
    app.hud.armor = app.survival.hud_armor();
    app.hud.xp_level = app.progression.hud_level();
    app.hud.xp_progress = app.progression.hud_xp_progress();
    app.hud.player_pos = (
        app.player.position.x,
        app.player.position.y,
        app.player.position.z,
    );
}

// ---------------------------------------------------------------------------
// Block interaction
// ---------------------------------------------------------------------------

/// Attempts to break the block the player is looking at.
pub fn break_block(app: &mut App) {
    let hit = raycast::raycast(
        app.player.eye_position(),
        app.player.look_direction(),
        REACH_DISTANCE,
        &|bx, by, bz| app.world.is_block_solid(bx, by, bz),
    );
    if let Some(hit) = hit {
        let block = app.world.get_block(hit.block_pos);
        app.world.set_block(hit.block_pos, BlockId::Air);
        let cp = hit.block_pos.chunk_pos();
        app.chunk_meshes.retain(|m| m.chunk_pos != cp);
        app.mesh_queue.push(cp);

        // Bridge: collect drops, add XP, play sound, track stats
        app.inventory.on_block_broken(block as u16);
        app.progression.on_block_mined(block as u16);
        app.sounds.on_block_break(hit.point);
    }
}

/// Attempts to place a block on the face the player is looking at.
pub fn place_block(app: &mut App) {
    let hit = raycast::raycast(
        app.player.eye_position(),
        app.player.look_direction(),
        REACH_DISTANCE,
        &|bx, by, bz| app.world.is_block_solid(bx, by, bz),
    );
    if let Some(hit) = hit {
        let normal = hit.face.normal();
        let place_pos = BlockPos::new(
            hit.block_pos.x + normal.x,
            hit.block_pos.y + normal.y,
            hit.block_pos.z + normal.z,
        );
        // Don't place inside the player
        let player_block = BlockPos::new(
            app.player.position.x.floor() as i32,
            app.player.position.y.floor() as i32,
            app.player.position.z.floor() as i32,
        );
        let player_head = BlockPos::new(player_block.x, player_block.y + 1, player_block.z);
        if place_pos != player_block && place_pos != player_head {
            // Use block from inventory if available, otherwise cobblestone
            let block_to_place = app
                .inventory
                .on_block_place()
                .and_then(|id| mc_core::block::BlockId::from_raw(id))
                .unwrap_or(BlockId::Cobblestone);
            app.world.set_block(place_pos, block_to_place);
            let cp = place_pos.chunk_pos();
            app.chunk_meshes.retain(|m| m.chunk_pos != cp);
            app.mesh_queue.push(cp);
            app.sounds.on_block_place(hit.point);
        }
    }
}

// ---------------------------------------------------------------------------
// Physics tick
// ---------------------------------------------------------------------------

/// Runs one physics tick: processes movement input, applies gravity,
/// handles jumping, and resolves collisions.
pub fn tick(app: &mut App, dt: f32) {
    let mut wish_dir = Vec3::ZERO;
    if app.keys_held.contains(&KeyCode::KeyW) {
        wish_dir += app.player.forward_xz();
    }
    if app.keys_held.contains(&KeyCode::KeyS) {
        wish_dir -= app.player.forward_xz();
    }
    if app.keys_held.contains(&KeyCode::KeyD) {
        wish_dir += app.player.right_xz();
    }
    if app.keys_held.contains(&KeyCode::KeyA) {
        wish_dir -= app.player.right_xz();
    }
    wish_dir = wish_dir.normalize_or_zero();

    let speed = if app.keys_held.contains(&KeyCode::ShiftLeft)
        || app.keys_held.contains(&KeyCode::ShiftRight)
    {
        SNEAK_SPEED
    } else if app.keys_held.contains(&KeyCode::ControlLeft) {
        SPRINT_SPEED
    } else {
        WALK_SPEED
    };

    app.player.velocity.x = wish_dir.x * speed;
    app.player.velocity.z = wish_dir.z * speed;
    app.player.velocity.y += GRAVITY * dt;

    if app.player.on_ground && app.keys_held.contains(&KeyCode::Space) {
        app.player.velocity.y = JUMP_VELOCITY;
        app.player.on_ground = false;
    }

    let frame_vel = app.player.velocity * dt;
    let resolved = collision::move_and_slide(app.player.position, frame_vel, &|bx, by, bz| {
        app.world.is_block_solid(bx, by, bz)
    });

    app.player.on_ground = frame_vel.y < 0.0 && resolved.y.abs() < 1e-6;
    app.player.position += resolved;

    if app.player.on_ground {
        app.player.velocity.y = 0.0;
    }
}
