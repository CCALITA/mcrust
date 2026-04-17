use std::collections::HashMap;

use glam::Vec3;

use crate::entity::EntityId;

/// Generic component storage mapping entity IDs to component data.
pub struct ComponentStore<T> {
    data: HashMap<EntityId, T>,
}

impl<T> ComponentStore<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: EntityId, component: T) {
        self.data.insert(id, component);
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        self.data.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        self.data.get_mut(&id)
    }

    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        self.data.remove(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.data.iter().map(|(&id, val)| (id, val))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.data.iter_mut().map(|(&id, val)| (id, val))
    }
}

impl<T> Default for ComponentStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Core components
// ---------------------------------------------------------------------------

/// World position.
#[derive(Debug, Clone, Copy)]
pub struct Position(pub Vec3);

/// Per-tick velocity.
#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vec3);

/// Facing direction (yaw around Y-axis, pitch up/down).
#[derive(Debug, Clone, Copy)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

/// Hit-points with helpers.
#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
}

/// Mob species.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MobKind {
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Pig,
    Cow,
    Sheep,
    Chicken,
}

/// Identifies the entity as a particular mob type.
#[derive(Debug, Clone, Copy)]
pub struct MobComponent {
    pub kind: MobKind,
}

/// Gravity acceleration (default -32.0 blocks/s^2).
#[derive(Debug, Clone, Copy)]
pub struct Gravity(pub f32);

/// Axis-aligned hitbox dimensions.
#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
}

// ---------------------------------------------------------------------------
// ECS World
// ---------------------------------------------------------------------------

use crate::entity::EntityManager;

/// Lightweight ECS world that owns all entity and component data.
pub struct World {
    pub entities: EntityManager,
    pub positions: ComponentStore<Position>,
    pub velocities: ComponentStore<Velocity>,
    pub rotations: ComponentStore<Rotation>,
    pub healths: ComponentStore<Health>,
    pub mobs: ComponentStore<MobComponent>,
    pub gravities: ComponentStore<Gravity>,
    pub colliders: ComponentStore<Collider>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            positions: ComponentStore::new(),
            velocities: ComponentStore::new(),
            rotations: ComponentStore::new(),
            healths: ComponentStore::new(),
            mobs: ComponentStore::new(),
            gravities: ComponentStore::new(),
            colliders: ComponentStore::new(),
        }
    }

    /// Spawn a mob with default components for the given kind.
    pub fn spawn_mob(&mut self, kind: MobKind, pos: Vec3) -> EntityId {
        let id = self.entities.spawn();

        self.positions.insert(id, Position(pos));
        self.velocities.insert(id, Velocity(Vec3::ZERO));
        self.healths.insert(
            id,
            Health {
                current: 20.0,
                max: 20.0,
            },
        );
        self.mobs.insert(id, MobComponent { kind });
        self.gravities.insert(id, Gravity(-32.0));
        self.colliders.insert(
            id,
            Collider {
                width: 0.6,
                height: 1.8,
            },
        );

        id
    }

    /// Remove an entity and all its component data.
    pub fn despawn(&mut self, id: EntityId) {
        self.entities.despawn(id);
        self.positions.remove(id);
        self.velocities.remove(id);
        self.rotations.remove(id);
        self.healths.remove(id);
        self.mobs.remove(id);
        self.gravities.remove(id);
        self.colliders.remove(id);
    }

    /// Advance one simulation step:
    /// 1. Apply gravity to velocity for entities with both components.
    /// 2. Apply velocity to position for entities with both components.
    pub fn tick(&mut self, dt: f32) {
        // Gravity -> velocity
        let gravity_updates: Vec<(EntityId, f32)> =
            self.gravities.iter().map(|(id, g)| (id, g.0)).collect();

        for (id, g) in gravity_updates {
            if let Some(vel) = self.velocities.get_mut(id) {
                vel.0.y += g * dt;
            }
        }

        // Velocity -> position
        let velocity_updates: Vec<(EntityId, Vec3)> =
            self.velocities.iter().map(|(id, v)| (id, v.0)).collect();

        for (id, v) in velocity_updates {
            if let Some(pos) = self.positions.get_mut(id) {
                pos.0 += v * dt;
            }
        }
    }
}

impl Default for World {
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

    // -- ComponentStore tests -----------------------------------------------

