//! Compass and clock item mechanics.
//!
//! Provides angle calculations for compass pointing, lodestone compass targeting,
//! clock animation frames, and dimension-aware compass spinning behavior.

use std::f32::consts::{PI, TAU};

/// Number of compass animation frames for a full rotation.
const COMPASS_FRAMES: u8 = 32;

/// Number of clock animation frames for a full day cycle.
const CLOCK_FRAMES: u8 = 64;

/// Nether dimension ID.
const DIMENSION_NETHER: u8 = 1;

/// End dimension ID.
const DIMENSION_END: u8 = 2;

/// Calculate the angle in radians from the player's facing direction to a target position.
///
/// Uses atan2 to determine the direction from the player to the target,
/// then subtracts the player's yaw to get the relative angle.
///
/// Returns an angle in radians, normalized to `[-PI, PI)`.
pub fn compass_angle(
    player_x: f32,
    player_z: f32,
    player_yaw: f32,
    target_x: f32,
    target_z: f32,
) -> f32 {
    let dx = target_x - player_x;
    let dz = target_z - player_z;
    let world_angle = dz.atan2(dx);
    let relative = world_angle - player_yaw;
    normalize_angle(relative)
}

/// Map a compass angle to an animation frame index (0..31).
///
/// Divides the full rotation into 32 equal segments. Frame 0 corresponds to
/// the direction the compass is pointing straight ahead.
pub fn compass_frame(angle: f32) -> u8 {
    // Normalize to [0, TAU)
    let normalized = ((angle % TAU) + TAU) % TAU;
    let raw = normalized / TAU * f32::from(COMPASS_FRAMES);
    (raw.round() as u8) % COMPASS_FRAMES
}

/// Map a time-of-day value (0.0..1.0) to a clock animation frame (0..63).
///
/// The full day cycle is divided into 64 equal segments.
pub fn clock_frame(time_of_day: f32) -> u8 {
    let clamped = time_of_day.clamp(0.0, 1.0);
    let raw = clamped * f32::from(CLOCK_FRAMES);
    (raw.round() as u8) % CLOCK_FRAMES
}

/// Return a human-readable name for the time of day.
///
/// Time thresholds:
/// - 0.0  => "Dawn"
/// - 0.1  => "Morning"
/// - 0.25 => "Noon"
/// - 0.4  => "Afternoon"
/// - 0.5  => "Dusk"
/// - 0.6  => "Evening"
/// - 0.75 => "Midnight"
/// - 0.85 => "Night"
pub fn clock_time_name(time_of_day: f32) -> &'static str {
    let t = time_of_day.clamp(0.0, 1.0);
    if t < 0.1 {
        "Dawn"
    } else if t < 0.25 {
        "Morning"
    } else if t < 0.4 {
        "Noon"
    } else if t < 0.5 {
        "Afternoon"
    } else if t < 0.6 {
        "Dusk"
    } else if t < 0.75 {
        "Evening"
    } else if t < 0.85 {
        "Midnight"
    } else {
        "Night"
    }
}

/// Calculate the compass angle pointing toward a lodestone position.
///
/// Behaves identically to [`compass_angle`] but is semantically distinct
/// for lodestone-bound compasses.
pub fn lodestone_compass_angle(
    player_x: f32,
    player_z: f32,
    player_yaw: f32,
    lodestone_x: f32,
    lodestone_z: f32,
) -> f32 {
    compass_angle(player_x, player_z, player_yaw, lodestone_x, lodestone_z)
}

/// Returns `true` if a compass spins randomly in the given dimension.
///
/// Compasses spin in the Nether (dimension 1) and the End (dimension 2).
pub fn compass_spins_in_dimension(dimension: u8) -> bool {
    dimension == DIMENSION_NETHER || dimension == DIMENSION_END
}

