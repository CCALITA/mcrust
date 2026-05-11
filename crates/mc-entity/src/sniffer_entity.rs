//! Sniffer mob entity — ancient passive mob that digs for seeds.

/// Base health for a sniffer.
pub fn sniffer_base_health() -> f32 {
    14.0
}

/// Walking speed for a sniffer (blocks per second).
pub fn sniffer_walk_speed() -> f32 {
    0.1
}

/// Sniffers always produce an egg when breeding (instead of a baby).
pub fn sniffer_breeding_always_egg() -> bool {
    true
}

/// Runtime state for a sniffer mob.
#[derive(Debug, Clone, PartialEq)]
pub struct SnifferMobState {
    pub health: f32,
    pub digging: bool,
    pub happy: bool,
}

impl SnifferMobState {
    /// Create a new sniffer with default health and idle state.
    pub fn new() -> Self {
        Self {
            health: sniffer_base_health(),
            digging: false,
            happy: false,
        }
    }
}

/// Tick the sniffer's digging behavior.
///
/// If `near_valid_block` is true and the sniffer is not already digging, it
/// begins sniffing. After accumulating enough time (`dt`), it completes a dig
/// and returns the block position `(0, -1, 0)` relative to the sniffer as the
/// dug location. Returns `None` when no dig completes this tick.
pub fn tick_sniffer(
    state: &mut SnifferMobState,
    dt: f32,
    near_valid_block: bool,
) -> Option<(i32, i32, i32)> {
    if state.health <= 0.0 {
        state.digging = false;
        return None;
    }

    if near_valid_block && !state.digging {
        state.digging = true;
    }

    if state.digging {
        // Simplified: complete dig after accumulating 2 seconds of dt
        if dt >= 2.0 {
            state.digging = false;
            state.happy = true;
            return Some((0, -1, 0));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let s = SnifferMobState::new();
        assert_eq!(s.health, 14.0);
        assert!(!s.digging);
        assert!(!s.happy);
    }

    #[test]
    fn test_base_health() {
        assert_eq!(sniffer_base_health(), 14.0);
    }

    #[test]
    fn test_walk_speed() {
        assert_eq!(sniffer_walk_speed(), 0.1);
    }

    #[test]
    fn test_breeding_always_egg() {
        assert!(sniffer_breeding_always_egg());
    }

    #[test]
    fn test_tick_starts_digging() {
        let mut s = SnifferMobState::new();
        let result = tick_sniffer(&mut s, 0.5, true);
        assert!(s.digging);
        assert_eq!(result, None);
    }

    #[test]
    fn test_tick_completes_dig() {
        let mut s = SnifferMobState::new();
        s.digging = true;
        let result = tick_sniffer(&mut s, 2.0, true);
        assert_eq!(result, Some((0, -1, 0)));
        assert!(!s.digging);
        assert!(s.happy);
    }

    #[test]
    fn test_tick_dead_sniffer() {
        let mut s = SnifferMobState::new();
        s.health = 0.0;
        s.digging = true;
        let result = tick_sniffer(&mut s, 2.0, true);
        assert_eq!(result, None);
        assert!(!s.digging);
    }

    #[test]
    fn test_tick_no_valid_block() {
        let mut s = SnifferMobState::new();
        let result = tick_sniffer(&mut s, 2.0, false);
        assert_eq!(result, None);
        assert!(!s.digging);
    }
}
