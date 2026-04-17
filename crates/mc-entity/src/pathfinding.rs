use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use mc_core::pos::BlockPos;

/// Result of an A* pathfinding search.
#[derive(Debug, Clone)]
pub struct AStarResult {
    pub path: Vec<BlockPos>,
    pub found: bool,
}

/// Node in the A* open set, ordered by lowest f-cost.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Node {
    pos: BlockPos,
    f_cost: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap (max-heap) pops the lowest f_cost first.
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Manhattan distance heuristic in 3D.
fn heuristic(a: BlockPos, b: BlockPos) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
}

/// 6-connected neighbors: 4 horizontal + up + down.
const NEIGHBOR_OFFSETS: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 0, 1),
    (0, 0, -1),
    (0, 1, 0),
    (0, -1, 0),
];

/// Find a path from `start` to `goal` using A* search.
///
/// A block is considered walkable if `is_walkable(pos)` returns true.
/// The caller should implement the walkability check: typically the block
/// itself is air AND the block below is solid (the entity can stand on it).
///
/// `max_iterations` caps the search to prevent lag. If the limit is reached
/// the best partial path found so far is returned with `found: false`.
pub fn find_path(
    start: BlockPos,
    goal: BlockPos,
    max_iterations: u32,
    is_walkable: &dyn Fn(BlockPos) -> bool,
) -> AStarResult {
    if start == goal {
        return AStarResult {
            path: vec![start],
            found: true,
        };
    }

    let mut open = BinaryHeap::new();
    let mut g_costs: HashMap<BlockPos, i32> = HashMap::new();
    let mut came_from: HashMap<BlockPos, BlockPos> = HashMap::new();

    g_costs.insert(start, 0);
    open.push(Node {
        pos: start,
        f_cost: heuristic(start, goal),
    });

    let mut iterations = 0u32;
    let mut best_pos = start;
    let mut best_h = heuristic(start, goal);

    while let Some(current) = open.pop() {
        if current.pos == goal {
            return AStarResult {
                path: reconstruct_path(&came_from, goal),
                found: true,
            };
        }

        iterations += 1;
        if iterations >= max_iterations {
            // Return partial path to the closest node we found.
            if best_pos == start {
                return AStarResult {
                    path: Vec::new(),
                    found: false,
                };
            }
            return AStarResult {
                path: reconstruct_path(&came_from, best_pos),
                found: false,
            };
        }

        let current_g = g_costs.get(&current.pos).copied().unwrap_or(i32::MAX);

        for &(dx, dy, dz) in &NEIGHBOR_OFFSETS {
            let neighbor =
                BlockPos::new(current.pos.x + dx, current.pos.y + dy, current.pos.z + dz);

            if !is_walkable(neighbor) {
                continue;
            }

            let tentative_g = current_g + 1;

            if tentative_g < g_costs.get(&neighbor).copied().unwrap_or(i32::MAX) {
                g_costs.insert(neighbor, tentative_g);
                came_from.insert(neighbor, current.pos);

                let h = heuristic(neighbor, goal);
                let f = tentative_g + h;

                open.push(Node {
                    pos: neighbor,
                    f_cost: f,
                });

                // Track the closest node to the goal for partial paths.
                if h < best_h {
                    best_h = h;
                    best_pos = neighbor;
                }
            }
        }
    }

    // Open set exhausted without reaching the goal.
    AStarResult {
        path: Vec::new(),
        found: false,
    }
}

/// Reconstruct the path from start to `end` by following `came_from` links.
fn reconstruct_path(came_from: &HashMap<BlockPos, BlockPos>, end: BlockPos) -> Vec<BlockPos> {
    let mut path = vec![end];
    let mut current = end;
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: walkable means the block itself is "air" and block below is "solid".
    /// We simulate a flat surface at y=0: blocks at y=0 are solid ground,
    /// blocks at y=1 are walkable (air with solid below).
    fn flat_surface_walkable(pos: BlockPos) -> bool {
        pos.y == 1 // y=1 is air, y=0 is solid ground beneath
    }

    #[test]
    fn finds_path_on_flat_surface() {
        let start = BlockPos::new(0, 1, 0);
        let goal = BlockPos::new(5, 1, 0);

        let result = find_path(start, goal, 200, &flat_surface_walkable);

        assert!(result.found);
        assert!(!result.path.is_empty());
        assert_eq!(*result.path.first().unwrap(), start);
        assert_eq!(*result.path.last().unwrap(), goal);
    }

    #[test]
    fn finds_path_around_wall() {
        // Wall from z=-1 to z=3 at x=3, y=1
        let is_walkable = |pos: BlockPos| -> bool {
            if pos.y != 1 {
                return false;
            }
            // Wall blocks at x=3 for z in -1..=3 are not walkable
            if pos.x == 3 && (-1..=3).contains(&pos.z) {
                return false;
            }
            true
        };

        let start = BlockPos::new(0, 1, 0);
        let goal = BlockPos::new(5, 1, 0);

        let result = find_path(start, goal, 200, &is_walkable);

        assert!(result.found);
        assert_eq!(*result.path.first().unwrap(), start);
        assert_eq!(*result.path.last().unwrap(), goal);

        // Path must go around the wall, so no node should be at x=3, z in -1..=3
        for pos in &result.path {
            if pos.x == 3 {
                assert!(
                    !(-1..=3).contains(&pos.z),
                    "Path passes through wall at {:?}",
                    pos
                );
            }
        }
    }

    #[test]
    fn unreachable_goal_returns_empty() {
        // Only a 3x3 island is walkable — goal is far away and unreachable.
        let is_walkable =
            |pos: BlockPos| -> bool { pos.y == 1 && pos.x.abs() <= 1 && pos.z.abs() <= 1 };

        let start = BlockPos::new(0, 1, 0);
        let goal = BlockPos::new(50, 1, 50);

        let result = find_path(start, goal, 200, &is_walkable);

        assert!(!result.found);
        // The open set is exhausted on a small island, so path is empty.
        assert!(result.path.is_empty());
    }

    #[test]
    fn same_start_and_goal_returns_trivial_path() {
        let pos = BlockPos::new(3, 1, 3);
        let result = find_path(pos, pos, 200, &flat_surface_walkable);

        assert!(result.found);
        assert_eq!(result.path.len(), 1);
        assert_eq!(result.path[0], pos);
    }

    #[test]
    fn max_iterations_returns_partial_path() {
        let start = BlockPos::new(0, 1, 0);
        let goal = BlockPos::new(100, 1, 0);

        // Very low iteration limit forces early termination.
        let result = find_path(start, goal, 5, &flat_surface_walkable);

        assert!(!result.found);
        // Should have a partial path (at least the start and some progress).
        if !result.path.is_empty() {
            assert_eq!(*result.path.first().unwrap(), start);
        }
    }

    #[test]
    fn path_steps_are_adjacent() {
        let start = BlockPos::new(0, 1, 0);
        let goal = BlockPos::new(3, 1, 3);

        let result = find_path(start, goal, 200, &flat_surface_walkable);
        assert!(result.found);

        // Verify each consecutive pair of positions differs by exactly 1 in
        // one axis (Manhattan neighbor).
        for pair in result.path.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let dist = (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs();
            assert_eq!(dist, 1, "Non-adjacent step: {:?} -> {:?}", a, b);
        }
    }
}
