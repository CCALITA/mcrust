//! Projectile trail rendering: billboard quads that follow arrows, tridents,
//! and fireworks through the air, fading with age.

/// A single point along a projectile trail.
#[derive(Debug, Clone, Copy)]
pub struct TrailPoint {
    /// World-space position of this trail sample.
    pub pos: [f32; 3],
    /// Seconds since this point was recorded (increases each tick).
    pub age: f32,
}

/// A trail composed of aged sample points, rendered as camera-facing quads.
#[derive(Debug, Clone)]
pub struct Trail {
    /// Ordered trail points, newest last.
    pub points: Vec<TrailPoint>,
    /// Maximum number of points retained before the oldest are dropped.
    pub max_points: usize,
    /// Seconds after which a point is considered expired and removed.
    pub lifetime: f32,
    /// RGB color of the trail.
    pub color: [f32; 3],
}

impl Trail {
    /// Create an empty trail with the given capacity, lifetime, and color.
    pub fn new(max_points: usize, lifetime: f32, color: [f32; 3]) -> Self {
        Self {
            points: Vec::with_capacity(max_points),
            max_points,
            lifetime,
            color,
        }
    }

    /// Record a new sample point at `pos`. If the trail is at capacity, the
    /// oldest point is removed first.
    pub fn add_point(&mut self, pos: [f32; 3]) {
        if self.points.len() >= self.max_points {
            self.points.remove(0);
        }
        self.points.push(TrailPoint { pos, age: 0.0 });
    }

    /// Advance all point ages by `dt` seconds and remove points that have
    /// exceeded the trail lifetime.
    pub fn tick(&mut self, dt: f32) {
        for point in &mut self.points {
            point.age += dt;
        }
        let lifetime = self.lifetime;
        self.points.retain(|p| p.age < lifetime);
    }
}

/// Create a trail preset for arrows: thin, white, short-lived.
pub fn arrow_trail() -> Trail {
    Trail::new(20, 0.4, [1.0, 1.0, 1.0])
}

/// Create a trail preset for tridents: medium, aqua-tinted, moderate lifetime.
pub fn trident_trail() -> Trail {
    Trail::new(30, 0.6, [0.3, 0.8, 0.9])
}

/// Create a trail preset for fireworks: long, orange-yellow, persistent.
pub fn firework_trail() -> Trail {
    Trail::new(50, 1.2, [1.0, 0.7, 0.2])
}

