//! Slime block bounce mechanics and entity interaction.
//!
//! Slime blocks provide full velocity rebound on impact, push adjacent entities,
//! stick to most blocks (except honey), and prevent fall damage.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Gravity constant used for bounce height calculation (blocks/tick²).
const GRAVITY: f32 = 0.08;

/// Drag factor applied each tick to vertical velocity.
const DRAG: f32 = 0.98;

// ---------------------------------------------------------------------------
// Bounce mechanics
// ---------------------------------------------------------------------------

/// Returns the bounce velocity from a slime block impact.
///
/// The slime block provides a full rebound: the returned velocity is the
/// negation of the impact velocity.
pub fn slime_bounce_velocity(impact_velocity: f32) -> f32 {
    -impact_velocity
}

/// Returns `true` because slime blocks push adjacent entities when moved by pistons.
pub fn slime_block_pushes_entities() -> bool {
    true
}

/// Returns `true` because slime blocks are sticky and attach to adjacent blocks
/// when moved by pistons.
pub fn slime_block_sticky() -> bool {
    true
}

/// Returns `true` because slime blocks do not stick to honey blocks.
pub fn slime_does_not_stick_to_honey() -> bool {
    true
}

/// Calculates the approximate bounce height from a given fall distance.
///
/// Uses the kinematic relationship between fall distance and resulting
/// bounce velocity, accounting for Minecraft's drag model.
pub fn slime_bounce_height(fall_distance: f32) -> f32 {
    if fall_distance <= 0.0 {
        return 0.0;
    }
    // Approximate upward velocity from fall distance using v = sqrt(2 * g * d)
    let velocity = (2.0 * GRAVITY * fall_distance).sqrt();
    // Bounce height accounting for drag: h ≈ v² / (2 * g) * drag²
    (velocity * velocity * DRAG * DRAG) / (2.0 * GRAVITY)
}

/// Returns `true` because landing on a slime block prevents fall damage.
pub fn slime_prevents_fall_damage() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounce_velocity_negates_impact() {
        assert_eq!(slime_bounce_velocity(5.0), -5.0);
        assert_eq!(slime_bounce_velocity(-3.0), 3.0);
        assert_eq!(slime_bounce_velocity(0.0), -0.0);
    }

    #[test]
    fn bounce_velocity_preserves_magnitude() {
        let speed = 7.5;
        assert_eq!(slime_bounce_velocity(speed).abs(), speed);
    }

    #[test]
    fn pushes_entities() {
        assert!(slime_block_pushes_entities());
    }

    #[test]
    fn is_sticky() {
        assert!(slime_block_sticky());
    }

    #[test]
    fn does_not_stick_to_honey() {
        assert!(slime_does_not_stick_to_honey());
    }

    #[test]
    fn prevents_fall_damage() {
        assert!(slime_prevents_fall_damage());
    }

    #[test]
    fn bounce_height_zero_for_no_fall() {
        assert_eq!(slime_bounce_height(0.0), 0.0);
        assert_eq!(slime_bounce_height(-1.0), 0.0);
    }

    #[test]
    fn bounce_height_increases_with_fall_distance() {
        let h1 = slime_bounce_height(5.0);
        let h2 = slime_bounce_height(10.0);
        assert!(h2 > h1, "higher fall should produce higher bounce");
    }

    #[test]
    fn bounce_height_less_than_fall_due_to_drag() {
        let fall = 10.0;
        let height = slime_bounce_height(fall);
        assert!(height < fall, "bounce height should be less than fall distance due to drag");
        assert!(height > 0.0);
    }
}
