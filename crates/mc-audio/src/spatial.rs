//! 3D spatial audio: distance attenuation, stereo panning, and Doppler effect.
//!
//! Provides [`Spatial3D`] to represent the listener's position and orientation,
//! plus free functions to compute volume falloff, left/right panning, and
//! pitch shifts for sound sources moving relative to the listener.

/// Maximum distance (in blocks) at which a sound is still audible.
pub const MAX_AUDIBLE_DISTANCE: f32 = 64.0;

/// Speed of sound in metres per second, used for Doppler calculations.
pub const SOUND_SPEED: f32 = 343.0;

/// Listener state for 3D audio calculations.
#[derive(Debug, Clone, PartialEq)]
pub struct Spatial3D {
    /// World-space position of the listener.
    pub listener_pos: [f32; 3],
    /// Horizontal rotation in radians (0 = +Z, pi/2 = -X).
    pub listener_yaw: f32,
    /// World-space velocity of the listener (used for Doppler).
    pub listener_velocity: [f32; 3],
}

impl Spatial3D {
    /// Creates a new listener with the given position and yaw, at rest.
    pub fn new(pos: [f32; 3], yaw: f32) -> Self {
        Self {
            listener_pos: pos,
            listener_yaw: yaw,
            listener_velocity: [0.0; 3],
        }
    }
}

