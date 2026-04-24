/// Tipped arrow effects derived from lingering potions.
///
/// Each potion type maps to an effect with a duration and amplifier,
/// along with a unique particle color for rendering.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TOTAL_TYPES: usize = 15;

// ---------------------------------------------------------------------------
// TippedArrowEffect
// ---------------------------------------------------------------------------

/// Describes the status effect applied when a tipped arrow hits a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TippedArrowEffect {
    /// Numeric effect id (e.g. poison, regen, slowness).
    pub effect_type: u8,
    /// Duration of the effect in game ticks (0 for instant effects).
    pub duration_ticks: u32,
    /// Amplifier level (0 = level I).
    pub amplifier: u8,
}

// ---------------------------------------------------------------------------
// Lookup
// ---------------------------------------------------------------------------

/// Create a [`TippedArrowEffect`] for the given `potion_type`.
///
/// Returns `None` if `potion_type` is not a recognised tipped arrow variant.
///
/// # Potion types
///
/// | id | name           | duration | amplifier |
/// |----|----------------|----------|-----------|
/// |  0 | Poison         |      375 |         0 |
/// |  1 | Healing        |  instant |         0 |
/// |  2 | Harming        |  instant |         0 |
/// |  3 | Regeneration   |      100 |         0 |
/// |  4 | Slowness       |      220 |         0 |
/// |  5 | Fire Resistance|       60 |         0 |
/// |  6 | Night Vision   |       60 |         0 |
/// |  7 | Strength       |       60 |         0 |
/// |  8 | Weakness       |      220 |         0 |
/// |  9 | Swiftness      |       60 |         0 |
/// | 10 | Water Breathing|       60 |         0 |
/// | 11 | Invisibility   |       60 |         0 |
/// | 12 | Slow Falling   |       30 |         0 |
/// | 13 | Turtle Master  |       20 |         0 |
/// | 14 | Luck           |      100 |         0 |
pub fn create_tipped_arrow(potion_type: u8) -> Option<TippedArrowEffect> {
    let (effect_type, duration_ticks, amplifier) = match potion_type {
        0  => (0,  375, 0), // Poison
        1  => (1,    0, 0), // Healing (instant)
        2  => (2,    0, 0), // Harming (instant)
        3  => (3,  100, 0), // Regeneration
        4  => (4,  220, 0), // Slowness
        5  => (5,   60, 0), // Fire Resistance
        6  => (6,   60, 0), // Night Vision
        7  => (7,   60, 0), // Strength
        8  => (8,  220, 0), // Weakness
        9  => (9,   60, 0), // Swiftness
        10 => (10,  60, 0), // Water Breathing
        11 => (11,  60, 0), // Invisibility
        12 => (12,  30, 0), // Slow Falling
        13 => (13,  20, 0), // Turtle Master (slow + resistance)
        14 => (14, 100, 0), // Luck
        _  => return None,
    };
    Some(TippedArrowEffect {
        effect_type,
        duration_ticks,
        amplifier,
    })
}

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

