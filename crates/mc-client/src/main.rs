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
use mc_physics::collision;
use mc_render::mesh::{ChunkMesh, NeighborChunks};
use mc_render::{Camera, Renderer};
use mc_world::ChunkManager;

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
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera: Camera,
    player: PlayerState,
    world: ChunkManager,
    chunk_meshes: Vec<ChunkMesh>,
    /// Chunks that still need meshing (carried across frames).
    mesh_queue: Vec<ChunkPos>,
    keys_held: HashSet<KeyCode>,
    cursor_grabbed: bool,
    last_tick: Instant,
    tick_accumulator: Duration,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            camera: Camera::new(Vec3::new(0.0, 65.62, 0.0), 16.0 / 9.0),
            player: PlayerState::new(Vec3::new(0.0, 65.0, 0.0)),
            world: ChunkManager::new(RENDER_DISTANCE),
            chunk_meshes: Vec::new(),
            mesh_queue: Vec::new(),
            keys_held: HashSet::new(),
            cursor_grabbed: false,
            last_tick: Instant::now(),
            tick_accumulator: Duration::ZERO,
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
        let resolved = collision::move_and_slide(self.player.position, frame_vel, &|bx, by, bz| {
            self.world.is_block_solid(bx, by, bz)
        });

        self.player.on_ground = frame_vel.y < 0.0 && resolved.y.abs() < 1e-6;
        self.player.position += resolved;

        if self.player.on_ground {
            self.player.velocity.y = 0.0;
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let frame_time = now - self.last_tick;
        self.last_tick = now;

        self.tick_accumulator += frame_time;
        let tick_dt = TICK_DURATION.as_secs_f32();
        // Cap accumulated ticks to prevent spiral-of-death on first frame
        let max_ticks = 4;
        let mut tick_count = 0;
        while self.tick_accumulator >= TICK_DURATION && tick_count < max_ticks {
            self.tick_accumulator -= TICK_DURATION;
            self.tick(tick_dt);
            tick_count += 1;
        }
        // Discard excess accumulated time
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
            // Remove old meshes for dirty chunks
            self.chunk_meshes.retain(|m| !dirty.contains(&m.chunk_pos));
            // Queue dirty chunks for meshing
            self.mesh_queue.extend(dirty);
        }

        // Mesh a limited number of chunks per frame to avoid overwhelming the GPU
        const MAX_MESHES_PER_FRAME: usize = 16;
        if !self.mesh_queue.is_empty() {
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

        // Render
        if let Some(renderer) = &self.renderer {
            match renderer.render_frame(&self.camera, &self.chunk_meshes) {
                Ok(()) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    if let Some(r) = &mut self.renderer {
                        let (w, h) = r.size();
                        log::warn!("Surface lost/outdated, reconfiguring");
                        r.resize(w, h);
                    }
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

                // Reset timer so we don't accumulate seconds of gravity from init time
                self.last_tick = Instant::now();
                self.tick_accumulator = Duration::ZERO;

                // Update camera aspect ratio
                self.camera.aspect = 1280.0 / 720.0;

                self.grab_cursor();
                log::info!("Game ready");
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
                            if key_code == KeyCode::Escape {
                                if self.cursor_grabbed {
                                    self.release_cursor();
                                } else {
                                    event_loop.exit();
                                }
                            } else {
                                self.keys_held.insert(key_code);
                            }
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
                if !self.cursor_grabbed {
                    self.grab_cursor();
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
            if self.cursor_grabbed {
                self.player.yaw -= dx as f32 * MOUSE_SENSITIVITY;
                self.player.pitch -= dy as f32 * MOUSE_SENSITIVITY;
                let half_pi = std::f32::consts::FRAC_PI_2 - 0.01;
                self.player.pitch = self.player.pitch.clamp(-half_pi, half_pi);
            }
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
