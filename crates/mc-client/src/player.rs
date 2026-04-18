use glam::Vec3;
use mc_physics::collision;

// ---------------------------------------------------------------------------
// Physics constants
// ---------------------------------------------------------------------------

pub const GRAVITY: f32 = -32.0;
pub const JUMP_VELOCITY: f32 = 8.5;
pub const WALK_SPEED: f32 = 4.3;
pub const SPRINT_SPEED: f32 = 5.6;
pub const SNEAK_SPEED: f32 = 1.3;
pub const MOUSE_SENSITIVITY: f32 = 0.003;
pub const REACH_DISTANCE: f32 = 5.0;

/// Spawn position.
pub const SPAWN_POSITION: Vec3 = Vec3::new(0.0, 100.0, 0.0);

// ---------------------------------------------------------------------------
// PlayerState
// ---------------------------------------------------------------------------

pub struct PlayerState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub yaw: f32,
    pub pitch: f32,
}

impl PlayerState {
    pub fn new(spawn: Vec3) -> Self {
        Self {
            position: spawn,
            velocity: Vec3::ZERO,
            on_ground: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn eye_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, collision::PLAYER_EYE_HEIGHT, 0.0)
    }

    pub fn forward_xz(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, self.yaw.cos()).normalize_or_zero()
    }

    pub fn right_xz(&self) -> Vec3 {
        let fwd = self.forward_xz();
        Vec3::new(fwd.z, 0.0, -fwd.x)
    }

    /// Full 3D look direction (includes pitch).
    pub fn look_direction(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
        .normalize_or_zero()
    }
}
