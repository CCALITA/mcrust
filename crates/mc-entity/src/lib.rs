pub mod behavior;
pub mod component;
pub mod entity;
pub mod spawning;

pub use behavior::{behavior_tick, behavior_tick_with_state, MobAction, MobBehavior};
pub use component::{
    Collider, ComponentStore, Gravity, Health, MobComponent, MobKind, Position, Rotation, Velocity,
    World,
};
pub use entity::{EntityId, EntityManager};
pub use spawning::{
    default_spawn_configs, MobSpawnConfig, SpawnSystem, DEFAULT_HOSTILE_CAP, DEFAULT_PASSIVE_CAP,
};
