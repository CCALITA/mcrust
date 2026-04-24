//! Sniffer mob digging behaviour and ancient seed drops.
//!
//! The sniffer digs in soil blocks for a fixed duration, then yields an
//! [`AncientSeed`] variant.  After completing a dig it enters a cooldown
//! before it can dig again.

/// The two ancient seed types a sniffer can unearth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AncientSeed {
    Torchflower,
    PitcherPod,
}

/// Duration of a single dig, in seconds.
const DIG_DURATION: f32 = 8.0;

/// Cooldown after a successful dig, in seconds.
const DIG_COOLDOWN_AFTER_COMPLETE: f32 = 300.0;

/// Block IDs considered valid for sniffer digging.
///
/// In Minecraft the sniffer digs in dirt, grass block, podzol, coarse dirt,
/// rooted dirt, moss block, mud, and muddy mangrove roots.  We represent them
/// here as string tags; in a full implementation these would be `BlockId`
/// values from `mc-core`.
const VALID_DIG_BLOCKS: &[&str] = &[
    "dirt",
    "grass_block",
    "podzol",
    "coarse_dirt",
    "rooted_dirt",
    "moss_block",
    "mud",
    "muddy_mangrove_roots",
];

/// Mutable state for a sniffer's digging behaviour.
#[derive(Debug, Clone, PartialEq)]
pub struct SnifferState {
    pub digging: bool,
    pub dig_target: Option<(i32, i32, i32)>,
    pub dig_progress: f32,
    pub cooldown: f32,
}

impl SnifferState {
    /// Create a new idle sniffer state.
    pub fn new() -> Self {
        Self {
            digging: false,
            dig_target: None,
            dig_progress: 0.0,
            cooldown: 0.0,
        }
    }
}

impl Default for SnifferState {
    fn default() -> Self {
        Self::new()
    }
}

/// Duration of a single sniffer dig, in seconds.
pub fn dig_duration() -> f32 {
    DIG_DURATION
}

/// Cooldown imposed after a successful dig, in seconds.
pub fn dig_cooldown_after_complete() -> f32 {
    DIG_COOLDOWN_AFTER_COMPLETE
}

/// Returns `true` if the given block tag is a valid digging target.
pub fn valid_dig_block(block: &str) -> bool {
    VALID_DIG_BLOCKS.contains(&block)
}

/// Begin digging at the given position.
///
/// Returns an updated [`SnifferState`] with digging active, or `None` if the
/// sniffer is still on cooldown or already digging.
pub fn start_digging(state: &SnifferState, target: (i32, i32, i32)) -> Option<SnifferState> {
    if state.digging || state.cooldown > 0.0 {
        return None;
    }
    Some(SnifferState {
        digging: true,
        dig_target: Some(target),
        dig_progress: 0.0,
        cooldown: 0.0,
    })
}

