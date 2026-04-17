use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::Vec3;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

use mc_core::block::BlockId;
use mc_core::pos::{BlockPos, ChunkPos};
use mc_physics::collision;
use mc_physics::raycast;
use mc_render::frustum::{self, Frustum};
use mc_render::mesh::{ChunkMesh, NeighborChunks};
use mc_render::sky::DayNightCycle;
use mc_render::{Camera, Renderer};
use mc_ui::hud::HudState;
use mc_world::ChunkManager;
use mc_world::nether::DimensionId;

// ---------------------------------------------------------------------------
// Physics constants
// ---------------------------------------------------------------------------

const GRAVITY: f32 = -32.0;
const JUMP_VELOCITY: f32 = 8.5;
const WALK_SPEED: f32 = 4.3;
const SPRINT_SPEED: f32 = 5.6;
const SNEAK_SPEED: f32 = 1.3;
const MOUSE_SENSITIVITY: f32 = 0.003;
const TICK_DURATION: Duration = Duration::from_millis(50); // 20 tps
const RENDER_DISTANCE: i32 = 8;
const REACH_DISTANCE: f32 = 5.0;

// ---------------------------------------------------------------------------
// Game state machine
// ---------------------------------------------------------------------------

#[allow(dead_code)]
enum GameState {
    MainMenu,
    Loading {
        chunks_loaded: usize,
        chunks_needed: usize,
    },
    Playing,
    Paused,
    Dead {
        respawn_timer: f32,
    },
}

/// Number of chunks needed before transitioning from Loading to Playing.
/// For an initial load distance of 4 we need (2*4+1)^2 = 81 chunks.
const INITIAL_LOAD_DISTANCE: i32 = 2;
const CHUNKS_NEEDED_FOR_PLAY: usize =
    ((2 * INITIAL_LOAD_DISTANCE + 1) * (2 * INITIAL_LOAD_DISTANCE + 1)) as usize;

/// Void death threshold — player falls below this Y coordinate.
const VOID_DEATH_Y: f32 = -100.0;

/// Spawn position.
const SPAWN_POSITION: Vec3 = Vec3::new(0.0, 100.0, 0.0);

// ---------------------------------------------------------------------------
// PlayerState
// ---------------------------------------------------------------------------

struct PlayerState {
    position: Vec3,
    velocity: Vec3,
    on_ground: bool,
    yaw: f32,
    pitch: f32,
}

impl PlayerState {
    fn new(spawn: Vec3) -> Self {
        Self {
            position: spawn,
            velocity: Vec3::ZERO,
            on_ground: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    fn eye_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, collision::PLAYER_EYE_HEIGHT, 0.0)
    }

