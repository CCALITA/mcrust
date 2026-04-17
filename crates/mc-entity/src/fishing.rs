use glam::Vec3;

// ---------------------------------------------------------------------------
// Loot types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FishType {
    Cod,
    Salmon,
    TropicalFish,
    Pufferfish,
}

const FISH_TYPES: [FishType; 4] = [
    FishType::Cod,
    FishType::Salmon,
    FishType::TropicalFish,
    FishType::Pufferfish,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JunkType {
    Leather,
    Stick,
    String,
    Bowl,
    Tripwire,
    RottenFlesh,
    WaterBottle,
    Bone,
    InkSac,
    LilyPad,
}

const JUNK_TYPES: [JunkType; 10] = [
    JunkType::Leather,
    JunkType::Stick,
    JunkType::String,
    JunkType::Bowl,
    JunkType::Tripwire,
    JunkType::RottenFlesh,
    JunkType::WaterBottle,
    JunkType::Bone,
    JunkType::InkSac,
    JunkType::LilyPad,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreasureType {
    Bow,
    FishingRod,
    EnchantedBook,
    NameTag,
    Saddle,
    Nautilus,
}

const TREASURE_TYPES: [TreasureType; 6] = [
    TreasureType::Bow,
    TreasureType::FishingRod,
    TreasureType::EnchantedBook,
    TreasureType::NameTag,
    TreasureType::Saddle,
    TreasureType::Nautilus,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FishingLoot {
    Fish(FishType),
    Junk(JunkType),
    Treasure(TreasureType),
}

// ---------------------------------------------------------------------------
// Fishing state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum FishingState {
    Idle,
    Cast {
        bobber_pos: Vec3,
        wait_time: f32,
        bite_timer: f32,
        has_bite: bool,
    },
    Reeling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishingAction {
    None,
    BobberSplash,
    FishEscaped,
}

// ---------------------------------------------------------------------------
// Deterministic hashing helper
// ---------------------------------------------------------------------------

/// Simple deterministic hash combining seed and tick for reproducible randomness.
fn hash_u64(seed: u64, tick: u64) -> u64 {
    let mut h = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(tick);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

/// Return a float in `[0.0, 1.0)` from a hash value.
fn hash_to_f32(h: u64) -> f32 {
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// Return a float in `[lo, hi)` from seed + tick.
fn rand_range(seed: u64, tick: u64, lo: f32, hi: f32) -> f32 {
    lo + hash_to_f32(hash_u64(seed, tick)) * (hi - lo)
}

/// Return an index in `[0, count)` from seed + tick.
fn rand_index(seed: u64, tick: u64, count: usize) -> usize {
    (hash_u64(seed, tick) % count as u64) as usize
}

// ---------------------------------------------------------------------------
// Fishing system
// ---------------------------------------------------------------------------

pub struct FishingSystem;

impl FishingSystem {
    /// Cast a fishing line. The bobber lands 5-10 blocks along `look_dir`
    /// from `player_pos`.
    pub fn cast(player_pos: Vec3, look_dir: Vec3, seed: u64) -> FishingState {
        let distance = rand_range(seed, 0, 5.0, 10.0);
        let dir = if look_dir.length_squared() > f32::EPSILON {
            look_dir.normalize()
        } else {
            Vec3::Z
        };
        let bobber_pos = player_pos + dir * distance;
        let wait_time = rand_range(seed, 1, 5.0, 30.0);

        FishingState::Cast {
            bobber_pos,
            wait_time,
            bite_timer: 0.0,
            has_bite: false,
        }
    }

    /// Advance the fishing state by `dt` seconds.
    ///
    /// While in the `Cast` state the bobber waits for a fish to bite. When
    /// `wait_time` elapses, a bite occurs (`BobberSplash`). The player then
    /// has a 2-second window to reel in; if the window expires the fish
    /// escapes and the timer is reset.
    pub fn tick(state: &mut FishingState, dt: f32, seed: u64, tick: u64) -> FishingAction {
        let FishingState::Cast {
            wait_time,
            bite_timer,
            has_bite,
            ..
        } = state
        else {
            return FishingAction::None;
        };

        if *has_bite {
            // Bite window: 2 seconds
            *bite_timer += dt;
            if *bite_timer >= 2.0 {
                // Fish escaped — reset for another bite cycle
                *has_bite = false;
                *bite_timer = 0.0;
                *wait_time = rand_range(seed, tick, 5.0, 30.0);
                return FishingAction::FishEscaped;
            }
            return FishingAction::None;
        }

        // Waiting for a bite
        *bite_timer += dt;
        if *bite_timer >= *wait_time {
            *has_bite = true;
            *bite_timer = 0.0;
            return FishingAction::BobberSplash;
        }

        FishingAction::None
    }

    /// Reel in the line. If a fish is biting, determine and return the loot.
    /// Otherwise return `None` (wasted reel).
    ///
    /// `luck_of_sea` is the Luck of the Sea enchantment level (0-3).
    /// It shifts loot probabilities: Fish `85 - luck*1`%, Junk `10 - luck*2`%,
    /// Treasure `5 + luck*3`%.
    pub fn reel(
        state: &mut FishingState,
        luck_of_sea: u8,
        seed: u64,
        tick: u64,
    ) -> Option<FishingLoot> {
        let had_bite = matches!(state, FishingState::Cast { has_bite: true, .. });
        *state = FishingState::Idle;

        if !had_bite {
            return None;
        }

        let luck = luck_of_sea.min(3) as f32;
        let fish_pct = (85.0 - luck) / 100.0;
        let junk_pct = (10.0 - luck * 2.0).max(0.0) / 100.0;
        // treasure_pct fills the remainder so they always sum to 1.0
        let _treasure_pct = 1.0 - fish_pct - junk_pct;

        let roll = hash_to_f32(hash_u64(seed, tick));

        let loot = if roll < fish_pct {
            let idx = rand_index(seed, tick.wrapping_add(1), FISH_TYPES.len());
            FishingLoot::Fish(FISH_TYPES[idx])
        } else if roll < fish_pct + junk_pct {
            let idx = rand_index(seed, tick.wrapping_add(2), JUNK_TYPES.len());
            FishingLoot::Junk(JUNK_TYPES[idx])
        } else {
            let idx = rand_index(seed, tick.wrapping_add(3), TREASURE_TYPES.len());
            FishingLoot::Treasure(TREASURE_TYPES[idx])
        };

        Some(loot)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- cast ---------------------------------------------------------------

    #[test]
    fn cast_places_bobber_in_correct_direction() {
        let player = Vec3::new(0.0, 64.0, 0.0);
        let look = Vec3::new(1.0, 0.0, 0.0);
        let state = FishingSystem::cast(player, look, 42);

        if let FishingState::Cast { bobber_pos, .. } = state {
            // Bobber should be 5-10 blocks in +X
            let diff = bobber_pos - player;
            assert!(diff.x >= 5.0, "bobber too close: {}", diff.x);
            assert!(diff.x <= 10.0, "bobber too far: {}", diff.x);
            assert!(
                diff.y.abs() < f32::EPSILON,
                "bobber Y should match look dir"
            );
            assert!(
                diff.z.abs() < f32::EPSILON,
                "bobber Z should be zero for +X look"
            );
        } else {
            panic!("expected Cast state after casting");
        }
    }

    #[test]
    fn cast_with_different_seeds_varies_distance() {
        let player = Vec3::ZERO;
        let look = Vec3::X;
        let mut distances = Vec::new();
        for seed in 0..20 {
            if let FishingState::Cast { bobber_pos, .. } = FishingSystem::cast(player, look, seed) {
                distances.push((bobber_pos - player).length());
            }
        }
        // Not all distances should be identical
        let first = distances[0];
        assert!(
            distances.iter().any(|d| (d - first).abs() > 0.01),
            "different seeds should produce different distances"
        );
    }

    #[test]
    fn cast_with_zero_look_dir_uses_fallback() {
        let state = FishingSystem::cast(Vec3::ZERO, Vec3::ZERO, 99);
        if let FishingState::Cast { bobber_pos, .. } = state {
            // Fallback is +Z
            assert!(bobber_pos.z > 0.0, "should fall back to +Z direction");
        } else {
            panic!("expected Cast state");
        }
    }

    // -- tick / bite timing -------------------------------------------------

    #[test]
    fn bite_happens_within_time_window() {
        let player = Vec3::ZERO;
        let look = Vec3::X;
        let seed = 12345u64;
        let mut state = FishingSystem::cast(player, look, seed);

        let wait = if let FishingState::Cast { wait_time, .. } = &state {
            *wait_time
        } else {
            panic!("expected Cast");
        };

        // Advance time just past the wait_time in small steps
        let mut elapsed = 0.0f32;
        let mut got_splash = false;
        while elapsed < wait + 1.0 {
            let action = FishingSystem::tick(&mut state, 0.1, seed, elapsed as u64);
            elapsed += 0.1;
            if action == FishingAction::BobberSplash {
                got_splash = true;
                break;
            }
        }

        assert!(got_splash, "should get BobberSplash within wait window");
        if let FishingState::Cast { has_bite, .. } = &state {
            assert!(has_bite, "has_bite should be true after splash");
        }
    }

    #[test]
    fn fish_escapes_after_bite_window_expires() {
        let player = Vec3::ZERO;
        let look = Vec3::X;
        let seed = 777u64;
        let mut state = FishingSystem::cast(player, look, seed);

        // Force a bite by advancing past wait_time
        let mut elapsed = 0.0f32;
        loop {
            let action = FishingSystem::tick(&mut state, 0.1, seed, elapsed as u64);
            elapsed += 0.1;
            if action == FishingAction::BobberSplash {
                break;
            }
            assert!(elapsed < 35.0, "timed out waiting for bite");
        }

        // Now advance past the 2-second bite window without reeling
        let mut escaped = false;
        for _ in 0..30 {
            let action = FishingSystem::tick(&mut state, 0.1, seed, elapsed as u64);
            elapsed += 0.1;
            if action == FishingAction::FishEscaped {
                escaped = true;
                break;
            }
        }

        assert!(escaped, "fish should escape after 2s bite window");
        if let FishingState::Cast { has_bite, .. } = &state {
            assert!(!has_bite, "has_bite should be false after escape");
        }
    }

    #[test]
    fn tick_on_idle_state_returns_none() {
        let mut state = FishingState::Idle;
        let action = FishingSystem::tick(&mut state, 1.0, 0, 0);
        assert_eq!(action, FishingAction::None);
    }

    // -- reel ---------------------------------------------------------------

    #[test]
    fn reel_with_bite_returns_loot() {
        let mut state = FishingState::Cast {
            bobber_pos: Vec3::new(5.0, 64.0, 0.0),
            wait_time: 10.0,
            bite_timer: 0.0,
            has_bite: true,
        };
        let loot = FishingSystem::reel(&mut state, 0, 42, 100);
        assert!(loot.is_some(), "should get loot when bite is active");
        assert_eq!(state, FishingState::Idle);
    }

    #[test]
    fn reel_without_bite_returns_none() {
        let mut state = FishingState::Cast {
            bobber_pos: Vec3::new(5.0, 64.0, 0.0),
            wait_time: 10.0,
            bite_timer: 3.0,
            has_bite: false,
        };
        let loot = FishingSystem::reel(&mut state, 0, 42, 100);
        assert!(loot.is_none(), "should get nothing without a bite");
        assert_eq!(state, FishingState::Idle);
    }

    #[test]
    fn reel_on_idle_returns_none() {
        let mut state = FishingState::Idle;
        let loot = FishingSystem::reel(&mut state, 0, 42, 100);
        assert!(loot.is_none());
    }

    // -- loot distribution --------------------------------------------------

    #[test]
    fn loot_distribution_roughly_matches_expected_ratios() {
        let iterations = 100_000;
        let mut fish_count = 0u32;
        let mut junk_count = 0u32;
        let mut treasure_count = 0u32;

        for i in 0..iterations {
            let mut state = FishingState::Cast {
                bobber_pos: Vec3::ZERO,
                wait_time: 10.0,
                bite_timer: 0.0,
                has_bite: true,
            };
            if let Some(loot) = FishingSystem::reel(&mut state, 0, i as u64, i as u64 * 7) {
                match loot {
                    FishingLoot::Fish(_) => fish_count += 1,
                    FishingLoot::Junk(_) => junk_count += 1,
                    FishingLoot::Treasure(_) => treasure_count += 1,
                }
            }
        }

        let total = (fish_count + junk_count + treasure_count) as f32;
        let fish_pct = fish_count as f32 / total * 100.0;
        let junk_pct = junk_count as f32 / total * 100.0;
        let treasure_pct = treasure_count as f32 / total * 100.0;

        // Expected: Fish 85%, Junk 10%, Treasure 5% — allow +/- 3%
        assert!(
            (fish_pct - 85.0).abs() < 3.0,
            "fish % = {fish_pct}, expected ~85%"
        );
        assert!(
            (junk_pct - 10.0).abs() < 3.0,
            "junk % = {junk_pct}, expected ~10%"
        );
        assert!(
            (treasure_pct - 5.0).abs() < 3.0,
            "treasure % = {treasure_pct}, expected ~5%"
        );
    }

    #[test]
    fn luck_of_sea_increases_treasure_rate() {
        let iterations = 100_000;

        let mut treasure_no_luck = 0u32;
        let mut treasure_max_luck = 0u32;

        for i in 0..iterations {
            let seed = i as u64;
            let tick = i as u64 * 13;

            let mut state0 = FishingState::Cast {
                bobber_pos: Vec3::ZERO,
                wait_time: 10.0,
                bite_timer: 0.0,
                has_bite: true,
            };
            if let Some(FishingLoot::Treasure(_)) = FishingSystem::reel(&mut state0, 0, seed, tick)
            {
                treasure_no_luck += 1;
            }

            let mut state3 = FishingState::Cast {
                bobber_pos: Vec3::ZERO,
                wait_time: 10.0,
                bite_timer: 0.0,
                has_bite: true,
            };
            if let Some(FishingLoot::Treasure(_)) = FishingSystem::reel(&mut state3, 3, seed, tick)
            {
                treasure_max_luck += 1;
            }
        }

        assert!(
            treasure_max_luck > treasure_no_luck,
            "luck 3 ({treasure_max_luck}) should yield more treasure than luck 0 ({treasure_no_luck})"
        );

        // Luck 3: treasure = 5 + 3*3 = 14%
        let pct_max = treasure_max_luck as f32 / iterations as f32 * 100.0;
        assert!(
            (pct_max - 14.0).abs() < 3.0,
            "luck 3 treasure % = {pct_max}, expected ~14%"
        );
    }

    // -- all loot variants reachable ----------------------------------------

    #[test]
    fn all_fish_types_are_reachable() {
        let mut seen = std::collections::HashSet::new();
        for i in 0..10_000u64 {
            let mut state = FishingState::Cast {
                bobber_pos: Vec3::ZERO,
                wait_time: 10.0,
                bite_timer: 0.0,
                has_bite: true,
            };
            if let Some(FishingLoot::Fish(f)) = FishingSystem::reel(&mut state, 0, i, i * 3) {
                seen.insert(f);
            }
        }
        for ft in &FISH_TYPES {
            assert!(seen.contains(ft), "FishType {:?} never appeared", ft);
        }
    }

    #[test]
    fn all_treasure_types_are_reachable() {
        let mut seen = std::collections::HashSet::new();
        // Use luck 3 to increase treasure chance
        for i in 0..100_000u64 {
            let mut state = FishingState::Cast {
                bobber_pos: Vec3::ZERO,
                wait_time: 10.0,
                bite_timer: 0.0,
                has_bite: true,
            };
            if let Some(FishingLoot::Treasure(t)) = FishingSystem::reel(&mut state, 3, i, i * 3) {
                seen.insert(t);
            }
        }
        for tt in &TREASURE_TYPES {
            assert!(seen.contains(tt), "TreasureType {:?} never appeared", tt);
        }
    }
}
