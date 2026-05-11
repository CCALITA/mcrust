use mc_core::block::BlockId;
use mc_core::pos::BlockPos;

/// Maximum number of blocks a piston can push in a single line.
pub const PUSH_LIMIT: usize = 12;

/// Runtime state of a piston block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PistonState {
    pub extended: bool,
    pub facing: u8,
    pub sticky: bool,
}

impl PistonState {
    /// Creates a new retracted piston facing the given direction.
    pub fn new(facing: u8, sticky: bool) -> Self {
        Self {
            extended: false,
            facing,
            sticky,
        }
    }
}

/// Returns the maximum number of blocks a piston can push.
pub fn push_limit() -> u8 {
    PUSH_LIMIT as u8
}

/// Extends a piston if the block count in front does not exceed the push limit.
///
/// Mutates `state.extended` to `true` on success. Returns `false` if
/// `blocks_count` exceeds the push limit.
pub fn piston_extend(state: &mut PistonState, blocks_count: u8) -> bool {
    if blocks_count > push_limit() {
        return false;
    }
    state.extended = true;
    true
}

/// Retracts a sticky piston. Returns `true` if the piston was extended and
/// is now retracted, `false` if it was already retracted or is not sticky.
pub fn piston_retract_sticky(state: &mut PistonState) -> bool {
    if !state.extended || !state.sticky {
        return false;
    }
    state.extended = false;
    true
}

/// Returns `true` if the given block can be pushed by a piston.
///
/// Most solid blocks are pushable. Bedrock and obsidian cannot be moved.
pub fn can_push_block(block: BlockId) -> bool {
    !matches!(block, BlockId::Bedrock | BlockId::Obsidian)
}

/// Converts a `facing` byte (0..=5) to a `(dx, dy, dz)` offset.
///
/// The mapping matches `Direction` repr values:
///   0 = Up (+Y), 1 = Down (-Y), 2 = North (-Z),
///   3 = South (+Z), 4 = East (+X), 5 = West (-X).
fn facing_offset(facing: u8) -> (i32, i32, i32) {
    match facing {
        0 => (0, 1, 0),
        1 => (0, -1, 0),
        2 => (0, 0, -1),
        3 => (0, 0, 1),
        4 => (1, 0, 0),
        5 => (-1, 0, 0),
        _ => (0, 0, 0),
    }
}

/// Offsets a `BlockPos` by `steps` positions in the given facing direction.
fn offset_pos(pos: BlockPos, facing: u8, steps: i32) -> BlockPos {
    let (dx, dy, dz) = facing_offset(facing);
    BlockPos::new(pos.x + dx * steps, pos.y + dy * steps, pos.z + dz * steps)
}

/// Pushes a line of blocks starting from the position in front of the piston.
///
/// Scans up to [`PUSH_LIMIT`] blocks in the facing direction. If the line
/// contains an unpushable block or exceeds the push limit, returns `false`
/// and leaves the world unchanged. Otherwise, shifts every block forward by
/// one position and clears the original piston-face position to `Air`.
///
/// # Parameters
///
/// * `pos` - The position of the piston block itself.
/// * `facing` - Direction the piston faces (0..=5, matching `Direction` repr).
/// * `get_block` - Closure that reads a block at a given position.
/// * `set_block` - Closure that writes a block at a given position.
pub fn push_line(
    pos: BlockPos,
    facing: u8,
    get_block: impl Fn(BlockPos) -> BlockId,
    mut set_block: impl FnMut(BlockPos, BlockId),
) -> bool {
    // Count pushable blocks in front of the piston.
    let mut count = 0;
    loop {
        let check = offset_pos(pos, facing, count as i32 + 1);
        let block = get_block(check);
        if block.is_air() {
            break;
        }
        if !can_push_block(block) {
            return false;
        }
        count += 1;
        if count > PUSH_LIMIT {
            return false;
        }
    }

    // Shift blocks from the far end back toward the piston so we never
    // overwrite a block before reading it.
    for i in (1..=count).rev() {
        let src = offset_pos(pos, facing, i as i32);
        let dst = offset_pos(pos, facing, i as i32 + 1);
        set_block(dst, get_block(src));
    }

    // Clear the position directly in front of the piston.
    if count > 0 {
        set_block(offset_pos(pos, facing, 1), BlockId::Air);
    }

    true
}

