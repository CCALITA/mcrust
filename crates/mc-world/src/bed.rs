//! Bed placement and sleep mechanics.
//!
//! Handles bed block state (color, part, facing, occupancy),
//! validates sleep conditions, computes spawn points adjacent to the bed head,
//! and advances the world time to dawn when all players sleep.

/// The 16 dye colors available for beds, matching Minecraft's palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BedColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

/// Which half of a two-block bed this block represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BedPart {
    Head,
    Foot,
}

/// Full block state of a bed block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedState {
    pub color: BedColor,
    pub part: BedPart,
    /// Cardinal facing direction encoded as 0=south, 1=west, 2=north, 3=east.
    pub facing: u8,
    pub occupied: bool,
}

/// Reasons why a player cannot sleep in a bed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepError {
    /// It is not night time.
    NotNight,
    /// Hostile mobs are too close.
    MonstersNearby,
    /// The player is more than 3 blocks from the bed.
    TooFarFromBed,
    /// Another player is already in the bed.
    BedOccupied,
    /// The space above the bed is obstructed.
    BedObstructed,
}

/// Time-of-day threshold at which night begins (Minecraft day cycle 0.0..1.0).
const NIGHT_THRESHOLD: f32 = 0.729;

/// Maximum distance (in blocks) a player may be from a bed to sleep.
const MAX_BED_DISTANCE: f32 = 3.0;

/// Time-of-day value representing dawn.
const DAWN_TIME: f32 = 0.0;

/// Check whether all conditions for sleeping are met.
///
/// Night is defined as `time_of_day > 0.729` or `time_of_day < 0.0`.
/// The player must be within 3.0 blocks of the bed, the bed must not be
/// occupied, and no hostile mobs may be nearby.
///
/// Returns `Ok(())` on success, or the first failing condition as a
/// [`SleepError`].
pub fn can_sleep(
    time_of_day: f32,
    has_monsters_nearby: bool,
    bed_occupied: bool,
    distance_to_bed: f32,
) -> Result<(), SleepError> {
    if time_of_day <= NIGHT_THRESHOLD && time_of_day >= 0.0 {
        return Err(SleepError::NotNight);
    }
    if has_monsters_nearby {
        return Err(SleepError::MonstersNearby);
    }
    if bed_occupied {
        return Err(SleepError::BedOccupied);
    }
    if distance_to_bed > MAX_BED_DISTANCE {
        return Err(SleepError::TooFarFromBed);
    }
    Ok(())
}

/// Compute the respawn position adjacent to the bed head.
///
/// The offset is applied in the direction the bed faces:
/// - 0 (south) -> z + 1
/// - 1 (west)  -> x - 1
/// - 2 (north) -> z - 1
/// - 3 (east)  -> x + 1
///
/// Any other facing value defaults to south (z + 1).
pub fn set_spawn_point(bed_pos: (i32, i32, i32), facing: u8) -> (i32, i32, i32) {
    let (x, y, z) = bed_pos;
    match facing {
        0 => (x, y, z + 1),     // south
        1 => (x - 1, y, z),     // west
        2 => (x, y, z - 1),     // north
        3 => (x + 1, y, z),     // east
        _ => (x, y, z + 1),     // default to south
    }
}

