/// Target block mechanics — redstone signal output based on hit position.

/// Returns the redstone signal strength (1–15) based on distance from center.
/// At center (0.0) returns 15, linearly decreasing to 1 at edge (0.5).
pub fn target_signal_strength(hit_distance_from_center: f32) -> u8 {
    let clamped = hit_distance_from_center.clamp(0.0, 0.5);
    let strength = 15.0 - (14.0 * clamped / 0.5);
    (strength.round() as u8).max(1)
}

/// Returns how long the target block emits a signal, in ticks.
/// Projectiles: 10 ticks. Other entities: 20 ticks.
pub fn target_signal_duration_ticks(is_projectile: bool) -> u32 {
    if is_projectile { 10 } else { 20 }
}

/// Returns the center position of a target block within its block space.
pub fn target_block_center() -> [f32; 3] {
    [0.5, 0.5, 0.5]
}

/// Computes the distance from a hit position to the center of a block.
pub fn distance_to_center(hit_pos: [f32; 3], block_pos: [f32; 3]) -> f32 {
    let center = [
        block_pos[0] + 0.5,
        block_pos[1] + 0.5,
        block_pos[2] + 0.5,
    ];
    let dx = hit_pos[0] - center[0];
    let dy = hit_pos[1] - center[1];
    let dz = hit_pos[2] - center[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Returns the block ID for the target block.
pub fn target_block_id() -> u16 {
    750
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_strength_at_center_is_15() {
        assert_eq!(target_signal_strength(0.0), 15);
    }

    #[test]
    fn signal_strength_at_edge_is_1() {
        assert_eq!(target_signal_strength(0.5), 1);
    }

    #[test]
    fn signal_strength_midpoint() {
        assert_eq!(target_signal_strength(0.25), 8);
    }

    #[test]
    fn signal_strength_clamps_beyond_edge() {
        assert_eq!(target_signal_strength(1.0), 1);
    }

    #[test]
    fn signal_strength_clamps_negative() {
        assert_eq!(target_signal_strength(-0.1), 15);
    }

    #[test]
    fn duration_projectile() {
        assert_eq!(target_signal_duration_ticks(true), 10);
    }

    #[test]
    fn duration_other() {
        assert_eq!(target_signal_duration_ticks(false), 20);
    }

    #[test]
    fn block_center() {
        assert_eq!(target_block_center(), [0.5, 0.5, 0.5]);
    }

    #[test]
    fn distance_to_center_at_center() {
        let d = distance_to_center([1.5, 2.5, 3.5], [1.0, 2.0, 3.0]);
        assert!((d - 0.0).abs() < 1e-6);
    }

    #[test]
    fn distance_to_center_offset() {
        let d = distance_to_center([1.5, 2.5, 4.0], [1.0, 2.0, 3.0]);
        assert!((d - 0.5).abs() < 1e-6);
    }

    #[test]
    fn block_id() {
        assert_eq!(target_block_id(), 750);
    }
}
