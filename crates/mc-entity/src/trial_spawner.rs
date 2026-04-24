//! Trial spawner mechanics for trial chambers.
//!
//! Trial spawners activate when players are nearby, scale difficulty with
//! player count, and enter a 30-minute cooldown after all mobs are spawned.
//! Ominous trial spawners double max mobs and add tougher mob types.

/// Mob kind identifiers returned by trial spawner functions.
pub const MOB_ZOMBIE: u8 = 0;
pub const MOB_SKELETON: u8 = 1;
pub const MOB_SPIDER: u8 = 2;
pub const MOB_BREEZE: u8 = 3;
pub const MOB_BOGGED: u8 = 4;

/// Spawn interval in seconds — one mob every 1.5 seconds.
const SPAWN_INTERVAL: f32 = 1.5;

/// State of a trial spawner block.
#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnerState {
    pub active: bool,
    pub mobs_spawned: u8,
    pub max_mobs: u8,
    pub cooldown: f32,
    pub players_nearby: u8,
    pub ominous: bool,
}

impl TrialSpawnerState {
    /// Create a new inactive trial spawner with default values.
    pub fn new() -> Self {
        Self {
            active: false,
            mobs_spawned: 0,
            max_mobs: 0,
            cooldown: 0.0,
            players_nearby: 0,
            ominous: false,
        }
    }
}

impl Default for TrialSpawnerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Activate a trial spawner when players are detected nearby.
///
/// Sets the spawner to active, records the player count, and calculates the
/// maximum number of mobs to spawn based on player count and ominous state.
pub fn activate_spawner(state: &mut TrialSpawnerState, players: u8) {
    state.active = true;
    state.players_nearby = players;
    state.max_mobs = max_mobs_for_players(players, state.ominous);
    state.mobs_spawned = 0;
    state.cooldown = 0.0;
}

/// Calculate the maximum number of mobs for a given player count.
///
/// Base formula: 2 * players. Ominous mode doubles that further.
pub fn max_mobs_for_players(players: u8, ominous: bool) -> u8 {
    let base = 2u16 * u16::from(players);
    let total = if ominous { base * 2 } else { base };
    // Clamp to u8 range.
    total.min(u16::from(u8::MAX)) as u8
}

/// Tick the trial spawner, advancing its internal timer.
///
/// If the spawner is active and has not yet spawned all mobs, it spawns one
/// mob every [`SPAWN_INTERVAL`] seconds. Returns `Some(mob_kind)` when a mob
/// should be spawned this tick, or `None` otherwise.
///
/// When all mobs have been spawned, the spawner deactivates and enters
/// cooldown.
pub fn tick_spawner(state: &mut TrialSpawnerState, dt: f32) -> Option<u8> {
    if !state.active {
        return None;
    }

    if state.mobs_spawned >= state.max_mobs {
        // All mobs spawned — enter cooldown.
        state.active = false;
        state.cooldown = spawner_cooldown();
        return None;
    }

    state.cooldown += dt;

    if state.cooldown >= SPAWN_INTERVAL {
        state.cooldown -= SPAWN_INTERVAL;
        state.mobs_spawned += 1;

        // Derive a seed from current state for mob selection.
        let seed = u64::from(state.mobs_spawned)
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(u64::from(state.players_nearby));
        Some(select_mob(state.ominous, seed))
    } else {
        None
    }
}

/// Cooldown duration in seconds after all mobs are spawned (30 minutes).
pub fn spawner_cooldown() -> f32 {
    1800.0
}

