// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Suspicion level at which the warden becomes hostile and attacks.
pub const ANGER_THRESHOLD: u8 = 150;

/// Damage dealt by the warden's sonic boom attack.
const SONIC_BOOM_DAMAGE: f32 = 10.0;

/// Maximum range (blocks) at which the warden can detect vibrations.
const DETECTION_RANGE: f32 = 16.0;

/// Maximum darkness intensity produced by the pulsing effect.
const DARKNESS_MAX_INTENSITY: f32 = 0.8;

// ---------------------------------------------------------------------------
// WardenAnger
// ---------------------------------------------------------------------------

/// Tracks the warden's suspicion and anger state toward nearby entities.
#[derive(Debug, Clone, PartialEq)]
pub struct WardenAnger {
    /// Current suspicion level (0–255). Attacks trigger at [`ANGER_THRESHOLD`].
    pub suspicion: u8,
    /// Entity id the warden is currently targeting, if any.
    pub anger_target: Option<u64>,
    /// Whether the warden is still emerging from the ground.
    pub emerging: bool,
}

impl WardenAnger {
    /// Create a new warden anger state with zero suspicion and no target.
    pub fn new() -> Self {
        Self {
            suspicion: 0,
            anger_target: None,
            emerging: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Detection helpers
// ---------------------------------------------------------------------------

/// Add `amount` to the warden's suspicion, clamping at `u8::MAX`.
///
/// Returns `true` when suspicion reaches or exceeds [`ANGER_THRESHOLD`],
/// meaning the warden should attack.
pub fn add_suspicion(state: &mut WardenAnger, amount: u8) -> bool {
    state.suspicion = state.suspicion.saturating_add(amount);
    state.suspicion >= ANGER_THRESHOLD
}

/// Compute a sin-based darkness pulse intensity in the range `0.0..=0.8`.
///
/// `time` is an arbitrary elapsed-time value in seconds; the pulse cycles
/// smoothly using a sine wave.
pub fn darkness_pulse(time: f32) -> f32 {
    let raw = (time.sin() + 1.0) * 0.5; // 0.0..=1.0
    raw * DARKNESS_MAX_INTENSITY
}

/// Damage dealt by the warden's sonic boom attack.
pub fn sonic_boom_damage() -> f32 {
    SONIC_BOOM_DAMAGE
}

/// Maximum range (blocks) at which the warden detects vibrations.
pub fn warden_detection_range() -> f32 {
    DETECTION_RANGE
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction -------------------------------------------------------

    #[test]
    fn new_warden_anger_has_correct_defaults() {
        let w = WardenAnger::new();

        assert_eq!(w.suspicion, 0);
        assert_eq!(w.anger_target, None);
        assert!(!w.emerging);
    }

    // -- add_suspicion ------------------------------------------------------

    #[test]
    fn add_suspicion_below_threshold_returns_false() {
        let mut w = WardenAnger::new();

        let triggered = add_suspicion(&mut w, 50);

        assert!(!triggered);
        assert_eq!(w.suspicion, 50);
    }

    #[test]
    fn add_suspicion_reaching_threshold_returns_true() {
        let mut w = WardenAnger::new();

        let triggered = add_suspicion(&mut w, ANGER_THRESHOLD);

        assert!(triggered);
        assert_eq!(w.suspicion, ANGER_THRESHOLD);
    }

    #[test]
    fn add_suspicion_exceeding_threshold_returns_true() {
        let mut w = WardenAnger::new();
        w.suspicion = 100;

        let triggered = add_suspicion(&mut w, 60);

        assert!(triggered);
        assert_eq!(w.suspicion, 160);
    }

    #[test]
    fn add_suspicion_saturates_at_u8_max() {
        let mut w = WardenAnger::new();
        w.suspicion = 200;

        let triggered = add_suspicion(&mut w, 200);

        assert!(triggered);
        assert_eq!(w.suspicion, u8::MAX);
    }

    #[test]
    fn add_suspicion_accumulates_across_calls() {
        let mut w = WardenAnger::new();

        add_suspicion(&mut w, 40);
        add_suspicion(&mut w, 40);
        let triggered = add_suspicion(&mut w, 40);

        assert!(!triggered);
        assert_eq!(w.suspicion, 120);
    }

    // -- darkness_pulse -----------------------------------------------------

    #[test]
    fn darkness_pulse_at_zero_is_half_max() {
        // sin(0) = 0 → (0+1)*0.5 = 0.5 → 0.5 * 0.8 = 0.4
        let v = darkness_pulse(0.0);
        assert!((v - 0.4).abs() < 1e-5);
    }

    #[test]
    fn darkness_pulse_at_pi_half_is_max() {
        // sin(π/2) = 1 → (1+1)*0.5 = 1.0 → 1.0 * 0.8 = 0.8
        let v = darkness_pulse(std::f32::consts::FRAC_PI_2);
        assert!((v - 0.8).abs() < 1e-5);
    }

    #[test]
    fn darkness_pulse_at_three_pi_half_is_zero() {
        // sin(3π/2) = -1 → (-1+1)*0.5 = 0.0 → 0.0 * 0.8 = 0.0
        let v = darkness_pulse(3.0 * std::f32::consts::FRAC_PI_2);
        assert!(v.abs() < 1e-5);
    }

    #[test]
    fn darkness_pulse_stays_in_range() {
        for i in 0..1000 {
            let t = i as f32 * 0.01;
            let v = darkness_pulse(t);
            assert!(v >= 0.0 && v <= 0.8 + 1e-6, "out of range at t={t}: {v}");
        }
    }

    // -- sonic_boom_damage --------------------------------------------------

    #[test]
    fn sonic_boom_damage_is_10() {
        assert!((sonic_boom_damage() - 10.0).abs() < f32::EPSILON);
    }

    // -- warden_detection_range ---------------------------------------------

    #[test]
    fn detection_range_is_16() {
        assert!((warden_detection_range() - 16.0).abs() < f32::EPSILON);
    }

    // -- ANGER_THRESHOLD constant -------------------------------------------

    #[test]
    fn anger_threshold_is_150() {
        assert_eq!(ANGER_THRESHOLD, 150);
    }
}
