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
    /// 3. Apply direction-based AI movement.
    /// 4. Tick the entity world (applies velocity and gravity).
    /// 5. Despawn entities more than 128 blocks from the player.
    /// 6. Increment the tick counter.
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

        // 3 — direction-based AI movement
        self.apply_mob_ai(player_pos, dt);

        // 4 — physics tick
        self.entity_world.tick(dt);

        // 5 — despawn far-away entities
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

        // 6 — advance tick counter
        self.tick_counter += 1;
    }

    /// Apply simple direction-based AI to each mob.
    ///
    /// - Hostile mobs (kind discriminant < 4) move toward the player.
    /// - Passive mobs (kind discriminant >= 4) wander with a small random offset.
    /// - Mobs beyond the AI range (64 blocks) do not move.
    pub fn apply_mob_ai(&mut self, player_pos: Vec3, dt: f32) {
        const HOSTILE_SPEED: f32 = 0.05;
        const PASSIVE_SPEED: f32 = 0.02;
        const AI_RANGE: f32 = 64.0;

        // Collect mob IDs and their kind discriminants to avoid borrow conflicts.
        let mob_data: Vec<(EntityId, u8)> = self
            .entity_world
            .mobs
            .iter()
            .map(|(id, mob)| (id, mob.kind as u8))
            .collect();

        for (id, kind_disc) in mob_data {
            let Some(pos_component) = self.entity_world.positions.get_mut(id) else {
                continue;
            };

            let mob_pos = pos_component.0;
            let dist_to_player = mob_pos.distance(player_pos);

            // Skip mobs beyond AI range.
            if dist_to_player > AI_RANGE {
                continue;
            }

            if kind_disc < 4 {
                // Hostile: move toward player.
                let diff = player_pos - mob_pos;
                let direction = if diff.length() > 0.001 {
                    diff.normalize()
                } else {
                    Vec3::ZERO
                };
                pos_component.0 += direction * HOSTILE_SPEED * dt;
            } else {
                // Passive: wander with small deterministic pseudo-random offset.
                let wander = pseudo_random_wander(id, self.tick_counter);
                pos_component.0 += wander * PASSIVE_SPEED * dt;
            }
        }
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

/// Produce a deterministic pseudo-random unit-length wander direction on the XZ
/// plane, seeded by the entity ID and the current tick.
fn pseudo_random_wander(id: EntityId, tick: u64) -> Vec3 {
    // Simple hash combining entity id and tick for reproducible randomness.
    let seed = id.0.wrapping_mul(6364136223846793005).wrapping_add(tick);
    let angle = (seed as f32) * 0.0001;
    Vec3::new(angle.cos(), 0.0, angle.sin())
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

    #[test]
    fn hostile_mob_moves_toward_player() {
        let mut world = MobWorld::new();
        let player_pos = Vec3::new(10.0, 64.0, 10.0);
        let zombie_start = Vec3::new(0.0, 64.0, 0.0);

        world
            .entity_world
            .spawn_mob(MobKind::Zombie, zombie_start);

        let dt = 0.05;
        // Run several ticks to accumulate movement.
        for _ in 0..20 {
            world.tick(player_pos, 0.3, dt);
        }

        let positions = world.mob_positions();
        let zombie_pos = positions
            .iter()
            .find(|(_, kind)| *kind == MobKind::Zombie as u8)
            .map(|(pos, _)| *pos);

        if let Some(pos) = zombie_pos {
            // Compare XZ distance only (gravity affects Y independently).
            let xz_player = Vec3::new(player_pos.x, 0.0, player_pos.z);
            let xz_start = Vec3::new(zombie_start.x, 0.0, zombie_start.z);
            let xz_pos = Vec3::new(pos.x, 0.0, pos.z);

            let new_xz_dist = xz_pos.distance(xz_player);
            let original_xz_dist = xz_start.distance(xz_player);
            assert!(
                new_xz_dist < original_xz_dist,
                "hostile mob should move toward player on XZ: original={original_xz_dist}, new={new_xz_dist}"
            );
        }
    }

    #[test]
    fn passive_mob_wanders_randomly() {
        let mut world = MobWorld::new();
        let player_pos = Vec3::new(10.0, 64.0, 10.0);
        let pig_start = Vec3::new(5.0, 64.0, 5.0);

        world.entity_world.spawn_mob(MobKind::Pig, pig_start);

        let dt = 0.05;
        for _ in 0..20 {
            world.tick(player_pos, 0.3, dt);
        }

        let positions = world.mob_positions();
        let pig_pos = positions
            .iter()
            .find(|(_, kind)| *kind == MobKind::Pig as u8)
            .map(|(pos, _)| *pos);

        if let Some(pos) = pig_pos {
            // Passive mob should have moved (wander), but not necessarily toward player.
            let moved = (pos - pig_start).length();
            assert!(
                moved > 0.0001,
                "passive mob should wander; moved only {moved}"
            );
        }
    }

    #[test]
    fn mob_beyond_ai_range_does_not_move_via_ai() {
        let mut world = MobWorld::new();
        let player_pos = Vec3::new(0.0, 64.0, 0.0);
        // Place zombie at exactly 65 blocks away (beyond AI_RANGE of 64).
        let zombie_start = Vec3::new(65.0, 64.0, 0.0);

        world
            .entity_world
            .spawn_mob(MobKind::Zombie, zombie_start);

        // Apply AI only (not the full tick which includes physics gravity).
        world.apply_mob_ai(player_pos, 0.05);

        let positions = world.mob_positions();
        let zombie_pos = positions
            .iter()
            .find(|(_, kind)| *kind == MobKind::Zombie as u8)
            .map(|(pos, _)| *pos);

        if let Some(pos) = zombie_pos {
            // Position should remain unchanged since mob is out of AI range.
            let moved = (pos - zombie_start).length();
            assert!(
                moved < 0.0001,
                "mob beyond AI range should not move via AI; moved {moved}"
            );
        }
    }

    #[test]
    fn pseudo_random_wander_returns_unit_length() {
        let id = EntityId(42);
        let wander = pseudo_random_wander(id, 100);
        let len = wander.length();
        assert!(
            (len - 1.0).abs() < 0.001,
            "wander vector should be unit length, got {len}"
        );
    }
}
