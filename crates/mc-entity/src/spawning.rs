use glam::Vec3;

use crate::component::MobKind;

/// Configuration for how a particular mob kind spawns.
#[derive(Debug, Clone)]
pub struct MobSpawnConfig {
    pub mob_kind: MobKind,
    pub min_group: u8,
    pub max_group: u8,
    pub spawn_weight: u32,
    pub hostile: bool,
}

/// Returns Minecraft-like default spawn configurations for every mob kind.
pub fn default_spawn_configs() -> Vec<MobSpawnConfig> {
    vec![
        MobSpawnConfig {
            mob_kind: MobKind::Zombie,
            min_group: 1,
            max_group: 4,
            spawn_weight: 100,
            hostile: true,
        },
        MobSpawnConfig {
            mob_kind: MobKind::Skeleton,
            min_group: 1,
            max_group: 4,
            spawn_weight: 80,
            hostile: true,
        },
        MobSpawnConfig {
            mob_kind: MobKind::Creeper,
            min_group: 1,
            max_group: 1,
            spawn_weight: 100,
            hostile: true,
        },
        MobSpawnConfig {
            mob_kind: MobKind::Spider,
            min_group: 1,
            max_group: 3,
            spawn_weight: 100,
            hostile: true,
        },
        MobSpawnConfig {
            mob_kind: MobKind::Pig,
            min_group: 1,
            max_group: 4,
            spawn_weight: 10,
            hostile: false,
        },
        MobSpawnConfig {
            mob_kind: MobKind::Cow,
            min_group: 1,
            max_group: 4,
            spawn_weight: 8,
            hostile: false,
        },
        MobSpawnConfig {
            mob_kind: MobKind::Sheep,
            min_group: 1,
            max_group: 4,
            spawn_weight: 12,
            hostile: false,
        },
        MobSpawnConfig {
            mob_kind: MobKind::Chicken,
            min_group: 1,
            max_group: 4,
            spawn_weight: 10,
            hostile: false,
        },
    ]
}

/// Deterministic hash function for spawn randomness.
/// Uses a simple xorshift-based mixer so that tests are reproducible.
fn deterministic_hash(seed: u64, tick: u64, index: u64) -> u64 {
    let mut h = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(tick)
        .wrapping_mul(1_442_695_040_888_963_407)
        .wrapping_add(index);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 31;
    h
}

/// Default mob caps used by `SpawnSystem`.
pub const DEFAULT_HOSTILE_CAP: u32 = 70;
pub const DEFAULT_PASSIVE_CAP: u32 = 10;

/// Minimum distance from the player at which mobs can spawn (blocks).
const MIN_SPAWN_DISTANCE: f32 = 24.0;
/// Maximum distance from the player at which mobs can spawn (blocks).
const MAX_SPAWN_DISTANCE: f32 = 128.0;

/// Handles mob spawning decisions each tick.
pub struct SpawnSystem {
    configs: Vec<MobSpawnConfig>,
}

impl SpawnSystem {
    pub fn new(configs: Vec<MobSpawnConfig>) -> Self {
        Self { configs }
    }

