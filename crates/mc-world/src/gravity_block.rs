use glam::Vec3;
use mc_core::block::BlockId;

const GRAVITY: f32 = -32.0;
const TERMINAL_VELOCITY: f32 = -78.4;

/// Returns `true` if the given block is affected by gravity (Sand, Gravel).
pub fn is_gravity_block(block: BlockId) -> bool {
    matches!(block, BlockId::Sand | BlockId::Gravel)
}

/// A block that is currently falling through the world.
#[derive(Debug, Clone, PartialEq)]
pub struct FallingBlock {
    pub block_id: u16,
    pub position: Vec3,
    pub velocity: Vec3,
}

/// The result of ticking a falling block for one frame.
#[derive(Debug, Clone, PartialEq)]
pub enum FallingBlockAction {
    /// The block is still falling.
    Falling,
    /// The block has landed at the given grid position.
    Landed((i32, i32, i32)),
}

/// Checks whether the block at `(bx, by, bz)` should begin falling.
///
/// A gravity-affected block should fall when the block directly below it is air.
pub fn check_should_fall(
    bx: i32,
    by: i32,
    bz: i32,
    is_air: &dyn Fn(i32, i32, i32) -> bool,
) -> bool {
    is_air(bx, by - 1, bz)
}

/// Advances a falling block by `dt` seconds and determines whether it is
/// still falling or has landed on a solid block.
pub fn tick_falling(
    fb: &FallingBlock,
    dt: f32,
    is_solid: &dyn Fn(i32, i32, i32) -> bool,
) -> (FallingBlock, FallingBlockAction) {
    let new_velocity = Vec3::new(
        fb.velocity.x,
        (fb.velocity.y + GRAVITY * dt).max(TERMINAL_VELOCITY),
        fb.velocity.z,
    );

    let new_position = Vec3::new(
        fb.position.x + new_velocity.x * dt,
        fb.position.y + new_velocity.y * dt,
        fb.position.z + new_velocity.z * dt,
    );

    let grid_x = new_position.x.floor() as i32;
    let grid_y = new_position.y.floor() as i32;
    let grid_z = new_position.z.floor() as i32;

    // The block has landed if it is moving downward and either:
    // - the destination cell itself is solid (block would clip into terrain), or
    // - the cell directly below the destination is solid (block rests on top).
    let destination_solid = is_solid(grid_x, grid_y, grid_z);
    let below_solid = is_solid(grid_x, grid_y - 1, grid_z);

    if new_velocity.y < 0.0 && (destination_solid || below_solid) {
        // Land on top of the highest solid block.
        let landed_y = if destination_solid { grid_y + 1 } else { grid_y };
        let landed = FallingBlock {
            block_id: fb.block_id,
            position: Vec3::new(grid_x as f32 + 0.5, landed_y as f32, grid_z as f32 + 0.5),
            velocity: Vec3::ZERO,
        };
        (landed, FallingBlockAction::Landed((grid_x, landed_y, grid_z)))
    } else {
        let updated = FallingBlock {
            block_id: fb.block_id,
            position: new_position,
            velocity: new_velocity,
        };
        (updated, FallingBlockAction::Falling)
    }
}

