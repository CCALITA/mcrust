//! Entity component system, mob AI, combat, and survival mechanics.
//!
//! Provides [`EntityManager`] with ECS components, mob [`AiSystem`] and pathfinding,
//! [`DamageEvent`]-based combat, hunger/armor/effects, projectiles, vehicles, and villager trading.

pub mod advancement;
pub mod ai;
pub mod anvil_damage;
pub mod armor;
pub mod behavior;
pub mod boat;
pub mod combat;
pub mod component;
pub mod crossbow;
pub mod decoration;
pub mod difficulty;
pub mod drowning;
pub mod elytra;
pub mod enchant_visual;
pub mod drops;
pub mod effects;
pub mod entity;
pub mod equipment;
pub mod experience;
pub mod fall_damage;
pub mod fishing;
pub mod fishing_rod;
pub mod food;
pub mod golem;
pub mod loot;
pub mod loot_blocks;
pub mod loot_mobs;
pub mod mending;
pub mod painting;
pub mod movement_effects;
pub mod pathfinding;
pub mod projectile;
pub mod raid;
pub mod scaffolding;
pub mod shield;
pub mod spawning;
pub mod special_mobs;
pub mod statistics;
pub mod survival;
pub mod taming;
pub mod totem;
pub mod tool_speed;
pub mod tool_use;
pub mod trident;
pub mod vehicle;
pub mod visibility;
pub mod villager;
pub mod villager_job;
pub mod villager_trades;
pub mod wither;

pub use advancement::{
    ADVANCEMENT_REGISTRY, AdvancementId, AdvancementProperties, AdvancementTracker,
    AdvancementTrigger,
};
pub use ai::{AiComponent, AiGoal, AiSystem};
pub use armor::{
    ArmorMaterial, ArmorPiece, ArmorSet, ArmorSlot, apply_armor_damage, calculate_damage_reduction,
};
pub use behavior::{MobAction, MobBehavior, behavior_tick, behavior_tick_with_state};
pub use combat::{
    DamageEvent, DamageType, apply_damage, attack_cooldown, calculate_fall_damage,
    calculate_knockback, calculate_melee_damage,
};
pub use crossbow::{
    CrossbowState, LoadedProjectile, charge_duration, piercing_remaining_targets,
};
pub use component::{
    Collider, ComponentStore, Gravity, Health, MobComponent, MobKind, Position, Rotation, Velocity,
    World,
};
pub use difficulty::{Difficulty, regional_difficulty};
pub use drops::{DropSystem, ItemDrop, XpOrb, spawn_block_drops, spawn_mob_drops};
pub use effects::{
    ActiveEffect, EffectManager, StatusEffect, apply_jump_modifier, apply_slowness_modifier,
    apply_speed_modifier, apply_strength_modifier,
};
pub use entity::{EntityId, EntityManager};
pub use equipment::{ElytraState, ShieldState, elytra_physics, firework_boost};
pub use experience::{
    ExperienceComponent, add_xp, remove_xp_for_enchanting, total_xp_for_level, xp_for_next_level,
    xp_from_block, xp_from_mob, xp_from_smelting,
};
pub use fishing::{
    FishType, FishingAction, FishingLoot, FishingState, FishingSystem, JunkType, TreasureType,
};
pub use food::{FoodItem, can_eat, eat_duration, food_count, food_data};
pub use loot::{LootCondition, LootContext, LootEntry, LootPool, LootTable};
pub use loot_blocks::block_loot_table;
pub use loot_mobs::mob_loot_table;
pub use pathfinding::{AStarResult, find_path};
pub use projectile::{
    Projectile, ProjectileEvent, ProjectileType, arrow_damage, ender_pearl_teleport_damage,
    snowball_knockback, tick_projectile,
};
pub use raid::{Raid, RaidEvent, RaidWave, default_raid_waves};
pub use spawning::{
    DEFAULT_HOSTILE_CAP, DEFAULT_PASSIVE_CAP, MobSpawnConfig, SpawnSystem, default_spawn_configs,
};
pub use statistics::{StatisticId, StatisticsTracker};
pub use survival::{
    EXHAUSTION_JUMP, EXHAUSTION_SPRINT_PER_METER, EXHAUSTION_WALK_PER_METER, HungerComponent,
    HungerSystem, food_values,
};
pub use taming::{
    BabyMob, BreedingComponent, FeedResult, TameableComponent, feed_animal, try_breed, try_tame,
};
pub use totem::{
    ABSORPTION_EFFECT, FIRE_RESIST_EFFECT, REGEN_EFFECT, TotemEffect, TotemSaveResult,
    check_totem_save, totem_animation_duration, totem_item_id,
};
pub use tool_use::{
    BreakProgress, DurabilityComponent, calculate_break_time, preferred_tool, use_tool,
};
pub use trident::{
    TridentState, channeling_strikes, impaling_bonus, return_velocity, riptide_boost,
    throw_trident, tick_trident, trident_damage,
};
pub use villager::{
    TradeOffer, TradeResult, VillagerData, VillagerProfession, execute_trade, xp_for_level,
};
pub use villager_job::{
    VillagerJobBinding, WorkstationType, bind_villager, find_workstation, try_restock, unbind,
    workstation_profession,
};
pub use villager_trades::default_trades;

pub use special_mobs::{
    EndermanState, PotionType, WitchAction, choose_witch_action, is_player_looking_at,
    slime_damage, slime_health, split_on_death, teleport_away,
};
pub use boat::{
    BoatState, BoatType, boat_collision_box, boat_max_speed, dismount as boat_dismount,
    mount_passenger, tick_boat,
};
pub use vehicle::{Vehicle, VehicleType, apply_input, boat_tick, minecart_tick};

pub use decoration::{Banner, BannerPattern, PaintingVariant, choose_painting};
pub use painting::{painting_name, painting_size, paintings_fitting, total_paintings};
pub use golem::{
    GolemAction, GolemEffect, IronGolem, SnowGolem, check_iron_golem_pattern,
    check_snow_golem_pattern, iron_golem_tick, snow_golem_tick,
};
pub use wither::{
    DamageResult, Wither, WitherPhase, WitherSkull, XP_REWARD, wither_damage, wither_tick,
};
