use glam::{Mat4, Vec3};

/// A plane in 3D space defined by its normal and distance from origin.
///
/// The plane equation is: normal . point + distance = 0.
/// Points with signed_distance > 0 are on the positive (inside) half-space.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    /// Signed distance from a point to this plane.
    /// Positive means the point is on the normal side (inside the frustum).
    pub fn signed_distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

/// A view frustum defined by 6 planes, extracted from a view-projection matrix.
///
/// Used for frustum culling — quickly rejecting geometry that is entirely
/// outside the camera's field of view.
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Extract 6 frustum planes from a combined view-projection matrix
    /// using the Griess-Hartmann method.
    ///
    /// Plane order: left, right, bottom, top, near, far.
    pub fn from_view_projection(vp: Mat4) -> Self {
        let row0 = vp.row(0);
        let row1 = vp.row(1);
        let row2 = vp.row(2);
        let row3 = vp.row(3);

        let raw_planes = [
            row3 + row0, // left
            row3 - row0, // right
            row3 + row1, // bottom
            row3 - row1, // top
            row3 + row2, // near
            row3 - row2, // far
        ];

        let planes = raw_planes.map(|p| {
            let normal = Vec3::new(p.x, p.y, p.z);
            let length = normal.length();
            if length < f32::EPSILON {
                Plane {
                    normal: Vec3::ZERO,
                    distance: 0.0,
                }
            } else {
                Plane {
                    normal: normal / length,
                    distance: p.w / length,
                }
            }
        });

        Self { planes }
    }

    /// Test whether an AABB is at least partially inside the frustum.
    ///
    /// For each frustum plane, the AABB vertex most in the direction of the
    /// plane normal (the "p-vertex") is tested. If it lies behind the plane,
    /// the entire AABB is outside the frustum.
    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Select the p-vertex: for each axis, pick max if the normal
            // component is positive, min otherwise.
            let p_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );

            if plane.signed_distance(p_vertex) < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Create an axis-aligned bounding box for a chunk column.
