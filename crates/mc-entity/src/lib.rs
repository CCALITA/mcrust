pub mod ai;
pub mod behavior;
pub mod boss;
pub mod combat;
pub mod component;
pub mod entity;
pub mod experience;
pub mod pathfinding;
pub mod spawning;
pub mod survival;

pub use ai::{AiComponent, AiGoal, AiSystem};
pub use behavior::{MobAction, MobBehavior, behavior_tick, behavior_tick_with_state};
pub use boss::{
    DamageResult, DragonPhase, EnderCrystal, EnderDragon, XP_REWARD, crystal_heal_tick,
    dragon_damage, dragon_tick,
};
pub use combat::{
    DamageEvent, DamageType, apply_damage, attack_cooldown, calculate_fall_damage,
    calculate_knockback, calculate_melee_damage,
};
pub use component::{
    Collider, ComponentStore, Gravity, Health, MobComponent, MobKind, Position, Rotation, Velocity,
    World,
};
pub use entity::{EntityId, EntityManager};
pub use experience::{
    ExperienceComponent, add_xp, remove_xp_for_enchanting, total_xp_for_level, xp_for_next_level,
    xp_from_block, xp_from_mob, xp_from_smelting,
};
pub use pathfinding::{AStarResult, find_path};
pub use spawning::{
    DEFAULT_HOSTILE_CAP, DEFAULT_PASSIVE_CAP, MobSpawnConfig, SpawnSystem, default_spawn_configs,
};
pub use survival::{
    EXHAUSTION_JUMP, EXHAUSTION_SPRINT_PER_METER, EXHAUSTION_WALK_PER_METER, HungerComponent,
    HungerSystem, food_values,
};
