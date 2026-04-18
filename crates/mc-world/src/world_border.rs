/// World border system for constraining player movement within a square boundary.
///
/// The border can be resized over time (lerp), deals damage to entities outside,
/// and provides warning indicators when players approach the edge.

#[derive(Debug, Clone)]
pub struct WorldBorder {
    pub center_x: f64,
    pub center_z: f64,
    pub size: f64,
    pub target_size: f64,
    pub lerp_speed: f64,
    pub damage_per_block: f64,
    pub warning_distance: u32,
    pub warning_time: u32,
}

impl WorldBorder {
    /// Create a new world border centered at (0, 0) with the given size.
    ///
    /// Defaults: damage = 0.2 per block outside, warning distance = 5 blocks.
    pub fn new(size: f64) -> Self {
        Self {
            center_x: 0.0,
            center_z: 0.0,
            size,
            target_size: size,
            lerp_speed: 0.0,
            damage_per_block: 0.2,
            warning_distance: 5,
            warning_time: 0,
        }
    }

    /// Move the border center to the given coordinates.
    pub fn set_center(&mut self, x: f64, z: f64) {
        self.center_x = x;
        self.center_z = z;
    }

    /// Begin resizing the border to `new_size` over `transition_secs` seconds.
    ///
    /// If `transition_secs` is zero or negative, the size changes instantly.
    pub fn set_size(&mut self, new_size: f64, transition_secs: f64) {
        self.target_size = new_size;
        if transition_secs <= 0.0 {
            self.size = new_size;
            self.lerp_speed = 0.0;
        } else {
            self.lerp_speed = (new_size - self.size).abs() / transition_secs;
        }
    }

    /// Advance the border interpolation by `dt` seconds.
    pub fn tick(&mut self, dt: f64) {
        if (self.size - self.target_size).abs() < f64::EPSILON {
            self.lerp_speed = 0.0;
            return;
        }

        let delta = self.lerp_speed * dt;
        if self.size < self.target_size {
            self.size = (self.size + delta).min(self.target_size);
        } else {
            self.size = (self.size - delta).max(self.target_size);
        }

        if (self.size - self.target_size).abs() < f64::EPSILON {
            self.size = self.target_size;
            self.lerp_speed = 0.0;
        }
    }

    /// Half of the current border size (distance from center to edge along one axis).
    pub fn half_size(&self) -> f64 {
        self.size / 2.0
    }

    /// Minimum X coordinate of the border.
    pub fn min_x(&self) -> f64 {
        self.center_x - self.half_size()
    }

    /// Maximum X coordinate of the border.
    pub fn max_x(&self) -> f64 {
        self.center_x + self.half_size()
    }

    /// Minimum Z coordinate of the border.
    pub fn min_z(&self) -> f64 {
        self.center_z - self.half_size()
    }

    /// Maximum Z coordinate of the border.
    pub fn max_z(&self) -> f64 {
        self.center_z + self.half_size()
    }

    /// Returns `true` if the position (x, z) is within the border.
    pub fn is_inside(&self, x: f64, z: f64) -> bool {
        x >= self.min_x() && x <= self.max_x() && z >= self.min_z() && z <= self.max_z()
    }

    /// Signed distance to the nearest border edge.
    ///
    /// Positive means inside the border, negative means outside.
    pub fn distance_to_border(&self, x: f64, z: f64) -> f64 {
        let dx_min = x - self.min_x();
        let dx_max = self.max_x() - x;
        let dz_min = z - self.min_z();
        let dz_max = self.max_z() - z;

        // Minimum distance to any edge; positive when inside, negative when outside
        dx_min.min(dx_max).min(dz_min).min(dz_max)
    }

    /// Clamp a position to stay within the border.
    pub fn clamp_position(&self, x: f64, z: f64) -> (f64, f64) {
        let clamped_x = x.clamp(self.min_x(), self.max_x());
        let clamped_z = z.clamp(self.min_z(), self.max_z());
        (clamped_x, clamped_z)
    }

    /// Damage dealt at position (x, z).
    ///
    /// Returns 0.0 if inside the border, otherwise `damage_per_block * distance_outside`.
    pub fn damage_at(&self, x: f64, z: f64) -> f64 {
        let dist = self.distance_to_border(x, z);
        if dist >= 0.0 {
            0.0
        } else {
            self.damage_per_block * (-dist)
        }
    }

