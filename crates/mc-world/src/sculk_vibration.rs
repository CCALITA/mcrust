//! Sculk sensor vibration mechanics.
//!
//! Defines vibration events, their frequencies, sensor ranges, and activation
//! logic for sculk sensors and calibrated sculk sensors.

/// A vibration event that sculk sensors can detect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VibrationEvent {
    Step,
    Swim,
    BlockBreak,
    BlockPlace,
    FluidPlace,
    FluidPickup,
    EntityDie,
    Eat,
    Projectile,
    Explode,
    Lightning,
    Instrument,
    SculkCharge,
    ItemPickup,
    ItemDrop,
    Teleport,
    ContainerOpen,
    ContainerClose,
    PistonExtend,
    PistonContract,
}

/// Returns the vibration frequency (1-15) for a given event.
///
/// Frequencies match the vanilla Minecraft sculk sensor frequency table.
pub fn vibration_frequency(event: VibrationEvent) -> u8 {
    match event {
        VibrationEvent::Step => 1,
        VibrationEvent::FluidPickup | VibrationEvent::FluidPlace => 2,
        VibrationEvent::Eat => 3,
        VibrationEvent::ContainerOpen | VibrationEvent::ContainerClose => 4,
        VibrationEvent::Swim => 5,
        VibrationEvent::BlockBreak => 6,
        VibrationEvent::BlockPlace => 7,
        VibrationEvent::ItemDrop => 8,
        VibrationEvent::ItemPickup => 9,
        VibrationEvent::PistonExtend | VibrationEvent::PistonContract => 10,
        VibrationEvent::Instrument => 11,
        VibrationEvent::SculkCharge => 12,
        VibrationEvent::Projectile => 13,
        VibrationEvent::EntityDie => 14,
        VibrationEvent::Explode | VibrationEvent::Lightning | VibrationEvent::Teleport => 15,
    }
}

/// Returns the detection range for a normal sculk sensor (8 blocks).
pub fn sculk_sensor_range() -> u8 {
    8
}

/// Returns the detection range for a calibrated sculk sensor (16 blocks).
pub fn calibrated_range() -> u8 {
    16
}

/// Determines whether a sculk sensor should activate for a given event.
///
/// Activates when the distance from the vibration source to the sensor is
/// within the specified range.
pub fn should_sensor_activate(event: VibrationEvent, distance: f32, range: u8) -> bool {
    let _ = vibration_frequency(event);
    distance <= range as f32
}