/// Select a mob kind based on ominous state and a seed value.
///
/// Normal pool: zombie, skeleton, spider (3 options).
/// Ominous pool: zombie, skeleton, spider, breeze, bogged (5 options).
pub fn select_mob(ominous: bool, seed: u64) -> u8 {
    // Simple xorshift mixer for determinism.
    let mut h = seed;
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 31;

    if ominous {
        let choices = [MOB_ZOMBIE, MOB_SKELETON, MOB_SPIDER, MOB_BREEZE, MOB_BOGGED];
        choices[(h % 5) as usize]
    } else {
        let choices = [MOB_ZOMBIE, MOB_SKELETON, MOB_SPIDER];
        choices[(h % 3) as usize]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_spawner_is_inactive() {
        let state = TrialSpawnerState::new();
        assert!(!state.active);
        assert_eq!(state.mobs_spawned, 0);
        assert_eq!(state.max_mobs, 0);
        assert_eq!(state.cooldown, 0.0);
        assert_eq!(state.players_nearby, 0);
        assert!(!state.ominous);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(TrialSpawnerState::default(), TrialSpawnerState::new());
    }

    #[test]
    fn max_mobs_scales_with_players() {
        assert_eq!(max_mobs_for_players(1, false), 2);
        assert_eq!(max_mobs_for_players(2, false), 4);
        assert_eq!(max_mobs_for_players(3, false), 6);
        assert_eq!(max_mobs_for_players(5, false), 10);
    }

    #[test]
    fn ominous_doubles_max_mobs() {
        assert_eq!(max_mobs_for_players(1, true), 4);
        assert_eq!(max_mobs_for_players(2, true), 8);
        assert_eq!(max_mobs_for_players(3, true), 12);
    }

    #[test]
    fn max_mobs_clamps_to_u8() {
        // 200 players * 2 * 2 = 800, clamped to 255
        assert_eq!(max_mobs_for_players(200, true), 255);
    }

    #[test]
    fn activate_sets_state_correctly() {
        let mut state = TrialSpawnerState::new();
        activate_spawner(&mut state, 3);

        assert!(state.active);
        assert_eq!(state.players_nearby, 3);
        assert_eq!(state.max_mobs, 6);
        assert_eq!(state.mobs_spawned, 0);
        assert_eq!(state.cooldown, 0.0);
    }

    #[test]
    fn activate_ominous_spawner() {
        let mut state = TrialSpawnerState::new();
        state.ominous = true;
        activate_spawner(&mut state, 2);

        assert!(state.active);
        assert_eq!(state.max_mobs, 8);
    }

    #[test]
    fn tick_spawns_mob_after_interval() {
        let mut state = TrialSpawnerState::new();
        activate_spawner(&mut state, 1);

        // Not enough time yet.
        assert!(tick_spawner(&mut state, 1.0).is_none());
        assert_eq!(state.mobs_spawned, 0);

        // Cross the 1.5s threshold.
        let result = tick_spawner(&mut state, 0.5);
        assert!(result.is_some());
        assert_eq!(state.mobs_spawned, 1);
    }

    #[test]
    fn tick_spawns_nothing_when_inactive() {
        let mut state = TrialSpawnerState::new();
        assert!(tick_spawner(&mut state, 2.0).is_none());
    }

    #[test]
    fn tick_deactivates_when_all_mobs_spawned() {
        let mut state = TrialSpawnerState::new();
        activate_spawner(&mut state, 1); // max_mobs = 2

        // Spawn first mob.
        tick_spawner(&mut state, SPAWN_INTERVAL);
        assert!(state.active);
        assert_eq!(state.mobs_spawned, 1);

        // Spawn second mob.
        tick_spawner(&mut state, SPAWN_INTERVAL);
        assert!(state.active);
        assert_eq!(state.mobs_spawned, 2);

        // Next tick should deactivate and enter cooldown.
        let result = tick_spawner(&mut state, SPAWN_INTERVAL);
        assert!(result.is_none());
        assert!(!state.active);
        assert_eq!(state.cooldown, spawner_cooldown());
    }

    #[test]
    fn spawner_cooldown_is_30_minutes() {
        assert_eq!(spawner_cooldown(), 1800.0);
    }

    #[test]
    fn select_mob_normal_returns_valid_kinds() {
        for seed in 0..100 {
            let kind = select_mob(false, seed);
            assert!(
                kind == MOB_ZOMBIE || kind == MOB_SKELETON || kind == MOB_SPIDER,
                "unexpected normal mob kind: {kind}"
            );
        }
    }

    #[test]
    fn select_mob_ominous_includes_breeze_and_bogged() {
        let mut found_breeze = false;
        let mut found_bogged = false;

        for seed in 0..1000 {
            let kind = select_mob(true, seed);
            assert!(
                kind <= MOB_BOGGED,
                "unexpected ominous mob kind: {kind}"
            );
            if kind == MOB_BREEZE {
                found_breeze = true;
            }
            if kind == MOB_BOGGED {
                found_bogged = true;
            }
        }

        assert!(found_breeze, "breeze never selected in ominous mode");
        assert!(found_bogged, "bogged never selected in ominous mode");
    }

    #[test]
    fn select_mob_deterministic() {
        let a = select_mob(false, 42);
        let b = select_mob(false, 42);
        assert_eq!(a, b);

        let c = select_mob(true, 99);
        let d = select_mob(true, 99);
        assert_eq!(c, d);
    }

    #[test]
    fn full_spawn_cycle() {
        let mut state = TrialSpawnerState::new();
        activate_spawner(&mut state, 2); // max_mobs = 4

        let mut spawned = Vec::new();

        // Tick enough times to spawn all 4 mobs.
        for _ in 0..20 {
            if let Some(kind) = tick_spawner(&mut state, 0.5) {
                spawned.push(kind);
            }
            if !state.active {
                break;
            }
        }

        assert_eq!(spawned.len(), 4, "should spawn exactly 4 mobs");
        assert!(!state.active, "spawner should be inactive after cycle");
        assert_eq!(state.cooldown, spawner_cooldown());
    }

    #[test]
    fn activate_resets_previous_state() {
        let mut state = TrialSpawnerState::new();
        activate_spawner(&mut state, 1);

        // Spawn some mobs.
        tick_spawner(&mut state, SPAWN_INTERVAL);

        // Reactivate with more players.
        activate_spawner(&mut state, 3);
        assert_eq!(state.mobs_spawned, 0);
        assert_eq!(state.max_mobs, 6);
        assert_eq!(state.players_nearby, 3);
    }
}
