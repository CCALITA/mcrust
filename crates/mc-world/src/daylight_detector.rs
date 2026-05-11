//! Daylight detector block that outputs a redstone signal based on sunlight level.

/// A daylight detector block that can be toggled between normal and inverted modes.
#[derive(Debug, Clone, PartialEq)]
pub struct DaylightDetector {
    pub inverted: bool,
}

impl DaylightDetector {
    /// Creates a new daylight detector in normal (non-inverted) mode.
    #[must_use]
    pub fn new() -> Self {
        Self { inverted: false }
    }
}

impl Default for DaylightDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the redstone signal strength (0–15) based on time of day, sky light, and inversion.
#[must_use]
pub fn daylight_signal(time_of_day: f32, sky_light: u8, inverted: bool) -> u8 {
    let base = daylight_at_time(time_of_day);
    let scaled = (base as u16 * sky_light as u16) / 15;
    let normal = scaled.min(15) as u8;
    if inverted {
        inverted_signal(normal)
    } else {
        normal
    }
}

/// Toggles a daylight detector between normal and inverted modes.
pub fn toggle_invert(detector: &mut DaylightDetector) {
    detector.inverted = !detector.inverted;
}

/// Returns the natural daylight level (0–15) for a given time of day.
///
/// `time` is in the range 0.0–24000.0 (Minecraft ticks).
/// Returns 15 at noon (6000), 0 around midnight (18000).
#[must_use]
pub fn daylight_at_time(time: f32) -> u8 {
    let time = time % 24000.0;
    if time < 0.0 {
        return 0;
    }
    // 6000 = noon (max light), 18000 = midnight (no light)
    // Linear ramp: 0→6000 rises 0→15, 6000→12000 falls 15→0, 12000→24000 stays 0
    if time <= 6000.0 {
        let ratio = time / 6000.0;
        (ratio * 15.0).round() as u8
    } else if time <= 12000.0 {
        let ratio = (12000.0 - time) / 6000.0;
        (ratio * 15.0).round() as u8
    } else {
        0
    }
}

/// Returns the inverted signal: `15 - normal`.
#[must_use]
pub fn inverted_signal(normal: u8) -> u8 {
    15u8.saturating_sub(normal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_detector_is_not_inverted() {
        let d = DaylightDetector::new();
        assert!(!d.inverted);
    }

    #[test]
    fn toggle_invert_flips() {
        let mut d = DaylightDetector::new();
        toggle_invert(&mut d);
        assert!(d.inverted);
        toggle_invert(&mut d);
        assert!(!d.inverted);
    }

    #[test]
    fn daylight_at_noon_is_max() {
        assert_eq!(daylight_at_time(6000.0), 15);
    }

    #[test]
    fn daylight_at_midnight_is_zero() {
        assert_eq!(daylight_at_time(18000.0), 0);
    }

    #[test]
    fn daylight_at_dawn() {
        // 3000 = halfway to noon → ~8
        let val = daylight_at_time(3000.0);
        assert_eq!(val, 8); // 3000/6000 * 15 = 7.5 → rounds to 8
    }

    #[test]
    fn daylight_at_dusk() {
        // 9000 = halfway from noon to 12000 → ~8
        let val = daylight_at_time(9000.0);
        assert_eq!(val, 8);
    }

    #[test]
    fn daylight_wraps_around() {
        assert_eq!(daylight_at_time(30000.0), daylight_at_time(6000.0));
    }

    #[test]
    fn inverted_signal_values() {
        assert_eq!(inverted_signal(0), 15);
        assert_eq!(inverted_signal(15), 0);
        assert_eq!(inverted_signal(7), 8);
    }

    #[test]
    fn daylight_signal_normal_full_sky() {
        let sig = daylight_signal(6000.0, 15, false);
        assert_eq!(sig, 15);
    }

    #[test]
    fn daylight_signal_normal_half_sky() {
        let sig = daylight_signal(6000.0, 7, false);
        assert_eq!(sig, 7); // 15*7/15 = 7
    }

    #[test]
    fn daylight_signal_inverted() {
        let sig = daylight_signal(6000.0, 15, true);
        assert_eq!(sig, 0); // inverted: 15-15=0
    }

    #[test]
    fn daylight_signal_at_midnight_inverted() {
        let sig = daylight_signal(18000.0, 15, true);
        assert_eq!(sig, 15); // inverted: 15-0=15
    }

    #[test]
    fn default_trait() {
        let d = DaylightDetector::default();
        assert!(!d.inverted);
    }
}