    /// Attempt to spawn mobs near the player.
    ///
    /// * `player_pos` — current player world position.
    /// * `mob_cap` — maximum number of mobs of the relevant category.
    /// * `current_count` — how many mobs of that category already exist.
    /// * `seed` — world seed for deterministic randomness.
    /// * `tick` — current game tick (contributes to randomness).
    /// * `time_of_day` — 0.0..1.0 where 0.0 = dawn, 0.5 = dusk (hostile mobs
    ///   spawn only during "night": time_of_day >= 0.5 or time_of_day < 0.1).
    ///
    /// Returns a list of `(MobKind, position)` pairs to spawn this tick.
    pub fn try_spawn(
        &self,
        player_pos: Vec3,
        mob_cap: u32,
        current_count: u32,
        seed: u64,
        tick: u64,
        time_of_day: f32,
    ) -> Vec<(MobKind, Vec3)> {
        if current_count >= mob_cap {
            return Vec::new();
        }

        let is_night = time_of_day >= 0.5 || time_of_day < 0.1;
        let remaining = mob_cap.saturating_sub(current_count);

        // Collect eligible configs for the current time of day.
        let eligible: Vec<&MobSpawnConfig> = self
            .configs
            .iter()
            .filter(|c| if c.hostile { is_night } else { true })
            .collect();

        if eligible.is_empty() {
            return Vec::new();
        }

        let total_weight: u64 = eligible.iter().map(|c| u64::from(c.spawn_weight)).sum();
        if total_weight == 0 {
            return Vec::new();
        }

        let mut result: Vec<(MobKind, Vec3)> = Vec::new();
        let mut attempt: u64 = 0;

        // Each tick we try a few spawn attempts (up to 4) to mirror Minecraft's
        // approach of spreading spawns across ticks rather than doing everything
        // at once.
        let max_attempts: u64 = 4;

        while attempt < max_attempts && (result.len() as u32) < remaining {
            let h = deterministic_hash(seed, tick, attempt);

            // Weighted selection of a config.
            let roll = h % total_weight;
            let mut cumulative: u64 = 0;
            let mut chosen: Option<&MobSpawnConfig> = None;
            for cfg in &eligible {
                cumulative += u64::from(cfg.spawn_weight);
                if roll < cumulative {
                    chosen = Some(cfg);
                    break;
                }
            }

            let cfg = match chosen {
                Some(c) => c,
                None => {
                    attempt += 1;
                    continue;
                }
            };

            // Determine group size.
            let range = u64::from(cfg.max_group - cfg.min_group + 1);
            let group_size_hash = deterministic_hash(seed, tick, attempt.wrapping_add(1000));
            let group_size = cfg.min_group + (group_size_hash % range) as u8;

            for g in 0..u64::from(group_size) {
                if (result.len() as u32) >= remaining {
                    break;
                }

                let pos = spawn_position(player_pos, seed, tick, attempt * 100 + g);
                result.push((cfg.mob_kind, pos));
            }

            attempt += 1;
        }

        result
    }
}

impl Default for SpawnSystem {
    fn default() -> Self {
        Self::new(default_spawn_configs())
    }
}