    /// Returns `true` if the position is within `warning_distance` blocks of the border edge.
    pub fn is_warning(&self, x: f64, z: f64) -> bool {
        let dist = self.distance_to_border(x, z);
        dist < self.warning_distance as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inside_outside_check() {
        let border = WorldBorder::new(100.0);
        // Inside
        assert!(border.is_inside(0.0, 0.0));
        assert!(border.is_inside(49.0, 49.0));
        assert!(border.is_inside(-50.0, -50.0));
        // On the edge counts as inside
        assert!(border.is_inside(50.0, 50.0));
        // Outside
        assert!(!border.is_inside(51.0, 0.0));
        assert!(!border.is_inside(0.0, -51.0));
        assert!(!border.is_inside(51.0, 51.0));
    }

    #[test]
    fn damage_zero_inside() {
        let border = WorldBorder::new(100.0);
        assert_eq!(border.damage_at(0.0, 0.0), 0.0);
        assert_eq!(border.damage_at(49.0, 49.0), 0.0);
        assert_eq!(border.damage_at(50.0, 50.0), 0.0);
    }

    #[test]
    fn damage_increases_outside() {
        let border = WorldBorder::new(100.0);
        // 1 block outside => 0.2 damage
        let d1 = border.damage_at(51.0, 0.0);
        assert!((d1 - 0.2).abs() < f64::EPSILON);
        // 5 blocks outside => 1.0 damage
        let d5 = border.damage_at(55.0, 0.0);
        assert!((d5 - 1.0).abs() < f64::EPSILON);
        // Further outside => more damage
        let d10 = border.damage_at(60.0, 0.0);
        assert!(d10 > d5);
    }

    #[test]
    fn lerp_progresses() {
        let mut border = WorldBorder::new(100.0);
        border.set_size(200.0, 10.0);
        assert!((border.lerp_speed - 10.0).abs() < f64::EPSILON);

        // After 5 seconds, should be halfway
        border.tick(5.0);
        assert!((border.size - 150.0).abs() < f64::EPSILON);

        // After another 5 seconds, should reach target
        border.tick(5.0);
        assert!((border.size - 200.0).abs() < f64::EPSILON);
        assert!((border.lerp_speed).abs() < f64::EPSILON);
    }

    #[test]
    fn lerp_shrinks() {
        let mut border = WorldBorder::new(200.0);
        border.set_size(100.0, 5.0);
        // speed = |100-200| / 5 = 20
        assert!((border.lerp_speed - 20.0).abs() < f64::EPSILON);

        border.tick(2.5);
        assert!((border.size - 150.0).abs() < f64::EPSILON);

        border.tick(2.5);
        assert!((border.size - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clamping() {
        let border = WorldBorder::new(100.0);
        // Inside stays the same
        assert_eq!(border.clamp_position(10.0, 20.0), (10.0, 20.0));
        // Outside gets clamped
        assert_eq!(border.clamp_position(100.0, 0.0), (50.0, 0.0));
        assert_eq!(border.clamp_position(-100.0, -100.0), (-50.0, -50.0));
        assert_eq!(border.clamp_position(0.0, 75.0), (0.0, 50.0));
    }

    #[test]
    fn warning_zone() {
        let border = WorldBorder::new(100.0);
        // warning_distance = 5; edge is at x=50
        // At x=46, distance_to_border = 46 - (-50) = nope, min(96, 4, 96, 4) = 4 < 5
        assert!(border.is_warning(46.0, 0.0));
        // At x=44, distance = min(94, 6, 50, 50) = 6 >= 5
        assert!(!border.is_warning(44.0, 0.0));
        // Center is well inside
        assert!(!border.is_warning(0.0, 0.0));
        // Outside is also warning (dist < 0 < 5)
        assert!(border.is_warning(55.0, 0.0));
    }

    #[test]
    fn set_center_moves_border() {
        let mut border = WorldBorder::new(100.0);
        border.set_center(100.0, 100.0);
        // Border now spans x=[50, 150], z=[50, 150]
        assert!(border.is_inside(100.0, 100.0));
        assert!(border.is_inside(50.0, 50.0));
        assert!(border.is_inside(150.0, 150.0));
        assert!(!border.is_inside(0.0, 0.0));
        assert!(!border.is_inside(49.0, 100.0));
        assert!(!border.is_inside(100.0, 151.0));
    }

    #[test]
    fn distance_to_border_positive_inside_negative_outside() {
        let border = WorldBorder::new(100.0);
        // At center, distance to nearest edge is 50
        assert!((border.distance_to_border(0.0, 0.0) - 50.0).abs() < f64::EPSILON);
        // 1 block outside
        let d = border.distance_to_border(51.0, 0.0);
        assert!((d - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn instant_resize_when_zero_transition() {
        let mut border = WorldBorder::new(100.0);
        border.set_size(50.0, 0.0);
        assert!((border.size - 50.0).abs() < f64::EPSILON);
        assert!((border.lerp_speed).abs() < f64::EPSILON);
    }

    #[test]
    fn half_size_and_edges() {
        let border = WorldBorder::new(200.0);
        assert!((border.half_size() - 100.0).abs() < f64::EPSILON);
        assert!((border.min_x() - (-100.0)).abs() < f64::EPSILON);
        assert!((border.max_x() - 100.0).abs() < f64::EPSILON);
        assert!((border.min_z() - (-100.0)).abs() < f64::EPSILON);
        assert!((border.max_z() - 100.0).abs() < f64::EPSILON);
    }
}
