// ---------------------------------------------------------------------------
// Village raid system
// ---------------------------------------------------------------------------

use glam::Vec3;

// ---------------------------------------------------------------------------
// Raid wave
// ---------------------------------------------------------------------------

/// A single wave in a village raid.
///
/// Each entry in `mobs` is `(mob_kind, count)` where `mob_kind` follows the
/// conventions in `mc-core` (e.g. MobKind discriminant values).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaidWave {
    pub wave_number: u8,
    pub mobs: Vec<(u8, u8)>,
}

// ---------------------------------------------------------------------------
// Raid event
// ---------------------------------------------------------------------------

/// Events emitted by the raid system during `tick` and `check_wave_clear`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaidEvent {
    /// The current wave has been cleared of all hostile mobs.
    WaveCleared,
    /// The next wave is about to begin. Contains the wave number (1-indexed).
    NextWave(u8),
    /// All waves have been cleared; the raid is over and the village won.
    RaidVictory,
    /// The raid timer expired before all waves were cleared.
    RaidDefeat,
}

// ---------------------------------------------------------------------------
// Raid
// ---------------------------------------------------------------------------

/// Maximum time (in seconds) allowed for the entire raid before it results in
/// defeat.
const RAID_TIMEOUT: f32 = 2400.0;

/// Persistent state for an active village raid.
#[derive(Debug, Clone)]
pub struct Raid {
    pub waves: Vec<RaidWave>,
    pub current_wave: u8,
    pub active: bool,
    pub center: Vec3,
    pub timer: f32,
}

impl Raid {
    /// Create a new raid centred on `center` with the default 7-wave
    /// progression. The raid is not yet active; call [`start`](Self::start) to
    /// begin the first wave.
    pub fn new(center: Vec3) -> Self {
        Self {
            waves: default_raid_waves(),
            current_wave: 0,
            active: false,
            center,
            timer: 0.0,
        }
    }

    /// Activate the raid and reset the timer.
    pub fn start(&mut self) {
        self.active = true;
        self.current_wave = 1;
        self.timer = 0.0;
    }

    /// Advance to the next wave and return the list of mobs to spawn.
    ///
    /// Each returned entry is `(mob_kind, position)`. Mob positions are spread
    /// in a circle around the raid centre.
    pub fn next_wave(&mut self) -> Vec<(u8, Vec3)> {
        let wave = match self
            .waves
            .iter()
            .find(|w| w.wave_number == self.current_wave)
        {
            Some(w) => w,
            None => return Vec::new(),
        };

        let mut spawns: Vec<(u8, Vec3)> = Vec::new();
        let mut index: usize = 0;

        for &(mob_kind, count) in &wave.mobs {
            for _ in 0..count {
                let angle = (index as f32 / 12.0) * std::f32::consts::TAU;
                let radius = 32.0;
                let pos = Vec3::new(
                    self.center.x + angle.cos() * radius,
                    self.center.y,
                    self.center.z + angle.sin() * radius,
                );
                spawns.push((mob_kind, pos));
                index += 1;
            }
        }

        spawns
    }

    /// Check whether the current wave has been cleared and return the
    /// appropriate event.
    ///
    /// * `alive` — number of raid mobs still alive for the current wave.
    pub fn check_wave_clear(&mut self, alive: u32) -> RaidEvent {
        if alive > 0 {
            return RaidEvent::WaveCleared; // not actually cleared — caller should ignore
        }

        let total_waves = self.waves.len() as u8;

        if self.current_wave >= total_waves {
            self.active = false;
            return RaidEvent::RaidVictory;
        }

        self.current_wave += 1;
        RaidEvent::NextWave(self.current_wave)
    }

    /// Per-frame update. Returns the most relevant event for this tick.
    ///
    /// * `alive` — number of raid mobs still alive for the current wave.
    /// * `dt` — delta time in seconds since the last tick.
    pub fn tick(&mut self, alive: u32, dt: f32) -> RaidEvent {
        if !self.active {
            return RaidEvent::RaidDefeat;
        }

        self.timer += dt;

        if self.timer >= RAID_TIMEOUT {
            self.active = false;
            return RaidEvent::RaidDefeat;
        }

        if alive == 0 {
            return self.check_wave_clear(alive);
        }

        RaidEvent::WaveCleared // wave still in progress
    }
}

// ---------------------------------------------------------------------------
// Default raid waves
// ---------------------------------------------------------------------------