/// Compute a spawn position between `MIN_SPAWN_DISTANCE` and
/// `MAX_SPAWN_DISTANCE` blocks from the player using deterministic hashing.
fn spawn_position(player_pos: Vec3, seed: u64, tick: u64, index: u64) -> Vec3 {
    let angle_hash = deterministic_hash(seed, tick, index.wrapping_add(500));
    let dist_hash = deterministic_hash(seed, tick, index.wrapping_add(700));

    // Angle in radians (0..2pi).
    let angle = (angle_hash % 36_000) as f32 / 36_000.0 * std::f32::consts::TAU;

    // Distance in MIN..MAX range.
    let dist_range = MAX_SPAWN_DISTANCE - MIN_SPAWN_DISTANCE;
    let distance = MIN_SPAWN_DISTANCE + (dist_hash % (dist_range as u64 * 100)) as f32 / 100.0;

    Vec3::new(
        player_pos.x + angle.cos() * distance,
        player_pos.y, // spawn at player Y (ground level approximation)
        player_pos.z + angle.sin() * distance,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_configs_contains_all_mob_kinds() {
        let configs = default_spawn_configs();
        assert_eq!(configs.len(), 8);

        let kinds: Vec<MobKind> = configs.iter().map(|c| c.mob_kind).collect();
        assert!(kinds.contains(&MobKind::Zombie));
        assert!(kinds.contains(&MobKind::Skeleton));
        assert!(kinds.contains(&MobKind::Creeper));
        assert!(kinds.contains(&MobKind::Spider));
        assert!(kinds.contains(&MobKind::Pig));
        assert!(kinds.contains(&MobKind::Cow));
        assert!(kinds.contains(&MobKind::Sheep));
        assert!(kinds.contains(&MobKind::Chicken));
    }

    #[test]
    fn spawn_positions_are_within_range() {
        let system = SpawnSystem::default();
        let player_pos = Vec3::new(100.0, 64.0, 100.0);

        // Use night so hostile mobs can spawn.
        let spawns = system.try_spawn(player_pos, 70, 0, 42, 100, 0.6);

        assert!(!spawns.is_empty(), "should produce at least one spawn");

        for (_kind, pos) in &spawns {
            let dx = pos.x - player_pos.x;
            let dz = pos.z - player_pos.z;
            let horizontal_dist = (dx * dx + dz * dz).sqrt();

            assert!(
                horizontal_dist >= MIN_SPAWN_DISTANCE - 0.1,
                "spawn too close: {horizontal_dist}"
            );
            assert!(
                horizontal_dist <= MAX_SPAWN_DISTANCE + 0.1,
                "spawn too far: {horizontal_dist}"
            );
        }
    }

    #[test]
    fn mob_cap_respected() {
        let system = SpawnSystem::default();
        let player_pos = Vec3::ZERO;

        // Already at cap.
        let spawns = system.try_spawn(player_pos, 70, 70, 1, 1, 0.6);
        assert!(spawns.is_empty(), "should not spawn when at mob cap");

        // Over cap.
        let spawns = system.try_spawn(player_pos, 70, 80, 1, 1, 0.6);
        assert!(spawns.is_empty(), "should not spawn when over mob cap");
    }

    #[test]
    fn mob_cap_limits_spawn_count() {
        let system = SpawnSystem::default();
        let player_pos = Vec3::ZERO;

        // Only 2 slots remaining — should never exceed that.
        let spawns = system.try_spawn(player_pos, 70, 68, 42, 500, 0.7);
        assert!(
            spawns.len() <= 2,
            "spawned {} but only 2 slots available",
            spawns.len()
        );
    }

    #[test]
    fn hostile_mobs_only_spawn_at_night() {
        let system = SpawnSystem::default();
        let player_pos = Vec3::ZERO;

        // Daytime (0.3) — only passive mobs should spawn.
        let spawns = system.try_spawn(player_pos, 70, 0, 42, 100, 0.3);

        let hostile_kinds = [
            MobKind::Zombie,
            MobKind::Skeleton,
            MobKind::Creeper,
            MobKind::Spider,
        ];

        for (kind, _pos) in &spawns {
            assert!(
                !hostile_kinds.contains(kind),
                "hostile mob {:?} spawned during daytime",
                kind
            );
        }
    }

    #[test]
    fn passive_mobs_can_spawn_anytime() {
        // Use a passive-only config to guarantee spawns.
        let configs = vec![MobSpawnConfig {
            mob_kind: MobKind::Pig,
            min_group: 1,
            max_group: 4,
            spawn_weight: 100,
            hostile: false,
        }];
        let system = SpawnSystem::new(configs);
        let player_pos = Vec3::ZERO;

        // Daytime.
        let day_spawns = system.try_spawn(player_pos, 10, 0, 42, 100, 0.3);
        assert!(
            !day_spawns.is_empty(),
            "passive mobs should spawn during day"
        );

        // Nighttime.
        let night_spawns = system.try_spawn(player_pos, 10, 0, 42, 200, 0.7);
        assert!(
            !night_spawns.is_empty(),
            "passive mobs should spawn during night"
        );
    }

    #[test]
    fn deterministic_spawns_are_reproducible() {
        let system = SpawnSystem::default();
        let player_pos = Vec3::new(50.0, 64.0, 50.0);

        let spawns_a = system.try_spawn(player_pos, 70, 0, 123, 456, 0.7);
        let spawns_b = system.try_spawn(player_pos, 70, 0, 123, 456, 0.7);

        assert_eq!(spawns_a.len(), spawns_b.len());
        for (a, b) in spawns_a.iter().zip(spawns_b.iter()) {
            assert_eq!(a.0, b.0);
            assert!((a.1 - b.1).length() < f32::EPSILON);
        }
    }

    #[test]
    fn empty_configs_produce_no_spawns() {
        let system = SpawnSystem::new(Vec::new());
        let spawns = system.try_spawn(Vec3::ZERO, 70, 0, 1, 1, 0.7);
        assert!(spawns.is_empty());
    }
}