/// Advance the world time to dawn (0.0) after all players have slept.
///
/// Returns the new time-of-day value (always `0.0`).
pub fn skip_night(time_of_day: f32) -> f32 {
    let _ = time_of_day;
    DAWN_TIME
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- BedColor ----

    #[test]
    fn bed_color_has_16_variants() {
        let colors = [
            BedColor::White,
            BedColor::Orange,
            BedColor::Magenta,
            BedColor::LightBlue,
            BedColor::Yellow,
            BedColor::Lime,
            BedColor::Pink,
            BedColor::Gray,
            BedColor::LightGray,
            BedColor::Cyan,
            BedColor::Purple,
            BedColor::Blue,
            BedColor::Brown,
            BedColor::Green,
            BedColor::Red,
            BedColor::Black,
        ];
        assert_eq!(colors.len(), 16);
    }

    #[test]
    fn bed_color_equality() {
        assert_eq!(BedColor::Red, BedColor::Red);
        assert_ne!(BedColor::Red, BedColor::Blue);
    }

    // ---- BedPart ----

    #[test]
    fn bed_part_head_and_foot_are_distinct() {
        assert_ne!(BedPart::Head, BedPart::Foot);
    }

    // ---- BedState ----

    #[test]
    fn bed_state_construction() {
        let state = BedState {
            color: BedColor::Red,
            part: BedPart::Head,
            facing: 0,
            occupied: false,
        };
        assert_eq!(state.color, BedColor::Red);
        assert_eq!(state.part, BedPart::Head);
        assert_eq!(state.facing, 0);
        assert!(!state.occupied);
    }

    // ---- can_sleep: success ----

    #[test]
    fn can_sleep_succeeds_at_night() {
        let result = can_sleep(0.80, false, false, 2.0);
        assert!(result.is_ok());
    }

    #[test]
    fn can_sleep_succeeds_just_above_threshold() {
        // 0.73 is above 0.729, so it is night
        let result = can_sleep(0.73, false, false, 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn can_sleep_succeeds_at_exact_max_distance() {
        let result = can_sleep(0.80, false, false, 3.0);
        assert!(result.is_ok());
    }

    #[test]
    fn can_sleep_succeeds_with_negative_time() {
        // time_of_day < 0.0 counts as night
        let result = can_sleep(-0.01, false, false, 2.0);
        assert!(result.is_ok());
    }

    // ---- can_sleep: NotNight error ----

    #[test]
    fn cannot_sleep_during_daytime() {
        let result = can_sleep(0.50, false, false, 2.0);
        assert_eq!(result, Err(SleepError::NotNight));
    }

    #[test]
    fn cannot_sleep_at_exact_threshold() {
        // 0.729 is exactly the threshold — not yet night
        let result = can_sleep(0.729, false, false, 2.0);
        assert_eq!(result, Err(SleepError::NotNight));
    }

    #[test]
    fn cannot_sleep_at_dawn() {
        let result = can_sleep(0.0, false, false, 2.0);
        assert_eq!(result, Err(SleepError::NotNight));
    }

    // ---- can_sleep: MonstersNearby error ----

    #[test]
    fn cannot_sleep_with_monsters_nearby() {
        let result = can_sleep(0.80, true, false, 2.0);
        assert_eq!(result, Err(SleepError::MonstersNearby));
    }

    // ---- can_sleep: BedOccupied error ----

    #[test]
    fn cannot_sleep_in_occupied_bed() {
        let result = can_sleep(0.80, false, true, 2.0);
        assert_eq!(result, Err(SleepError::BedOccupied));
    }

    // ---- can_sleep: TooFarFromBed error ----

    #[test]
    fn cannot_sleep_when_too_far() {
        let result = can_sleep(0.80, false, false, 5.0);
        assert_eq!(result, Err(SleepError::TooFarFromBed));
    }

    #[test]
    fn cannot_sleep_just_beyond_max_distance() {
        let result = can_sleep(0.80, false, false, 3.01);
        assert_eq!(result, Err(SleepError::TooFarFromBed));
    }

    // ---- can_sleep: error priority (first failing condition) ----

    #[test]
    fn not_night_takes_priority_over_monsters() {
        let result = can_sleep(0.50, true, true, 5.0);
        assert_eq!(result, Err(SleepError::NotNight));
    }

    #[test]
    fn monsters_takes_priority_over_occupied() {
        let result = can_sleep(0.80, true, true, 5.0);
        assert_eq!(result, Err(SleepError::MonstersNearby));
    }

    #[test]
    fn occupied_takes_priority_over_distance() {
        let result = can_sleep(0.80, false, true, 5.0);
        assert_eq!(result, Err(SleepError::BedOccupied));
    }

    // ---- set_spawn_point ----

    #[test]
    fn spawn_point_south_facing() {
        let pos = set_spawn_point((10, 64, 20), 0);
        assert_eq!(pos, (10, 64, 21));
    }

    #[test]
    fn spawn_point_west_facing() {
        let pos = set_spawn_point((10, 64, 20), 1);
        assert_eq!(pos, (9, 64, 20));
    }

    #[test]
    fn spawn_point_north_facing() {
        let pos = set_spawn_point((10, 64, 20), 2);
        assert_eq!(pos, (10, 64, 19));
    }

    #[test]
    fn spawn_point_east_facing() {
        let pos = set_spawn_point((10, 64, 20), 3);
        assert_eq!(pos, (11, 64, 20));
    }

    #[test]
    fn spawn_point_invalid_facing_defaults_to_south() {
        let pos = set_spawn_point((10, 64, 20), 255);
        assert_eq!(pos, (10, 64, 21));
    }

    #[test]
    fn spawn_point_preserves_y_coordinate() {
        let pos = set_spawn_point((0, 100, 0), 0);
        assert_eq!(pos.1, 100);
    }

    #[test]
    fn spawn_point_with_negative_coordinates() {
        let pos = set_spawn_point((-5, 64, -10), 2); // north
        assert_eq!(pos, (-5, 64, -11));
    }

    // ---- skip_night ----

    #[test]
    fn skip_night_returns_dawn() {
        let result = skip_night(0.80);
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn skip_night_from_midnight() {
        let result = skip_night(0.999);
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn skip_night_idempotent_at_dawn() {
        let result = skip_night(0.0);
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    // ---- SleepError: all variants covered ----

    #[test]
    fn all_sleep_error_variants_are_distinct() {
        let errors = [
            SleepError::NotNight,
            SleepError::MonstersNearby,
            SleepError::TooFarFromBed,
            SleepError::BedOccupied,
            SleepError::BedObstructed,
        ];
        for (i, a) in errors.iter().enumerate() {
            for (j, b) in errors.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn sleep_error_debug_format() {
        let err = SleepError::BedObstructed;
        let debug = format!("{:?}", err);
        assert!(debug.contains("BedObstructed"));
    }
}