/// Returns the standard 7-wave raid progression with increasing mob counts.
///
/// Mob-kind IDs:
///   0 = Pillager, 1 = Vindicator, 2 = Ravager, 3 = Evoker, 4 = Witch
pub fn default_raid_waves() -> Vec<RaidWave> {
    vec![
        RaidWave {
            wave_number: 1,
            mobs: vec![(0, 4), (1, 1)],
        },
        RaidWave {
            wave_number: 2,
            mobs: vec![(0, 3), (1, 2), (4, 1)],
        },
        RaidWave {
            wave_number: 3,
            mobs: vec![(0, 4), (1, 2), (2, 1)],
        },
        RaidWave {
            wave_number: 4,
            mobs: vec![(0, 5), (1, 3), (4, 1), (2, 1)],
        },
        RaidWave {
            wave_number: 5,
            mobs: vec![(0, 5), (1, 4), (4, 2), (2, 1), (3, 1)],
        },
        RaidWave {
            wave_number: 6,
            mobs: vec![(0, 5), (1, 4), (4, 2), (2, 2), (3, 1)],
        },
        RaidWave {
            wave_number: 7,
            mobs: vec![(0, 6), (1, 5), (4, 2), (2, 2), (3, 2)],
        },
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- default_raid_waves ---------------------------------------------------

    #[test]
    fn default_waves_has_seven_waves() {
        let waves = default_raid_waves();
        assert_eq!(waves.len(), 7);
        for (i, wave) in waves.iter().enumerate() {
            assert_eq!(wave.wave_number, (i + 1) as u8);
        }
    }

    #[test]
    fn mob_counts_increase_across_waves() {
        let waves = default_raid_waves();
        let counts: Vec<u8> = waves
            .iter()
            .map(|w| w.mobs.iter().map(|&(_, c)| c).sum())
            .collect();

        for window in counts.windows(2) {
            assert!(
                window[1] >= window[0],
                "wave mob count should not decrease: {} -> {}",
                window[0],
                window[1]
            );
        }
    }

    // -- Raid::new ------------------------------------------------------------

    #[test]
    fn new_raid_is_not_active() {
        let raid = Raid::new(Vec3::ZERO);
        assert!(!raid.active);
        assert_eq!(raid.current_wave, 0);
        assert_eq!(raid.timer, 0.0);
    }

    #[test]
    fn new_raid_has_seven_waves() {
        let raid = Raid::new(Vec3::ZERO);
        assert_eq!(raid.waves.len(), 7);
    }

    // -- Raid::start ----------------------------------------------------------

    #[test]
    fn start_activates_raid() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();
        assert!(raid.active);
        assert_eq!(raid.current_wave, 1);
    }

    // -- Raid::next_wave ------------------------------------------------------

    #[test]
    fn next_wave_spawns_mobs_around_center() {
        let center = Vec3::new(100.0, 64.0, 200.0);
        let mut raid = Raid::new(center);
        raid.start();

        let spawns = raid.next_wave();
        assert!(!spawns.is_empty());

        for (_kind, pos) in &spawns {
            let dx = pos.x - center.x;
            let dz = pos.z - center.z;
            let dist = (dx * dx + dz * dz).sqrt();
            assert!(
                (dist - 32.0).abs() < 0.5,
                "spawn should be ~32 blocks from center, got {dist}"
            );
        }
    }

    #[test]
    fn next_wave_returns_correct_mob_count_for_wave_1() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();
        let spawns = raid.next_wave();
        // Wave 1: (0, 4) + (1, 1) = 5 mobs
        assert_eq!(spawns.len(), 5);
    }

    // -- Raid::check_wave_clear -----------------------------------------------

    #[test]
    fn check_wave_clear_advances_to_next_wave() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();
        assert_eq!(raid.current_wave, 1);

        let event = raid.check_wave_clear(0);
        assert_eq!(event, RaidEvent::NextWave(2));
        assert_eq!(raid.current_wave, 2);
    }

    #[test]
    fn victory_after_clearing_all_waves() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();

        for wave_num in 1..=7u8 {
            assert_eq!(raid.current_wave, wave_num);
            let event = raid.check_wave_clear(0);

            if wave_num < 7 {
                assert_eq!(event, RaidEvent::NextWave(wave_num + 1));
            } else {
                assert_eq!(event, RaidEvent::RaidVictory);
                assert!(!raid.active);
            }
        }
    }

    #[test]
    fn wave_not_cleared_when_mobs_alive() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();

        let event = raid.check_wave_clear(5);
        assert_eq!(event, RaidEvent::WaveCleared);
        // Wave should not advance
        assert_eq!(raid.current_wave, 1);
    }

    // -- Raid::tick -----------------------------------------------------------

    #[test]
    fn tick_returns_defeat_when_not_active() {
        let mut raid = Raid::new(Vec3::ZERO);
        let event = raid.tick(0, 1.0);
        assert_eq!(event, RaidEvent::RaidDefeat);
    }

    #[test]
    fn tick_advances_timer() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();
        raid.tick(5, 10.0);
        assert!((raid.timer - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_returns_defeat_on_timeout() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();

        let event = raid.tick(5, RAID_TIMEOUT + 1.0);
        assert_eq!(event, RaidEvent::RaidDefeat);
        assert!(!raid.active);
    }

    #[test]
    fn tick_clears_wave_when_alive_is_zero() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();

        let event = raid.tick(0, 1.0);
        assert_eq!(event, RaidEvent::NextWave(2));
    }

    #[test]
    fn full_raid_progression_via_tick() {
        let mut raid = Raid::new(Vec3::ZERO);
        raid.start();

        for wave_num in 1..=7u8 {
            assert_eq!(raid.current_wave, wave_num);

            // Simulate some fighting time
            let event = raid.tick(3, 5.0);
            assert_eq!(event, RaidEvent::WaveCleared);

            // Clear the wave
            let event = raid.tick(0, 1.0);
            if wave_num < 7 {
                assert_eq!(event, RaidEvent::NextWave(wave_num + 1));
            } else {
                assert_eq!(event, RaidEvent::RaidVictory);
            }
        }

        assert!(!raid.active);
    }

    // -- Wave progression mob counts ------------------------------------------

    #[test]
    fn wave_7_has_more_mobs_than_wave_1() {
        let waves = default_raid_waves();
        let w1_count: u8 = waves[0].mobs.iter().map(|&(_, c)| c).sum();
        let w7_count: u8 = waves[6].mobs.iter().map(|&(_, c)| c).sum();
        assert!(
            w7_count > w1_count,
            "wave 7 ({w7_count}) should have more mobs than wave 1 ({w1_count})"
        );
    }
}
