// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Detection range for ramming targets (blocks).
const RAM_RANGE: f32 = 16.0;

/// Ram cooldown for normal goats (seconds).
const NORMAL_RAM_COOLDOWN: f32 = 30.0;

/// Ram cooldown for screamer goats (seconds).
const SCREAMER_RAM_COOLDOWN: f32 = 5.0;

/// Ram damage for normal goats.
const NORMAL_RAM_DAMAGE: f32 = 1.0;

/// Ram damage for screamer goats.
const SCREAMER_RAM_DAMAGE: f32 = 3.0;

/// Knockback strength for normal goats.
const NORMAL_RAM_KNOCKBACK: f32 = 1.5;

/// Knockback strength for screamer goats.
const SCREAMER_RAM_KNOCKBACK: f32 = 3.0;

// ---------------------------------------------------------------------------
// Goat state
// ---------------------------------------------------------------------------

/// State for a goat entity.
#[derive(Debug, Clone, PartialEq)]
pub struct GoatState {
    /// Whether this goat is a screamer variant.
    pub screamer: bool,
    /// Remaining cooldown before the goat can ram again (seconds).
    pub ram_cooldown: f32,
    /// Whether the goat is currently preparing to ram.
    pub preparing: bool,
}

impl GoatState {
    /// Create a new goat state.
    pub fn new(screamer: bool) -> Self {
        Self {
            screamer,
            ram_cooldown: 0.0,
            preparing: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Pure query functions
// ---------------------------------------------------------------------------

/// Damage dealt by a ram attack.
pub fn ram_damage(screamer: bool) -> f32 {
    if screamer { SCREAMER_RAM_DAMAGE } else { NORMAL_RAM_DAMAGE }
}

/// Knockback strength of a ram attack.
pub fn ram_knockback(screamer: bool) -> f32 {
    if screamer { SCREAMER_RAM_KNOCKBACK } else { NORMAL_RAM_KNOCKBACK }
}

/// Maximum range at which a goat detects ram targets.
pub fn ram_range() -> f32 {
    RAM_RANGE
}

/// Cooldown duration after a ram (seconds).
pub fn ram_cooldown_duration(screamer: bool) -> f32 {
    if screamer { SCREAMER_RAM_COOLDOWN } else { NORMAL_RAM_COOLDOWN }
}

/// Whether ramming a log block drops a goat horn.
pub fn drops_horn_on_ram(hit_log: bool) -> bool {
    hit_log
}

// ---------------------------------------------------------------------------
// Tick
// ---------------------------------------------------------------------------

/// Advance goat AI by one tick.
///
/// Returns `true` when the goat performs a ram this tick.
/// `target_distance` is the distance to the nearest valid ram target
/// (pass `f32::MAX` when no target exists).
pub fn tick_goat(state: &mut GoatState, target_distance: f32, dt: f32) -> bool {
    // Tick down cooldown.
    if state.ram_cooldown > 0.0 {
        state.ram_cooldown = (state.ram_cooldown - dt).max(0.0);
    }

    // Cannot ram while on cooldown.
    if state.ram_cooldown > 0.0 {
        state.preparing = false;
        return false;
    }

    let in_range = target_distance <= RAM_RANGE;

    if !state.preparing && in_range {
        // Begin preparing to ram.
        state.preparing = true;
        return false;
    }

    if state.preparing && in_range {
        // Already preparing — execute the ram.
        state.preparing = false;
        state.ram_cooldown = ram_cooldown_duration(state.screamer);
        return true;
    }

    // Target moved out of range — cancel preparation.
    state.preparing = false;
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction ---------------------------------------------------------

    #[test]
    fn new_normal_goat() {
        let goat = GoatState::new(false);
        assert!(!goat.screamer);
        assert_eq!(goat.ram_cooldown, 0.0);
        assert!(!goat.preparing);
    }

    #[test]
    fn new_screamer_goat() {
        let goat = GoatState::new(true);
        assert!(goat.screamer);
        assert_eq!(goat.ram_cooldown, 0.0);
        assert!(!goat.preparing);
    }

    // -- Damage ---------------------------------------------------------------

    #[test]
    fn normal_ram_damage() {
        assert_eq!(ram_damage(false), 1.0);
    }

    #[test]
    fn screamer_ram_damage() {
        assert_eq!(ram_damage(true), 3.0);
    }

    // -- Knockback ------------------------------------------------------------

    #[test]
    fn normal_ram_knockback() {
        assert_eq!(ram_knockback(false), 1.5);
    }

    #[test]
    fn screamer_ram_knockback() {
        assert_eq!(ram_knockback(true), 3.0);
    }

    // -- Range ----------------------------------------------------------------

    #[test]
    fn ram_range_is_16() {
        assert_eq!(ram_range(), 16.0);
    }

    // -- Cooldown duration ----------------------------------------------------

    #[test]
    fn normal_cooldown_duration() {
        assert_eq!(ram_cooldown_duration(false), 30.0);
    }

    #[test]
    fn screamer_cooldown_duration() {
        assert_eq!(ram_cooldown_duration(true), 5.0);
    }

    // -- Horn drops -----------------------------------------------------------

    #[test]
    fn drops_horn_when_hitting_log() {
        assert!(drops_horn_on_ram(true));
    }

    #[test]
    fn no_horn_when_not_hitting_log() {
        assert!(!drops_horn_on_ram(false));
    }

    // -- Tick: ram lifecycle ---------------------------------------------------

    #[test]
    fn tick_prepares_then_rams() {
        let mut goat = GoatState::new(false);
        let dt = 0.05;

        // First tick in range — starts preparing, does not ram yet.
        let rammed = tick_goat(&mut goat, 10.0, dt);
        assert!(!rammed);
        assert!(goat.preparing);

        // Second tick still in range — executes ram.
        let rammed = tick_goat(&mut goat, 10.0, dt);
        assert!(rammed);
        assert!(!goat.preparing);
        assert!(goat.ram_cooldown > 0.0);
    }

    #[test]
    fn tick_does_not_ram_when_out_of_range() {
        let mut goat = GoatState::new(false);
        let rammed = tick_goat(&mut goat, 20.0, 0.05);
        assert!(!rammed);
        assert!(!goat.preparing);
    }

    #[test]
    fn tick_cancels_preparation_when_target_leaves_range() {
        let mut goat = GoatState::new(false);

        // Start preparing.
        tick_goat(&mut goat, 10.0, 0.05);
        assert!(goat.preparing);

        // Target moves out of range.
        let rammed = tick_goat(&mut goat, 20.0, 0.05);
        assert!(!rammed);
        assert!(!goat.preparing);
    }

    #[test]
    fn tick_respects_cooldown() {
        let mut goat = GoatState::new(false);

        // Prepare and ram.
        tick_goat(&mut goat, 10.0, 0.05);
        tick_goat(&mut goat, 10.0, 0.05);
        assert!(goat.ram_cooldown > 0.0);

        // Immediately try again — should not prepare or ram.
        let rammed = tick_goat(&mut goat, 10.0, 0.05);
        assert!(!rammed);
        assert!(!goat.preparing);
    }

    #[test]
    fn cooldown_ticks_down() {
        let mut goat = GoatState::new(true);

        // Ram to start cooldown.
        tick_goat(&mut goat, 10.0, 0.05);
        tick_goat(&mut goat, 10.0, 0.05);
        let initial_cd = goat.ram_cooldown;

        // Tick with no target — cooldown should decrease.
        tick_goat(&mut goat, f32::MAX, 1.0);
        assert!(goat.ram_cooldown < initial_cd);
    }

    #[test]
    fn cooldown_does_not_go_negative() {
        let mut goat = GoatState::new(true);
        goat.ram_cooldown = 0.5;

        tick_goat(&mut goat, f32::MAX, 10.0);
        assert_eq!(goat.ram_cooldown, 0.0);
    }

    #[test]
    fn screamer_can_ram_again_faster() {
        let mut screamer = GoatState::new(true);
        let mut normal = GoatState::new(false);

        // Both ram.
        tick_goat(&mut screamer, 10.0, 0.05);
        tick_goat(&mut screamer, 10.0, 0.05);
        tick_goat(&mut normal, 10.0, 0.05);
        tick_goat(&mut normal, 10.0, 0.05);

        assert!(screamer.ram_cooldown < normal.ram_cooldown);
    }
}
