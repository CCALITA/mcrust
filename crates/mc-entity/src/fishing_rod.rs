//! Fishing rod cast/bite/reel minigame state machine.
//!
//! Tracks bobber state after a cast, advances a bite timer, and resolves the
//! reel-in into a loot item id. This module is state only — rendering and
//! input wiring live in the client crate.

/// Minimum randomized time until a bite, in seconds.
pub const BITE_TIME_MIN: f32 = 5.0;
/// Maximum randomized time until a bite, in seconds.
pub const BITE_TIME_MAX: f32 = 30.0;

/// Item ids returned by [`reel_in`] when the player successfully catches.
pub const ITEM_LURE: u16 = 3030;
pub const ITEM_COD: u16 = 3015;
pub const ITEM_SALMON: u16 = 3017;
pub const ITEM_CLOWNFISH: u16 = 3031;
pub const ITEM_PUFFERFISH: u16 = 3032;

/// Per-tick state for the fishing rod bobber.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FishingState {
    pub cast: bool,
    pub bobber_pos: [f32; 3],
    pub bite_timer: f32,
    pub time_to_bite: f32,
    pub has_bite: bool,
}

impl FishingState {
    pub fn new() -> Self {
        Self {
            cast: false,
            bobber_pos: [0.0, 0.0, 0.0],
            bite_timer: 0.0,
            time_to_bite: 0.0,
            has_bite: false,
        }
    }
}

impl Default for FishingState {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level state transition reported each tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishingEvent {
    NotCast,
    Casting,
    Waiting,
    Biting,
    Caught(u16),
    Lost,
}

fn hash_u64(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^= x >> 33;
    x
}

fn rand_range(seed: u64, min: f32, max: f32) -> f32 {
    let h = hash_u64(seed);
    // Use top 24 bits for a [0,1) fraction.
    let frac = ((h >> 40) as f32) / ((1u64 << 24) as f32);
    min + frac * (max - min)
}

/// Cast the bobber to `target`, randomizing bite time from `seed`.
pub fn cast_line(state: &mut FishingState, target: [f32; 3], seed: u64) {
    state.cast = true;
    state.bobber_pos = target;
    state.bite_timer = 0.0;
    state.time_to_bite = rand_range(seed, BITE_TIME_MIN, BITE_TIME_MAX);
    state.has_bite = false;
}

/// Advance fishing state by `dt` seconds.
///
/// If the bobber is out of water, the line is "lost" and state is reset.
pub fn tick_fishing(state: &mut FishingState, dt: f32, in_water: bool) -> FishingEvent {
    if !state.cast {
        return FishingEvent::NotCast;
    }
    if !in_water {
        *state = FishingState::new();
        return FishingEvent::Lost;
    }
    if state.has_bite {
        return FishingEvent::Biting;
    }
    state.bite_timer += dt;
    if state.bite_timer >= state.time_to_bite {
        state.has_bite = true;
        return FishingEvent::Biting;
    }
    if state.bite_timer < 0.25 {
        FishingEvent::Casting
    } else {
        FishingEvent::Waiting
    }
}

/// Reel in the line. Returns `Some(item_id)` when the rod was biting.
/// Always resets state.
pub fn reel_in(state: &mut FishingState, has_bite: bool, seed: u64) -> Option<u16> {
    let result = if has_bite {
        let h = hash_u64(seed ^ 0x9E37_79B9_7F4A_7C15);
        let roll = (h % 100) as u8;
        Some(match roll {
            0..=4 => ITEM_PUFFERFISH,
            5..=9 => ITEM_CLOWNFISH,
            10..=14 => ITEM_LURE,
            15..=54 => ITEM_SALMON,
            _ => ITEM_COD,
        })
    } else {
        None
    };
    *state = FishingState::new();
    result
}

/// Per-tick probability of a bite, before random roll.
/// Base 0.005, +0.001 per lure level, +0.001 in rain (`weather == 1`).
pub fn bite_chance_per_tick(weather: u8, lure_level: u8) -> f32 {
    let mut c = 0.005_f32;
    c += 0.001 * lure_level as f32;
    if weather == 1 {
        c += 0.001;
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast_sets_state() {
        let mut s = FishingState::new();
        assert!(!s.cast);
        cast_line(&mut s, [1.0, 64.0, 2.0], 42);
        assert!(s.cast);
        assert_eq!(s.bobber_pos, [1.0, 64.0, 2.0]);
        assert!(s.time_to_bite >= BITE_TIME_MIN && s.time_to_bite <= BITE_TIME_MAX);
        assert_eq!(s.bite_timer, 0.0);
        assert!(!s.has_bite);
    }

    #[test]
    fn tick_advances_timer_and_eventually_bites() {
        let mut s = FishingState::new();
        cast_line(&mut s, [0.0, 0.0, 0.0], 7);
        let before = s.bite_timer;
        let ev = tick_fishing(&mut s, 0.5, true);
        assert!(s.bite_timer > before);
        assert!(matches!(ev, FishingEvent::Casting | FishingEvent::Waiting));

        // Fast-forward past the bite time.
        let _ = tick_fishing(&mut s, BITE_TIME_MAX + 1.0, true);
        assert!(s.has_bite);
    }

    #[test]
    fn tick_not_cast_returns_notcast() {
        let mut s = FishingState::new();
        assert_eq!(tick_fishing(&mut s, 1.0, true), FishingEvent::NotCast);
    }

    #[test]
    fn tick_out_of_water_loses_line() {
        let mut s = FishingState::new();
        cast_line(&mut s, [0.0, 0.0, 0.0], 1);
        assert_eq!(tick_fishing(&mut s, 0.1, false), FishingEvent::Lost);
        assert!(!s.cast);
    }

    #[test]
    fn reel_returns_item_when_biting() {
        let mut s = FishingState::new();
        cast_line(&mut s, [0.0, 0.0, 0.0], 1);
        s.has_bite = true;
        let item = reel_in(&mut s, true, 123);
        assert!(item.is_some());
        let id = item.unwrap();
        assert!(matches!(
            id,
            ITEM_LURE | ITEM_COD | ITEM_SALMON | ITEM_CLOWNFISH | ITEM_PUFFERFISH
        ));
        // state reset
        assert!(!s.cast);
        assert!(!s.has_bite);
    }

    #[test]
    fn reel_returns_none_when_not_biting() {
        let mut s = FishingState::new();
        cast_line(&mut s, [0.0, 0.0, 0.0], 1);
        assert!(reel_in(&mut s, false, 123).is_none());
        assert!(!s.cast);
    }

    #[test]
    fn bite_chance_with_weather_and_lure() {
        assert!((bite_chance_per_tick(0, 0) - 0.005).abs() < 1e-6);
        assert!((bite_chance_per_tick(1, 0) - 0.006).abs() < 1e-6);
        assert!((bite_chance_per_tick(0, 3) - 0.008).abs() < 1e-6);
        assert!((bite_chance_per_tick(1, 3) - 0.009).abs() < 1e-6);
    }
}
