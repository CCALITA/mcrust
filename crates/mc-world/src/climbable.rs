use mc_core::block::BlockId;
use std::collections::VecDeque;

/// Speed at which a player moves upward while climbing, in blocks per tick.
pub const CLIMBING_SPEED: f32 = 0.12;

/// Maximum horizontal distance scaffolding can extend from a supported column.
pub const SCAFFOLDING_MAX_DISTANCE: u8 = 6;

/// Returns `true` if the block is climbable (ladder-like).
///
/// Uses `Torch` as a placeholder for Ladder/Vine since no dedicated Ladder
/// `BlockId` variant exists yet.
pub fn is_climbable(block: BlockId) -> bool {
    matches!(block, BlockId::Torch)
}

/// Returns `true` if scaffolding can be placed at `pos`.
///
/// Scaffolding is valid when:
/// - The block directly below is solid (provided by `get_block`), **or**
/// - There is scaffolding within `SCAFFOLDING_MAX_DISTANCE` blocks horizontally
///   that ultimately connects to a supported column.
///
/// `get_block` should return `true` if the block at the given coordinates is
/// solid (ground / support).
pub fn can_place_scaffolding(
    pos: (i32, i32, i32),
    get_block: &dyn Fn(i32, i32, i32) -> bool,
) -> bool {
    let (x, y, z) = pos;

    // Direct support: solid block below.
    if get_block(x, y - 1, z) {
        return true;
    }

    // Check whether any scaffolding within range connects to a supported
    // position.  We do this by searching outward from `pos` for a scaffolding
    // block that itself sits on solid ground.
    //
    // For this check we assume any position that has a solid block below it
    // counts as supported scaffolding within range.
    for dx in -(SCAFFOLDING_MAX_DISTANCE as i32)..=(SCAFFOLDING_MAX_DISTANCE as i32) {
        for dz in -(SCAFFOLDING_MAX_DISTANCE as i32)..=(SCAFFOLDING_MAX_DISTANCE as i32) {
            let dist = dx.unsigned_abs() as u8 + dz.unsigned_abs() as u8;
            if dist == 0 || dist > SCAFFOLDING_MAX_DISTANCE {
                continue;
            }
            if get_block(x + dx, y - 1 + 0, z + dz) {
                // There is a solid block below (x+dx, y, z+dz),
                // meaning scaffolding there would be supported, and it is
                // within range.
                return true;
            }
        }
    }

    false
}

/// Computes the scaffolding distance for `pos` via BFS from the nearest
/// supported scaffolding block.
///
/// A scaffolding block is *supported* when `is_supported(x, y, z)` returns
/// `true` (e.g., solid block directly below).  The distance is the minimum
/// number of horizontal Manhattan steps to reach the nearest supported
/// scaffolding through a connected chain of scaffolding blocks.
///
/// Returns `u8::MAX` if no supported scaffolding is reachable.
pub fn scaffolding_distance(
    pos: (i32, i32, i32),
    is_scaffolding: &dyn Fn(i32, i32, i32) -> bool,
    is_supported: &dyn Fn(i32, i32, i32) -> bool,
) -> u8 {
    let (px, py, pz) = pos;

    // If the start position itself is not scaffolding, return max.
    if !is_scaffolding(px, py, pz) {
        return u8::MAX;
    }

    // If the start is directly supported, distance is 0.
    if is_supported(px, py, pz) {
        return 0;
    }

    // BFS outward from `pos` through scaffolding blocks.  We track the
    // Manhattan distance from the origin and stop as soon as we find a
    // supported block or exceed the maximum search radius.
    let mut queue: VecDeque<(i32, i32, i32, u8)> = VecDeque::new();
    let mut visited: Vec<(i32, i32, i32)> = Vec::new();

    queue.push_back((px, py, pz, 0));
    visited.push((px, py, pz));

    let neighbors: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    while let Some((cx, cy, cz, dist)) = queue.pop_front() {
        for (dx, dz) in &neighbors {
            let nx = cx + dx;
            let nz = cz + dz;

            if visited.contains(&(nx, cy, nz)) {
                continue;
            }

            if !is_scaffolding(nx, cy, nz) {
                continue;
            }

            let new_dist = dist + 1;

            if is_supported(nx, cy, nz) {
                return new_dist;
            }

            visited.push((nx, cy, nz));

            // Only continue BFS within a reasonable radius to avoid unbounded
            // search.  We cap at SCAFFOLDING_MAX_DISTANCE * 2 to be generous.
            if new_dist < SCAFFOLDING_MAX_DISTANCE * 2 {
                queue.push_back((nx, cy, nz, new_dist));
            }
        }
    }

    u8::MAX
}