///
/// Chunk coordinates (cx, cz) map to world blocks:
/// - X: `cx * 16 .. cx * 16 + 16`
/// - Y: `-64 .. 320` (Minecraft world height range)
/// - Z: `cz * 16 .. cz * 16 + 16`
///
/// Returns `(min, max)` corners of the AABB.
pub fn chunk_aabb(cx: i32, cz: i32) -> (Vec3, Vec3) {
    let min = Vec3::new((cx * 16) as f32, -64.0, (cz * 16) as f32);
    let max = Vec3::new((cx * 16 + 16) as f32, 320.0, (cz * 16 + 16) as f32);
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    /// Helper: create a camera looking along -Z from the given position
    /// and return the frustum extracted from its view-projection matrix.
    fn frustum_looking_neg_z(position: Vec3) -> Frustum {
        let view = Mat4::look_at_rh(position, position + Vec3::NEG_Z, Vec3::Y);
        let proj = Mat4::perspective_rh(70.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
        Frustum::from_view_projection(proj * view)
    }

    #[test]
    fn chunk_at_origin_is_inside_frustum() {
        let frustum = frustum_looking_neg_z(Vec3::new(8.0, 128.0, 16.0));
        let (min, max) = chunk_aabb(0, 0);
        assert!(
            frustum.contains_aabb(min, max),
            "Chunk (0,0) should be inside the frustum when camera looks at it"
        );
    }

    #[test]
    fn chunk_far_behind_camera_is_outside() {
        let frustum = frustum_looking_neg_z(Vec3::new(8.0, 128.0, 16.0));
        // Chunk at (0, 100) is at z=1600..1616, far behind a camera at z=16 looking -Z
        let (min, max) = chunk_aabb(0, 100);
        assert!(
            !frustum.contains_aabb(min, max),
            "Chunk (0,100) should be outside the frustum (behind camera)"
        );
    }

    #[test]
    fn aabb_fully_outside_left_plane() {
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        // Far to the left, should be outside
        let min = Vec3::new(-5000.0, -10.0, -50.0);
        let max = Vec3::new(-4000.0, 10.0, -40.0);
        assert!(
            !frustum.contains_aabb(min, max),
            "AABB far to the left should be outside"
        );
    }

    #[test]
    fn aabb_fully_outside_right_plane() {
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        let min = Vec3::new(4000.0, -10.0, -50.0);
        let max = Vec3::new(5000.0, 10.0, -40.0);
        assert!(
            !frustum.contains_aabb(min, max),
            "AABB far to the right should be outside"
        );
    }

    #[test]
    fn aabb_fully_outside_top_plane() {
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        let min = Vec3::new(-1.0, 4000.0, -50.0);
        let max = Vec3::new(1.0, 5000.0, -40.0);
        assert!(
            !frustum.contains_aabb(min, max),
            "AABB far above should be outside"
        );
    }

    #[test]
    fn aabb_fully_outside_bottom_plane() {
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        let min = Vec3::new(-1.0, -5000.0, -50.0);
        let max = Vec3::new(1.0, -4000.0, -40.0);
        assert!(
            !frustum.contains_aabb(min, max),
            "AABB far below should be outside"
        );
    }

    #[test]
    fn aabb_fully_outside_near_plane() {
        // Camera at origin looking -Z; near = 0.1
        // An AABB fully in front of the camera (positive Z) is behind the near plane
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        let min = Vec3::new(-1.0, -1.0, 10.0);
        let max = Vec3::new(1.0, 1.0, 20.0);
        assert!(
            !frustum.contains_aabb(min, max),
            "AABB behind the camera should be outside (near plane)"
        );
    }

    #[test]
    fn aabb_fully_outside_far_plane() {
        // Camera at origin, far = 1000; AABB at z = -2000..-1500
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        let min = Vec3::new(-1.0, -1.0, -2000.0);
        let max = Vec3::new(1.0, 1.0, -1500.0);
        assert!(
            !frustum.contains_aabb(min, max),
            "AABB beyond the far plane should be outside"
        );
    }

    #[test]
    fn aabb_straddling_near_plane_is_inside() {
        // AABB that straddles the near plane (partially inside).
        // Camera at origin looking -Z; near = 0.1.
        // AABB from z=-1 to z=0.5 straddles the near plane.
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        let min = Vec3::new(-1.0, -1.0, -1.0);
        let max = Vec3::new(1.0, 1.0, 0.5);
        assert!(
            frustum.contains_aabb(min, max),
            "AABB straddling the near plane should be considered inside"
        );
    }

    #[test]
    fn aabb_straddling_left_edge_is_inside() {
        // A large AABB that straddles the left frustum edge
        let frustum = frustum_looking_neg_z(Vec3::ZERO);
        let min = Vec3::new(-500.0, -1.0, -10.0);
        let max = Vec3::new(0.0, 1.0, -5.0);
        assert!(
            frustum.contains_aabb(min, max),
            "AABB straddling the left edge should be considered inside"
        );
    }

    #[test]
    fn chunk_aabb_dimensions_are_correct() {
        let (min, max) = chunk_aabb(3, -2);
        assert_eq!(min.x, 48.0); // 3 * 16
        assert_eq!(max.x, 64.0); // 3 * 16 + 16
        assert_eq!(min.y, -64.0);
        assert_eq!(max.y, 320.0);
        assert_eq!(min.z, -32.0); // -2 * 16
        assert_eq!(max.z, -16.0); // -2 * 16 + 16
    }

    #[test]
    fn plane_signed_distance_positive_for_inside() {
        let plane = Plane {
            normal: Vec3::Z,
            distance: 0.0,
        };
        assert!(plane.signed_distance(Vec3::new(0.0, 0.0, 5.0)) > 0.0);
        assert!(plane.signed_distance(Vec3::new(0.0, 0.0, -5.0)) < 0.0);
    }
}
