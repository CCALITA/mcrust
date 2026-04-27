//! Bubble column particle effects for underwater columns created by magma
//! blocks (downward pull) and soul sand (upward push).
//!
//! Each bubble column spawns rising or sinking bubble particles with
//! deterministic placement via a seed-based hash.

/// Describes a bubble column at a world position.
#[derive(Debug, Clone)]
pub struct BubbleColumn {
    /// `true` for soul-sand (upward) columns, `false` for magma (downward).
    pub upward: bool,
    /// World-space position of the column base.
    pub pos: [f32; 3],
}

/// Spawn `count` bubble particles for the given column.
///
/// Returns `(position, velocity)` pairs. Positions are scattered within a
/// 1-block horizontal area centered on the column, and velocities point
/// upward or downward depending on `col.upward`.
pub fn spawn_bubble_particles(
    col: &BubbleColumn,
    count: u8,
    seed: u64,
) -> Vec<([f32; 3], [f32; 3])> {
    let speed = bubble_rise_speed(col.upward);
    let mut particles = Vec::with_capacity(count as usize);
    for i in 0..count {
        let h = hash_u64(seed, i as u32);

        // Scatter within a 1-block horizontal area centered on the column.
        let x_off = ((h & 0xFFFF) as f32 / 32767.5) - 1.0;
        let z_off = (((h >> 16) & 0xFFFF) as f32 / 32767.5) - 1.0;

        // Slight vertical offset so bubbles don't all start at the same height.
        let y_off = ((h >> 32) & 0xFFFF) as f32 / 65535.0;

        let pos = [
            col.pos[0] + x_off * 0.5,
            col.pos[1] + y_off,
            col.pos[2] + z_off * 0.5,
        ];

        let vy = if col.upward { speed } else { -speed };

        // Small horizontal drift from the whirlpool effect.
        let pull = whirlpool_pull_strength();
        let vx = ((h >> 48) & 0xFF) as f32 / 255.0 * pull - pull * 0.5;
        let vz = ((h >> 56) & 0xFF) as f32 / 255.0 * pull - pull * 0.5;

        particles.push((pos, [vx, vy, vz]));
    }
    particles
}

/// Vertical speed of bubbles in a column.
///
/// Upward (soul sand) columns push at 0.5 blocks/s; downward (magma) columns
/// pull at 0.1 blocks/s.
pub fn bubble_rise_speed(upward: bool) -> f32 {
    if upward { 0.5 } else { 0.1 }
}

/// Horizontal pull strength of the whirlpool effect (blocks/s).
pub fn whirlpool_pull_strength() -> f32 {
    0.3
}

/// RGBA color for bubble particles — white and translucent.
pub fn bubble_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 0.4]
}

/// Returns `true` — magma blocks create downward bubble columns.
pub fn magma_creates_downward() -> bool {
    true
}

/// Returns `true` — soul sand creates upward bubble columns.
pub fn soul_sand_creates_upward() -> bool {
    true
}

/// Deterministic 64-bit hash from a seed and particle index.
fn hash_u64(seed: u64, index: u32) -> u64 {
    let mut h = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(index as u64)
        .wrapping_mul(1_442_695_040_888_963_407);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_returns_correct_count() {
        let col = BubbleColumn { upward: true, pos: [0.0, 64.0, 0.0] };
        let particles = spawn_bubble_particles(&col, 10, 42);
        assert_eq!(particles.len(), 10);
    }

    #[test]
    fn spawn_zero_count_returns_empty() {
        let col = BubbleColumn { upward: true, pos: [0.0, 64.0, 0.0] };
        let particles = spawn_bubble_particles(&col, 0, 42);
        assert!(particles.is_empty());
    }

    #[test]
    fn spawn_upward_has_positive_y_velocity() {
        let col = BubbleColumn { upward: true, pos: [0.0, 64.0, 0.0] };
        let particles = spawn_bubble_particles(&col, 20, 99);
        for (_, vel) in &particles {
            assert!(vel[1] > 0.0, "upward column should have positive vy: {}", vel[1]);
        }
    }

    #[test]
    fn spawn_downward_has_negative_y_velocity() {
        let col = BubbleColumn { upward: false, pos: [0.0, 64.0, 0.0] };
        let particles = spawn_bubble_particles(&col, 20, 99);
        for (_, vel) in &particles {
            assert!(vel[1] < 0.0, "downward column should have negative vy: {}", vel[1]);
        }
    }

    #[test]
    fn spawn_is_deterministic() {
        let col = BubbleColumn { upward: true, pos: [5.0, 64.0, 5.0] };
        let a = spawn_bubble_particles(&col, 10, 42);
        let b = spawn_bubble_particles(&col, 10, 42);
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.0, pb.0);
            assert_eq!(pa.1, pb.1);
        }
    }

    #[test]
    fn spawn_different_seeds_differ() {
        let col = BubbleColumn { upward: true, pos: [0.0, 64.0, 0.0] };
        let a = spawn_bubble_particles(&col, 5, 1);
        let b = spawn_bubble_particles(&col, 5, 2);
        let any_differ = a.iter().zip(b.iter()).any(|(pa, pb)| pa.0 != pb.0);
        assert!(any_differ, "different seeds should produce different positions");
    }

    #[test]
    fn spawn_positions_near_column() {
        let col = BubbleColumn { upward: true, pos: [10.0, 64.0, 10.0] };
        let particles = spawn_bubble_particles(&col, 50, 77);
        for (pos, _) in &particles {
            let dx = (pos[0] - col.pos[0]).abs();
            let dz = (pos[2] - col.pos[2]).abs();
            assert!(dx <= 0.5, "x offset too large: {dx}");
            assert!(dz <= 0.5, "z offset too large: {dz}");
        }
    }

    #[test]
    fn bubble_rise_speed_upward() {
        assert!((bubble_rise_speed(true) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn bubble_rise_speed_downward() {
        assert!((bubble_rise_speed(false) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn whirlpool_pull_strength_value() {
        assert!((whirlpool_pull_strength() - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn bubble_color_is_white_translucent() {
        let c = bubble_color();
        assert_eq!(c[0], 1.0);
        assert_eq!(c[1], 1.0);
        assert_eq!(c[2], 1.0);
        assert!(c[3] > 0.0 && c[3] < 1.0, "alpha should be translucent: {}", c[3]);
    }

    #[test]
    fn magma_creates_downward_returns_true() {
        assert!(magma_creates_downward());
    }

    #[test]
    fn soul_sand_creates_upward_returns_true() {
        assert!(soul_sand_creates_upward());
    }
}
