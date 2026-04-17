pub mod combat;
pub mod component;
pub mod entity;
pub mod survival;

pub use combat::{
    DamageEvent, DamageType, apply_damage, attack_cooldown, calculate_fall_damage,
    calculate_knockback, calculate_melee_damage,
};
pub use component::{
    Collider, ComponentStore, Gravity, Health, MobComponent, MobKind, Position, Rotation, Velocity,
    World,
};
pub use entity::{EntityId, EntityManager};
pub use survival::{
    EXHAUSTION_JUMP, EXHAUSTION_SPRINT_PER_METER, EXHAUSTION_WALK_PER_METER, HungerComponent,
    HungerSystem, food_values,
};
