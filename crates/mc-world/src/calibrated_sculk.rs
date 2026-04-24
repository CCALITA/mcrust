/// Calibrated sculk sensor — a directional sculk sensor that filters vibrations
/// by frequency and emits a redstone signal when activated.

/// Standard detection range for a calibrated sculk sensor (blocks).
pub fn calibrated_range() -> u8 {
    16
}

/// Standard cooldown duration after activation (game ticks).
pub fn standard_cooldown_ticks() -> u32 {
    10
}

/// A calibrated sculk sensor that responds only to a specific vibration frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalibratedSculkSensor {
    /// Whether the sensor is currently active (emitting a redstone signal).
    pub active: bool,
    /// The vibration frequency this sensor passes through (1..=15).
    pub filtered_frequency: u8,
    /// Remaining cooldown ticks before the sensor can activate again.
    pub cooldown: u32,
    /// Direction the sensor is facing (0..5 for the six cardinal directions).
    pub facing: u8,
}

impl CalibratedSculkSensor {
    /// Creates a new calibrated sculk sensor facing the given direction and
    /// filtering the given frequency.  Starts inactive with no cooldown.
    pub fn new(facing: u8, frequency: u8) -> Self {
        Self {
            active: false,
            filtered_frequency: frequency,
            cooldown: 0,
            facing,
        }
    }
}

/// Returns `true` if the sensor should activate for a vibration event.
///
/// Conditions:
/// - `event_freq` must match the sensor's `filtered_frequency`
/// - `distance` must be within [`calibrated_range`]
/// - sensor must not be on cooldown
/// - sensor must not already be active
pub fn should_activate(sensor: &CalibratedSculkSensor, event_freq: u8, distance: f32) -> bool {
    event_freq == sensor.filtered_frequency
        && distance <= calibrated_range() as f32
        && sensor.cooldown == 0
        && !sensor.active
}

/// Activates the sensor, setting it active and starting the standard cooldown.
pub fn activate(sensor: &mut CalibratedSculkSensor) {
    sensor.active = true;
    sensor.cooldown = standard_cooldown_ticks();
}

/// Advances the sensor by `dt_ticks` game ticks.
///
/// Decrements the cooldown and deactivates the sensor when the cooldown reaches
/// zero.  Returns `true` if the sensor just deactivated during this tick.
pub fn tick_sensor(sensor: &mut CalibratedSculkSensor, dt_ticks: u32) -> bool {
    if sensor.cooldown == 0 {
        return false;
    }

    if sensor.cooldown <= dt_ticks {
        sensor.cooldown = 0;
        sensor.active = false;
        return true;
    }

    sensor.cooldown -= dt_ticks;
    false
}

/// Returns the redstone signal output of the sensor: 15 when active, 0 otherwise.
pub fn sensor_redstone_output(sensor: &CalibratedSculkSensor) -> u8 {
    if sensor.active { 15 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Construction ----

    #[test]
    fn new_sensor_starts_inactive() {
        let sensor = CalibratedSculkSensor::new(2, 5);
        assert!(!sensor.active);
        assert_eq!(sensor.cooldown, 0);
        assert_eq!(sensor.facing, 2);
        assert_eq!(sensor.filtered_frequency, 5);
    }

    // ---- Constants ----

    #[test]
    fn calibrated_range_is_16() {
        assert_eq!(calibrated_range(), 16);
    }

    #[test]
    fn standard_cooldown_is_10() {
        assert_eq!(standard_cooldown_ticks(), 10);
    }

    // ---- should_activate ----

    #[test]
    fn activates_on_matching_frequency_in_range() {
        let sensor = CalibratedSculkSensor::new(0, 8);
        assert!(should_activate(&sensor, 8, 10.0));
    }

    #[test]
    fn rejects_mismatched_frequency() {
        let sensor = CalibratedSculkSensor::new(0, 8);
        assert!(!should_activate(&sensor, 7, 10.0));
    }

    #[test]
    fn rejects_out_of_range_distance() {
        let sensor = CalibratedSculkSensor::new(0, 8);
        assert!(!should_activate(&sensor, 8, 17.0));
    }

    #[test]
    fn accepts_exact_range_boundary() {
        let sensor = CalibratedSculkSensor::new(0, 8);
        assert!(should_activate(&sensor, 8, 16.0));
    }

    #[test]
    fn rejects_when_on_cooldown() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        sensor.cooldown = 3;
        assert!(!should_activate(&sensor, 8, 5.0));
    }

    #[test]
    fn rejects_when_already_active() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        sensor.active = true;
        assert!(!should_activate(&sensor, 8, 5.0));
    }

    // ---- activate ----

    #[test]
    fn activate_sets_active_and_cooldown() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        activate(&mut sensor);
        assert!(sensor.active);
        assert_eq!(sensor.cooldown, standard_cooldown_ticks());
    }

    // ---- tick_sensor ----

    #[test]
    fn tick_decrements_cooldown() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        activate(&mut sensor);
        let deactivated = tick_sensor(&mut sensor, 1);
        assert!(!deactivated);
        assert_eq!(sensor.cooldown, standard_cooldown_ticks() - 1);
        assert!(sensor.active);
    }

    #[test]
    fn tick_deactivates_when_cooldown_expires() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        activate(&mut sensor);
        let deactivated = tick_sensor(&mut sensor, standard_cooldown_ticks());
        assert!(deactivated);
        assert!(!sensor.active);
        assert_eq!(sensor.cooldown, 0);
    }

    #[test]
    fn tick_deactivates_when_dt_exceeds_cooldown() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        activate(&mut sensor);
        let deactivated = tick_sensor(&mut sensor, standard_cooldown_ticks() + 5);
        assert!(deactivated);
        assert!(!sensor.active);
        assert_eq!(sensor.cooldown, 0);
    }

    #[test]
    fn tick_noop_when_no_cooldown() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        let deactivated = tick_sensor(&mut sensor, 1);
        assert!(!deactivated);
    }

    // ---- sensor_redstone_output ----

    #[test]
    fn output_is_15_when_active() {
        let mut sensor = CalibratedSculkSensor::new(0, 8);
        activate(&mut sensor);
        assert_eq!(sensor_redstone_output(&sensor), 15);
    }

    #[test]
    fn output_is_0_when_inactive() {
        let sensor = CalibratedSculkSensor::new(0, 8);
        assert_eq!(sensor_redstone_output(&sensor), 0);
    }

    // ---- Full lifecycle ----

    #[test]
    fn full_activation_lifecycle() {
        let mut sensor = CalibratedSculkSensor::new(1, 12);

        // Should activate for matching event
        assert!(should_activate(&sensor, 12, 8.0));
        activate(&mut sensor);
        assert_eq!(sensor_redstone_output(&sensor), 15);

        // Should not activate again while active
        assert!(!should_activate(&sensor, 12, 8.0));

        // Tick through cooldown
        for _ in 0..9 {
            let deactivated = tick_sensor(&mut sensor, 1);
            assert!(!deactivated);
            assert!(sensor.active);
        }

        // Final tick deactivates
        let deactivated = tick_sensor(&mut sensor, 1);
        assert!(deactivated);
        assert!(!sensor.active);
        assert_eq!(sensor_redstone_output(&sensor), 0);

        // Can activate again
        assert!(should_activate(&sensor, 12, 8.0));
    }
}