    #[test]
    fn component_store_insert_get_remove() {
        let mut store = ComponentStore::<i32>::new();
        let id = EntityId(1);

        store.insert(id, 42);
        assert_eq!(store.get(id), Some(&42));

        let removed = store.remove(id);
        assert_eq!(removed, Some(42));
        assert!(store.get(id).is_none());
    }

    #[test]
    fn component_store_get_mut() {
        let mut store = ComponentStore::<i32>::new();
        let id = EntityId(0);
        store.insert(id, 10);

        if let Some(val) = store.get_mut(id) {
            *val = 20;
        }
        assert_eq!(store.get(id), Some(&20));
    }

    #[test]
    fn component_store_iter() {
        let mut store = ComponentStore::<&str>::new();
        store.insert(EntityId(0), "a");
        store.insert(EntityId(1), "b");

        let items: Vec<_> = store.iter().collect();
        assert_eq!(items.len(), 2);
    }

    // -- Health tests -------------------------------------------------------

    #[test]
    fn health_damage_reduces_current() {
        let mut h = Health {
            current: 20.0,
            max: 20.0,
        };
        h.damage(5.0);
        assert!((h.current - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn health_damage_clamps_at_zero() {
        let mut h = Health {
            current: 3.0,
            max: 20.0,
        };
        h.damage(10.0);
        assert!((h.current).abs() < f32::EPSILON);
        assert!(h.is_dead());
    }

    #[test]
    fn health_heal_restores_up_to_max() {
        let mut h = Health {
            current: 10.0,
            max: 20.0,
        };
        h.heal(100.0);
        assert!((h.current - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn health_is_dead_at_zero() {
        let h = Health {
            current: 0.0,
            max: 20.0,
        };
        assert!(h.is_dead());
    }

    // -- World tests --------------------------------------------------------

    #[test]
    fn world_spawn_mob_creates_all_components() {
        let mut world = World::new();
        let id = world.spawn_mob(MobKind::Zombie, Vec3::new(1.0, 2.0, 3.0));

        assert!(world.entities.is_alive(id));
        assert!(world.positions.get(id).is_some());
        assert!(world.velocities.get(id).is_some());
        assert!(world.healths.get(id).is_some());
        assert!(world.mobs.get(id).is_some());
        assert!(world.gravities.get(id).is_some());
        assert!(world.colliders.get(id).is_some());

        let pos = world.positions.get(id).unwrap();
        assert!((pos.0.x - 1.0).abs() < f32::EPSILON);
        assert!((pos.0.y - 2.0).abs() < f32::EPSILON);
        assert!((pos.0.z - 3.0).abs() < f32::EPSILON);

        let mob = world.mobs.get(id).unwrap();
        assert_eq!(mob.kind, MobKind::Zombie);
    }

    #[test]
    fn world_despawn_removes_all_components() {
        let mut world = World::new();
        let id = world.spawn_mob(MobKind::Pig, Vec3::ZERO);
        world.despawn(id);

        assert!(!world.entities.is_alive(id));
        assert!(world.positions.get(id).is_none());
        assert!(world.velocities.get(id).is_none());
        assert!(world.healths.get(id).is_none());
        assert!(world.mobs.get(id).is_none());
        assert!(world.gravities.get(id).is_none());
        assert!(world.colliders.get(id).is_none());
    }

    #[test]
    fn world_tick_applies_velocity_to_position() {
        let mut world = World::new();
        let id = world.entities.spawn();
        world.positions.insert(id, Position(Vec3::ZERO));
        world
            .velocities
            .insert(id, Velocity(Vec3::new(1.0, 0.0, 0.0)));

        // No gravity for this entity
        world.tick(1.0);

        let pos = world.positions.get(id).unwrap();
        assert!((pos.0.x - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn world_tick_applies_gravity_to_velocity() {
        let mut world = World::new();
        let id = world.entities.spawn();
        world.positions.insert(id, Position(Vec3::ZERO));
        world.velocities.insert(id, Velocity(Vec3::ZERO));
        world.gravities.insert(id, Gravity(-32.0));

        world.tick(1.0);

        let vel = world.velocities.get(id).unwrap();
        assert!((vel.0.y - (-32.0)).abs() < f32::EPSILON);

        // Position should also have changed (gravity applied first, then velocity)
        let pos = world.positions.get(id).unwrap();
        assert!((pos.0.y - (-32.0)).abs() < f32::EPSILON);
    }
}
