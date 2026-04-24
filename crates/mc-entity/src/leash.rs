// ---------------------------------------------------------------------------
// Leash mechanics and llama caravan system
// ---------------------------------------------------------------------------

/// Maximum distance (in blocks) before a leash breaks.
const LEASH_BREAK_DISTANCE: f32 = 10.0;

/// Maximum number of llamas allowed in a single caravan chain.
const DEFAULT_MAX_CHAIN: u8 = 10;

/// Base speed multiplier applied to all caravan members.
const CARAVAN_BASE_MULTIPLIER: f32 = 1.0;

/// Per-link speed reduction for each position further back in the chain.
const CARAVAN_SPEED_REDUCTION_PER_LINK: f32 = 0.05;

/// Minimum speed multiplier so trailing llamas never stop entirely.
const CARAVAN_MIN_MULTIPLIER: f32 = 0.5;

// ---------------------------------------------------------------------------
// Leash state
// ---------------------------------------------------------------------------

/// Tracks whether an entity is leashed and to which entity.
#[derive(Debug, Clone, PartialEq)]
pub struct LeashState {
    /// Entity id of the entity this mob is leashed to (`None` if unleashed).
    pub leashed_to: Option<u64>,
    /// Maximum distance in blocks before the leash breaks.
    pub max_distance: f32,
}

impl LeashState {
    /// Create a new unleashed state with the default break distance.
    pub fn new() -> Self {
        Self {
            leashed_to: None,
            max_distance: LEASH_BREAK_DISTANCE,
        }
    }
}

impl Default for LeashState {
    fn default() -> Self {
        Self::new()
    }
}

/// Attach a leash from `entity` to `target_id`.
///
/// Overwrites any existing leash. The max distance is preserved.
pub fn attach_leash(state: &mut LeashState, target_id: u64) {
    state.leashed_to = Some(target_id);
}

/// Detach the leash, returning the previously leashed entity id if any.
pub fn detach_leash(state: &mut LeashState) -> Option<u64> {
    state.leashed_to.take()
}

/// Returns the distance at which a leash breaks (10.0 blocks).
pub fn leash_break_distance() -> f32 {
    LEASH_BREAK_DISTANCE
}

// ---------------------------------------------------------------------------
// Llama caravan
// ---------------------------------------------------------------------------

/// A llama caravan: one leader followed by a chain of followers.
///
/// In Minecraft, when a player leashes a llama, nearby llamas may form a
/// caravan behind it, up to a maximum chain length.
#[derive(Debug, Clone, PartialEq)]
pub struct LlamaCaravan {
    /// Entity id of the caravan leader (the leashed llama).
    pub leader: u64,
    /// Ordered list of follower entity ids, from front to back.
    pub followers: Vec<u64>,
    /// Maximum number of followers allowed in this caravan.
    pub max_chain: u8,
}

impl LlamaCaravan {
    /// Create a new caravan with the given leader and default max chain length.
    pub fn new(leader: u64) -> Self {
        Self {
            leader,
            followers: Vec::new(),
            max_chain: DEFAULT_MAX_CHAIN,
        }
    }

    /// Create a new caravan with a custom maximum chain length.
    pub fn with_max_chain(leader: u64, max_chain: u8) -> Self {
        Self {
            leader,
            followers: Vec::new(),
            max_chain,
        }
    }

    /// Returns the total number of llamas in the caravan (leader + followers).
    pub fn len(&self) -> usize {
        1 + self.followers.len()
    }

    /// Returns `true` if the caravan has no followers (leader only).
    pub fn is_empty(&self) -> bool {
        self.followers.is_empty()
    }
}

/// Attempt to add a llama to the caravan.
///
/// Returns `true` if the llama was added, `false` if the caravan is already
/// at maximum capacity or the llama is already the leader.
pub fn attempt_join_caravan(caravan: &mut LlamaCaravan, llama_id: u64) -> bool {
    if llama_id == caravan.leader {
        return false;
    }
    if caravan.followers.len() >= caravan.max_chain as usize {
        return false;
    }
    if caravan.followers.contains(&llama_id) {
        return false;
    }
    caravan.followers.push(llama_id);
    true
}

