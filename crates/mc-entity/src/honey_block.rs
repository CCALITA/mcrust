//! Honey block physics: slowdown, reduced jump, sliding, and fall damage reduction.

/// Movement speed multiplier when on a honey block.
pub fn honey_slowdown_factor() -> f32 {
    0.4
}

/// Jump height multiplier when on a honey block.
pub fn honey_jump_factor() -> f32 {
    0.5
}

/// Vertical slide speed when against the side of a honey block.
pub fn honey_slide_speed() -> f32 {
    0.05
}

/// Fall damage multiplier when landing on a honey block.
pub fn honey_fall_damage_multiplier() -> f32 {
    0.2
}

/// Whether entities stick to honey blocks (e.g. pushed by pistons).
pub fn honey_sticks_entities() -> bool {
    true
}

/// Returns the effective movement speed, applying honey slowdown when `on_honey` is true.
pub fn apply_honey_movement(base_speed: f32, on_honey: bool) -> f32 {
    if on_honey {
        base_speed * honey_slowdown_factor()
    } else {
        base_speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honey_slowdown_factor() {
        assert!((honey_slowdown_factor() - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_honey_jump_factor() {
        assert!((honey_jump_factor() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_honey_slide_speed() {
        assert!((honey_slide_speed() - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_honey_fall_damage_multiplier() {
        assert!((honey_fall_damage_multiplier() - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_honey_sticks_entities() {
        assert!(honey_sticks_entities());
    }

    #[test]
    fn test_apply_honey_movement_on_honey() {
        let result = apply_honey_movement(10.0, true);
        assert!((result - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_honey_movement_off_honey() {
        let result = apply_honey_movement(10.0, false);
        assert!((result - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_honey_movement_zero_speed() {
        assert!((apply_honey_movement(0.0, true)).abs() < f32::EPSILON);
    }
}