/// Returns `true` if scaffolding at the given distance should fall (break).
pub fn should_scaffolding_fall(distance: u8) -> bool {
    distance > SCAFFOLDING_MAX_DISTANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Distance calculation ────────────────────────────────────────────

    #[test]
    fn distance_zero_when_directly_supported() {
        let dist = scaffolding_distance(
            (0, 1, 0),
            &|_, _, _| true,  // everything is scaffolding
            &|x, y, z| x == 0 && y == 1 && z == 0, // only origin supported
        );
        assert_eq!(dist, 0);
    }

    #[test]
    fn distance_one_when_adjacent_to_support() {
        // Supported scaffolding at (0,1,0); query at (1,1,0).
        let is_scaffolding = |x: i32, y: i32, z: i32| {
            (x == 0 || x == 1) && y == 1 && z == 0
        };
        let is_supported = |x: i32, y: i32, z: i32| {
            x == 0 && y == 1 && z == 0
        };

        let dist = scaffolding_distance((1, 1, 0), &is_scaffolding, &is_supported);
        assert_eq!(dist, 1);
    }

    #[test]
    fn distance_through_chain() {
        // Chain: (0,1,0) supported -> (1,1,0) -> (2,1,0) -> (3,1,0)
        let is_scaffolding = |x: i32, y: i32, z: i32| {
            y == 1 && z == 0 && (0..=3).contains(&x)
        };
        let is_supported = |x: i32, y: i32, z: i32| {
            x == 0 && y == 1 && z == 0
        };

        assert_eq!(scaffolding_distance((3, 1, 0), &is_scaffolding, &is_supported), 3);
    }

    #[test]
    fn distance_max_when_no_support_reachable() {
        // Isolated scaffolding with no support.
        let is_scaffolding = |x: i32, y: i32, z: i32| {
            x == 5 && y == 1 && z == 5
        };
        let is_supported = |_: i32, _: i32, _: i32| false;

        let dist = scaffolding_distance((5, 1, 5), &is_scaffolding, &is_supported);
        assert_eq!(dist, u8::MAX);
    }

    #[test]
    fn distance_max_when_not_scaffolding() {
        let dist = scaffolding_distance(
            (0, 0, 0),
            &|_, _, _| false,
            &|_, _, _| true,
        );
        assert_eq!(dist, u8::MAX);
    }

    // ── Support chain ───────────────────────────────────────────────────

    #[test]
    fn support_chain_within_max_distance() {
        // Chain of 6 scaffolding blocks from the supported column.
        let is_scaffolding = |x: i32, y: i32, z: i32| {
            y == 1 && z == 0 && (0..=6).contains(&x)
        };
        let is_supported = |x: i32, y: i32, z: i32| {
            x == 0 && y == 1 && z == 0
        };

        let dist = scaffolding_distance((6, 1, 0), &is_scaffolding, &is_supported);
        assert_eq!(dist, 6);
        assert!(!should_scaffolding_fall(dist));
    }

    #[test]
    fn support_chain_beyond_max_distance_falls() {
        // Chain of 7 scaffolding blocks — the 7th should fall.
        let is_scaffolding = |x: i32, y: i32, z: i32| {
            y == 1 && z == 0 && (0..=7).contains(&x)
        };
        let is_supported = |x: i32, y: i32, z: i32| {
            x == 0 && y == 1 && z == 0
        };

        let dist = scaffolding_distance((7, 1, 0), &is_scaffolding, &is_supported);
        assert_eq!(dist, 7);
        assert!(should_scaffolding_fall(dist));
    }

    // ── Fall condition ──────────────────────────────────────────────────

    #[test]
    fn scaffolding_does_not_fall_at_max_distance() {
        assert!(!should_scaffolding_fall(6));
    }

    #[test]
    fn scaffolding_falls_beyond_max_distance() {
        assert!(should_scaffolding_fall(7));
        assert!(should_scaffolding_fall(10));
        assert!(should_scaffolding_fall(u8::MAX));
    }

    #[test]
    fn scaffolding_does_not_fall_at_zero() {
        assert!(!should_scaffolding_fall(0));
    }

    // ── Climbing speed constant ─────────────────────────────────────────

    #[test]
    fn climbing_speed_is_expected_value() {
        assert!((CLIMBING_SPEED - 0.12).abs() < f32::EPSILON);
    }

    // ── is_climbable ────────────────────────────────────────────────────

    #[test]
    fn torch_is_climbable_placeholder() {
        assert!(is_climbable(BlockId::Torch));
    }

    #[test]
    fn non_climbable_blocks() {
        assert!(!is_climbable(BlockId::Air));
        assert!(!is_climbable(BlockId::Stone));
        assert!(!is_climbable(BlockId::OakLog));
        assert!(!is_climbable(BlockId::Water));
    }

    // ── can_place_scaffolding ───────────────────────────────────────────

    #[test]
    fn can_place_on_solid_below() {
        let get_block = |_x: i32, _y: i32, _z: i32| true;
        assert!(can_place_scaffolding((0, 5, 0), &get_block));
    }

    #[test]
    fn cannot_place_without_support() {
        let get_block = |_x: i32, _y: i32, _z: i32| false;
        assert!(!can_place_scaffolding((0, 5, 0), &get_block));
    }

    #[test]
    fn can_place_near_supported_scaffolding() {
        // Solid block below position (3, 5, 0) — within range of (0, 5, 0).
        let get_block = |x: i32, y: i32, z: i32| {
            x == 3 && y == 4 && z == 0
        };
        assert!(can_place_scaffolding((0, 5, 0), &get_block));
    }

    #[test]
    fn cannot_place_when_support_too_far() {
        // Solid block at distance 7 (beyond max 6).
        let get_block = |x: i32, y: i32, z: i32| {
            x == 7 && y == 4 && z == 0
        };
        assert!(!can_place_scaffolding((0, 5, 0), &get_block));
    }
}