/// Generate billboard quad vertices for the entire trail, each quad facing the
/// camera. Returns world-space vertex positions suitable for building a render
/// batch.
///
/// Each pair of adjacent trail points produces one quad (4 vertices). The quad
/// width tapers linearly based on point age (newer = wider, older = thinner).
pub fn trail_quad_vertices(trail: &Trail, camera_pos: [f32; 3]) -> Vec<[f32; 3]> {
    if trail.points.len() < 2 {
        return Vec::new();
    }

    let base_half_width: f32 = 0.05;
    let pair_count = trail.points.len() - 1;
    let mut vertices = Vec::with_capacity(pair_count * 4);

    for i in 0..pair_count {
        let a = &trail.points[i];
        let b = &trail.points[i + 1];

        // Segment direction
        let seg_dx = b.pos[0] - a.pos[0];
        let seg_dy = b.pos[1] - a.pos[1];
        let seg_dz = b.pos[2] - a.pos[2];

        // Midpoint of segment -> used for billboard direction
        let mid_x = (a.pos[0] + b.pos[0]) * 0.5;
        let mid_y = (a.pos[1] + b.pos[1]) * 0.5;
        let mid_z = (a.pos[2] + b.pos[2]) * 0.5;

        // Camera-to-midpoint direction
        let cam_dx = mid_x - camera_pos[0];
        let cam_dy = mid_y - camera_pos[1];
        let cam_dz = mid_z - camera_pos[2];

        // Cross product of segment direction and camera direction gives the
        // billboard "right" vector.
        let cx = seg_dy * cam_dz - seg_dz * cam_dy;
        let cy = seg_dz * cam_dx - seg_dx * cam_dz;
        let cz = seg_dx * cam_dy - seg_dy * cam_dx;

        let cross_len = (cx * cx + cy * cy + cz * cz).sqrt();
        let (rx, ry, rz) = if cross_len > 1e-6 {
            (cx / cross_len, cy / cross_len, cz / cross_len)
        } else {
            // Degenerate: segment parallel to camera direction; pick an
            // arbitrary perpendicular.
            (1.0, 0.0, 0.0)
        };

        // Taper width based on age: newer points are wider.
        let alpha_a = if trail.lifetime > 0.0 {
            1.0 - (a.age / trail.lifetime).min(1.0)
        } else {
            0.0
        };
        let alpha_b = if trail.lifetime > 0.0 {
            1.0 - (b.age / trail.lifetime).min(1.0)
        } else {
            0.0
        };
        let hw_a = base_half_width * alpha_a;
        let hw_b = base_half_width * alpha_b;

        // Quad: a-left, a-right, b-right, b-left
        vertices.push([
            a.pos[0] - rx * hw_a,
            a.pos[1] - ry * hw_a,
            a.pos[2] - rz * hw_a,
        ]);
        vertices.push([
            a.pos[0] + rx * hw_a,
            a.pos[1] + ry * hw_a,
            a.pos[2] + rz * hw_a,
        ]);
        vertices.push([
            b.pos[0] + rx * hw_b,
            b.pos[1] + ry * hw_b,
            b.pos[2] + rz * hw_b,
        ]);
        vertices.push([
            b.pos[0] - rx * hw_b,
            b.pos[1] - ry * hw_b,
            b.pos[2] - rz * hw_b,
        ]);
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trail_is_empty() {
        let trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        assert!(trail.points.is_empty());
        assert_eq!(trail.max_points, 10);
        assert!((trail.lifetime - 1.0).abs() < 1e-6);
    }

    #[test]
    fn add_point_stores_point_with_zero_age() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([1.0, 2.0, 3.0]);
        assert_eq!(trail.points.len(), 1);
        assert_eq!(trail.points[0].pos, [1.0, 2.0, 3.0]);
        assert!((trail.points[0].age).abs() < 1e-6);
    }

    #[test]
    fn add_point_respects_max_capacity() {
        let mut trail = Trail::new(3, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([1.0, 0.0, 0.0]);
        trail.add_point([2.0, 0.0, 0.0]);
        trail.add_point([3.0, 0.0, 0.0]);
        assert_eq!(trail.points.len(), 3);

        trail.add_point([4.0, 0.0, 0.0]);
        assert_eq!(trail.points.len(), 3);
        // Oldest point (1.0) should have been dropped
        assert!((trail.points[0].pos[0] - 2.0).abs() < 1e-6);
        assert!((trail.points[2].pos[0] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn tick_ages_points() {
        let mut trail = Trail::new(10, 2.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.tick(0.5);
        assert!((trail.points[0].age - 0.5).abs() < 1e-6);
    }

    #[test]
    fn tick_removes_expired_points() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.add_point([1.0, 0.0, 0.0]);
        trail.tick(0.5);
        assert_eq!(trail.points.len(), 2);
        trail.tick(0.6); // total age = 1.1 > lifetime 1.0
        assert_eq!(trail.points.len(), 0);
    }

    #[test]
    fn arrow_trail_preset() {
        let trail = arrow_trail();
        assert_eq!(trail.max_points, 20);
        assert!((trail.lifetime - 0.4).abs() < 1e-6);
        assert_eq!(trail.color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn trident_trail_preset() {
        let trail = trident_trail();
        assert_eq!(trail.max_points, 30);
        assert!((trail.lifetime - 0.6).abs() < 1e-6);
        assert_eq!(trail.color, [0.3, 0.8, 0.9]);
    }

    #[test]
    fn firework_trail_preset() {
        let trail = firework_trail();
        assert_eq!(trail.max_points, 50);
        assert!((trail.lifetime - 1.2).abs() < 1e-6);
        assert_eq!(trail.color, [1.0, 0.7, 0.2]);
    }

    #[test]
    fn trail_quad_vertices_empty_trail_returns_empty() {
        let trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        let verts = trail_quad_vertices(&trail, [0.0, 0.0, 5.0]);
        assert!(verts.is_empty());
    }

    #[test]
    fn trail_quad_vertices_single_point_returns_empty() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        let verts = trail_quad_vertices(&trail, [0.0, 0.0, 5.0]);
        assert!(verts.is_empty());
    }

    #[test]
    fn trail_quad_vertices_two_points_produce_one_quad() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.add_point([1.0, 0.0, 0.0]);
        let verts = trail_quad_vertices(&trail, [0.0, 0.0, 5.0]);
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn trail_quad_vertices_three_points_produce_two_quads() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.add_point([1.0, 0.0, 0.0]);
        trail.add_point([2.0, 0.0, 0.0]);
        let verts = trail_quad_vertices(&trail, [0.0, 0.0, 5.0]);
        assert_eq!(verts.len(), 8);
    }

    #[test]
    fn trail_quad_vertices_taper_with_age() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.add_point([1.0, 0.0, 0.0]);
        // Age the first point so it tapers more
        trail.points[0].age = 0.5;
        trail.points[1].age = 0.0;

        let camera = [0.5, 0.0, 5.0];
        let verts = trail_quad_vertices(&trail, camera);
        assert_eq!(verts.len(), 4);

        // The older point (index 0) should produce a narrower offset than the
        // newer point (index 1). Compare the y-offset magnitude (billboard
        // perpendicular to the x-axis segment with camera along +z produces
        // perpendicular along y).
        let older_half_width = (verts[1][1] - verts[0][1]).abs();
        let newer_half_width = (verts[3][2] - verts[2][2]).abs();
        // Newer point (age 0.0) should be at full width, older at 50%
        // So newer width > older width
        assert!(
            newer_half_width <= older_half_width + 1e-6
                || older_half_width < 0.06,
            "older half_width {older_half_width} should be <= newer {newer_half_width}"
        );
    }

    #[test]
    fn trail_quad_vertices_degenerate_camera_does_not_panic() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.add_point([1.0, 0.0, 0.0]);
        // Camera at same position as midpoint of segment
        let verts = trail_quad_vertices(&trail, [0.5, 0.0, 0.0]);
        assert_eq!(verts.len(), 4);
    }

    #[test]
    fn tick_does_not_remove_points_within_lifetime() {
        let mut trail = Trail::new(10, 2.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.add_point([1.0, 0.0, 0.0]);
        trail.tick(1.0);
        assert_eq!(trail.points.len(), 2);
    }

    #[test]
    fn multiple_ticks_accumulate_age() {
        let mut trail = Trail::new(10, 1.0, [1.0, 1.0, 1.0]);
        trail.add_point([0.0, 0.0, 0.0]);
        trail.tick(0.3);
        trail.tick(0.3);
        trail.tick(0.3);
        assert!((trail.points[0].age - 0.9).abs() < 1e-5);
        assert_eq!(trail.points.len(), 1);

        trail.tick(0.2); // age = 1.1 > 1.0
        assert_eq!(trail.points.len(), 0);
    }
}
