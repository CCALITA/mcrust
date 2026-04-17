use glam::Vec3;

use crate::aabb::Aabb;

/// Player dimensions (same as Minecraft: 0.6 wide, 1.8 tall)
pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;

pub fn player_aabb(feet_pos: Vec3) -> Aabb {
    let half_w = PLAYER_WIDTH / 2.0;
    Aabb::new(
        Vec3::new(feet_pos.x - half_w, feet_pos.y, feet_pos.z - half_w),
        Vec3::new(
            feet_pos.x + half_w,
            feet_pos.y + PLAYER_HEIGHT,
            feet_pos.z + half_w,
        ),
    )
}

/// Check if a block at (bx, by, bz) is solid and its AABB intersects with the given AABB.
pub fn block_aabb(bx: i32, by: i32, bz: i32) -> Aabb {
    Aabb::new(
        Vec3::new(bx as f32, by as f32, bz as f32),
        Vec3::new(bx as f32 + 1.0, by as f32 + 1.0, bz as f32 + 1.0),
    )
}

/// Resolve movement against solid blocks in a chunk map.
/// Returns the adjusted velocity after collision.
pub fn move_and_slide(
    pos: Vec3,
    velocity: Vec3,
    get_block_solid: &dyn Fn(i32, i32, i32) -> bool,
) -> Vec3 {
    let mut result = velocity;

    // Resolve each axis independently (Y first for ground detection)
    for axis in [1, 0, 2] {
        let mut test_pos = pos;
        test_pos[axis] += result[axis];
        let aabb = player_aabb(test_pos);

        let min_b = aabb.min.floor();
        let max_b = aabb.max.ceil();

        let mut collided = false;
        for bx in (min_b.x as i32)..(max_b.x as i32) {
            for by in (min_b.y as i32)..(max_b.y as i32) {
                for bz in (min_b.z as i32)..(max_b.z as i32) {
                    if get_block_solid(bx, by, bz) {
                        let bb = block_aabb(bx, by, bz);
                        if aabb.intersects(&bb) {
                            collided = true;
                        }
                    }
                }
            }
        }

        if collided {
            result[axis] = 0.0;
        }
    }

    result
}
