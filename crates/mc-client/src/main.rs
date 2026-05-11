mod block_interaction;
mod command_system;
mod game_loop;
mod inventory_system;
mod mob_system;
mod player;
mod progression_system;
mod save_system;
mod sound_system;
mod survival_system;
mod world_tick;

use player::*;

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

use mc_core::pos::ChunkPos;
use mc_render::mesh::ChunkMesh;
use mc_render::sky::DayNightCycle;
use mc_render::{Camera, Renderer};
use mc_ui::hud::HudState;
use mc_world::ChunkManager;
use mc_world::nether::DimensionId;

const TICK_DURATION: Duration = Duration::from_millis(50); // 20 tps
const RENDER_DISTANCE: i32 = 8;

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

struct App {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) renderer: Option<Renderer>,
    pub(crate) camera: Camera,
    pub(crate) sky: DayNightCycle,
    pub(crate) player: PlayerState,
    pub(crate) world: ChunkManager,
    pub(crate) chunk_meshes: Vec<ChunkMesh>,
    pub(crate) mesh_queue: Vec<ChunkPos>,
    pub(crate) keys_held: HashSet<KeyCode>,
    pub(crate) cursor_grabbed: bool,
    pub(crate) skip_frames: u8,
    pub(crate) last_tick: Instant,
    pub(crate) tick_accumulator: Duration,
    pub(crate) state: GameState,
    pub(crate) hud: HudState,
    // Bridge modules connecting library systems to the game loop
    pub(crate) mob_world: mob_system::MobWorld,
    pub(crate) inventory: inventory_system::PlayerInventory,
    pub(crate) survival: survival_system::SurvivalState,
    pub(crate) world_state: world_tick::WorldTickState,
    pub(crate) progression: progression_system::ProgressionState,
    pub(crate) sounds: sound_system::GameSoundSystem,
    pub(crate) save: save_system::SaveSystem,
    pub(crate) chat: command_system::ChatState,
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
            skip_frames: 0,
            last_tick: Instant::now(),
            tick_accumulator: Duration::ZERO,
            state: GameState::Loading {
                chunks_loaded: 0,
                chunks_needed: CHUNKS_NEEDED_FOR_PLAY,
            },
            hud: HudState::default(),
            mob_world: mob_system::MobWorld::new(),
            inventory: inventory_system::PlayerInventory::new(),
            survival: survival_system::SurvivalState::new(),
            world_state: world_tick::WorldTickState::new(42),
            progression: progression_system::ProgressionState::new(),
            sounds: sound_system::GameSoundSystem::new(),
            save: save_system::SaveSystem::new("saves"),
            chat: command_system::ChatState::new(),
        }
    }

    pub(crate) fn grab_cursor(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let _ = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            window.set_cursor_visible(false);
            self.cursor_grabbed = true;
        }
    }

    pub(crate) fn release_cursor(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.cursor_grabbed = false;
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let frame_time = now - self.last_tick;
        self.last_tick = now;

        // Skip rendering after resize/cursor-grab to let Metal surface stabilize.
        // Without this, get_current_texture() can hit a SIGBUS on macOS when the
        // surface backing memory is being remapped by the GPU driver.
        if self.skip_frames > 0 {
            self.skip_frames -= 1;
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            return;
        }

        match self.state {
            GameState::MainMenu => {
                log::info!("Press any key to start");
                game_loop::render_scene(
                    &self.renderer,
                    &self.camera,
                    &self.sky,
                    &self.chunk_meshes,
                    1280.0,
                    720.0,
                );
            }

            GameState::Loading { chunks_needed, .. } => {
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
                game_loop::process_mesh_queue(
                    &self.renderer,
                    &self.world,
                    &mut self.mesh_queue,
                    &mut self.chunk_meshes,
                );

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

                game_loop::render_scene(
                    &self.renderer,
                    &self.camera,
                    &self.sky,
                    &self.chunk_meshes,
                    1280.0,
                    720.0,
                );

                // Transition to Playing once enough chunks are loaded
                if loaded >= chunks_needed {
                    log::info!("World loaded! Entering Playing state.");
                    self.state = GameState::Playing;
                    // NOTE: Don't grab cursor here — it can invalidate the Metal
                    // surface on macOS, causing a bus error. Grab on next frame.
                }
            }

            GameState::Playing => {
                if let Some(new_state) = game_loop::playing_frame(self, frame_time) {
                    self.state = new_state;
                }
            }

            GameState::Paused => {
                // Don't tick physics, don't advance sky. Just sync camera and render.
                self.camera.position = self.player.eye_position();
                self.camera.yaw = self.player.yaw;
                self.camera.pitch = self.player.pitch;

                game_loop::render_scene(
                    &self.renderer,
                    &self.camera,
                    &self.sky,
                    &self.chunk_meshes,
                    1280.0,
                    720.0,
                );
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

                game_loop::render_scene(
                    &self.renderer,
                    &self.camera,
                    &self.sky,
                    &self.chunk_meshes,
                    1280.0,
                    720.0,
                );
            }
        }
    }
}

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
                // Skip next frame to let Metal surface stabilize after reconfigure
                self.skip_frames = 2;
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
                    game_loop::break_block(self);
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
                    game_loop::place_block(self);
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
                    log::info!("Respawning...");
                    self.player.position = SPAWN_POSITION;
                    self.player.velocity = Vec3::ZERO;
                    self.player.on_ground = false;
                    self.survival.respawn();
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