/// Normalize an angle to the range `[-PI, PI)`.
fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle % TAU;
    if a >= PI {
        a -= TAU;
    } else if a < -PI {
        a += TAU;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const EPSILON: f32 = 1e-5;

    #[test]
    fn compass_pointing_north_when_facing_south() {
        // Target is at north (negative Z in Minecraft convention, but here
        // we use a simple 2D plane). Target directly north: same X, Z < player.
        // Player facing south: yaw = -PI/2 (pointing in -Z direction).
        // With our atan2(dz, dx) convention:
        //   dx = 0, dz = -10 => world_angle = atan2(-10, 0) = -PI/2
        //   relative = -PI/2 - (-PI/2) = 0 ... but let's use a clearer scenario.

        // Player at origin, facing south (yaw = PI).
        // Target at (0, -10) — due north.
        // world_angle = atan2(-10, 0) = -PI/2
        // relative = -PI/2 - PI = -3PI/2 => normalized to PI/2
        let angle = compass_angle(0.0, 0.0, PI, 0.0, -10.0);
        assert!(
            (angle - FRAC_PI_2).abs() < EPSILON,
            "Expected PI/2, got {angle}"
        );
    }

    #[test]
    fn compass_pointing_straight_ahead() {
        // Player at origin, facing east (yaw = 0). Target at (10, 0) — due east.
        // world_angle = atan2(0, 10) = 0
        // relative = 0 - 0 = 0
        let angle = compass_angle(0.0, 0.0, 0.0, 10.0, 0.0);
        assert!(angle.abs() < EPSILON, "Expected 0, got {angle}");
    }

    #[test]
    fn compass_frame_north() {
        // angle 0 => frame 0
        assert_eq!(compass_frame(0.0), 0);
    }

    #[test]
    fn compass_frame_halfway() {
        // angle PI => frame 16 (halfway around)
        assert_eq!(compass_frame(PI), 16);
    }

    #[test]
    fn compass_frame_wraps() {
        // Full rotation should wrap to frame 0
        assert_eq!(compass_frame(TAU), 0);
    }

    #[test]
    fn clock_at_noon_is_frame_16() {
        // 0.25 * 64 = 16
        assert_eq!(clock_frame(0.25), 16);
    }

    #[test]
    fn clock_at_midnight_is_frame_48() {
        // 0.75 * 64 = 48
        assert_eq!(clock_frame(0.75), 48);
    }

    #[test]
    fn clock_at_start_is_frame_0() {
        assert_eq!(clock_frame(0.0), 0);
    }

    #[test]
    fn clock_at_end_clamps() {
        // 1.0 should clamp; 1.0 * 64 = 64 => 64 % 64 = 0
        assert_eq!(clock_frame(1.0), 0);
    }

    #[test]
    fn time_name_dawn() {
        assert_eq!(clock_time_name(0.0), "Dawn");
        assert_eq!(clock_time_name(0.05), "Dawn");
    }

    #[test]
    fn time_name_morning() {
        assert_eq!(clock_time_name(0.1), "Morning");
        assert_eq!(clock_time_name(0.2), "Morning");
    }

    #[test]
    fn time_name_noon() {
        assert_eq!(clock_time_name(0.25), "Noon");
        assert_eq!(clock_time_name(0.35), "Noon");
    }

    #[test]
    fn time_name_afternoon() {
        assert_eq!(clock_time_name(0.4), "Afternoon");
        assert_eq!(clock_time_name(0.45), "Afternoon");
    }

    #[test]
    fn time_name_dusk() {
        assert_eq!(clock_time_name(0.5), "Dusk");
        assert_eq!(clock_time_name(0.55), "Dusk");
    }

    #[test]
    fn time_name_evening() {
        assert_eq!(clock_time_name(0.6), "Evening");
        assert_eq!(clock_time_name(0.7), "Evening");
    }

    #[test]
    fn time_name_midnight() {
        assert_eq!(clock_time_name(0.75), "Midnight");
        assert_eq!(clock_time_name(0.8), "Midnight");
    }

    #[test]
    fn time_name_night() {
        assert_eq!(clock_time_name(0.85), "Night");
        assert_eq!(clock_time_name(0.95), "Night");
    }

    #[test]
    fn nether_compass_spins() {
        assert!(compass_spins_in_dimension(1));
    }

    #[test]
    fn end_compass_spins() {
        assert!(compass_spins_in_dimension(2));
    }

    #[test]
    fn overworld_compass_does_not_spin() {
        assert!(!compass_spins_in_dimension(0));
    }

    #[test]
    fn lodestone_compass_matches_regular() {
        let regular = compass_angle(5.0, 5.0, 1.0, 10.0, 20.0);
        let lodestone = lodestone_compass_angle(5.0, 5.0, 1.0, 10.0, 20.0);
        assert!((regular - lodestone).abs() < EPSILON);
    }
}
