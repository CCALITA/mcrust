use glam::Vec3;
use mc_core::direction::Direction;
use mc_core::pos::BlockPos;

/// Result of a successful block raycast.
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    /// The block position that was hit.
    pub block_pos: BlockPos,
    /// The face of the block that was hit (used to place blocks adjacent).
    pub face: Direction,
    /// Distance from the ray origin to the hit point.
    pub distance: f32,
    /// Exact world-space hit point on the block face.
    pub point: Vec3,
}

/// Cast a ray through the voxel grid using the DDA algorithm.
///
/// Returns the first solid block hit, along with the face that was entered
/// and the exact intersection point.
///
/// # Arguments
/// * `origin` - Ray start position in world space.
/// * `direction` - Ray direction (does not need to be normalized, but must be non-zero).
/// * `max_distance` - Maximum ray travel distance.
/// * `is_solid` - Callback that returns `true` if the block at (x, y, z) is solid.
pub fn raycast(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    is_solid: &dyn Fn(i32, i32, i32) -> bool,
) -> Option<RaycastHit> {
    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    // Current grid cell (floor to get block coordinates).
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    // Check if we start inside a solid block.
    if is_solid(x, y, z) {
        return Some(RaycastHit {
            block_pos: BlockPos::new(x, y, z),
            face: Direction::Up, // arbitrary when starting inside
            distance: 0.0,
            point: origin,
        });
    }

    // Step direction (+1 or -1) and the face entered when stepping along each axis.
    let step_x: i32 = if dir.x >= 0.0 { 1 } else { -1 };
    let step_y: i32 = if dir.y >= 0.0 { 1 } else { -1 };
    let step_z: i32 = if dir.z >= 0.0 { 1 } else { -1 };

    // Face entered when we step along each axis.
    let face_x = if step_x > 0 {
        Direction::West
    } else {
        Direction::East
    };
    let face_y = if step_y > 0 {
        Direction::Down
    } else {
        Direction::Up
    };
    let face_z = if step_z > 0 {
        Direction::North
    } else {
        Direction::South
    };

    // Distance along the ray to the next grid boundary for each axis.
    let t_max_x = if dir.x.abs() > f32::EPSILON {
        let boundary = if step_x > 0 { (x + 1) as f32 } else { x as f32 };
        (boundary - origin.x) / dir.x
    } else {
        f32::MAX
    };

    let t_max_y = if dir.y.abs() > f32::EPSILON {
        let boundary = if step_y > 0 { (y + 1) as f32 } else { y as f32 };
        (boundary - origin.y) / dir.y
    } else {
        f32::MAX
    };

    let t_max_z = if dir.z.abs() > f32::EPSILON {
        let boundary = if step_z > 0 { (z + 1) as f32 } else { z as f32 };
        (boundary - origin.z) / dir.z
    } else {
        f32::MAX
    };

    let mut t_max = Vec3::new(t_max_x, t_max_y, t_max_z);

    // Distance along the ray between successive grid boundaries on each axis.
    let t_delta = Vec3::new(
        if dir.x.abs() > f32::EPSILON {
            (1.0 / dir.x).abs()
        } else {
            f32::MAX
        },
        if dir.y.abs() > f32::EPSILON {
            (1.0 / dir.y).abs()
        } else {
            f32::MAX
        },
        if dir.z.abs() > f32::EPSILON {
            (1.0 / dir.z).abs()
        } else {
            f32::MAX
        },
    );

    loop {
        // Advance along the axis with the smallest t_max.
        let (t_current, last_face);
        if t_max.x < t_max.y && t_max.x < t_max.z {
            t_current = t_max.x;
            last_face = face_x;
            x += step_x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            t_current = t_max.y;
            last_face = face_y;
            y += step_y;
            t_max.y += t_delta.y;
        } else {
            t_current = t_max.z;
            last_face = face_z;
            z += step_z;
            t_max.z += t_delta.z;
        }

        if t_current > max_distance {
            return None;
        }

        if is_solid(x, y, z) {
            let point = origin + dir * t_current;
            return Some(RaycastHit {
                block_pos: BlockPos::new(x, y, z),
                face: last_face,
                distance: t_current,
                point,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: only the block at the given position is solid.
    fn solid_at(sx: i32, sy: i32, sz: i32) -> impl Fn(i32, i32, i32) -> bool {
        move |x, y, z| x == sx && y == sy && z == sz
    }

    /// Helper: a horizontal layer of solid blocks at the given y.
    fn solid_layer(layer_y: i32) -> impl Fn(i32, i32, i32) -> bool {
        move |_x, y, _z| y == layer_y
    }

    #[test]
    fn ray_straight_down_hits_top_face() {
        let origin = Vec3::new(0.5, 10.5, 0.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);
        let hit = raycast(origin, direction, 20.0, &solid_layer(5)).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(0, 5, 0));
        assert_eq!(hit.face, Direction::Up);
        assert!((hit.distance - 4.5).abs() < 1e-4);
        assert!((hit.point.y - 6.0).abs() < 1e-4);
    }

    #[test]
    fn ray_along_positive_x_hits_west_face() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let hit = raycast(origin, direction, 20.0, &solid_at(5, 0, 0)).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(5, 0, 0));
        assert_eq!(hit.face, Direction::West);
        assert!((hit.distance - 4.5).abs() < 1e-4);
        assert!((hit.point.x - 5.0).abs() < 1e-4);
    }

    #[test]
    fn ray_misses_when_max_distance_too_short() {
        let origin = Vec3::new(0.5, 10.5, 0.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);
        let hit = raycast(origin, direction, 2.0, &solid_layer(5));

        assert!(hit.is_none());
    }

    #[test]
    fn ray_misses_when_no_solid_blocks() {
        let origin = Vec3::new(0.5, 10.5, 0.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);
        let hit = raycast(origin, direction, 100.0, &|_, _, _| false);

        assert!(hit.is_none());
    }

    #[test]
    fn ray_starting_inside_solid_returns_immediately() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let hit = raycast(origin, direction, 10.0, &|_, _, _| true).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(0, 0, 0));
        assert!((hit.distance - 0.0).abs() < 1e-6);
        assert_eq!(hit.point, origin);
    }

    #[test]
    fn ray_along_negative_x_hits_east_face() {
        let origin = Vec3::new(10.5, 0.5, 0.5);
        let direction = Vec3::new(-1.0, 0.0, 0.0);
        let hit = raycast(origin, direction, 20.0, &solid_at(3, 0, 0)).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(3, 0, 0));
        assert_eq!(hit.face, Direction::East);
        assert!((hit.point.x - 4.0).abs() < 1e-4);
    }

    #[test]
    fn ray_along_positive_z_hits_north_face() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let hit = raycast(origin, direction, 20.0, &solid_at(0, 0, 7)).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(0, 0, 7));
        assert_eq!(hit.face, Direction::North);
        assert!((hit.point.z - 7.0).abs() < 1e-4);
    }

    #[test]
    fn ray_along_negative_z_hits_south_face() {
        let origin = Vec3::new(0.5, 0.5, 10.5);
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let hit = raycast(origin, direction, 20.0, &solid_at(0, 0, 3)).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(0, 0, 3));
        assert_eq!(hit.face, Direction::South);
        assert!((hit.point.z - 4.0).abs() < 1e-4);
    }

    #[test]
    fn ray_diagonal_hits_correct_block() {
        // Ray at 45 degrees in XY plane aimed at a block at (5, 5, 0).
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 1.0, 0.0).normalize();
        let hit = raycast(origin, direction, 20.0, &solid_at(5, 5, 0)).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(5, 5, 0));
    }

    #[test]
    fn zero_direction_returns_none() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let hit = raycast(origin, Vec3::ZERO, 10.0, &|_, _, _| true);
        assert!(hit.is_none());
    }

    #[test]
    fn ray_upward_hits_bottom_face() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(0.0, 1.0, 0.0);
        let hit = raycast(origin, direction, 20.0, &solid_at(0, 8, 0)).unwrap();

        assert_eq!(hit.block_pos, BlockPos::new(0, 8, 0));
        assert_eq!(hit.face, Direction::Down);
        assert!((hit.point.y - 8.0).abs() < 1e-4);
    }

    #[test]
    fn hit_distance_is_accurate() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let hit = raycast(origin, direction, 50.0, &solid_at(10, 0, 0)).unwrap();

        // Origin at 0.5, block starts at 10.0, so distance = 9.5
        assert!((hit.distance - 9.5).abs() < 1e-4);
    }
}
