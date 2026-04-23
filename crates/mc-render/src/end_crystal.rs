//! End crystal rendering: animation state, rotation/bob oscillation, and beam geometry.

use std::f32::consts::PI;

/// Visual state of an End Crystal entity for rendering.
#[derive(Debug, Clone, Copy)]
pub struct EndCrystalState {
    pub position: [f32; 3],
    pub bob_offset: f32,
    pub rotation: f32,
    pub has_base: bool,
}

/// Returns the rotation angle (radians) for the given elapsed time.
///
/// One full rotation (2*PI) every 3 seconds.
pub fn rotation_angle(time: f32) -> f32 {
    (time / 3.0) * 2.0 * PI
}

/// Returns the vertical bob offset for the given elapsed time.
///
/// Sine wave with amplitude 0.1 blocks and period 2 seconds.
pub fn bob_offset(time: f32) -> f32 {
    (time * PI).sin() * 0.1
}

/// Returns `(rotation_angle, bob_y_offset)` for the given elapsed time.
pub fn animate_crystal(time: f32) -> (f32, f32) {
    (rotation_angle(time), bob_offset(time))
}

/// Generates vertices for a beam connecting a crystal position to a target position.
///
/// Returns 8 vertices (2 segments x 4 vertices each) forming quads along the beam axis.
pub fn crystal_beam_vertices(crystal_pos: [f32; 3], target_pos: [f32; 3]) -> Vec<[f32; 3]> {
    let mid = [
        (crystal_pos[0] + target_pos[0]) * 0.5,
        (crystal_pos[1] + target_pos[1]) * 0.5,
        (crystal_pos[2] + target_pos[2]) * 0.5,
    ];

    // Direction vector from crystal to target.
    let dx = target_pos[0] - crystal_pos[0];
    let dy = target_pos[1] - crystal_pos[1];
    let dz = target_pos[2] - crystal_pos[2];
    let length = (dx * dx + dy * dy + dz * dz).sqrt();

    // Half-width of the beam quad perpendicular to the beam direction.
    let half_width: f32 = 0.05;

    // Build a perpendicular vector for the quad width.
    let (perp_x, perp_z) = if length > 1e-6 {
        let inv = half_width / (dx * dx + dz * dz).sqrt().max(1e-6);
        (-dz * inv, dx * inv)
    } else {
        (half_width, 0.0)
    };

    vec![
        // Segment 1: crystal_pos -> mid (4 vertices forming a quad)
        [crystal_pos[0] - perp_x, crystal_pos[1], crystal_pos[2] - perp_z],
        [crystal_pos[0] + perp_x, crystal_pos[1], crystal_pos[2] + perp_z],
        [mid[0] + perp_x, mid[1], mid[2] + perp_z],
        [mid[0] - perp_x, mid[1], mid[2] - perp_z],
        // Segment 2: mid -> target_pos (4 vertices forming a quad)
        [mid[0] - perp_x, mid[1], mid[2] - perp_z],
        [mid[0] + perp_x, mid[1], mid[2] + perp_z],
        [target_pos[0] + perp_x, target_pos[1], target_pos[2] + perp_z],
        [target_pos[0] - perp_x, target_pos[1], target_pos[2] - perp_z],
    ]
}

/// Returns the color of the End Crystal health beam: pink `[1.0, 0.4, 0.8]`.
pub fn crystal_health_beam_color() -> [f32; 3] {
    [1.0, 0.4, 0.8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_completes_full_cycle_in_three_seconds() {
        let angle = rotation_angle(3.0);
        let expected = 2.0 * PI;
        assert!(
            (angle - expected).abs() < 1e-5,
            "expected {expected}, got {angle}"
        );
    }

    #[test]
    fn rotation_at_zero_is_zero() {
        assert!((rotation_angle(0.0)).abs() < 1e-6);
    }

    #[test]
    fn bob_stays_within_range() {
        for i in 0..100 {
            let t = i as f32 * 0.07;
            let offset = bob_offset(t);
            assert!(
                offset >= -0.1 - 1e-6 && offset <= 0.1 + 1e-6,
                "bob_offset({t}) = {offset} is out of range"
            );
        }
    }

    #[test]
    fn bob_period_is_two_seconds() {
        // sin(t * PI) has period 2 (since sin completes a cycle at 2*PI => t = 2).
        let a = bob_offset(0.0);
        let b = bob_offset(2.0);
        assert!(
            (a - b).abs() < 1e-5,
            "bob_offset should repeat every 2 seconds: t=0 -> {a}, t=2 -> {b}"
        );
    }

    #[test]
    fn animate_returns_rotation_and_bob() {
        let (rot, bob) = animate_crystal(1.5);
        assert!((rot - rotation_angle(1.5)).abs() < 1e-6);
        assert!((bob - bob_offset(1.5)).abs() < 1e-6);
    }

    #[test]
    fn beam_vertex_count_is_eight() {
        let verts = crystal_beam_vertices([0.0, 10.0, 0.0], [0.0, 20.0, 0.0]);
        assert_eq!(verts.len(), 8, "expected 8 vertices, got {}", verts.len());
    }

    #[test]
    fn beam_vertices_span_from_crystal_to_target() {
        let crystal = [1.0, 5.0, 3.0];
        let target = [4.0, 15.0, 6.0];
        let verts = crystal_beam_vertices(crystal, target);

        // First segment starts near crystal, second segment ends near target.
        // Check Y coordinates bracket correctly.
        let first_y = verts[0][1];
        let last_y = verts[7][1];
        assert!(
            (first_y - crystal[1]).abs() < 1e-4,
            "first vertex Y should be near crystal Y"
        );
        assert!(
            (last_y - target[1]).abs() < 1e-4,
            "last vertex Y should be near target Y"
        );
    }

    #[test]
    fn health_beam_color_is_pink() {
        let color = crystal_health_beam_color();
        assert!((color[0] - 1.0).abs() < 1e-6);
        assert!((color[1] - 0.4).abs() < 1e-6);
        assert!((color[2] - 0.8).abs() < 1e-6);
    }
}
