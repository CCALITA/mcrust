use std::collections::{HashMap, HashSet, VecDeque};

use mc_core::pos::BlockPos;

// ---------------------------------------------------------------------------
// Rail types and shapes
// ---------------------------------------------------------------------------

/// The four rail variants found in Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RailType {
    Normal,
    Powered,
    Detector,
    Activator,
}

/// Possible orientations and curves for a rail piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscNorth,
    AscSouth,
    AscEast,
    AscWest,
    CurveNE,
    CurveSE,
    CurveSW,
    CurveNW,
}

// ---------------------------------------------------------------------------
// Rail block identification
// ---------------------------------------------------------------------------

/// Block-id ranges reserved for rail variants.
///
/// Since `BlockId` does not yet contain rail variants, we define placeholder
/// raw id ranges here.  Call sites that place rails into the world should use
/// these constants until proper `BlockId` variants are added.
const RAIL_ID_START: u16 = 200;
const RAIL_ID_END: u16 = 203; // inclusive: Normal=200, Powered=201, Detector=202, Activator=203

/// Returns `true` if the raw block id represents any kind of rail.
pub fn is_rail(block_id: u16) -> bool {
    (RAIL_ID_START..=RAIL_ID_END).contains(&block_id)
}

/// Maps a raw block id to its `RailType`, returning `None` for non-rail blocks.
pub fn rail_type_from_id(block_id: u16) -> Option<RailType> {
    match block_id {
        200 => Some(RailType::Normal),
        201 => Some(RailType::Powered),
        202 => Some(RailType::Detector),
        203 => Some(RailType::Activator),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Shape determination
// ---------------------------------------------------------------------------

/// Determine the visual shape a rail should adopt based on which cardinal
/// neighbors also contain rails.
///
/// Rules (matching vanilla Minecraft):
/// - Two opposing neighbors (N+S or E+W) -> straight rail.
/// - Two adjacent neighbors -> curve.
/// - Fewer or more than two neighbors -> default to `NorthSouth`.
pub fn determine_rail_shape(
    has_north: bool,
    has_south: bool,
    has_east: bool,
    has_west: bool,
) -> RailShape {
    let count = has_north as u8 + has_south as u8 + has_east as u8 + has_west as u8;

    if count == 2 {
        // Straight rails
        if has_north && has_south {
            return RailShape::NorthSouth;
        }
        if has_east && has_west {
            return RailShape::EastWest;
        }

        // Curves (two adjacent neighbors)
        if has_north && has_east {
            return RailShape::CurveNE;
        }
        if has_south && has_east {
            return RailShape::CurveSE;
        }
        if has_south && has_west {
            return RailShape::CurveSW;
        }
        if has_north && has_west {
            return RailShape::CurveNW;
        }
    }

    // Default for 0, 1, 3, or 4 neighbors.
    RailShape::NorthSouth
}

// ---------------------------------------------------------------------------
// Powered rail mechanics
// ---------------------------------------------------------------------------

/// Maximum speed a minecart can reach on powered rails (blocks/second).
const MAX_RAIL_SPEED: f32 = 8.0;

/// Acceleration applied per tick when on an active powered rail.
const POWERED_ACCEL: f32 = 0.5;

/// Deceleration (braking) applied per tick when on an unpowered powered rail.
const POWERED_BRAKE: f32 = 0.5;

/// Compute the new speed of a minecart sitting on a powered rail.
///
/// - **Powered**: accelerate toward `MAX_RAIL_SPEED`.
/// - **Unpowered**: brake toward 0.
///
/// The returned speed is always non-negative and capped at `MAX_RAIL_SPEED`.
pub fn powered_rail_effect(current_speed: f32, is_powered: bool) -> f32 {
    if is_powered {
        (current_speed + POWERED_ACCEL).min(MAX_RAIL_SPEED)
    } else {
        (current_speed - POWERED_BRAKE).max(0.0)
    }
}

// ---------------------------------------------------------------------------
// Detector rail
// ---------------------------------------------------------------------------

/// Returns the redstone signal strength a detector rail should emit.
///
/// A detector rail outputs a full-strength signal (15) when a minecart is on
/// it, and 0 otherwise.
pub fn detector_rail_signal(has_minecart: bool) -> u8 {
    if has_minecart { 15 } else { 0 }
}

// ---------------------------------------------------------------------------
// Rail network (pathfinding helper)
// ---------------------------------------------------------------------------

/// A lightweight graph of connected rail positions used for minecart
/// pathfinding and rail connectivity queries.
pub struct RailNetwork {
    /// Adjacency list: for each rail position, the set of neighboring rail
    /// positions it connects to.
    adjacency: HashMap<BlockPos, HashSet<BlockPos>>,
}

impl RailNetwork {
    /// Create an empty rail network.
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    /// Register a rail at `pos` and connect it to any existing neighboring
    /// rails in the four cardinal directions.
    pub fn add_rail(&mut self, pos: BlockPos) {
        let neighbors = cardinal_neighbors(pos);
        let connected: Vec<BlockPos> = neighbors
            .iter()
            .copied()
            .filter(|n| self.adjacency.contains_key(n))
            .collect();

        // First pass: register reverse links from neighbors back to `pos`.
        for &n in &connected {
            self.adjacency.entry(n).or_default().insert(pos);
        }

        // Second pass: build the forward adjacency set for `pos`.
        let entry = self.adjacency.entry(pos).or_default();
        for &n in &connected {
            entry.insert(n);
        }
    }

    /// Remove a rail from the network, disconnecting it from its neighbors.
    pub fn remove_rail(&mut self, pos: &BlockPos) {
        if let Some(neighbors) = self.adjacency.remove(pos) {
            for n in neighbors {
                if let Some(adj) = self.adjacency.get_mut(&n) {
                    adj.remove(pos);
                }
            }
        }
    }

    /// Returns the set of positions directly connected to `pos`, or an empty
    /// slice if `pos` is not in the network.
    pub fn neighbors(&self, pos: &BlockPos) -> Vec<BlockPos> {
        self.adjacency
            .get(pos)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// BFS search for a path from `start` to `goal` along connected rails.
    ///
    /// Returns the ordered list of positions (including both endpoints) if a
    /// path exists, or `None` otherwise.
    pub fn find_path(&self, start: BlockPos, goal: BlockPos) -> Option<Vec<BlockPos>> {
        if !self.adjacency.contains_key(&start) || !self.adjacency.contains_key(&goal) {
            return None;
        }

        let mut visited: HashSet<BlockPos> = HashSet::new();
        let mut queue: VecDeque<BlockPos> = VecDeque::new();
        let mut came_from: HashMap<BlockPos, BlockPos> = HashMap::new();

        visited.insert(start);
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if current == goal {
                // Reconstruct path.
                let mut path = vec![goal];
                let mut cursor = goal;
                while cursor != start {
                    cursor = *came_from.get(&cursor)?;
                    path.push(cursor);
                }
                path.reverse();
                return Some(path);
            }

            if let Some(adj) = self.adjacency.get(&current) {
                for &neighbor in adj {
                    if visited.insert(neighbor) {
                        came_from.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }

    /// Returns the number of rail positions in the network.
    pub fn len(&self) -> usize {
        self.adjacency.len()
    }

    /// Returns `true` when the network has no rails.
    pub fn is_empty(&self) -> bool {
        self.adjacency.is_empty()
    }
}

impl Default for RailNetwork {
    fn default() -> Self {
        Self::new()
    }
}

/// Four cardinal neighbor offsets (north, south, east, west) at the same Y.
fn cardinal_neighbors(pos: BlockPos) -> [BlockPos; 4] {
    [
        BlockPos::new(pos.x, pos.y, pos.z - 1), // north (-Z)
        BlockPos::new(pos.x, pos.y, pos.z + 1), // south (+Z)
        BlockPos::new(pos.x + 1, pos.y, pos.z), // east  (+X)
        BlockPos::new(pos.x - 1, pos.y, pos.z), // west  (-X)
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // is_rail identification
    // ------------------------------------------------------------------

    #[test]
    fn is_rail_recognizes_all_rail_ids() {
        assert!(is_rail(200));
        assert!(is_rail(201));
        assert!(is_rail(202));
        assert!(is_rail(203));
    }

    #[test]
    fn is_rail_rejects_non_rail_ids() {
        assert!(!is_rail(0));
        assert!(!is_rail(1));
        assert!(!is_rail(199));
        assert!(!is_rail(204));
        assert!(!is_rail(u16::MAX));
    }

    #[test]
    fn rail_type_from_id_maps_correctly() {
        assert_eq!(rail_type_from_id(200), Some(RailType::Normal));
        assert_eq!(rail_type_from_id(201), Some(RailType::Powered));
        assert_eq!(rail_type_from_id(202), Some(RailType::Detector));
        assert_eq!(rail_type_from_id(203), Some(RailType::Activator));
        assert_eq!(rail_type_from_id(100), None);
    }

    // ------------------------------------------------------------------
    // Shape determination — straight rails
    // ------------------------------------------------------------------

    #[test]
    fn straight_north_south() {
        assert_eq!(
            determine_rail_shape(true, true, false, false),
            RailShape::NorthSouth
        );
    }

    #[test]
    fn straight_east_west() {
        assert_eq!(
            determine_rail_shape(false, false, true, true),
            RailShape::EastWest
        );
    }

    // ------------------------------------------------------------------
    // Shape determination — curves
    // ------------------------------------------------------------------

    #[test]
    fn curve_north_east() {
        assert_eq!(
            determine_rail_shape(true, false, true, false),
            RailShape::CurveNE
        );
    }

    #[test]
    fn curve_south_east() {
        assert_eq!(
            determine_rail_shape(false, true, true, false),
            RailShape::CurveSE
        );
    }

    #[test]
    fn curve_south_west() {
        assert_eq!(
            determine_rail_shape(false, true, false, true),
            RailShape::CurveSW
        );
    }

    #[test]
    fn curve_north_west() {
        assert_eq!(
            determine_rail_shape(true, false, false, true),
            RailShape::CurveNW
        );
    }

    // ------------------------------------------------------------------
    // Shape determination — edge cases
    // ------------------------------------------------------------------

    #[test]
    fn no_neighbors_defaults_to_north_south() {
        assert_eq!(
            determine_rail_shape(false, false, false, false),
            RailShape::NorthSouth
        );
    }

    #[test]
    fn single_neighbor_defaults_to_north_south() {
        assert_eq!(
            determine_rail_shape(true, false, false, false),
            RailShape::NorthSouth
        );
        assert_eq!(
            determine_rail_shape(false, false, true, false),
            RailShape::NorthSouth
        );
    }

    #[test]
    fn three_neighbors_defaults_to_north_south() {
        assert_eq!(
            determine_rail_shape(true, true, true, false),
            RailShape::NorthSouth
        );
    }

    #[test]
    fn four_neighbors_defaults_to_north_south() {
        assert_eq!(
            determine_rail_shape(true, true, true, true),
            RailShape::NorthSouth
        );
    }

    // ------------------------------------------------------------------
    // Powered rail effects
    // ------------------------------------------------------------------

    #[test]
    fn powered_rail_accelerates_toward_max() {
        let speed = powered_rail_effect(0.0, true);
        assert!(
            speed > 0.0,
            "powered rail should accelerate from standstill"
        );
        assert!(speed <= MAX_RAIL_SPEED);
    }

    #[test]
    fn powered_rail_caps_at_max_speed() {
        let speed = powered_rail_effect(7.8, true);
        assert!(speed <= MAX_RAIL_SPEED);

        let speed2 = powered_rail_effect(MAX_RAIL_SPEED, true);
        assert!((speed2 - MAX_RAIL_SPEED).abs() < f32::EPSILON);
    }

    #[test]
    fn unpowered_rail_brakes_toward_zero() {
        let speed = powered_rail_effect(4.0, false);
        assert!(speed < 4.0, "unpowered rail should decelerate");
        assert!(speed >= 0.0);
    }

    #[test]
    fn unpowered_rail_does_not_go_negative() {
        let speed = powered_rail_effect(0.1, false);
        assert!(speed >= 0.0);

        let speed2 = powered_rail_effect(0.0, false);
        assert!((speed2 - 0.0).abs() < f32::EPSILON);
    }

    // ------------------------------------------------------------------
    // Detector rail signal
    // ------------------------------------------------------------------

    #[test]
    fn detector_rail_occupied_emits_15() {
        assert_eq!(detector_rail_signal(true), 15);
    }

    #[test]
    fn detector_rail_empty_emits_0() {
        assert_eq!(detector_rail_signal(false), 0);
    }

    // ------------------------------------------------------------------
    // RailNetwork
    // ------------------------------------------------------------------

    #[test]
    fn empty_network() {
        let net = RailNetwork::new();
        assert!(net.is_empty());
        assert_eq!(net.len(), 0);
    }

    #[test]
    fn add_single_rail() {
        let mut net = RailNetwork::new();
        net.add_rail(BlockPos::new(0, 64, 0));
        assert_eq!(net.len(), 1);
        assert!(!net.is_empty());
    }

    #[test]
    fn adjacent_rails_connect() {
        let mut net = RailNetwork::new();
        let a = BlockPos::new(0, 64, 0);
        let b = BlockPos::new(1, 64, 0); // east of a
        net.add_rail(a);
        net.add_rail(b);

        let a_neighbors = net.neighbors(&a);
        let b_neighbors = net.neighbors(&b);
        assert!(a_neighbors.contains(&b));
        assert!(b_neighbors.contains(&a));
    }

    #[test]
    fn non_adjacent_rails_do_not_connect() {
        let mut net = RailNetwork::new();
        let a = BlockPos::new(0, 64, 0);
        let b = BlockPos::new(5, 64, 0);
        net.add_rail(a);
        net.add_rail(b);

        assert!(net.neighbors(&a).is_empty());
        assert!(net.neighbors(&b).is_empty());
    }

    #[test]
    fn remove_rail_disconnects() {
        let mut net = RailNetwork::new();
        let a = BlockPos::new(0, 64, 0);
        let b = BlockPos::new(1, 64, 0);
        net.add_rail(a);
        net.add_rail(b);

        net.remove_rail(&a);
        assert_eq!(net.len(), 1);
        assert!(net.neighbors(&b).is_empty());
    }

    #[test]
    fn find_path_straight_line() {
        let mut net = RailNetwork::new();
        // Build a 5-block east-west rail line.
        for x in 0..5 {
            net.add_rail(BlockPos::new(x, 64, 0));
        }

        let path = net
            .find_path(BlockPos::new(0, 64, 0), BlockPos::new(4, 64, 0))
            .expect("path should exist");

        assert_eq!(path.len(), 5);
        assert_eq!(path[0], BlockPos::new(0, 64, 0));
        assert_eq!(path[4], BlockPos::new(4, 64, 0));
    }

    #[test]
    fn find_path_returns_none_for_disconnected() {
        let mut net = RailNetwork::new();
        net.add_rail(BlockPos::new(0, 64, 0));
        net.add_rail(BlockPos::new(10, 64, 0));

        assert!(
            net.find_path(BlockPos::new(0, 64, 0), BlockPos::new(10, 64, 0))
                .is_none()
        );
    }

    #[test]
    fn find_path_returns_none_for_missing_nodes() {
        let net = RailNetwork::new();
        assert!(
            net.find_path(BlockPos::new(0, 64, 0), BlockPos::new(1, 64, 0))
                .is_none()
        );
    }

    #[test]
    fn find_path_l_shaped() {
        let mut net = RailNetwork::new();
        // East-west segment: (0,64,0) -> (2,64,0)
        for x in 0..=2 {
            net.add_rail(BlockPos::new(x, 64, 0));
        }
        // Turn south: (2,64,1) -> (2,64,3)
        for z in 1..=3 {
            net.add_rail(BlockPos::new(2, 64, z));
        }

        let path = net
            .find_path(BlockPos::new(0, 64, 0), BlockPos::new(2, 64, 3))
            .expect("L-shaped path should exist");

        assert_eq!(path.len(), 6); // 3 east + 3 south (corner counted once)
        assert_eq!(path[0], BlockPos::new(0, 64, 0));
        assert_eq!(*path.last().unwrap(), BlockPos::new(2, 64, 3));
    }

    #[test]
    fn default_trait_creates_empty_network() {
        let net = RailNetwork::default();
        assert!(net.is_empty());
    }
}