/// Retracts a piston. If the piston is sticky, pulls one block back.
///
/// # Parameters
///
/// * `pos` - The position of the piston block itself.
/// * `facing` - Direction the piston faces.
/// * `sticky` - Whether this is a sticky piston.
/// * `get_block` - Closure that reads a block at a given position.
/// * `set_block` - Closure that writes a block at a given position.
pub fn retract(
    pos: BlockPos,
    facing: u8,
    sticky: bool,
    get_block: impl Fn(BlockPos) -> BlockId,
    mut set_block: impl FnMut(BlockPos, BlockId),
) {
    // Clear the block directly in front (the piston head / extension).
    set_block(offset_pos(pos, facing, 1), BlockId::Air);

    if sticky {
        let pull_pos = offset_pos(pos, facing, 2);
        let block = get_block(pull_pos);
        if !block.is_air() && can_push_block(block) {
            set_block(offset_pos(pos, facing, 1), block);
            set_block(pull_pos, BlockId::Air);
        }
    }
}

/// Extends a piston: pushes blocks in front and marks state as extended.
///
/// Returns the updated `PistonState` on success, or `None` if the push
/// failed (unpushable block or push limit exceeded).
pub fn extend_piston(
    state: PistonState,
    pos: BlockPos,
    get_block: impl Fn(BlockPos) -> BlockId,
    set_block: impl FnMut(BlockPos, BlockId),
) -> Option<PistonState> {
    if state.extended {
        return Some(state);
    }
    if push_line(pos, state.facing, get_block, set_block) {
        Some(PistonState {
            extended: true,
            ..state
        })
    } else {
        None
    }
}

