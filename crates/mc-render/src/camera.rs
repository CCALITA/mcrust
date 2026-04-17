use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

/// Camera uniform data uploaded to the GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

/// First-person camera with position, yaw, and pitch.
pub struct Camera {
    pub position: Vec3,
    /// Yaw in radians (rotation around Y axis). 0 = looking along -Z.
    pub yaw: f32,
    /// Pitch in radians (rotation around X axis). Clamped to +-89 degrees.
    pub pitch: f32,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vec3, aspect: f32) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            fov_y: 70.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Forward direction vector on the XZ plane (ignores pitch for movement).
    fn forward_xz(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos()).normalize_or_zero()
    }

    /// Right direction vector on the XZ plane.
    fn right_xz(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()).normalize_or_zero()
    }

    /// Full forward direction (includes pitch).
    fn forward(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalize_or_zero()
    }

    /// Update facing direction from mouse delta (dx, dy in radians).
    pub fn update_direction(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        self.pitch += dy;

        let limit = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-limit, limit);
    }

    /// Move relative to facing direction.
    /// `forward` = forward/back, `right` = strafe, `up` = vertical.
    pub fn move_relative(&mut self, forward: f32, right: f32, up: f32) {
        self.position += self.forward_xz() * forward;
        self.position += self.right_xz() * right;
        self.position += Vec3::Y * up;
    }

    /// Build the view matrix (camera transform).
    pub fn view_matrix(&self) -> Mat4 {
        let target = self.position + self.forward();
        Mat4::look_at_rh(self.position, target, Vec3::Y)
    }

    /// Build the perspective projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    /// Combined view-projection matrix.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Build the camera uniform for uploading to the GPU.
    pub fn uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.view_projection_matrix().to_cols_array_2d(),
        }
    }
}
