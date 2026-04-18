use glam::Vec3;
use mc_entity::component::World as EntityWorld;
use mc_entity::entity::EntityId;
use mc_entity::spawning::{DEFAULT_HOSTILE_CAP, SpawnSystem};

/// Despawn distance — entities further than this from the player are removed.
const DESPAWN_DISTANCE: f32 = 128.0;

/// Fixed world seed used for deterministic spawn randomness.
const WORLD_SEED: u64 = 42;

/// Bridge between the entity/spawning subsystem and the client game loop.
///
/// Wraps an `EntityWorld`, a `SpawnSystem`, a tick counter, and a mob cap.
/// Each call to [`MobWorld::tick`] tries to spawn new mobs, advances entity
/// physics, and despawns entities that are too far from the player.
pub struct MobWorld {
    entity_world: EntityWorld,
    spawn_system: SpawnSystem,
    tick_counter: u64,
    mob_cap: u32,
}

impl MobWorld {
    pub fn new() -> Self {
        Self {
            entity_world: EntityWorld::new(),
            spawn_system: SpawnSystem::default(),
            tick_counter: 0,
            mob_cap: DEFAULT_HOSTILE_CAP,
        }
    }

    /// Advance one mob-system tick.
    ///
    /// 1. Try to spawn mobs around the player (uses `SpawnSystem::try_spawn`).
    /// 2. For each spawned mob, add it to the `EntityWorld` via `spawn_mob`.
    /// 3. Tick the entity world (applies velocity and gravity).
    /// 4. Despawn entities more than 128 blocks from the player.
    /// 5. Increment the tick counter.
    pub fn tick(&mut self, player_pos: Vec3, time_of_day: f32, dt: f32) {
        // 1 & 2 — spawn new mobs
        let current_count = self.entity_world.entities.count() as u32;
        let spawns = self.spawn_system.try_spawn(
            player_pos,
            self.mob_cap,
            current_count,
            WORLD_SEED,
            self.tick_counter,
            time_of_day,
        );

        for (kind, pos) in spawns {
            self.entity_world.spawn_mob(kind, pos);
        }

        // 3 — physics tick
        self.entity_world.tick(dt);

        // 4 — despawn far-away entities
        let to_despawn: Vec<EntityId> = self
            .entity_world
            .positions
            .iter()
            .filter_map(|(id, pos)| {
                let dist = (pos.0 - player_pos).length();
                if dist > DESPAWN_DISTANCE {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        for id in to_despawn {
            self.entity_world.despawn(id);
        }

        // 5 — advance tick counter
        self.tick_counter += 1;
    }

    /// Number of alive entities in the world.
    pub fn mob_count(&self) -> usize {
        self.entity_world.entities.count()
    }

    /// Positions and mob-kind discriminants for all living mobs.
    ///
    /// The `u8` is the discriminant of [`MobKind`] for use in rendering
    /// (e.g. choosing the right texture/model).
    pub fn mob_positions(&self) -> Vec<(Vec3, u8)> {
        self.entity_world
            .mobs
            .iter()
            .filter_map(|(id, mob)| {
                self.entity_world
                    .positions
                    .get(id)
                    .map(|pos| (pos.0, mob.kind as u8))
            })
            .collect()
    }
}

impl Default for MobWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mc_entity::component::MobKind;

    #[test]
    fn new_mob_world_starts_empty() {
        let world = MobWorld::new();
        assert_eq!(world.mob_count(), 0);
        assert!(world.mob_positions().is_empty());
    }

    #[test]
    fn tick_spawns_mobs_at_night() {
        let mut world = MobWorld::new();
        let player_pos = Vec3::new(0.0, 64.0, 0.0);

        // Night time — hostile mobs should spawn.
        world.tick(player_pos, 0.7, 0.05);

        assert!(
            world.mob_count() > 0,
            "expected mobs to spawn at night, got 0"
        );
    }

    #[test]
    fn mob_positions_returns_correct_count() {
        let mut world = MobWorld::new();
        let player_pos = Vec3::new(0.0, 64.0, 0.0);

        world.tick(player_pos, 0.7, 0.05);

        let positions = world.mob_positions();
        assert_eq!(positions.len(), world.mob_count());
    }

    #[test]
    fn despawns_entities_beyond_128_blocks() {
        let mut world = MobWorld::new();

        // Manually spawn a mob far from the player.
        world
            .entity_world
            .spawn_mob(MobKind::Zombie, Vec3::new(200.0, 64.0, 200.0));
        assert_eq!(world.mob_count(), 1);

        // Tick with player at origin — the zombie is ~283 blocks away.
        world.tick(Vec3::ZERO, 0.3, 0.05);

        // The manually spawned zombie should be despawned (>128 blocks).
        // New passive spawns may exist, but the original far-away zombie is gone.
        let positions = world.mob_positions();
        for (pos, _kind) in &positions {
            let dist = (*pos - Vec3::ZERO).length();
            assert!(
                dist <= DESPAWN_DISTANCE + 10.0, // small tolerance for physics movement
                "mob at distance {dist} should have been despawned"
            );
        }
    }

    #[test]
    fn mob_cap_limits_total_entities() {
        let mut world = MobWorld::new();
        let player_pos = Vec3::new(0.0, 64.0, 0.0);

        // Run many ticks to accumulate mobs.
        for _ in 0..500 {
            world.tick(player_pos, 0.7, 0.05);
        }

        assert!(
            world.mob_count() as u32 <= world.mob_cap,
            "mob count {} exceeds cap {}",
            world.mob_count(),
            world.mob_cap,
        );
    }
}