/// Retracts a piston and returns the updated state.
pub fn retract_piston(
    state: PistonState,
    pos: BlockPos,
    get_block: impl Fn(BlockPos) -> BlockId,
    set_block: impl FnMut(BlockPos, BlockId),
) -> PistonState {
    if !state.extended {
        return state;
    }
    retract(pos, state.facing, state.sticky, get_block, set_block);
    PistonState {
        extended: false,
        ..state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos::new(x, y, z)
    }

    /// Helper: read from a snapshot so the original map stays available for mutation.
    fn getter(snap: &HashMap<BlockPos, BlockId>) -> impl Fn(BlockPos) -> BlockId + '_ {
        move |p: BlockPos| *snap.get(&p).unwrap_or(&BlockId::Air)
    }

    #[test]
    fn push_limit_exceeded() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East (+X)

        let mut world: HashMap<BlockPos, BlockId> = HashMap::new();
        // Place 13 stone blocks in front of the piston (exceeds PUSH_LIMIT of 12).
        for i in 1..=13 {
            world.insert(pos(i, 0, 0), BlockId::Stone);
        }

        let snap = world.clone();
        let result = push_line(piston_pos, facing, getter(&snap), |p, b| {
            world.insert(p, b);
        });
        assert!(!result, "push should fail when exceeding PUSH_LIMIT");

        // World should be unchanged because push_line returns early.
        for i in 1..=13 {
            assert_eq!(*world.get(&pos(i, 0, 0)).unwrap(), BlockId::Stone);
        }
    }

    #[test]
    fn unpushable_block_prevents_push() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East

        let mut world: HashMap<BlockPos, BlockId> = HashMap::new();
        world.insert(pos(1, 0, 0), BlockId::Stone);
        world.insert(pos(2, 0, 0), BlockId::Bedrock);

        let snap = world.clone();
        let result = push_line(piston_pos, facing, getter(&snap), |_p, _b| {});
        assert!(!result, "push should fail when hitting Bedrock");
    }

    #[test]
    fn unpushable_obsidian_prevents_push() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East

        let mut world: HashMap<BlockPos, BlockId> = HashMap::new();
        world.insert(pos(1, 0, 0), BlockId::Obsidian);

        let snap = world.clone();
        let result = push_line(piston_pos, facing, getter(&snap), |_p, _b| {});
        assert!(!result, "push should fail when hitting Obsidian");
    }

    #[test]
    fn sticky_retract_pulls_one_block() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East
        let sticky = true;

        let mut w: HashMap<BlockPos, BlockId> = HashMap::new();
        // After extension, a block sits at position 2.
        w.insert(pos(2, 0, 0), BlockId::Sand);

        let snap = w.clone();
        retract(piston_pos, facing, sticky, getter(&snap), |p, b| {
            w.insert(p, b);
        });

        // Sand should have been pulled from pos(2,0,0) to pos(1,0,0).
        assert_eq!(
            *w.get(&pos(1, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Sand,
            "sticky piston should pull the block one position closer"
        );
        assert_eq!(
            *w.get(&pos(2, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Air,
            "original position should be cleared after sticky retract"
        );
    }

    #[test]
    fn non_sticky_retract_does_not_pull() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East
        let sticky = false;

        let mut w: HashMap<BlockPos, BlockId> = HashMap::new();
        w.insert(pos(2, 0, 0), BlockId::Sand);

        let snap = w.clone();
        retract(piston_pos, facing, sticky, getter(&snap), |p, b| {
            w.insert(p, b);
        });

        // Sand should stay at pos(2,0,0); pos(1,0,0) should be air.
        assert_eq!(
            *w.get(&pos(1, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Air,
            "non-sticky piston should not pull any block"
        );
        assert_eq!(
            *w.get(&pos(2, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Sand,
            "block behind piston head should remain"
        );
    }

    #[test]
    fn empty_push_succeeds() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East

        let w: HashMap<BlockPos, BlockId> = HashMap::new();
        let get = |p: BlockPos| *w.get(&p).unwrap_or(&BlockId::Air);
        let set = |_p: BlockPos, _b: BlockId| {};

        let result = push_line(piston_pos, facing, get, set);
        assert!(result, "push into empty space should succeed");
    }

    #[test]
    fn push_shifts_blocks_forward() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East

        let mut w: HashMap<BlockPos, BlockId> = HashMap::new();
        w.insert(pos(1, 0, 0), BlockId::Stone);
        w.insert(pos(2, 0, 0), BlockId::Dirt);

        let snapshot = w.clone();
        let get = |p: BlockPos| *snapshot.get(&p).unwrap_or(&BlockId::Air);
        let result = push_line(piston_pos, facing, get, |p, b| {
            w.insert(p, b);
        });

        assert!(result, "push should succeed with 2 blocks");
        assert_eq!(
            *w.get(&pos(1, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Air,
            "position in front of piston should be cleared"
        );
        assert_eq!(
            *w.get(&pos(2, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Stone,
            "first block should shift forward"
        );
        assert_eq!(
            *w.get(&pos(3, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Dirt,
            "second block should shift forward"
        );
    }

    #[test]
    fn extend_piston_updates_state() {
        let piston_pos = pos(0, 0, 0);
        let state = PistonState {
            extended: false,
            facing: 4,
            sticky: false,
        };

        let w: HashMap<BlockPos, BlockId> = HashMap::new();
        let get = |p: BlockPos| *w.get(&p).unwrap_or(&BlockId::Air);
        let set = |_p: BlockPos, _b: BlockId| {};

        let new_state = extend_piston(state, piston_pos, get, set);
        assert!(new_state.is_some());
        assert!(new_state.unwrap().extended);
    }

    #[test]
    fn extend_piston_fails_on_unpushable() {
        let piston_pos = pos(0, 0, 0);
        let state = PistonState {
            extended: false,
            facing: 4,
            sticky: false,
        };

        let mut w: HashMap<BlockPos, BlockId> = HashMap::new();
        w.insert(pos(1, 0, 0), BlockId::Bedrock);

        let get = |p: BlockPos| *w.get(&p).unwrap_or(&BlockId::Air);
        let set = |_p: BlockPos, _b: BlockId| {};

        let result = extend_piston(state, piston_pos, get, set);
        assert!(result.is_none(), "extend should fail against bedrock");
    }

    #[test]
    fn retract_piston_updates_state() {
        let piston_pos = pos(0, 0, 0);
        let state = PistonState {
            extended: true,
            facing: 4,
            sticky: true,
        };

        let mut w: HashMap<BlockPos, BlockId> = HashMap::new();
        w.insert(pos(2, 0, 0), BlockId::Sand);

        let snapshot = w.clone();
        let get = |p: BlockPos| *snapshot.get(&p).unwrap_or(&BlockId::Air);
        let new_state = retract_piston(state, piston_pos, get, |p, b| {
            w.insert(p, b);
        });

        assert!(!new_state.extended);
        assert_eq!(
            *w.get(&pos(1, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Sand,
        );
    }

    #[test]
    fn push_exactly_at_limit_succeeds() {
        let piston_pos = pos(0, 0, 0);
        let facing: u8 = 4; // East

        let mut w: HashMap<BlockPos, BlockId> = HashMap::new();
        // Place exactly 12 blocks (the limit).
        for i in 1..=12 {
            w.insert(pos(i, 0, 0), BlockId::Stone);
        }

        let snapshot = w.clone();
        let get = |p: BlockPos| *snapshot.get(&p).unwrap_or(&BlockId::Air);
        let result = push_line(piston_pos, facing, get, |p, b| {
            w.insert(p, b);
        });

        assert!(result, "push of exactly 12 blocks should succeed");
        // The first position should now be air.
        assert_eq!(*w.get(&pos(1, 0, 0)).unwrap_or(&BlockId::Air), BlockId::Air,);
        // The last block should have moved to position 13.
        assert_eq!(
            *w.get(&pos(13, 0, 0)).unwrap_or(&BlockId::Air),
            BlockId::Stone,
        );
    }

    #[test]
    fn can_push_block_common_blocks() {
        assert!(can_push_block(BlockId::Stone));
        assert!(can_push_block(BlockId::Dirt));
        assert!(can_push_block(BlockId::Sand));
        assert!(can_push_block(BlockId::GrassBlock));
        assert!(can_push_block(BlockId::OakPlanks));
    }

    #[test]
    fn can_push_block_immovable() {
        assert!(!can_push_block(BlockId::Bedrock));
        assert!(!can_push_block(BlockId::Obsidian));
    }

    #[test]
    fn piston_state_default_values() {
        let state = PistonState {
            extended: false,
            facing: 0,
            sticky: false,
        };
        assert!(!state.extended);
        assert_eq!(state.facing, 0);
        assert!(!state.sticky);
    }

    #[test]
    fn piston_state_new_creates_retracted_piston() {
        let state = PistonState::new(3, true);
        assert!(!state.extended);
        assert_eq!(state.facing, 3);
        assert!(state.sticky);
    }

    #[test]
    fn push_limit_returns_twelve() {
        assert_eq!(push_limit(), 12);
    }

    #[test]
    fn piston_extend_succeeds_within_limit() {
        let mut state = PistonState::new(4, false);
        assert!(piston_extend(&mut state, 12));
        assert!(state.extended);
    }

    #[test]
    fn piston_extend_fails_over_limit() {
        let mut state = PistonState::new(4, false);
        assert!(!piston_extend(&mut state, 13));
        assert!(!state.extended);
    }

    #[test]
    fn piston_extend_zero_blocks() {
        let mut state = PistonState::new(0, false);
        assert!(piston_extend(&mut state, 0));
        assert!(state.extended);
    }

    #[test]
    fn piston_retract_sticky_succeeds() {
        let mut state = PistonState::new(2, true);
        state.extended = true;
        assert!(piston_retract_sticky(&mut state));
        assert!(!state.extended);
    }

    #[test]
    fn piston_retract_sticky_fails_if_not_extended() {
        let mut state = PistonState::new(2, true);
        assert!(!piston_retract_sticky(&mut state));
    }

    #[test]
    fn piston_retract_sticky_fails_if_not_sticky() {
        let mut state = PistonState::new(2, false);
        state.extended = true;
        assert!(!piston_retract_sticky(&mut state));
        assert!(state.extended, "non-sticky piston should remain extended");
    }

    #[test]
    fn can_push_block_obsidian_returns_false() {
        assert!(!can_push_block(BlockId::Obsidian));
    }

    #[test]
    fn can_push_block_bedrock_returns_false() {
        assert!(!can_push_block(BlockId::Bedrock));
    }
}