/// Euclidean distance between two 3D points.
pub fn distance_3d(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Linear distance-based volume attenuation.
///
/// Returns 1.0 when the source is at the listener, 0.0 at `max_range`,
/// and clamps to 0.0 beyond `max_range`.
pub fn attenuate_volume(source_pos: [f32; 3], listener: &Spatial3D, max_range: f32) -> f32 {
    let dist = distance_3d(source_pos, listener.listener_pos);
    (1.0 - dist / max_range).clamp(0.0, 1.0)
}

/// Stereo panning based on the angle between the listener's facing
/// direction and the sound source.
///
/// Returns `(left_volume, right_volume)`:
/// - Front center: `(1.0, 1.0)`
/// - Full left:    `(1.0, 0.3)`
/// - Full right:   `(0.3, 1.0)`
/// - Behind:       interpolated symmetrically
pub fn pan_stereo(source_pos: [f32; 3], listener: &Spatial3D) -> (f32, f32) {
    let dx = source_pos[0] - listener.listener_pos[0];
    let dz = source_pos[2] - listener.listener_pos[2];

    // If the source is at the listener position, return centered.
    let horizontal_dist = (dx * dx + dz * dz).sqrt();
    if horizontal_dist < 1e-6 {
        return (1.0, 1.0);
    }

    // Listener forward direction from yaw (0 = +Z, pi/2 = -X).
    let forward_x = -listener.listener_yaw.sin();
    let forward_z = listener.listener_yaw.cos();

    // Right-hand perpendicular on the XZ plane.
    let right_x = forward_z;
    let right_z = -forward_x;

    // Normalised direction to source.
    let to_source_x = dx / horizontal_dist;
    let to_source_z = dz / horizontal_dist;

    // Dot product with right vector: positive => source is to the right.
    let right_dot = to_source_x * right_x + to_source_z * right_z;

    // Map right_dot [-1, 1] to volume.
    // right_dot = -1 => full left  (left=1.0, right=0.3)
    // right_dot =  0 => center     (left=1.0, right=1.0)
    // right_dot = +1 => full right (left=0.3, right=1.0)
    let min_vol = 0.3_f32;
    let left = 1.0 - (1.0 - min_vol) * right_dot.clamp(0.0, 1.0);
    let right = 1.0 - (1.0 - min_vol) * (-right_dot).clamp(0.0, 1.0);

    (left, right)
}

/// Doppler pitch multiplier.
///
/// Uses the classical Doppler formula:
///
/// ```text
/// pitch = (sound_speed + v_listener_towards) / (sound_speed + v_source_away)
/// ```
///
/// where velocities are projected onto the source-to-listener axis.
///
/// Returns a value > 1.0 when source and listener approach each other,
/// and < 1.0 when they recede.
pub fn doppler_pitch(
    source_pos: [f32; 3],
    source_velocity: [f32; 3],
    listener: &Spatial3D,
    sound_speed: f32,
) -> f32 {
    let dx = listener.listener_pos[0] - source_pos[0];
    let dy = listener.listener_pos[1] - source_pos[1];
    let dz = listener.listener_pos[2] - source_pos[2];

    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist < 1e-6 {
        return 1.0;
    }

    // Unit vector from source towards listener.
    let nx = dx / dist;
    let ny = dy / dist;
    let nz = dz / dist;

    // Listener velocity component towards source (positive = approaching).
    let v_listener = listener.listener_velocity[0] * (-nx)
        + listener.listener_velocity[1] * (-ny)
        + listener.listener_velocity[2] * (-nz);

    // Source velocity component away from listener (positive = receding).
    let v_source = source_velocity[0] * (-nx)
        + source_velocity[1] * (-ny)
        + source_velocity[2] * (-nz);

    (sound_speed + v_listener) / (sound_speed + v_source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // ---- distance_3d ----

    #[test]
    fn distance_same_point_is_zero() {
        let d = distance_3d([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]);
        assert!((d - 0.0).abs() < 1e-6);
    }

    #[test]
    fn distance_along_single_axis() {
        let d = distance_3d([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn distance_3d_full() {
        let d = distance_3d([1.0, 2.0, 3.0], [4.0, 6.0, 3.0]);
        // sqrt(9 + 16 + 0) = 5
        assert!((d - 5.0).abs() < 1e-6);
    }

    // ---- attenuate_volume ----

    #[test]
    fn attenuation_at_zero_distance() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let vol = attenuate_volume([0.0, 0.0, 0.0], &listener, MAX_AUDIBLE_DISTANCE);
        assert!((vol - 1.0).abs() < 1e-6);
    }

    #[test]
    fn attenuation_at_half_distance() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let vol = attenuate_volume([32.0, 0.0, 0.0], &listener, MAX_AUDIBLE_DISTANCE);
        assert!((vol - 0.5).abs() < 1e-6);
    }

    #[test]
    fn attenuation_at_max_distance() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let vol = attenuate_volume([64.0, 0.0, 0.0], &listener, MAX_AUDIBLE_DISTANCE);
        assert!((vol - 0.0).abs() < 1e-6);
    }

    #[test]
    fn attenuation_beyond_max_clamps_to_zero() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let vol = attenuate_volume([100.0, 0.0, 0.0], &listener, MAX_AUDIBLE_DISTANCE);
        assert!((vol - 0.0).abs() < 1e-6);
    }

    // ---- pan_stereo ----

    #[test]
    fn pan_front_center() {
        // Listener faces +Z, source is directly ahead.
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let (left, right) = pan_stereo([0.0, 0.0, 10.0], &listener);
        assert!((left - 1.0).abs() < 1e-6, "left={left}");
        assert!((right - 1.0).abs() < 1e-6, "right={right}");
    }

    #[test]
    fn pan_full_right() {
        // Listener faces +Z (yaw=0). Right is +X.
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let (left, right) = pan_stereo([10.0, 0.0, 0.0], &listener);
        assert!((left - 0.3).abs() < 1e-4, "left={left}");
        assert!((right - 1.0).abs() < 1e-4, "right={right}");
    }

    #[test]
    fn pan_full_left() {
        // Listener faces +Z (yaw=0). Left is -X.
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let (left, right) = pan_stereo([-10.0, 0.0, 0.0], &listener);
        assert!((left - 1.0).abs() < 1e-4, "left={left}");
        assert!((right - 0.3).abs() < 1e-4, "right={right}");
    }

    #[test]
    fn pan_behind_center() {
        // Listener faces +Z, source is directly behind (-Z).
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let (left, right) = pan_stereo([0.0, 0.0, -10.0], &listener);
        // Behind center should be symmetric.
        assert!((left - right).abs() < 1e-6, "left={left}, right={right}");
    }

    #[test]
    fn pan_source_at_listener_is_centered() {
        let listener = Spatial3D::new([5.0, 3.0, 7.0], 1.2);
        let (left, right) = pan_stereo([5.0, 3.0, 7.0], &listener);
        assert!((left - 1.0).abs() < 1e-6);
        assert!((right - 1.0).abs() < 1e-6);
    }

    // ---- doppler_pitch ----

    #[test]
    fn doppler_stationary_is_unity() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let pitch = doppler_pitch([10.0, 0.0, 0.0], [0.0, 0.0, 0.0], &listener, SOUND_SPEED);
        assert!((pitch - 1.0).abs() < 1e-6);
    }

    #[test]
    fn doppler_approaching_raises_pitch() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        // Source at +X, moving towards listener (-X velocity).
        let pitch = doppler_pitch([10.0, 0.0, 0.0], [-10.0, 0.0, 0.0], &listener, SOUND_SPEED);
        assert!(pitch > 1.0, "pitch={pitch}");
    }

    #[test]
    fn doppler_receding_lowers_pitch() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        // Source at +X, moving away from listener (+X velocity).
        let pitch = doppler_pitch([10.0, 0.0, 0.0], [10.0, 0.0, 0.0], &listener, SOUND_SPEED);
        assert!(pitch < 1.0, "pitch={pitch}");
    }

    #[test]
    fn doppler_listener_approaching_raises_pitch() {
        let mut listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        listener.listener_velocity = [10.0, 0.0, 0.0]; // Moving towards source at +X.
        let pitch = doppler_pitch([10.0, 0.0, 0.0], [0.0, 0.0, 0.0], &listener, SOUND_SPEED);
        assert!(pitch > 1.0, "pitch={pitch}");
    }

    #[test]
    fn doppler_source_at_listener_is_unity() {
        let listener = Spatial3D::new([5.0, 5.0, 5.0], 0.0);
        let pitch = doppler_pitch([5.0, 5.0, 5.0], [10.0, 0.0, 0.0], &listener, SOUND_SPEED);
        assert!((pitch - 1.0).abs() < 1e-6);
    }

    #[test]
    fn doppler_symmetric_approach_and_recede() {
        let listener = Spatial3D::new([0.0, 0.0, 0.0], 0.0);
        let speed = 20.0;
        let approaching =
            doppler_pitch([50.0, 0.0, 0.0], [-speed, 0.0, 0.0], &listener, SOUND_SPEED);
        let receding =
            doppler_pitch([50.0, 0.0, 0.0], [speed, 0.0, 0.0], &listener, SOUND_SPEED);
        // approaching * receding should be close to (c/(c+v)) * (c/(c-v)) pattern, not 1.0 exactly
        // but approaching > 1 and receding < 1.
        assert!(approaching > 1.0);
        assert!(receding < 1.0);
    }

    // ---- Spatial3D construction ----

    #[test]
    fn new_sets_velocity_to_zero() {
        let s = Spatial3D::new([1.0, 2.0, 3.0], PI);
        assert_eq!(s.listener_velocity, [0.0, 0.0, 0.0]);
        assert_eq!(s.listener_pos, [1.0, 2.0, 3.0]);
        assert!((s.listener_yaw - PI).abs() < 1e-6);
    }
}