/// Return the particle / trail colour for a tipped arrow as `[r, g, b]` in
/// the 0.0..1.0 range.
///
/// Returns a default grey for unknown potion types.
pub fn arrow_color(potion_type: u8) -> [f32; 3] {
    match potion_type {
        0  => [0.31, 0.60, 0.07], // Poison — green
        1  => [0.96, 0.24, 0.24], // Healing — red
        2  => [0.26, 0.05, 0.05], // Harming — dark red
        3  => [0.80, 0.30, 0.60], // Regeneration — pink
        4  => [0.35, 0.40, 0.50], // Slowness — blue-grey
        5  => [0.90, 0.58, 0.20], // Fire Resistance — orange
        6  => [0.12, 0.10, 0.55], // Night Vision — dark blue
        7  => [0.57, 0.15, 0.15], // Strength — maroon
        8  => [0.28, 0.28, 0.28], // Weakness — dark grey
        9  => [0.49, 0.78, 0.97], // Swiftness — light blue
        10 => [0.18, 0.35, 0.60], // Water Breathing — navy
        11 => [0.50, 0.50, 0.60], // Invisibility — silver
        12 => [0.95, 0.95, 0.80], // Slow Falling — cream
        13 => [0.10, 0.30, 0.10], // Turtle Master — dark green
        14 => [0.20, 0.80, 0.20], // Luck — bright green
        _  => [0.50, 0.50, 0.50], // Unknown — neutral grey
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

/// Total number of recognised tipped arrow types.
pub fn total_arrow_types() -> usize {
    TOTAL_TYPES
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- create_tipped_arrow --------------------------------------------------

    #[test]
    fn poison_arrow_has_correct_stats() {
        let effect = create_tipped_arrow(0).expect("valid type");
        assert_eq!(effect.effect_type, 0);
        assert_eq!(effect.duration_ticks, 375);
        assert_eq!(effect.amplifier, 0);
    }

    #[test]
    fn healing_arrow_is_instant() {
        let effect = create_tipped_arrow(1).expect("valid type");
        assert_eq!(effect.effect_type, 1);
        assert_eq!(effect.duration_ticks, 0);
    }

    #[test]
    fn harming_arrow_is_instant() {
        let effect = create_tipped_arrow(2).expect("valid type");
        assert_eq!(effect.effect_type, 2);
        assert_eq!(effect.duration_ticks, 0);
    }

    #[test]
    fn regen_arrow_has_100_tick_duration() {
        let effect = create_tipped_arrow(3).expect("valid type");
        assert_eq!(effect.duration_ticks, 100);
    }

    #[test]
    fn slowness_arrow_has_220_tick_duration() {
        let effect = create_tipped_arrow(4).expect("valid type");
        assert_eq!(effect.duration_ticks, 220);
    }

    #[test]
    fn fire_resist_arrow_has_60_tick_duration() {
        let effect = create_tipped_arrow(5).expect("valid type");
        assert_eq!(effect.duration_ticks, 60);
    }

    #[test]
    fn night_vision_arrow_has_60_tick_duration() {
        let effect = create_tipped_arrow(6).expect("valid type");
        assert_eq!(effect.duration_ticks, 60);
    }

    #[test]
    fn strength_arrow_has_60_tick_duration() {
        let effect = create_tipped_arrow(7).expect("valid type");
        assert_eq!(effect.duration_ticks, 60);
    }

    #[test]
    fn weakness_arrow_has_220_tick_duration() {
        let effect = create_tipped_arrow(8).expect("valid type");
        assert_eq!(effect.duration_ticks, 220);
    }

    #[test]
    fn swiftness_arrow_has_60_tick_duration() {
        let effect = create_tipped_arrow(9).expect("valid type");
        assert_eq!(effect.duration_ticks, 60);
    }

    #[test]
    fn water_breathing_arrow_has_60_tick_duration() {
        let effect = create_tipped_arrow(10).expect("valid type");
        assert_eq!(effect.duration_ticks, 60);
    }

    #[test]
    fn invisibility_arrow_has_60_tick_duration() {
        let effect = create_tipped_arrow(11).expect("valid type");
        assert_eq!(effect.duration_ticks, 60);
    }

    #[test]
    fn slow_falling_arrow_has_30_tick_duration() {
        let effect = create_tipped_arrow(12).expect("valid type");
        assert_eq!(effect.duration_ticks, 30);
    }

    #[test]
    fn turtle_master_arrow_has_20_tick_duration() {
        let effect = create_tipped_arrow(13).expect("valid type");
        assert_eq!(effect.duration_ticks, 20);
    }

    #[test]
    fn luck_arrow_has_100_tick_duration() {
        let effect = create_tipped_arrow(14).expect("valid type");
        assert_eq!(effect.duration_ticks, 100);
    }

    #[test]
    fn unknown_potion_type_returns_none() {
        assert!(create_tipped_arrow(15).is_none());
        assert!(create_tipped_arrow(255).is_none());
    }

    #[test]
    fn all_valid_types_return_some() {
        for i in 0..15u8 {
            assert!(
                create_tipped_arrow(i).is_some(),
                "potion_type {i} should be valid"
            );
        }
    }

    #[test]
    fn all_effects_have_zero_amplifier() {
        for i in 0..15u8 {
            let effect = create_tipped_arrow(i).expect("valid type");
            assert_eq!(effect.amplifier, 0, "type {i} should have amplifier 0");
        }
    }

    // -- arrow_color ----------------------------------------------------------

    #[test]
    fn each_type_has_unique_color() {
        let mut colors: Vec<[f32; 3]> = (0..15u8).map(arrow_color).collect();
        colors.sort_by(|a, b| {
            a[0].partial_cmp(&b[0])
                .unwrap()
                .then(a[1].partial_cmp(&b[1]).unwrap())
                .then(a[2].partial_cmp(&b[2]).unwrap())
        });
        colors.dedup();
        assert_eq!(colors.len(), 15, "all 15 types must have distinct colours");
    }

    #[test]
    fn unknown_type_returns_grey() {
        let c = arrow_color(200);
        assert_eq!(c, [0.50, 0.50, 0.50]);
    }

    #[test]
    fn colors_are_in_valid_range() {
        for i in 0..15u8 {
            let c = arrow_color(i);
            for channel in &c {
                assert!(
                    (0.0..=1.0).contains(channel),
                    "color channel out of range for type {i}"
                );
            }
        }
    }

    // -- total_arrow_types ----------------------------------------------------

    #[test]
    fn total_types_is_fifteen() {
        assert_eq!(total_arrow_types(), 15);
    }
}