/// Returns the redstone signal strength for a given vibration frequency.
///
/// The output equals the frequency, clamped to 1-15.
pub fn redstone_output_for_frequency(freq: u8) -> u8 {
    freq.clamp(1, 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_has_frequency_1() {
        assert_eq!(vibration_frequency(VibrationEvent::Step), 1);
    }

    #[test]
    fn fluid_events_have_frequency_2() {
        assert_eq!(vibration_frequency(VibrationEvent::FluidPickup), 2);
        assert_eq!(vibration_frequency(VibrationEvent::FluidPlace), 2);
    }

    #[test]
    fn eat_has_frequency_3() {
        assert_eq!(vibration_frequency(VibrationEvent::Eat), 3);
    }

    #[test]
    fn container_events_have_frequency_4() {
        assert_eq!(vibration_frequency(VibrationEvent::ContainerOpen), 4);
        assert_eq!(vibration_frequency(VibrationEvent::ContainerClose), 4);
    }

    #[test]
    fn swim_has_frequency_5() {
        assert_eq!(vibration_frequency(VibrationEvent::Swim), 5);
    }

    #[test]
    fn block_break_has_frequency_6() {
        assert_eq!(vibration_frequency(VibrationEvent::BlockBreak), 6);
    }

    #[test]
    fn block_place_has_frequency_7() {
        assert_eq!(vibration_frequency(VibrationEvent::BlockPlace), 7);
    }

    #[test]
    fn item_drop_has_frequency_8() {
        assert_eq!(vibration_frequency(VibrationEvent::ItemDrop), 8);
    }

    #[test]
    fn item_pickup_has_frequency_9() {
        assert_eq!(vibration_frequency(VibrationEvent::ItemPickup), 9);
    }

    #[test]
    fn piston_events_have_frequency_10() {
        assert_eq!(vibration_frequency(VibrationEvent::PistonExtend), 10);
        assert_eq!(vibration_frequency(VibrationEvent::PistonContract), 10);
    }

    #[test]
    fn instrument_has_frequency_11() {
        assert_eq!(vibration_frequency(VibrationEvent::Instrument), 11);
    }

    #[test]
    fn sculk_charge_has_frequency_12() {
        assert_eq!(vibration_frequency(VibrationEvent::SculkCharge), 12);
    }

    #[test]
    fn projectile_has_frequency_13() {
        assert_eq!(vibration_frequency(VibrationEvent::Projectile), 13);
    }

    #[test]
    fn entity_die_has_frequency_14() {
        assert_eq!(vibration_frequency(VibrationEvent::EntityDie), 14);
    }

    #[test]
    fn high_frequency_events_have_frequency_15() {
        assert_eq!(vibration_frequency(VibrationEvent::Explode), 15);
        assert_eq!(vibration_frequency(VibrationEvent::Lightning), 15);
        assert_eq!(vibration_frequency(VibrationEvent::Teleport), 15);
    }

    #[test]
    fn sculk_sensor_range_is_8() {
        assert_eq!(sculk_sensor_range(), 8);
    }

    #[test]
    fn calibrated_range_is_16() {
        assert_eq!(calibrated_range(), 16);
    }

    #[test]
    fn sensor_activates_within_range() {
        assert!(should_sensor_activate(VibrationEvent::Step, 5.0, 8));
    }

    #[test]
    fn sensor_activates_at_exact_range() {
        assert!(should_sensor_activate(VibrationEvent::Step, 8.0, 8));
    }

    #[test]
    fn sensor_does_not_activate_beyond_range() {
        assert!(!should_sensor_activate(VibrationEvent::Step, 8.1, 8));
    }

    #[test]
    fn sensor_activates_at_zero_distance() {
        assert!(should_sensor_activate(VibrationEvent::Explode, 0.0, 8));
    }

    #[test]
    fn calibrated_sensor_activates_at_extended_range() {
        assert!(should_sensor_activate(VibrationEvent::Step, 12.0, 16));
    }

    #[test]
    fn calibrated_sensor_rejects_beyond_extended_range() {
        assert!(!should_sensor_activate(VibrationEvent::Step, 16.1, 16));
    }

    #[test]
    fn redstone_output_matches_frequency() {
        assert_eq!(redstone_output_for_frequency(1), 1);
        assert_eq!(redstone_output_for_frequency(7), 7);
        assert_eq!(redstone_output_for_frequency(15), 15);
    }

    #[test]
    fn redstone_output_clamps_zero_to_one() {
        assert_eq!(redstone_output_for_frequency(0), 1);
    }

    #[test]
    fn redstone_output_clamps_above_15() {
        assert_eq!(redstone_output_for_frequency(20), 15);
    }

    #[test]
    fn all_frequencies_produce_valid_redstone_output() {
        let events = [
            VibrationEvent::Step,
            VibrationEvent::Swim,
            VibrationEvent::BlockBreak,
            VibrationEvent::BlockPlace,
            VibrationEvent::FluidPlace,
            VibrationEvent::FluidPickup,
            VibrationEvent::EntityDie,
            VibrationEvent::Eat,
            VibrationEvent::Projectile,
            VibrationEvent::Explode,
            VibrationEvent::Lightning,
            VibrationEvent::Instrument,
            VibrationEvent::SculkCharge,
            VibrationEvent::ItemPickup,
            VibrationEvent::ItemDrop,
            VibrationEvent::Teleport,
            VibrationEvent::ContainerOpen,
            VibrationEvent::ContainerClose,
            VibrationEvent::PistonExtend,
            VibrationEvent::PistonContract,
        ];
        for event in events {
            let freq = vibration_frequency(event);
            let output = redstone_output_for_frequency(freq);
            assert!(output >= 1 && output <= 15, "event {event:?} produced out-of-range output {output}");
        }
    }
}
