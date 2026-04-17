//! Weather system that cycles through Clear, Rain, and Thunder states
//! using deterministic hashing for reproducible "random" transitions.

/// The three weather states a world can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherState {
    Clear,
    Rain,
    Thunder,
}

/// Tracks current weather, transition timers, and rain intensity.
pub struct WeatherSystem {
    state: WeatherState,
    timer: u32,
    rain_strength: f32,
    seed: u64,
    tick_count: u64,
}

/// Deterministic hash combining a world seed with a tick counter.
///
/// Produces a pseudo-random `u64` without requiring external RNG crates,
/// making weather behaviour fully reproducible for a given seed.
fn weather_hash(seed: u64, tick: u64) -> u64 {
    let mut h = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(tick);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// Derive a timer duration centred on `avg` ticks from a hash value.
///
/// The returned value is in `avg/2 ..= avg*3/2`, giving a uniform spread
/// around the average.
fn timer_from_hash(hash: u64, avg: u32) -> u32 {
    let half = avg / 2;
    let range = avg; // avg/2 .. avg + avg/2
    half + (hash % range as u64) as u32
}

/// Rate at which `rain_strength` fades toward its target each tick.
const FADE_RATE: f32 = 0.01;

impl WeatherSystem {
    /// Create a new weather system seeded with `seed`.
    ///
    /// The world starts in `Clear` with a random timer derived from the seed.
    pub fn new(seed: u64) -> Self {
        let initial_hash = weather_hash(seed, 0);
        Self {
            state: WeatherState::Clear,
            timer: timer_from_hash(initial_hash, 12_000),
            rain_strength: 0.0,
            seed,
            tick_count: 0,
        }
    }

    /// Advance the weather by one game tick.
    ///
    /// Decrements the internal timer, transitions state when the timer
    /// reaches zero, and smoothly fades `rain_strength` toward its target.
    pub fn tick(&mut self) {
        self.tick_count += 1;

        // Timer countdown and state transition.
        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            let h = weather_hash(self.seed, self.tick_count);
            match self.state {
                WeatherState::Clear => {
                    self.state = WeatherState::Rain;
                    self.timer = timer_from_hash(h, 6_000);
                }
                WeatherState::Rain => {
                    // 50/50 chance: escalate to thunder or clear up.
                    if h.is_multiple_of(2) {
                        self.state = WeatherState::Thunder;
                        self.timer = timer_from_hash(h, 3_000);
                    } else {
                        self.state = WeatherState::Clear;
                        self.timer = timer_from_hash(h, 12_000);
                    }
                }
                WeatherState::Thunder => {
                    self.state = WeatherState::Rain;
                    self.timer = timer_from_hash(h, 6_000);
                }
            }
        }

        // Fade rain_strength toward the target for the current state.
        let target = match self.state {
            WeatherState::Clear => 0.0,
            WeatherState::Rain | WeatherState::Thunder => 1.0,
        };

        if (self.rain_strength - target).abs() < FADE_RATE {
            self.rain_strength = target;
        } else if self.rain_strength < target {
            self.rain_strength += FADE_RATE;
        } else {
            self.rain_strength -= FADE_RATE;
        }
    }

    /// `true` when the world is experiencing rain (or thunder, which includes rain).
    pub fn is_raining(&self) -> bool {
        matches!(self.state, WeatherState::Rain | WeatherState::Thunder)
    }

    /// `true` only during a thunderstorm.
    pub fn is_thundering(&self) -> bool {
        self.state == WeatherState::Thunder
    }

    /// Current rain intensity in `0.0..=1.0`.
    pub fn rain_strength(&self) -> f32 {
        self.rain_strength
    }

    /// During a thunderstorm, returns a random `(x, z)` coordinate within
    /// 128 blocks of the origin with a 1-in-1000 chance per tick.
    ///
    /// Returns `None` when it is not thundering or the random roll fails.
    pub fn lightning_strike(&self) -> Option<(i32, i32)> {
        if self.state != WeatherState::Thunder {
            return None;
        }
        let h = weather_hash(self.seed, self.tick_count.wrapping_add(0xdead));
        if !h.is_multiple_of(1000) {
            return None;
        }
        // Derive x, z in -128..=127 (256-wide range).
        let h2 = weather_hash(self.seed, self.tick_count.wrapping_add(0xbeef));
        let x = (h2 % 256) as i32 - 128;
        let z = ((h2 >> 16) % 256) as i32 - 128;
        Some((x, z))
    }

    /// Rain allows hostile mobs to spawn during daytime.
    pub fn affects_mob_spawning(&self) -> bool {
        self.is_raining()
    }

    /// Sky darkness multiplier: 0.0 for clear, 0.3 for rain, 0.5 for thunder.
    pub fn sky_darkness(&self) -> f32 {
        match self.state {
            WeatherState::Clear => 0.0,
            WeatherState::Rain => 0.3,
            WeatherState::Thunder => 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_clear() {
        let ws = WeatherSystem::new(42);
        assert_eq!(ws.state, WeatherState::Clear);
        assert!(!ws.is_raining());
        assert!(!ws.is_thundering());
        assert_eq!(ws.rain_strength(), 0.0);
    }

    #[test]
    fn transitions_cycle_through_states() {
        let mut ws = WeatherSystem::new(42);

        // Fast-forward until the first state change.
        let mut seen_rain = false;
        let mut seen_thunder = false;
        let mut seen_clear_again = false;

        for _ in 0..100_000 {
            ws.tick();
            match ws.state {
                WeatherState::Rain if !seen_rain => seen_rain = true,
                WeatherState::Thunder if seen_rain && !seen_thunder => seen_thunder = true,
                WeatherState::Clear if seen_rain && !seen_clear_again => seen_clear_again = true,
                _ => {}
            }
            if seen_rain && seen_clear_again {
                break;
            }
        }

        assert!(seen_rain, "should have transitioned to Rain");
        // At least one of thunder or clear-again should have been reached
        // (50/50 per rain exit, so over many transitions both are very likely).
        assert!(
            seen_thunder || seen_clear_again,
            "should have transitioned beyond Rain"
        );
    }

    #[test]
    fn rain_strength_fades_toward_target() {
        let mut ws = WeatherSystem::new(42);

        // Tick until it starts raining.
        while ws.state == WeatherState::Clear {
            ws.tick();
        }
        assert!(ws.is_raining());

        // Rain strength should climb toward 1.0.
        for _ in 0..200 {
            ws.tick();
            if ws.rain_strength() >= 0.99 {
                break;
            }
        }
        assert!(
            ws.rain_strength() > 0.5,
            "rain_strength should increase while raining, got {}",
            ws.rain_strength()
        );

        // Now fast-forward until it clears.
        let prev_state = ws.state;
        while ws.state != WeatherState::Clear || ws.state == prev_state {
            ws.tick();
            if ws.state == WeatherState::Clear {
                break;
            }
        }

        // Rain strength should fade toward 0.0.
        for _ in 0..200 {
            ws.tick();
            if ws.rain_strength() <= 0.01 {
                break;
            }
        }
        assert!(
            ws.rain_strength() < 0.5,
            "rain_strength should decrease after clearing, got {}",
            ws.rain_strength()
        );
    }

    #[test]
    fn lightning_only_during_thunder() {
        let mut ws = WeatherSystem::new(42);

        // During clear weather, lightning should never strike.
        for _ in 0..10_000 {
            assert!(ws.lightning_strike().is_none(), "no lightning when clear");
            ws.tick();
            if ws.state != WeatherState::Clear {
                break;
            }
        }

        // Fast-forward to thunder.
        for _ in 0..200_000 {
            ws.tick();
            if ws.is_thundering() {
                break;
            }
        }

        if ws.is_thundering() {
            // During thunder, lightning may or may not strike (1/1000 chance).
            let mut any_strike = false;
            for _ in 0..10_000 {
                if ws.lightning_strike().is_some() {
                    any_strike = true;
                    break;
                }
                ws.tick();
                if !ws.is_thundering() {
                    break;
                }
            }
            // We only assert if we stayed in thunder long enough;
            // with 1/1000 odds over several thousand ticks, a hit is very likely.
            if ws.is_thundering() {
                assert!(
                    any_strike,
                    "expected at least one lightning strike during thunder"
                );
            }
        }
    }

    #[test]
    fn lightning_coordinates_within_range() {
        // Brute-force a seed + tick that yields a strike.
        for seed in 0..100u64 {
            let mut ws = WeatherSystem::new(seed);
            // Force into thunder.
            ws.state = WeatherState::Thunder;
            ws.timer = 10_000;
            for _ in 0..5_000 {
                ws.tick();
                if let Some((x, z)) = ws.lightning_strike() {
                    assert!((-128..=127).contains(&x), "x={x} out of range");
                    assert!((-128..=127).contains(&z), "z={z} out of range");
                    return; // success
                }
            }
        }
        panic!("no lightning strike found in any tested seed");
    }

    #[test]
    fn sky_darkness_values() {
        let mut ws = WeatherSystem::new(42);

        ws.state = WeatherState::Clear;
        assert!((ws.sky_darkness() - 0.0).abs() < f32::EPSILON);

        ws.state = WeatherState::Rain;
        assert!((ws.sky_darkness() - 0.3).abs() < f32::EPSILON);

        ws.state = WeatherState::Thunder;
        assert!((ws.sky_darkness() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn affects_mob_spawning_matches_rain() {
        let mut ws = WeatherSystem::new(42);

        ws.state = WeatherState::Clear;
        assert!(!ws.affects_mob_spawning());

        ws.state = WeatherState::Rain;
        assert!(ws.affects_mob_spawning());

        ws.state = WeatherState::Thunder;
        assert!(ws.affects_mob_spawning());
    }

    #[test]
    fn weather_hash_is_deterministic() {
        let a = weather_hash(42, 100);
        let b = weather_hash(42, 100);
        assert_eq!(a, b);

        // Different inputs produce different hashes (not a hard guarantee but
        // practically certain for these inputs).
        let c = weather_hash(42, 101);
        assert_ne!(a, c);
    }

    #[test]
    fn timer_from_hash_within_bounds() {
        for tick in 0..1000u64 {
            let h = weather_hash(42, tick);
            let t = timer_from_hash(h, 12_000);
            assert!(t >= 6_000, "timer too low: {t}");
            assert!(t <= 18_000, "timer too high: {t}");
        }
    }
}