    fn forward_xz(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, self.yaw.cos()).normalize_or_zero()
    }

    fn right_xz(&self) -> Vec3 {
        let fwd = self.forward_xz();
        Vec3::new(fwd.z, 0.0, -fwd.x)
    }

    /// Full 3D look direction (includes pitch).
    fn look_direction(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
        .normalize_or_zero()
    }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera: Camera,
    sky: DayNightCycle,
    player: PlayerState,
    world: ChunkManager,
    chunk_meshes: Vec<ChunkMesh>,
    mesh_queue: Vec<ChunkPos>,
    keys_held: HashSet<KeyCode>,
    cursor_grabbed: bool,
    last_tick: Instant,
    tick_accumulator: Duration,
    state: GameState,
    hud: HudState,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            camera: Camera::new(SPAWN_POSITION, 16.0 / 9.0),
            sky: DayNightCycle::new(0.25),
            player: PlayerState::new(SPAWN_POSITION),
            world: ChunkManager::new(RENDER_DISTANCE),
            chunk_meshes: Vec::new(),
            mesh_queue: Vec::new(),
            keys_held: HashSet::new(),
            cursor_grabbed: false,
            last_tick: Instant::now(),
            tick_accumulator: Duration::ZERO,
            state: GameState::Loading {
                chunks_loaded: 0,
                chunks_needed: CHUNKS_NEEDED_FOR_PLAY,
            },
            hud: HudState::default(),
        }
    }

    fn grab_cursor(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let _ = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            window.set_cursor_visible(false);
            self.cursor_grabbed = true;
        }
    }

    fn release_cursor(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.cursor_grabbed = false;
        }
    }

    // -- Block interaction --------------------------------------------------

    fn break_block(&mut self) {
        let hit = raycast::raycast(
            self.player.eye_position(),
            self.player.look_direction(),
            REACH_DISTANCE,
            &|bx, by, bz| self.world.is_block_solid(bx, by, bz),
        );
        if let Some(hit) = hit {
            self.world.set_block(hit.block_pos, BlockId::Air);
            // Rebuild mesh for affected chunk(s)
            let cp = hit.block_pos.chunk_pos();
            self.chunk_meshes.retain(|m| m.chunk_pos != cp);
            self.mesh_queue.push(cp);
        }
    }

    fn place_block(&mut self) {
        let hit = raycast::raycast(
            self.player.eye_position(),
            self.player.look_direction(),
            REACH_DISTANCE,
            &|bx, by, bz| self.world.is_block_solid(bx, by, bz),
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
                self.player.position.x.floor() as i32,
                self.player.position.y.floor() as i32,
                self.player.position.z.floor() as i32,
            );
            let player_head = BlockPos::new(player_block.x, player_block.y + 1, player_block.z);
            if place_pos != player_block && place_pos != player_head {
                self.world.set_block(place_pos, BlockId::Cobblestone);
                let cp = place_pos.chunk_pos();
                self.chunk_meshes.retain(|m| m.chunk_pos != cp);
                self.mesh_queue.push(cp);
            }
        }
    }

    // -- Physics tick -------------------------------------------------------

    fn tick(&mut self, dt: f32) {
        let mut wish_dir = Vec3::ZERO;
        if self.keys_held.contains(&KeyCode::KeyW) {
            wish_dir += self.player.forward_xz();
        }
        if self.keys_held.contains(&KeyCode::KeyS) {
            wish_dir -= self.player.forward_xz();
        }
        if self.keys_held.contains(&KeyCode::KeyD) {
            wish_dir += self.player.right_xz();
        }
        if self.keys_held.contains(&KeyCode::KeyA) {
            wish_dir -= self.player.right_xz();
        }
        wish_dir = wish_dir.normalize_or_zero();

        let speed = if self.keys_held.contains(&KeyCode::ShiftLeft)
            || self.keys_held.contains(&KeyCode::ShiftRight)
        {
            SNEAK_SPEED
        } else if self.keys_held.contains(&KeyCode::ControlLeft) {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        self.player.velocity.x = wish_dir.x * speed;
        self.player.velocity.z = wish_dir.z * speed;
        self.player.velocity.y += GRAVITY * dt;

        if self.player.on_ground && self.keys_held.contains(&KeyCode::Space) {
            self.player.velocity.y = JUMP_VELOCITY;
            self.player.on_ground = false;
        }

        let frame_vel = self.player.velocity * dt;
        let resolved =
            collision::move_and_slide(self.player.position, frame_vel, &|bx, by, bz| {
                self.world.is_block_solid(bx, by, bz)
            });

        self.player.on_ground = frame_vel.y < 0.0 && resolved.y.abs() < 1e-6;
        self.player.position += resolved;

        if self.player.on_ground {
            self.player.velocity.y = 0.0;
        }
    }

    // -- Mesh processing (extracted for reuse across states) ----------------

    fn process_mesh_queue(&mut self) {
        const MAX_MESHES_PER_FRAME: usize = 16;
        if self.mesh_queue.is_empty() {
            return;
        }
        if let Some(renderer) = &self.renderer {
            let batch_size = self.mesh_queue.len().min(MAX_MESHES_PER_FRAME);
            let batch: Vec<ChunkPos> = self.mesh_queue.drain(..batch_size).collect();

            for pos in &batch {
                if let Some(chunk) = self.world.get_chunk(*pos) {
                    let neighbors = NeighborChunks {
                        east: self.world.get_chunk(ChunkPos::new(pos.x + 1, pos.z)),
                        west: self.world.get_chunk(ChunkPos::new(pos.x - 1, pos.z)),
                        south: self.world.get_chunk(ChunkPos::new(pos.x, pos.z + 1)),
                        north: self.world.get_chunk(ChunkPos::new(pos.x, pos.z - 1)),
                    };
                    if let Some(mesh) =
                        ChunkMesh::build(renderer.device(), chunk, *pos, &neighbors)
                    {
                        self.chunk_meshes.push(mesh);
                    }
                }
            }
        }
    }

    // -- Render scene (extracted for reuse across states) -------------------

    fn render_scene(&mut self) {
        let vp = self.camera.view_projection_matrix();
        let frustum = Frustum::from_view_projection(vp);

        // Sort meshes by distance to camera for better rendering and
        // keep only frustum-visible chunks. Cap at 128 draw calls to
        // prevent Metal driver overload on first frame.
        let cam_cx = (self.camera.position.x / 16.0).floor() as i32;
        let cam_cz = (self.camera.position.z / 16.0).floor() as i32;

        let mut visible_indices: Vec<usize> = self
            .chunk_meshes
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
            let m = &self.chunk_meshes[i];
            let dx = m.chunk_pos.x - cam_cx;
            let dz = m.chunk_pos.z - cam_cz;
            dx * dx + dz * dz
        });

        // Cap draw calls
        visible_indices.truncate(128);

        // Build a temporary Vec of references for rendering
        let visible_meshes: Vec<&ChunkMesh> = visible_indices
            .iter()
            .map(|&i| &self.chunk_meshes[i])
            .collect();

        if let Some(renderer) = &self.renderer {
            match renderer.render_frame_refs(&self.camera, &self.sky, &visible_meshes) {
                Ok(()) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    log::warn!("Surface lost/outdated — will reconfigure on next resize");
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    log::error!("Out of GPU memory!");
                }
                Err(e) => {
                    log::warn!("Render error: {e:?}");
                }
            }
        }
    }

    // -- Frame update (state-dispatched) ------------------------------------

    fn update(&mut self) {
        let now = Instant::now();
        let frame_time = now - self.last_tick;
        self.last_tick = now;

        match self.state {
            GameState::MainMenu => {
                log::info!("Press any key to start");
                self.render_scene();
            }

            GameState::Loading {
                chunks_needed, ..
            } => {
                // Update chunk loading around the player
                let player_chunk = ChunkPos::from_block(
                    self.player.position.x.floor() as i32,
                    self.player.position.z.floor() as i32,
                );
                self.world.update(player_chunk);

                // Process dirty chunks into the mesh queue
                let dirty = self.world.take_dirty();
                if !dirty.is_empty() {
                    self.chunk_meshes.retain(|m| !dirty.contains(&m.chunk_pos));
                    self.mesh_queue.extend(dirty);
                }
                self.process_mesh_queue();

                // Count loaded chunks
                let loaded = self.world.loaded_chunks().count();

                log::info!(
                    "Loading world: {loaded}/{chunks_needed} chunks ({:.0}%)",
                    (loaded as f32 / chunks_needed as f32 * 100.0).min(100.0)
                );

                // Update chunks_loaded in state
                self.state = GameState::Loading {
                    chunks_loaded: loaded,
                    chunks_needed,
                };

                // Sync camera for the loading-screen render
                self.camera.position = self.player.eye_position();
                self.camera.yaw = self.player.yaw;
                self.camera.pitch = self.player.pitch;

                self.render_scene();

                // Transition to Playing once enough chunks are loaded
                if loaded >= chunks_needed {
                    log::info!("World loaded! Entering Playing state.");
                    self.state = GameState::Playing;
                    self.grab_cursor();
                }
            }

            GameState::Playing => {
                // Physics ticks
                self.tick_accumulator += frame_time;
                let tick_dt = TICK_DURATION.as_secs_f32();
                let max_ticks = 4;
                let mut tick_count = 0;
                while self.tick_accumulator >= TICK_DURATION && tick_count < max_ticks {
                    self.tick_accumulator -= TICK_DURATION;
                    self.tick(tick_dt);
                    tick_count += 1;
                }
                if self.tick_accumulator > TICK_DURATION {
                    self.tick_accumulator = Duration::ZERO;
                }

                // Sync camera with player
                self.camera.position = self.player.eye_position();
                self.camera.yaw = self.player.yaw;
                self.camera.pitch = self.player.pitch;

                // Update chunk loading
                let player_chunk = ChunkPos::from_block(
                    self.player.position.x.floor() as i32,
                    self.player.position.z.floor() as i32,
                );
                self.world.update(player_chunk);

                // Rebuild dirty chunk meshes
                let dirty = self.world.take_dirty();
                if !dirty.is_empty() {
                    self.chunk_meshes.retain(|m| !dirty.contains(&m.chunk_pos));
                    self.mesh_queue.extend(dirty);
                }
                self.process_mesh_queue();

                // Advance day/night cycle
                let frame_dt = frame_time.as_secs_f32();
                self.sky.advance(frame_dt);

                // Update HUD from player data
                self.hud.health = 20.0;
                self.hud.hunger = 20;
                self.hud.player_pos = (
                    self.player.position.x,
                    self.player.position.y,
                    self.player.position.z,
                );

                // Render
                self.render_scene();

                // Check for void death
                if self.player.position.y < VOID_DEATH_Y {
                    log::info!("Player fell into the void!");
                    self.state = GameState::Dead {
                        respawn_timer: 0.0,
                    };
                    self.release_cursor();
                }
            }

            GameState::Paused => {
                // Don't tick physics, don't advance sky. Just sync camera and render.
                self.camera.position = self.player.eye_position();
                self.camera.yaw = self.player.yaw;
                self.camera.pitch = self.player.pitch;

                self.render_scene();
            }

            GameState::Dead { respawn_timer } => {
                let frame_dt = frame_time.as_secs_f32();
                let new_timer = respawn_timer + frame_dt;

                log::info!("You died! Press Space to respawn.");

                // Update respawn timer
                self.state = GameState::Dead {
                    respawn_timer: new_timer,
                };

                // Sync camera and render
                self.camera.position = self.player.eye_position();
                self.camera.yaw = self.player.yaw;
                self.camera.pitch = self.player.pitch;

                self.render_scene();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// winit 0.30 ApplicationHandler
// ---------------------------------------------------------------------------

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("MCRust")
            .with_inner_size(PhysicalSize::new(1280u32, 720u32));

        match event_loop.create_window(attrs) {
            Ok(window) => {
                log::info!("Window created: {:?}", window.inner_size());
                let window = Arc::new(window);

                let renderer = Renderer::new(window.clone());
                self.renderer = Some(renderer);
                self.window = Some(window);

                self.last_tick = Instant::now();
                self.tick_accumulator = Duration::ZERO;
                self.camera.aspect = 1280.0 / 720.0;

                // Don't grab cursor yet — we start in Loading state.
                // Cursor will be grabbed when transitioning to Playing.
                log::info!("Game ready — loading world...");
            }
            Err(e) => {
                log::error!("Failed to create window: {e}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting.");
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                log::info!("Resized to {}x{}", new_size.width, new_size.height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width, new_size.height);
                }
                if new_size.width > 0 && new_size.height > 0 {
                    self.camera.aspect = new_size.width as f32 / new_size.height as f32;
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.handle_key_press(key_code, event_loop);
                        }
                        ElementState::Released => {
                            self.keys_held.remove(&key_code);
                        }
                    }
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if matches!(self.state, GameState::Playing) && self.cursor_grabbed {
                    self.break_block();
                } else if matches!(self.state, GameState::Playing) {
                    self.grab_cursor();
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                if matches!(self.state, GameState::Playing) && self.cursor_grabbed {
                    self.place_block();
                }
            }

            WindowEvent::RedrawRequested => {
                self.update();
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if self.cursor_grabbed && matches!(self.state, GameState::Playing) {
                self.player.yaw -= dx as f32 * MOUSE_SENSITIVITY;
                self.player.pitch -= dy as f32 * MOUSE_SENSITIVITY;
                let half_pi = std::f32::consts::FRAC_PI_2 - 0.01;
                self.player.pitch = self.player.pitch.clamp(-half_pi, half_pi);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Input handling (state-aware)
// ---------------------------------------------------------------------------

impl App {
    fn handle_key_press(&mut self, key_code: KeyCode, event_loop: &ActiveEventLoop) {
        match self.state {
            GameState::MainMenu => {
                // Any key transitions to Loading
                log::info!("Starting game...");
                self.state = GameState::Loading {
                    chunks_loaded: 0,
                    chunks_needed: CHUNKS_NEEDED_FOR_PLAY,
                };
            }

            GameState::Loading { .. } => {
                // No input handling during loading (except Escape to quit)
                if key_code == KeyCode::Escape {
                    event_loop.exit();
                }
            }

            GameState::Playing => {
                match key_code {
                    KeyCode::Escape => {
                        log::info!("Game paused.");
                        self.state = GameState::Paused;
                        self.release_cursor();
                    }
                    KeyCode::F3 => {
                        self.hud.show_debug = !self.hud.show_debug;
                    }
                    // Hotbar slot selection: 1-9
                    KeyCode::Digit1 => self.hud.selected_slot = 0,
                    KeyCode::Digit2 => self.hud.selected_slot = 1,
                    KeyCode::Digit3 => self.hud.selected_slot = 2,
                    KeyCode::Digit4 => self.hud.selected_slot = 3,
                    KeyCode::Digit5 => self.hud.selected_slot = 4,
                    KeyCode::Digit6 => self.hud.selected_slot = 5,
                    KeyCode::Digit7 => self.hud.selected_slot = 6,
                    KeyCode::Digit8 => self.hud.selected_slot = 7,
                    KeyCode::Digit9 => self.hud.selected_slot = 8,
                    // Dimension switching (debug keys)
                    KeyCode::KeyN => {
                        self.world.switch_dimension(DimensionId::Nether);
                        self.chunk_meshes.clear();
                        self.mesh_queue.clear();
                        self.player.position = Vec3::new(0.0, 70.0, 0.0);
                        self.player.velocity = Vec3::ZERO;
                        log::info!("Switched to Nether");
                    }
                    KeyCode::KeyO => {
                        self.world.switch_dimension(DimensionId::Overworld);
                        self.chunk_meshes.clear();
                        self.mesh_queue.clear();
                        self.player.position = Vec3::new(0.0, 100.0, 0.0);
                        self.player.velocity = Vec3::ZERO;
                        log::info!("Switched to Overworld");
                    }
                    KeyCode::KeyJ => {
                        self.world.switch_dimension(DimensionId::End);
                        self.chunk_meshes.clear();
                        self.mesh_queue.clear();
                        self.player.position = Vec3::new(0.0, 70.0, 0.0);
                        self.player.velocity = Vec3::ZERO;
                        log::info!("Switched to The End");
                    }
                    _ => {
                        self.keys_held.insert(key_code);
                    }
                }
            }

            GameState::Paused => match key_code {
                KeyCode::Escape => {
                    log::info!("Game resumed.");
                    self.state = GameState::Playing;
                    self.grab_cursor();
                }
                _ => {}
            },

            GameState::Dead { .. } => match key_code {
                KeyCode::Space => {
                    // Respawn
                    log::info!("Respawning...");
                    self.player.position = SPAWN_POSITION;
                    self.player.velocity = Vec3::ZERO;
                    self.player.on_ground = false;
                    self.state = GameState::Playing;
                    self.grab_cursor();
                }
                KeyCode::Escape => {
                    event_loop.exit();
                }
                _ => {}
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("MCRust starting...");

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Event loop exited with error: {e}");
    }
}
