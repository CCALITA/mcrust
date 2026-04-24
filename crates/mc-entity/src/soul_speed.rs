//! Soul Speed enchantment system.
//!
//! Provides pure helper functions for the Soul Speed boots enchantment,
//! which increases movement speed on soul sand (id 88) and soul soil (id 87).
//! Also generates particle positions near the player's feet when the
//! enchantment is active.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum level of the Soul Speed enchantment (I, II, III).
pub const MAX_SOUL_SPEED_LEVEL: u8 = 3;

/// Block id for soul soil.
const SOUL_SOIL: u16 = 87;

/// Block id for soul sand.
const SOUL_SAND: u16 = 88;

/// Speed bonus per enchantment level when on a soul block.
const SPEED_BONUS_PER_LEVEL: f32 = 0.21;

/// Number of particles spawned per tick when Soul Speed is active.
const PARTICLE_COUNT: usize = 3;

// ---------------------------------------------------------------------------
// Block classification
// ---------------------------------------------------------------------------

/// Returns `true` when `block_id` is soul sand (88) or soul soil (87).
pub fn is_soul_block(block_id: u16) -> bool {
    block_id == SOUL_SAND || block_id == SOUL_SOIL
}

// ---------------------------------------------------------------------------
// Speed computation
// ---------------------------------------------------------------------------

/// Computes the raw speed bonus granted by Soul Speed.
///
/// Returns `0.21 * level` when the player stands on a soul block,
/// otherwise `0.0`. Levels above [`MAX_SOUL_SPEED_LEVEL`] are clamped.
pub fn soul_speed_bonus(level: u8, on_soul_block: bool) -> f32 {
    if !on_soul_block || level == 0 {
        return 0.0;
    }
    let clamped = level.min(MAX_SOUL_SPEED_LEVEL);
    SPEED_BONUS_PER_LEVEL * clamped as f32
}

/// Applies the Soul Speed bonus to a base movement speed.
///
/// `base + soul_speed_bonus(level, on_soul)`.
pub fn apply_soul_speed_to_speed(base: f32, level: u8, on_soul: bool) -> f32 {
    base + soul_speed_bonus(level, on_soul)
}

// ---------------------------------------------------------------------------
// Particles
// ---------------------------------------------------------------------------

/// Generates particle positions near the player's feet.
///
/// Returns three positions offset slightly from `pos` to simulate the
/// soul-block crumbling effect. The offsets are deterministic so that
/// tests stay stable.
pub fn soul_speed_particles(pos: [f32; 3]) -> Vec<[f32; 3]> {
    let offsets: [[f32; 3]; PARTICLE_COUNT] = [
        [-0.2, 0.0, -0.2],
        [0.0, 0.05, 0.0],
        [0.2, 0.0, 0.2],
    ];
    offsets
        .iter()
        .map(|o| [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]])
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- is_soul_block -------------------------------------------------------

    #[test]
    fn soul_sand_is_soul_block() {
        assert!(is_soul_block(88));
    }

    #[test]
    fn soul_soil_is_soul_block() {
        assert!(is_soul_block(87));
    }

    #[test]
    fn regular_block_is_not_soul_block() {
        assert!(!is_soul_block(1));   // stone
        assert!(!is_soul_block(0));   // air
        assert!(!is_soul_block(89));  // glowstone
        assert!(!is_soul_block(86));  // pumpkin
    }

    // -- soul_speed_bonus ----------------------------------------------------

    #[test]
    fn bonus_level_one_on_soul_block() {
        let b = soul_speed_bonus(1, true);
        assert!((b - 0.21).abs() < 1e-6, "got {b}");
    }

    #[test]
    fn bonus_level_two_on_soul_block() {
        let b = soul_speed_bonus(2, true);
        assert!((b - 0.42).abs() < 1e-6, "got {b}");
    }

    #[test]
    fn bonus_level_three_on_soul_block() {
        let b = soul_speed_bonus(3, true);
        assert!((b - 0.63).abs() < 1e-6, "got {b}");
    }

    #[test]
    fn bonus_clamped_above_max_level() {
        // Level 5 should be clamped to 3 -> 0.63.
        let b = soul_speed_bonus(5, true);
        assert!((b - 0.63).abs() < 1e-6, "got {b}");
    }

    #[test]
    fn bonus_zero_when_not_on_soul_block() {
        let b = soul_speed_bonus(3, false);
        assert!((b - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bonus_zero_when_level_zero() {
        let b = soul_speed_bonus(0, true);
        assert!((b - 0.0).abs() < f32::EPSILON);
    }

    // -- apply_soul_speed_to_speed -------------------------------------------

    #[test]
    fn applies_bonus_to_base_speed() {
        let result = apply_soul_speed_to_speed(4.0, 2, true);
        // 4.0 + 0.42 = 4.42
        assert!((result - 4.42).abs() < 1e-6, "got {result}");
    }

    #[test]
    fn no_bonus_off_soul_block() {
        let result = apply_soul_speed_to_speed(4.0, 3, false);
        assert!((result - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn no_bonus_level_zero() {
        let result = apply_soul_speed_to_speed(4.0, 0, true);
        assert!((result - 4.0).abs() < f32::EPSILON);
    }

    // -- soul_speed_particles ------------------------------------------------

    #[test]
    fn particles_returns_three_positions() {
        let particles = soul_speed_particles([10.0, 64.0, 20.0]);
        assert_eq!(particles.len(), 3);
    }

    #[test]
    fn particles_are_near_player_feet() {
        let pos = [10.0, 64.0, 20.0];
        let particles = soul_speed_particles(pos);
        for p in &particles {
            let dx = (p[0] - pos[0]).abs();
            let dy = (p[1] - pos[1]).abs();
            let dz = (p[2] - pos[2]).abs();
            assert!(dx <= 0.5, "x offset too large: {dx}");
            assert!(dy <= 0.5, "y offset too large: {dy}");
            assert!(dz <= 0.5, "z offset too large: {dz}");
        }
    }

    #[test]
    fn particles_deterministic() {
        let a = soul_speed_particles([1.0, 2.0, 3.0]);
        let b = soul_speed_particles([1.0, 2.0, 3.0]);
        assert_eq!(a, b);
    }

    // -- MAX_SOUL_SPEED_LEVEL ------------------------------------------------

    #[test]
    fn max_level_is_three() {
        assert_eq!(MAX_SOUL_SPEED_LEVEL, 3);
    }
}
