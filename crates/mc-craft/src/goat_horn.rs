//! Goat horn variant system.
//!
//! Defines the eight goat horn variants, their sound IDs, durations,
//! range, cooldown, and drop rules based on goat type (screamer vs normal).

/// The eight goat horn instrument variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoatHornVariant {
    Ponder,
    Sing,
    Seek,
    Feel,
    Admire,
    Call,
    Yearn,
    Dream,
}

/// All variants in declaration order, for iteration and counting.
const ALL_VARIANTS: [GoatHornVariant; 8] = [
    GoatHornVariant::Ponder,
    GoatHornVariant::Sing,
    GoatHornVariant::Seek,
    GoatHornVariant::Feel,
    GoatHornVariant::Admire,
    GoatHornVariant::Call,
    GoatHornVariant::Yearn,
    GoatHornVariant::Dream,
];

/// Variants dropped by normal (non-screamer) goats.
const NORMAL_VARIANTS: [GoatHornVariant; 4] = [
    GoatHornVariant::Ponder,
    GoatHornVariant::Sing,
    GoatHornVariant::Seek,
    GoatHornVariant::Feel,
];

/// Variants dropped by screamer goats.
const SCREAMER_VARIANTS: [GoatHornVariant; 4] = [
    GoatHornVariant::Admire,
    GoatHornVariant::Call,
    GoatHornVariant::Yearn,
    GoatHornVariant::Dream,
];

/// Base sound ID for goat horn variants (Ponder = 5000, Dream = 5007).
const HORN_SOUND_BASE: u16 = 5000;

impl GoatHornVariant {
    /// Human-readable name for this variant.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ponder => "Ponder",
            Self::Sing => "Sing",
            Self::Seek => "Seek",
            Self::Feel => "Feel",
            Self::Admire => "Admire",
            Self::Call => "Call",
            Self::Yearn => "Yearn",
            Self::Dream => "Dream",
        }
    }
}

/// Return the unique sound event ID for the given horn variant.
///
/// IDs are assigned sequentially from 5000 (Ponder) through 5007 (Dream).
pub fn horn_sound_id(variant: GoatHornVariant) -> u16 {
    HORN_SOUND_BASE + variant as u16
}

/// Maximum audible range of a goat horn blast, in blocks.
pub fn horn_range() -> f32 {
    256.0
}

/// Cooldown between consecutive horn uses, in seconds.
pub fn horn_cooldown() -> f32 {
    7.0
}

/// Duration of the horn sound for the given variant, in seconds.
pub fn horn_duration(variant: GoatHornVariant) -> f32 {
    match variant {
        GoatHornVariant::Ponder => 5.0,
        GoatHornVariant::Sing => 6.0,
        GoatHornVariant::Seek => 5.5,
        GoatHornVariant::Feel => 7.0,
        GoatHornVariant::Admire => 4.5,
        GoatHornVariant::Call => 6.5,
        GoatHornVariant::Yearn => 8.0,
        GoatHornVariant::Dream => 7.5,
    }
}

/// Determine which goat horn variant drops when a goat rams a block.
///
/// Screamer goats drop one of Admire/Call/Yearn/Dream.
/// Normal goats drop one of Ponder/Sing/Seek/Feel.
///
/// The `seed` value is used for deterministic selection (e.g. from world RNG).
pub fn horn_from_goat_drop(is_screamer: bool, seed: u64) -> GoatHornVariant {
    let pool = if is_screamer {
        &SCREAMER_VARIANTS
    } else {
        &NORMAL_VARIANTS
    };
    let index = (seed % pool.len() as u64) as usize;
    pool[index]
}

