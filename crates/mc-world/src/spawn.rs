//! Bed sleeping and spawn point system.
//!
//! Manages per-player spawn points set by sleeping in a bed,
//! validates sleep conditions, and advances the world time to morning.

/// A player's personal spawn point, set by sleeping in a bed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnPoint {
    pub position: (i32, i32, i32),
    pub dimension: u8,
}

/// Outcome of a `try_sleep` attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedResult {
    /// The player slept successfully and time can advance.
    SleptSuccessfully,
    /// It is not night time (time_of_day must be >= 0.75 or < 0.25 is already morning).
    NotNightTime,
    /// The space above the bed is obstructed.
    BedObstructed,
    /// The player is too far from the bed (> 3 blocks).
    TooFarFromBed,
    /// There is no bed at the target position.
    NoBedAtPosition,
}

/// Maximum distance (in blocks) a player may be from a bed to sleep.
const MAX_BED_DISTANCE: f32 = 3.0;

/// Time-of-day threshold at which night begins (0.75 = dusk in Minecraft's 0..1 day cycle).
const NIGHT_START: f32 = 0.75;

/// Time-of-day value representing morning (0.25 = sunrise).
const MORNING_TIME: f32 = 0.25;

/// Manages the world's default spawn and per-player bed spawn points.
pub struct SpawnManager {
    default_spawn: (i32, i32, i32),
    player_spawn: Option<SpawnPoint>,
}

impl SpawnManager {
    /// Create a new spawn manager whose default spawn is at `(0, surface_y, 0)`.
    pub fn new(surface_y: i32) -> Self {
        Self {
            default_spawn: (0, surface_y, 0),
            player_spawn: None,
        }
    }

    /// Set the player's personal spawn point from a bed.
    pub fn set_spawn(&mut self, position: (i32, i32, i32), dimension: u8) {
        self.player_spawn = Some(SpawnPoint {
            position,
            dimension,
        });
    }

    /// Return the effective spawn point: the player's bed spawn if set,
    /// otherwise the world default (dimension 0).
    pub fn get_spawn(&self) -> SpawnPoint {
        self.player_spawn.unwrap_or(SpawnPoint {
            position: self.default_spawn,
            dimension: 0,
        })
    }

    /// Clear the player's personal spawn point (e.g. when the bed is destroyed).
    pub fn clear_spawn(&mut self) {
        self.player_spawn = None;
    }

    /// Attempt to sleep in a bed, validating all conditions.
    ///
    /// * `time_of_day` — current world time in `0.0..1.0` (0.0 = midnight, 0.25 = sunrise, 0.75 = dusk).
    /// * `is_bed` — whether a bed block exists at the target position.
    /// * `is_obstructed` — whether the space above the bed is blocked.
    /// * `distance` — euclidean distance from the player to the bed.
    pub fn try_sleep(
        time_of_day: f32,
        is_bed: bool,
        is_obstructed: bool,
        distance: f32,
    ) -> BedResult {
        if !is_bed {
            return BedResult::NoBedAtPosition;
        }
        if time_of_day < NIGHT_START {
            return BedResult::NotNightTime;
        }
        if is_obstructed {
            return BedResult::BedObstructed;
        }
        if distance > MAX_BED_DISTANCE {
            return BedResult::TooFarFromBed;
        }
        BedResult::SleptSuccessfully
    }

    /// Advance the world time to morning (0.25).
    ///
    /// Called after all players in a world have slept successfully.
    pub fn advance_to_morning(time: &mut f32) {
        *time = MORNING_TIME;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Sleep condition tests ----

    #[test]
    fn sleeps_successfully_at_night_with_valid_bed() {
        let result = SpawnManager::try_sleep(0.80, true, false, 2.0);
        assert_eq!(result, BedResult::SleptSuccessfully);
    }

    #[test]
    fn rejects_sleep_during_daytime() {
        let result = SpawnManager::try_sleep(0.50, true, false, 2.0);
        assert_eq!(result, BedResult::NotNightTime);
    }

    #[test]
    fn rejects_sleep_when_no_bed_present() {
        let result = SpawnManager::try_sleep(0.80, false, false, 2.0);
        assert_eq!(result, BedResult::NoBedAtPosition);
    }

    #[test]
    fn rejects_sleep_when_bed_obstructed() {
        let result = SpawnManager::try_sleep(0.80, true, true, 2.0);
        assert_eq!(result, BedResult::BedObstructed);
    }

    #[test]
    fn rejects_sleep_when_too_far_from_bed() {
        let result = SpawnManager::try_sleep(0.80, true, false, 5.0);
        assert_eq!(result, BedResult::TooFarFromBed);
    }

    #[test]
    fn rejects_sleep_at_exactly_boundary_distance() {
        // distance > MAX_BED_DISTANCE (3.0), so 3.01 should fail
        let result = SpawnManager::try_sleep(0.80, true, false, 3.01);
        assert_eq!(result, BedResult::TooFarFromBed);
    }

    #[test]
    fn accepts_sleep_at_exact_max_distance() {
        // distance == MAX_BED_DISTANCE (3.0) is allowed (not >)
        let result = SpawnManager::try_sleep(0.80, true, false, 3.0);
        assert_eq!(result, BedResult::SleptSuccessfully);
    }

    #[test]
    fn rejects_sleep_at_dawn_boundary() {
        // 0.74 is still daytime (< 0.75)
        let result = SpawnManager::try_sleep(0.74, true, false, 2.0);
        assert_eq!(result, BedResult::NotNightTime);
    }

    #[test]
    fn accepts_sleep_at_night_start_boundary() {
        // 0.75 is exactly night start
        let result = SpawnManager::try_sleep(0.75, true, false, 2.0);
        assert_eq!(result, BedResult::SleptSuccessfully);
    }

    // ---- Spawn set/get/clear tests ----

    #[test]
    fn default_spawn_returns_world_origin() {
        let mgr = SpawnManager::new(64);
        let spawn = mgr.get_spawn();
        assert_eq!(spawn.position, (0, 64, 0));
        assert_eq!(spawn.dimension, 0);
    }

    #[test]
    fn set_spawn_overrides_default() {
        let mut mgr = SpawnManager::new(64);
        mgr.set_spawn((100, 70, -200), 1);
        let spawn = mgr.get_spawn();
        assert_eq!(spawn.position, (100, 70, -200));
        assert_eq!(spawn.dimension, 1);
    }

    #[test]
    fn clear_spawn_reverts_to_default() {
        let mut mgr = SpawnManager::new(64);
        mgr.set_spawn((100, 70, -200), 1);
        mgr.clear_spawn();
        let spawn = mgr.get_spawn();
        assert_eq!(spawn.position, (0, 64, 0));
        assert_eq!(spawn.dimension, 0);
    }

    #[test]
    fn set_spawn_can_be_updated() {
        let mut mgr = SpawnManager::new(64);
        mgr.set_spawn((10, 65, 10), 0);
        mgr.set_spawn((20, 72, 20), 0);
        let spawn = mgr.get_spawn();
        assert_eq!(spawn.position, (20, 72, 20));
    }

    // ---- Time advancement tests ----

    #[test]
    fn advance_to_morning_sets_correct_time() {
        let mut time = 0.90;
        SpawnManager::advance_to_morning(&mut time);
        assert!((time - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn advance_to_morning_from_midnight() {
        let mut time = 0.0;
        SpawnManager::advance_to_morning(&mut time);
        assert!((time - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn advance_to_morning_idempotent() {
        let mut time = 0.25;
        SpawnManager::advance_to_morning(&mut time);
        assert!((time - 0.25).abs() < f32::EPSILON);
    }
}
