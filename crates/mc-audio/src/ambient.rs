/// Ambient sound selection based on environmental conditions.
///
/// Evaluates dimension, depth, weather, and time-of-day to produce a list of
/// ambient sounds with associated volume levels.

/// Types of ambient sound loops the game can play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmbientSound {
    CaveAmbience,
    UnderwaterLoop,
    NetherLoop,
    EndLoop,
    Rain,
    Thunder,
    Wind,
    Cricket,
}

/// Environmental conditions used to select ambient sounds.
#[derive(Debug, Clone, Copy)]
pub struct AmbientConditions {
    /// Dimension id: 0 = overworld, 1 = nether, 2 = end.
    pub dimension: u8,
    /// Player Y coordinate.
    pub y: i32,
    /// Whether the player is submerged.
    pub is_underwater: bool,
    /// Whether the player is inside a cave.
    pub is_cave: bool,
    /// Weather state: 0 = clear, 1 = rain, 2 = thunder.
    pub weather: u8,
    /// Fraction of the day cycle in `0.0..1.0`.
    pub time_of_day: f32,
}

/// Selects ambient sounds and their volumes based on the given conditions.
///
/// Returns a list of `(AmbientSound, volume)` pairs. Multiple ambient sounds
/// can be active simultaneously (e.g. rain + cave ambience).
pub fn select_ambient(conditions: &AmbientConditions) -> Vec<(AmbientSound, f32)> {
    let mut sounds = Vec::new();

    // Underwater overrides surface sounds.
    if conditions.is_underwater {
        sounds.push((AmbientSound::UnderwaterLoop, 0.8));
        return sounds;
    }

    // Dimension-specific loops.
    match conditions.dimension {
        1 => {
            sounds.push((AmbientSound::NetherLoop, 0.6));
            return sounds;
        }
        2 => {
            sounds.push((AmbientSound::EndLoop, 0.5));
            return sounds;
        }
        _ => {}
    }

    // Weather layers (overworld only at this point).
    match conditions.weather {
        1 => {
            sounds.push((AmbientSound::Rain, 0.5));
        }
        2 => {
            sounds.push((AmbientSound::Rain, 0.7));
            sounds.push((AmbientSound::Thunder, 0.3));
        }
        _ => {}
    }

    // Cave ambience when underground.
    if conditions.is_cave && conditions.y < 40 {
        sounds.push((AmbientSound::CaveAmbience, 0.3));
    }

    // Cricket sounds at night (time_of_day 0.75-1.0 or 0.0-0.15).
    if conditions.time_of_day >= 0.75 || conditions.time_of_day <= 0.15 {
        sounds.push((AmbientSound::Cricket, 0.2));
    }

    sounds
}

/// Returns the probability (0.0-1.0) of cave ambience triggering at a given Y level.
///
/// - Above y=60: 0.0
/// - At y=0: 1.0
/// - Linearly interpolated between y=0 and y=60
/// - Below y=-64: 0.0 (capped)
pub fn cave_ambience_chance(y: i32) -> f32 {
    if y > 60 || y < -64 {
        return 0.0;
    }
    // Linearly interpolate: 1.0 at y=0, 0.0 at y=60.
    let clamped = y.clamp(0, 60) as f32;
    1.0 - (clamped / 60.0)
}