/// Total number of goat horn variants.
pub fn total_variants() -> usize {
    ALL_VARIANTS.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_variants_named() {
        let names: Vec<&str> = ALL_VARIANTS.iter().map(|v| v.name()).collect();
        assert_eq!(
            names,
            vec!["Ponder", "Sing", "Seek", "Feel", "Admire", "Call", "Yearn", "Dream"]
        );
    }

    #[test]
    fn total_variants_is_eight() {
        assert_eq!(total_variants(), 8);
    }

    #[test]
    fn sound_ids_unique() {
        let ids: HashSet<u16> = ALL_VARIANTS.iter().map(|v| horn_sound_id(*v)).collect();
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn sound_ids_in_range() {
        for variant in &ALL_VARIANTS {
            let id = horn_sound_id(*variant);
            assert!(
                (5000..=5007).contains(&id),
                "Sound ID {id} out of range for {:?}",
                variant
            );
        }
    }

    #[test]
    fn durations_positive() {
        for variant in &ALL_VARIANTS {
            let d = horn_duration(*variant);
            assert!(d > 0.0, "Duration for {:?} must be positive, got {d}", variant);
        }
    }

    #[test]
    fn specific_durations() {
        assert!((horn_duration(GoatHornVariant::Ponder) - 5.0).abs() < f32::EPSILON);
        assert!((horn_duration(GoatHornVariant::Sing) - 6.0).abs() < f32::EPSILON);
        assert!((horn_duration(GoatHornVariant::Seek) - 5.5).abs() < f32::EPSILON);
        assert!((horn_duration(GoatHornVariant::Feel) - 7.0).abs() < f32::EPSILON);
        assert!((horn_duration(GoatHornVariant::Admire) - 4.5).abs() < f32::EPSILON);
        assert!((horn_duration(GoatHornVariant::Call) - 6.5).abs() < f32::EPSILON);
        assert!((horn_duration(GoatHornVariant::Yearn) - 8.0).abs() < f32::EPSILON);
        assert!((horn_duration(GoatHornVariant::Dream) - 7.5).abs() < f32::EPSILON);
    }

    #[test]
    fn horn_range_value() {
        assert!((horn_range() - 256.0).abs() < f32::EPSILON);
    }

    #[test]
    fn horn_cooldown_value() {
        assert!((horn_cooldown() - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normal_goat_drops_normal_variant() {
        for seed in 0..100 {
            let variant = horn_from_goat_drop(false, seed);
            assert!(
                matches!(
                    variant,
                    GoatHornVariant::Ponder
                        | GoatHornVariant::Sing
                        | GoatHornVariant::Seek
                        | GoatHornVariant::Feel
                ),
                "Normal goat should not drop {:?}",
                variant
            );
        }
    }

    #[test]
    fn screamer_goat_drops_screamer_variant() {
        for seed in 0..100 {
            let variant = horn_from_goat_drop(true, seed);
            assert!(
                matches!(
                    variant,
                    GoatHornVariant::Admire
                        | GoatHornVariant::Call
                        | GoatHornVariant::Yearn
                        | GoatHornVariant::Dream
                ),
                "Screamer goat should not drop {:?}",
                variant
            );
        }
    }

    #[test]
    fn drop_covers_all_normal_variants() {
        let variants: HashSet<GoatHornVariant> =
            (0..100).map(|s| horn_from_goat_drop(false, s)).collect();
        assert!(variants.contains(&GoatHornVariant::Ponder));
        assert!(variants.contains(&GoatHornVariant::Sing));
        assert!(variants.contains(&GoatHornVariant::Seek));
        assert!(variants.contains(&GoatHornVariant::Feel));
    }

    #[test]
    fn drop_covers_all_screamer_variants() {
        let variants: HashSet<GoatHornVariant> =
            (0..100).map(|s| horn_from_goat_drop(true, s)).collect();
        assert!(variants.contains(&GoatHornVariant::Admire));
        assert!(variants.contains(&GoatHornVariant::Call));
        assert!(variants.contains(&GoatHornVariant::Yearn));
        assert!(variants.contains(&GoatHornVariant::Dream));
    }
}