/// Advance the dig by `dt` seconds.
///
/// Returns the new state and, when the dig completes, the [`AncientSeed`]
/// that was unearthed.  The `seed` parameter selects which ancient seed
/// variant is produced (deterministic, e.g. from world RNG).
pub fn tick_digging(state: &SnifferState, dt: f32, seed: u64) -> (SnifferState, Option<AncientSeed>) {
    if !state.digging {
        // Not digging — just tick cooldown toward zero.
        let new_cooldown = (state.cooldown - dt).max(0.0);
        let new_state = SnifferState {
            digging: false,
            dig_target: state.dig_target,
            dig_progress: state.dig_progress,
            cooldown: new_cooldown,
        };
        return (new_state, None);
    }

    let new_progress = state.dig_progress + dt;
    if new_progress >= DIG_DURATION {
        // Dig complete — produce a seed, enter cooldown.
        let ancient_seed = if seed % 2 == 0 {
            AncientSeed::Torchflower
        } else {
            AncientSeed::PitcherPod
        };
        let new_state = SnifferState {
            digging: false,
            dig_target: None,
            dig_progress: 0.0,
            cooldown: DIG_COOLDOWN_AFTER_COMPLETE,
        };
        (new_state, Some(ancient_seed))
    } else {
        // Still digging.
        let new_state = SnifferState {
            digging: true,
            dig_target: state.dig_target,
            dig_progress: new_progress,
            cooldown: 0.0,
        };
        (new_state, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_idle() {
        let state = SnifferState::new();
        assert!(!state.digging);
        assert!(state.dig_target.is_none());
        assert!((state.dig_progress).abs() < f32::EPSILON);
        assert!((state.cooldown).abs() < f32::EPSILON);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(SnifferState::default(), SnifferState::new());
    }

    #[test]
    fn dig_duration_value() {
        assert!((dig_duration() - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn dig_cooldown_value() {
        assert!((dig_cooldown_after_complete() - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn valid_dig_blocks_accepted() {
        assert!(valid_dig_block("dirt"));
        assert!(valid_dig_block("grass_block"));
        assert!(valid_dig_block("podzol"));
        assert!(valid_dig_block("coarse_dirt"));
        assert!(valid_dig_block("rooted_dirt"));
        assert!(valid_dig_block("moss_block"));
        assert!(valid_dig_block("mud"));
        assert!(valid_dig_block("muddy_mangrove_roots"));
    }

    #[test]
    fn invalid_dig_blocks_rejected() {
        assert!(!valid_dig_block("stone"));
        assert!(!valid_dig_block("sand"));
        assert!(!valid_dig_block("oak_planks"));
    }

    #[test]
    fn start_digging_from_idle() {
        let state = SnifferState::new();
        let result = start_digging(&state, (10, 64, -5));
        assert!(result.is_some());
        let next = result.unwrap();
        assert!(next.digging);
        assert_eq!(next.dig_target, Some((10, 64, -5)));
        assert!((next.dig_progress).abs() < f32::EPSILON);
    }

    #[test]
    fn start_digging_while_already_digging_fails() {
        let state = SnifferState {
            digging: true,
            dig_target: Some((0, 0, 0)),
            dig_progress: 3.0,
            cooldown: 0.0,
        };
        assert!(start_digging(&state, (1, 2, 3)).is_none());
    }

    #[test]
    fn start_digging_during_cooldown_fails() {
        let state = SnifferState {
            digging: false,
            dig_target: None,
            dig_progress: 0.0,
            cooldown: 100.0,
        };
        assert!(start_digging(&state, (1, 2, 3)).is_none());
    }

    #[test]
    fn tick_advances_progress() {
        let state = SnifferState {
            digging: true,
            dig_target: Some((0, 60, 0)),
            dig_progress: 0.0,
            cooldown: 0.0,
        };
        let (next, seed) = tick_digging(&state, 2.0, 0);
        assert!(seed.is_none());
        assert!(next.digging);
        assert!((next.dig_progress - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_completes_dig_and_yields_seed() {
        let state = SnifferState {
            digging: true,
            dig_target: Some((5, 62, 10)),
            dig_progress: 7.0,
            cooldown: 0.0,
        };
        let (next, seed) = tick_digging(&state, 1.5, 42);
        assert!(seed.is_some());
        assert!(!next.digging);
        assert!(next.dig_target.is_none());
        assert!((next.dig_progress).abs() < f32::EPSILON);
        assert!((next.cooldown - DIG_COOLDOWN_AFTER_COMPLETE).abs() < f32::EPSILON);
    }

    #[test]
    fn even_seed_yields_torchflower() {
        let state = SnifferState {
            digging: true,
            dig_target: Some((0, 0, 0)),
            dig_progress: 7.5,
            cooldown: 0.0,
        };
        let (_, seed) = tick_digging(&state, 1.0, 100);
        assert_eq!(seed, Some(AncientSeed::Torchflower));
    }

    #[test]
    fn odd_seed_yields_pitcher_pod() {
        let state = SnifferState {
            digging: true,
            dig_target: Some((0, 0, 0)),
            dig_progress: 7.5,
            cooldown: 0.0,
        };
        let (_, seed) = tick_digging(&state, 1.0, 101);
        assert_eq!(seed, Some(AncientSeed::PitcherPod));
    }

    #[test]
    fn tick_idle_decrements_cooldown() {
        let state = SnifferState {
            digging: false,
            dig_target: None,
            dig_progress: 0.0,
            cooldown: 10.0,
        };
        let (next, seed) = tick_digging(&state, 3.0, 0);
        assert!(seed.is_none());
        assert!((next.cooldown - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cooldown_does_not_go_negative() {
        let state = SnifferState {
            digging: false,
            dig_target: None,
            dig_progress: 0.0,
            cooldown: 2.0,
        };
        let (next, _) = tick_digging(&state, 5.0, 0);
        assert!((next.cooldown).abs() < f32::EPSILON);
    }

    #[test]
    fn full_lifecycle() {
        // Start idle, begin digging, tick to completion, enter cooldown, wait out cooldown.
        let s0 = SnifferState::new();
        let s1 = start_digging(&s0, (3, 65, -2)).unwrap();
        assert!(s1.digging);

        // Tick partway.
        let (s2, seed) = tick_digging(&s1, 4.0, 0);
        assert!(seed.is_none());
        assert!(s2.digging);

        // Tick to completion.
        let (s3, seed) = tick_digging(&s2, 5.0, 7);
        assert_eq!(seed, Some(AncientSeed::PitcherPod));
        assert!(!s3.digging);
        assert!(s3.cooldown > 0.0);

        // Cannot start digging during cooldown.
        assert!(start_digging(&s3, (0, 0, 0)).is_none());

        // Tick through full cooldown.
        let (s4, _) = tick_digging(&s3, DIG_COOLDOWN_AFTER_COMPLETE + 1.0, 0);
        assert!((s4.cooldown).abs() < f32::EPSILON);

        // Now can dig again.
        assert!(start_digging(&s4, (1, 1, 1)).is_some());
    }

    #[test]
    fn ancient_seed_enum_variants() {
        let seeds = [AncientSeed::Torchflower, AncientSeed::PitcherPod];
        assert_eq!(seeds.len(), 2);
        assert_ne!(seeds[0], seeds[1]);
    }
}