/// Called when a block update occurs. If the block at `pos` is a gravity block
/// and air is below it, returns a new `FallingBlock` entity to begin simulation.
pub fn on_block_update(pos: (i32, i32, i32), block: u16, is_air_below: bool) -> Option<FallingBlock> {
    let block_id = BlockId::from_raw(block)?;
    if !is_gravity_block(block_id) || !is_air_below {
        return None;
    }
    Some(FallingBlock {
        block_id: block,
        position: Vec3::new(pos.0 as f32 + 0.5, pos.1 as f32, pos.2 as f32 + 0.5),
        velocity: Vec3::ZERO,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sand_is_gravity_block() {
        assert!(is_gravity_block(BlockId::Sand));
    }

    #[test]
    fn gravel_is_gravity_block() {
        assert!(is_gravity_block(BlockId::Gravel));
    }

    #[test]
    fn stone_is_not_gravity_block() {
        assert!(!is_gravity_block(BlockId::Stone));
    }

    #[test]
    fn air_is_not_gravity_block() {
        assert!(!is_gravity_block(BlockId::Air));
    }

    #[test]
    fn should_fall_when_air_below() {
        let is_air = |_x: i32, _y: i32, _z: i32| true;
        assert!(check_should_fall(0, 10, 0, &is_air));
    }

    #[test]
    fn should_not_fall_when_solid_below() {
        let is_air = |_x: i32, _y: i32, _z: i32| false;
        assert!(!check_should_fall(0, 10, 0, &is_air));
    }

    #[test]
    fn sand_falls_when_air_below() {
        let fb = FallingBlock {
            block_id: BlockId::Sand as u16,
            position: Vec3::new(5.5, 10.0, 5.5),
            velocity: Vec3::ZERO,
        };
        let is_solid = |_x: i32, _y: i32, _z: i32| false;
        let (updated, action) = tick_falling(&fb, 0.05, &is_solid);
        assert_eq!(action, FallingBlockAction::Falling);
        assert!(updated.position.y < fb.position.y, "block should move downward");
        assert!(updated.velocity.y < 0.0, "velocity should be negative");
    }

    #[test]
    fn sand_lands_on_solid_block() {
        // Block is at y=3.05 with a small downward velocity; after one tick it
        // moves to floor grid_y=3, and the block at y=2 is solid, so it lands.
        let fb = FallingBlock {
            block_id: BlockId::Sand as u16,
            position: Vec3::new(5.5, 3.05, 5.5),
            velocity: Vec3::new(0.0, -0.5, 0.0),
        };
        let is_solid = |_x: i32, y: i32, _z: i32| y <= 2;
        let (landed, action) = tick_falling(&fb, 0.05, &is_solid);
        match action {
            FallingBlockAction::Landed((x, y, z)) => {
                assert_eq!(x, 5);
                assert_eq!(y, 3);
                assert_eq!(z, 5);
            }
            FallingBlockAction::Falling => {
                panic!("expected block to land on solid surface");
            }
        }
        assert_eq!(landed.velocity, Vec3::ZERO);
    }

    #[test]
    fn gravel_falls_when_air_below() {
        let fb = FallingBlock {
            block_id: BlockId::Gravel as u16,
            position: Vec3::new(3.5, 20.0, 3.5),
            velocity: Vec3::ZERO,
        };
        let is_solid = |_x: i32, _y: i32, _z: i32| false;
        let (updated, action) = tick_falling(&fb, 0.05, &is_solid);
        assert_eq!(action, FallingBlockAction::Falling);
        assert!(updated.position.y < fb.position.y);
    }

    #[test]
    fn gravel_lands_on_solid_block() {
        // Block is just above y=5 with a small downward velocity; after one tick
        // grid_y=5, and y=4 is solid, so it lands at (2, 5, 2).
        let fb = FallingBlock {
            block_id: BlockId::Gravel as u16,
            position: Vec3::new(2.5, 5.05, 2.5),
            velocity: Vec3::new(0.0, -0.5, 0.0),
        };
        let is_solid = |_x: i32, y: i32, _z: i32| y <= 4;
        let (_landed, action) = tick_falling(&fb, 0.05, &is_solid);
        assert!(matches!(action, FallingBlockAction::Landed((2, 5, 2))));
    }

    #[test]
    fn on_block_update_creates_falling_block_for_sand() {
        let result = on_block_update((10, 50, 10), BlockId::Sand as u16, true);
        assert!(result.is_some());
        let fb = result.unwrap();
        assert_eq!(fb.block_id, BlockId::Sand as u16);
        assert_eq!(fb.position, Vec3::new(10.5, 50.0, 10.5));
        assert_eq!(fb.velocity, Vec3::ZERO);
    }

    #[test]
    fn on_block_update_none_when_not_gravity_block() {
        let result = on_block_update((10, 50, 10), BlockId::Stone as u16, true);
        assert!(result.is_none());
    }

    #[test]
    fn on_block_update_none_when_air_below_is_false() {
        let result = on_block_update((10, 50, 10), BlockId::Sand as u16, false);
        assert!(result.is_none());
    }

    #[test]
    fn on_block_update_none_for_invalid_block_id() {
        let result = on_block_update((0, 0, 0), 9999, true);
        assert!(result.is_none());
    }

    #[test]
    fn velocity_does_not_exceed_terminal_velocity() {
        let fb = FallingBlock {
            block_id: BlockId::Sand as u16,
            position: Vec3::new(0.5, 200.0, 0.5),
            velocity: Vec3::new(0.0, TERMINAL_VELOCITY + 1.0, 0.0),
        };
        let is_solid = |_x: i32, _y: i32, _z: i32| false;
        // Simulate many ticks
        let mut current = fb;
        for _ in 0..1000 {
            let (updated, _) = tick_falling(&current, 0.05, &is_solid);
            current = updated;
        }
        assert!(
            current.velocity.y >= TERMINAL_VELOCITY,
            "velocity {} should not exceed terminal velocity {}",
            current.velocity.y,
            TERMINAL_VELOCITY
        );
    }
}
