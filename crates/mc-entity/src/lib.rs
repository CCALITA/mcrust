<<<<<<< HEAD
pub mod behavior;
||||||| 5cd2059
=======
pub mod combat;
>>>>>>> origin/feat/combat-survival-mechanics
pub mod component;
pub mod entity;
<<<<<<< HEAD
pub mod spawning;
||||||| 5cd2059
=======
pub mod survival;
>>>>>>> origin/feat/combat-survival-mechanics

<<<<<<< HEAD
pub use behavior::{behavior_tick, behavior_tick_with_state, MobAction, MobBehavior};
||||||| 5cd2059
=======
pub use combat::{
    DamageEvent, DamageType, apply_damage, attack_cooldown, calculate_fall_damage,
    calculate_knockback, calculate_melee_damage,
};
>>>>>>> origin/feat/combat-survival-mechanics
pub use component::{
    Collider, ComponentStore, Gravity, Health, MobComponent, MobKind, Position, Rotation, Velocity,
    World,
};
pub use entity::{EntityId, EntityManager};
<<<<<<< HEAD
pub use spawning::{
    default_spawn_configs, MobSpawnConfig, SpawnSystem, DEFAULT_HOSTILE_CAP, DEFAULT_PASSIVE_CAP,
||||||| 5cd2059
=======
pub use survival::{
    EXHAUSTION_JUMP, EXHAUSTION_SPRINT_PER_METER, EXHAUSTION_WALK_PER_METER, HungerComponent,
    HungerSystem, food_values,
>>>>>>> origin/feat/combat-survival-mechanics
};