/// Maps a rain intensity (0.0-1.0) to a volume (0.1-0.8).
///
/// A linear mapping where intensity 0.0 produces volume 0.1 and
/// intensity 1.0 produces volume 0.8.
pub fn rain_volume(intensity: f32) -> f32 {
    let clamped = intensity.clamp(0.0, 1.0);
    0.1 + clamped * 0.7
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overworld_conditions() -> AmbientConditions {
        AmbientConditions {
            dimension: 0,
            y: 64,
            is_underwater: false,
            is_cave: false,
            weather: 0,
            time_of_day: 0.5,
        }
    }

    // --- select_ambient tests ---

    #[test]
    fn underwater_overrides_surface_sounds() {
        let conditions = AmbientConditions {
            is_underwater: true,
            weather: 2, // thunder, but should be ignored
            is_cave: true,
            y: 10,
            time_of_day: 0.0,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0], (AmbientSound::UnderwaterLoop, 0.8));
    }

    #[test]
    fn nether_dimension_returns_nether_loop() {
        let conditions = AmbientConditions {
            dimension: 1,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0], (AmbientSound::NetherLoop, 0.6));
    }

    #[test]
    fn end_dimension_returns_end_loop() {
        let conditions = AmbientConditions {
            dimension: 2,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0], (AmbientSound::EndLoop, 0.5));
    }

    #[test]
    fn rain_weather_adds_rain_sound() {
        let conditions = AmbientConditions {
            weather: 1,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert!(sounds.contains(&(AmbientSound::Rain, 0.5)));
        assert!(!sounds.iter().any(|(s, _)| *s == AmbientSound::Thunder));
    }

    #[test]
    fn thunder_weather_layers_rain_and_thunder() {
        let conditions = AmbientConditions {
            weather: 2,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert!(sounds.contains(&(AmbientSound::Rain, 0.7)));
        assert!(sounds.contains(&(AmbientSound::Thunder, 0.3)));
    }

    #[test]
    fn cave_below_40_adds_cave_ambience() {
        let conditions = AmbientConditions {
            is_cave: true,
            y: 20,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert!(sounds.contains(&(AmbientSound::CaveAmbience, 0.3)));
    }

    #[test]
    fn cave_at_or_above_40_no_cave_ambience() {
        let conditions = AmbientConditions {
            is_cave: true,
            y: 40,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert!(!sounds.iter().any(|(s, _)| *s == AmbientSound::CaveAmbience));
    }

    #[test]
    fn nighttime_adds_crickets() {
        let conditions = AmbientConditions {
            time_of_day: 0.8,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert!(sounds.contains(&(AmbientSound::Cricket, 0.2)));
    }

    #[test]
    fn early_morning_adds_crickets() {
        let conditions = AmbientConditions {
            time_of_day: 0.1,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert!(sounds.contains(&(AmbientSound::Cricket, 0.2)));
    }

    #[test]
    fn daytime_no_crickets() {
        let sounds = select_ambient(&overworld_conditions());
        assert!(!sounds.iter().any(|(s, _)| *s == AmbientSound::Cricket));
    }

    #[test]
    fn weather_and_cave_layer_together() {
        let conditions = AmbientConditions {
            weather: 1,
            is_cave: true,
            y: 10,
            time_of_day: 0.9,
            ..overworld_conditions()
        };
        let sounds = select_ambient(&conditions);
        assert!(sounds.contains(&(AmbientSound::Rain, 0.5)));
        assert!(sounds.contains(&(AmbientSound::CaveAmbience, 0.3)));
        assert!(sounds.contains(&(AmbientSound::Cricket, 0.2)));
    }

    #[test]
    fn clear_day_overworld_surface_is_silent() {
        let sounds = select_ambient(&overworld_conditions());
        assert!(sounds.is_empty());
    }

    // --- cave_ambience_chance tests ---

    #[test]
    fn cave_chance_above_60_is_zero() {
        assert_eq!(cave_ambience_chance(61), 0.0);
        assert_eq!(cave_ambience_chance(100), 0.0);
    }

    #[test]
    fn cave_chance_at_zero_is_one() {
        assert!((cave_ambience_chance(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cave_chance_at_60_is_zero() {
        assert!((cave_ambience_chance(60) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cave_chance_at_30_is_half() {
        assert!((cave_ambience_chance(30) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn cave_chance_below_negative_64_is_zero() {
        assert_eq!(cave_ambience_chance(-65), 0.0);
        assert_eq!(cave_ambience_chance(-100), 0.0);
    }

    #[test]
    fn cave_chance_between_neg64_and_zero_is_one() {
        // Any y in -64..=0 should clamp to 0 and return 1.0
        assert!((cave_ambience_chance(-30) - 1.0).abs() < f32::EPSILON);
        assert!((cave_ambience_chance(-64) - 1.0).abs() < f32::EPSILON);
    }

    // --- rain_volume tests ---

    #[test]
    fn rain_volume_at_zero_intensity() {
        assert!((rain_volume(0.0) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn rain_volume_at_full_intensity() {
        assert!((rain_volume(1.0) - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn rain_volume_at_half_intensity() {
        assert!((rain_volume(0.5) - 0.45).abs() < f32::EPSILON);
    }

    #[test]
    fn rain_volume_clamps_above_one() {
        assert!((rain_volume(2.0) - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn rain_volume_clamps_below_zero() {
        assert!((rain_volume(-1.0) - 0.1).abs() < f32::EPSILON);
    }
}
