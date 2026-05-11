//! Tripwire hook block mechanics — placement, connection detection, and redstone output.

/// A tripwire hook attached to a block face.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripwireHook {
    /// Cardinal direction the hook faces (0=south, 1=west, 2=north, 3=east).
    pub facing: u8,
    /// Whether this hook is connected to another hook via tripwire string.
    pub connected: bool,
    /// Whether the tripwire is currently activated (entity tripped it).
    pub powered: bool,
}

impl TripwireHook {
    /// Create a new tripwire hook facing the given direction, initially disconnected and unpowered.
    pub fn new(facing: u8) -> Self {
        Self {
            facing,
            connected: false,
            powered: false,
        }
    }
}

/// Maximum length of a tripwire line between two hooks (in blocks).
pub fn tripwire_max_length() -> u8 {
    40
}

/// Check whether a tripwire circuit between two hooks should be activated.
///
/// Returns `true` when both hooks exist at valid positions forming a straight
/// line within [`tripwire_max_length`], and an entity is present in the line.
pub fn check_tripwire(
    hook1: (i32, i32, i32),
    hook2: (i32, i32, i32),
    entity_in_line: bool,
) -> bool {
    // Hooks must be at the same Y level
    if hook1.1 != hook2.1 {
        return false;
    }

    // Hooks must form a straight line (same X or same Z)
    let aligned = hook1.0 == hook2.0 || hook1.2 == hook2.2;
    if !aligned {
        return false;
    }

    let dx = (hook2.0 - hook1.0).unsigned_abs();
    let dz = (hook2.2 - hook1.2).unsigned_abs();
    let distance = dx + dz;

    if distance == 0 || distance > tripwire_max_length() as u32 {
        return false;
    }

    entity_in_line
}

/// Redstone signal strength emitted by an activated tripwire hook.
pub fn tripwire_signal_strength() -> u8 {
    15
}

/// Whether tripwire string breaks when cut (e.g. with shears).
pub fn tripwire_breaks_on_cut() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_hook_defaults() {
        let hook = TripwireHook::new(2);
        assert_eq!(hook.facing, 2);
        assert!(!hook.connected);
        assert!(!hook.powered);
    }

    #[test]
    fn max_length_is_40() {
        assert_eq!(tripwire_max_length(), 40);
    }

    #[test]
    fn signal_strength_is_15() {
        assert_eq!(tripwire_signal_strength(), 15);
    }

    #[test]
    fn breaks_on_cut() {
        assert!(tripwire_breaks_on_cut());
    }

    #[test]
    fn check_tripwire_aligned_with_entity() {
        assert!(check_tripwire((0, 64, 0), (10, 64, 0), true));
    }

    #[test]
    fn check_tripwire_aligned_no_entity() {
        assert!(!check_tripwire((0, 64, 0), (10, 64, 0), false));
    }

    #[test]
    fn check_tripwire_different_y() {
        assert!(!check_tripwire((0, 64, 0), (10, 65, 0), true));
    }

    #[test]
    fn check_tripwire_diagonal_rejected() {
        assert!(!check_tripwire((0, 64, 0), (5, 64, 5), true));
    }

    #[test]
    fn check_tripwire_exceeds_max_length() {
        assert!(!check_tripwire((0, 64, 0), (41, 64, 0), true));
    }

    #[test]
    fn check_tripwire_at_max_length() {
        assert!(check_tripwire((0, 64, 0), (40, 64, 0), true));
    }

    #[test]
    fn check_tripwire_same_position_rejected() {
        assert!(!check_tripwire((5, 64, 5), (5, 64, 5), true));
    }

    #[test]
    fn check_tripwire_z_aligned() {
        assert!(check_tripwire((3, 64, 0), (3, 64, 20), true));
    }
}