/// Calculate the speed multiplier for a llama at position `chain_position`
/// in the caravan (0 = leader, 1 = first follower, etc.).
///
/// Followers further back move slightly slower to create the visual chain
/// effect. The multiplier never drops below [`CARAVAN_MIN_MULTIPLIER`].
pub fn caravan_speed_multiplier(chain_position: u8) -> f32 {
    let reduction = chain_position as f32 * CARAVAN_SPEED_REDUCTION_PER_LINK;
    (CARAVAN_BASE_MULTIPLIER - reduction).max(CARAVAN_MIN_MULTIPLIER)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- LeashState -----------------------------------------------------------

    #[test]
    fn new_leash_state_is_unleashed() {
        let state = LeashState::new();
        assert!(state.leashed_to.is_none());
        assert!((state.max_distance - LEASH_BREAK_DISTANCE).abs() < f32::EPSILON);
    }

    #[test]
    fn default_leash_state_matches_new() {
        let a = LeashState::new();
        let b = LeashState::default();
        assert_eq!(a, b);
    }

    // -- attach / detach ------------------------------------------------------

    #[test]
    fn attach_leash_sets_target() {
        let mut state = LeashState::new();
        attach_leash(&mut state, 42);
        assert_eq!(state.leashed_to, Some(42));
    }

    #[test]
    fn attach_leash_overwrites_existing() {
        let mut state = LeashState::new();
        attach_leash(&mut state, 1);
        attach_leash(&mut state, 2);
        assert_eq!(state.leashed_to, Some(2));
    }

    #[test]
    fn detach_leash_returns_previous_target() {
        let mut state = LeashState::new();
        attach_leash(&mut state, 99);
        let prev = detach_leash(&mut state);
        assert_eq!(prev, Some(99));
        assert!(state.leashed_to.is_none());
    }

    #[test]
    fn detach_leash_returns_none_when_unleashed() {
        let mut state = LeashState::new();
        assert_eq!(detach_leash(&mut state), None);
    }

    #[test]
    fn max_distance_preserved_after_attach() {
        let mut state = LeashState::new();
        state.max_distance = 5.0;
        attach_leash(&mut state, 10);
        assert!((state.max_distance - 5.0).abs() < f32::EPSILON);
    }

    // -- leash_break_distance -------------------------------------------------

    #[test]
    fn break_distance_is_ten() {
        assert!((leash_break_distance() - 10.0).abs() < f32::EPSILON);
    }

    // -- LlamaCaravan ---------------------------------------------------------

    #[test]
    fn new_caravan_has_leader_only() {
        let caravan = LlamaCaravan::new(1);
        assert_eq!(caravan.leader, 1);
        assert!(caravan.followers.is_empty());
        assert_eq!(caravan.max_chain, DEFAULT_MAX_CHAIN);
        assert_eq!(caravan.len(), 1);
        assert!(caravan.is_empty());
    }

    #[test]
    fn caravan_with_custom_max_chain() {
        let caravan = LlamaCaravan::with_max_chain(5, 3);
        assert_eq!(caravan.leader, 5);
        assert_eq!(caravan.max_chain, 3);
    }

    // -- attempt_join_caravan -------------------------------------------------

    #[test]
    fn join_caravan_succeeds() {
        let mut caravan = LlamaCaravan::new(1);
        assert!(attempt_join_caravan(&mut caravan, 2));
        assert_eq!(caravan.followers, vec![2]);
        assert_eq!(caravan.len(), 2);
        assert!(!caravan.is_empty());
    }

    #[test]
    fn join_caravan_multiple_followers() {
        let mut caravan = LlamaCaravan::new(1);
        assert!(attempt_join_caravan(&mut caravan, 2));
        assert!(attempt_join_caravan(&mut caravan, 3));
        assert!(attempt_join_caravan(&mut caravan, 4));
        assert_eq!(caravan.followers, vec![2, 3, 4]);
        assert_eq!(caravan.len(), 4);
    }

    #[test]
    fn join_caravan_rejected_when_full() {
        let mut caravan = LlamaCaravan::with_max_chain(1, 2);
        assert!(attempt_join_caravan(&mut caravan, 10));
        assert!(attempt_join_caravan(&mut caravan, 11));
        assert!(!attempt_join_caravan(&mut caravan, 12));
        assert_eq!(caravan.followers.len(), 2);
    }

    #[test]
    fn join_caravan_rejected_for_leader() {
        let mut caravan = LlamaCaravan::new(1);
        assert!(!attempt_join_caravan(&mut caravan, 1));
        assert!(caravan.followers.is_empty());
    }

    #[test]
    fn join_caravan_rejected_for_duplicate() {
        let mut caravan = LlamaCaravan::new(1);
        assert!(attempt_join_caravan(&mut caravan, 2));
        assert!(!attempt_join_caravan(&mut caravan, 2));
        assert_eq!(caravan.followers.len(), 1);
    }

    // -- caravan_speed_multiplier ---------------------------------------------

    #[test]
    fn leader_speed_multiplier_is_one() {
        assert!((caravan_speed_multiplier(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn first_follower_speed_multiplier() {
        let expected = 1.0 - CARAVAN_SPEED_REDUCTION_PER_LINK;
        assert!((caravan_speed_multiplier(1) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_decreases_with_chain_position() {
        let m1 = caravan_speed_multiplier(1);
        let m2 = caravan_speed_multiplier(2);
        let m3 = caravan_speed_multiplier(3);
        assert!(m1 > m2);
        assert!(m2 > m3);
    }

    #[test]
    fn speed_multiplier_never_below_minimum() {
        // Even at extreme chain positions the multiplier is clamped.
        let m = caravan_speed_multiplier(255);
        assert!((m - CARAVAN_MIN_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_at_boundary() {
        // Position where reduction exactly equals 0.5 (1.0 - 0.05 * 10 = 0.5).
        let m = caravan_speed_multiplier(10);
        assert!((m - CARAVAN_MIN_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_just_above_minimum() {
        // Position 9: 1.0 - 0.05 * 9 = 0.55
        let expected = 1.0 - 9.0 * CARAVAN_SPEED_REDUCTION_PER_LINK;
        let m = caravan_speed_multiplier(9);
        assert!((m - expected).abs() < f32::EPSILON);
    }
}
