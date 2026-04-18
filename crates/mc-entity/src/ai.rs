use glam::Vec3;
use mc_core::pos::BlockPos;

use crate::component::World;
use crate::entity::EntityId;
use crate::pathfinding;

/// High-level goal driving a mob's behavior.
#[derive(Debug, Clone)]
pub enum AiGoal {
    Idle,
    Wander { timer: f32 },
    FollowTarget { target_pos: Vec3, speed: f32 },
    FleeFrom { pos: Vec3, speed: f32 },
    Attack { target_pos: Vec3 },
}

/// AI state attached to an entity.
#[derive(Debug, Clone)]
pub struct AiComponent {
    pub current_goal: AiGoal,
    pub path: Vec<BlockPos>,
    pub path_index: usize,
    pub retarget_timer: f32,
}

impl AiComponent {
    pub fn new() -> Self {
        Self {
            current_goal: AiGoal::Idle,
            path: Vec::new(),
            path_index: 0,
            retarget_timer: 0.0,
        }
    }
}

impl Default for AiComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// Move an entity position toward the next waypoint in a path.
///
/// Returns `true` when the entity has reached or passed the final waypoint.
pub fn walk_along_path(
    pos: &mut Vec3,
    path: &[BlockPos],
    path_index: &mut usize,
    speed: f32,
    dt: f32,
) -> bool {
    let mut remaining_step = speed * dt;

    loop {
        if *path_index >= path.len() {
            return true;
        }

        let target = path[*path_index].to_vec3() + Vec3::new(0.5, 0.0, 0.5);
        let diff = target - *pos;
        let horizontal_diff = Vec3::new(diff.x, 0.0, diff.z);
        let dist = horizontal_diff.length();

        if dist <= remaining_step {
            // Arrived at this waypoint — snap and consume the distance.
            remaining_step -= dist;
            pos.x = target.x;
            pos.z = target.z;
            pos.y = target.y;
            *path_index += 1;
            // Continue to the next waypoint with leftover step budget.
        } else {
            let direction = horizontal_diff / dist;
            *pos += direction * remaining_step;
            pos.y = target.y;
            return false;
        }
    }
}

/// Simple deterministic hash for generating pseudo-random values from entity
/// state without pulling in a PRNG crate. Not cryptographically secure —
/// only used for varying wander directions and idle timers.
fn simple_hash(a: u64, b: u32) -> u32 {
    let mut x = a.wrapping_mul(6364136223846793005).wrapping_add(b as u64);
    x ^= x >> 16;
    x = x.wrapping_mul(0x45d9f3b);
    x ^= x >> 16;
    x as u32
}

pub struct AiSystem;

impl AiSystem {
    /// Advance AI for every entity that has an `AiComponent`.
    ///
    /// `is_walkable` should return true for positions the mob can stand on
    /// (air block with solid ground below).
    pub fn tick(
        world: &mut World,
        ai_components: &mut crate::component::ComponentStore<AiComponent>,
        dt: f32,
        is_walkable: &dyn Fn(BlockPos) -> bool,
    ) {
        // Collect entity IDs so we can iterate without borrowing issues.
        let ids: Vec<EntityId> = ai_components.iter().map(|(id, _)| id).collect();

        for id in ids {
            let (current_pos, ai) = {
                let Some(pos_comp) = world.positions.get(id) else {
                    continue;
                };
                let Some(ai) = ai_components.get(id) else {
                    continue;
                };
                (pos_comp.0, ai.clone())
            };

            match ai.current_goal {
                AiGoal::Idle => {
                    Self::tick_idle(id, ai_components, dt);
                }
                AiGoal::Wander { .. } => {
                    Self::tick_wander(id, current_pos, world, ai_components, dt, is_walkable);
                }
                AiGoal::FollowTarget { target_pos, speed } => {
                    Self::tick_follow(
                        id,
                        current_pos,
                        target_pos,
                        speed,
                        world,
                        ai_components,
                        dt,
                        is_walkable,
                    );
                }
                AiGoal::FleeFrom { pos: threat, speed } => {
                    Self::tick_flee(
                        id,
                        current_pos,
                        threat,
                        speed,
                        world,
                        ai_components,
                        dt,
                        is_walkable,
                    );
                }
                AiGoal::Attack { target_pos } => {
                    // For now, Attack behaves like FollowTarget at speed 4.0.
                    Self::tick_follow(
                        id,
                        current_pos,
                        target_pos,
                        4.0,
                        world,
                        ai_components,
                        dt,
                        is_walkable,
                    );
                }
            }
        }
    }

    /// Idle: decrement retarget_timer, transition to Wander when it expires.
    fn tick_idle(
        id: EntityId,
        ai_components: &mut crate::component::ComponentStore<AiComponent>,
        dt: f32,
    ) {
        let Some(ai) = ai_components.get_mut(id) else {
            return;
        };

        ai.retarget_timer -= dt;
        if ai.retarget_timer <= 0.0 {
            // Pseudo-random wander duration between 3-6 seconds.
            let hash = simple_hash(id.0, 0);
            let wander_time = 3.0 + (hash % 300) as f32 / 100.0;
            ai.current_goal = AiGoal::Wander { timer: wander_time };
            ai.path.clear();
            ai.path_index = 0;
        }
    }

    /// Wander: pick a random nearby destination, pathfind, walk, then idle.
    fn tick_wander(
        id: EntityId,
        current_pos: Vec3,
        world: &mut World,
        ai_components: &mut crate::component::ComponentStore<AiComponent>,
        dt: f32,
        is_walkable: &dyn Fn(BlockPos) -> bool,
    ) {
        let Some(ai) = ai_components.get_mut(id) else {
            return;
        };

        // Decrement timer.
        let new_timer = match ai.current_goal {
            AiGoal::Wander { timer } => timer - dt,
            _ => return,
        };

        if new_timer <= 0.0 {
            // Wander time expired — go idle for 2-5 seconds.
            let hash = simple_hash(id.0, 1);
            let idle_time = 2.0 + (hash % 300) as f32 / 100.0;
            ai.current_goal = AiGoal::Idle;
            ai.retarget_timer = idle_time;
            ai.path.clear();
            ai.path_index = 0;
            return;
        }

        ai.current_goal = AiGoal::Wander { timer: new_timer };

        // If no path, pick a random destination.
        if ai.path.is_empty() || ai.path_index >= ai.path.len() {
            let hash = simple_hash(id.0, new_timer.to_bits());
            let dx = (hash % 11) as i32 - 5; // -5..5
            let dz = ((hash >> 8) % 11) as i32 - 5;

            let start = BlockPos::new(
                current_pos.x.floor() as i32,
                current_pos.y.floor() as i32,
                current_pos.z.floor() as i32,
            );
            let goal = BlockPos::new(start.x + dx, start.y, start.z + dz);

            let result = pathfinding::find_path(start, goal, 200, is_walkable);
            ai.path = result.path;
            ai.path_index = 0;

            // If pathfinding returned nothing, go idle.
            if ai.path.is_empty() {
                let hash2 = simple_hash(id.0, 2);
                let idle_time = 2.0 + (hash2 % 300) as f32 / 100.0;
                ai.current_goal = AiGoal::Idle;
                ai.retarget_timer = idle_time;
                return;
            }
        }

        // Walk along the path.
        let mut entity_pos = current_pos;
        let path = ai.path.clone();
        let mut idx = ai.path_index;
        let done = walk_along_path(&mut entity_pos, &path, &mut idx, 2.0, dt);
        ai.path_index = idx;

        if let Some(pos_comp) = world.positions.get_mut(id) {
            pos_comp.0 = entity_pos;
        }

        if done {
            let hash3 = simple_hash(id.0, 3);
            let idle_time = 2.0 + (hash3 % 300) as f32 / 100.0;
            if let Some(ai) = ai_components.get_mut(id) {
                ai.current_goal = AiGoal::Idle;
                ai.retarget_timer = idle_time;
                ai.path.clear();
                ai.path_index = 0;
            }
        }
    }

    /// FollowTarget: pathfind toward target, walk along path.
    #[allow(clippy::too_many_arguments)]
    fn tick_follow(
        id: EntityId,
        current_pos: Vec3,
        target_pos: Vec3,
        speed: f32,
        world: &mut World,
        ai_components: &mut crate::component::ComponentStore<AiComponent>,
        dt: f32,
        is_walkable: &dyn Fn(BlockPos) -> bool,
    ) {
        let Some(ai) = ai_components.get_mut(id) else {
            return;
        };

        ai.retarget_timer -= dt;

        // Re-pathfind periodically or if no path.
        if ai.path.is_empty() || ai.path_index >= ai.path.len() || ai.retarget_timer <= 0.0 {
            let start = BlockPos::new(
                current_pos.x.floor() as i32,
                current_pos.y.floor() as i32,
                current_pos.z.floor() as i32,
            );
            let goal = BlockPos::new(
                target_pos.x.floor() as i32,
                target_pos.y.floor() as i32,
                target_pos.z.floor() as i32,
            );

            let result = pathfinding::find_path(start, goal, 200, is_walkable);
            ai.path = result.path;
            ai.path_index = 0;
            ai.retarget_timer = 1.0; // Re-pathfind every second.
        }

        // Walk along path.
        let mut entity_pos = current_pos;
        let path = ai.path.clone();
        let mut idx = ai.path_index;
        walk_along_path(&mut entity_pos, &path, &mut idx, speed, dt);
        ai.path_index = idx;

        if let Some(pos_comp) = world.positions.get_mut(id) {
            pos_comp.0 = entity_pos;
        }
    }

    /// FleeFrom: pathfind away from threat.
    #[allow(clippy::too_many_arguments)]
    fn tick_flee(
        id: EntityId,
        current_pos: Vec3,
        threat_pos: Vec3,
        speed: f32,
        world: &mut World,
        ai_components: &mut crate::component::ComponentStore<AiComponent>,
        dt: f32,
        is_walkable: &dyn Fn(BlockPos) -> bool,
    ) {
        let Some(ai) = ai_components.get_mut(id) else {
            return;
        };

        ai.retarget_timer -= dt;

        if ai.path.is_empty() || ai.path_index >= ai.path.len() || ai.retarget_timer <= 0.0 {
            // Pick a position away from the threat.
            let away = current_pos - threat_pos;
            let away_dir = if away.length() > 0.001 {
                away.normalize()
            } else {
                Vec3::X
            };
            let flee_target = current_pos + away_dir * 10.0;

            let start = BlockPos::new(
                current_pos.x.floor() as i32,
                current_pos.y.floor() as i32,
                current_pos.z.floor() as i32,
            );
            let goal = BlockPos::new(
                flee_target.x.floor() as i32,
                flee_target.y.floor() as i32,
                flee_target.z.floor() as i32,
            );

            let result = pathfinding::find_path(start, goal, 200, is_walkable);
            ai.path = result.path;
            ai.path_index = 0;
            ai.retarget_timer = 1.0;
        }

        let mut entity_pos = current_pos;
        let path = ai.path.clone();
        let mut idx = ai.path_index;
        walk_along_path(&mut entity_pos, &path, &mut idx, speed, dt);
        ai.path_index = idx;

        if let Some(pos_comp) = world.positions.get_mut(id) {
            pos_comp.0 = entity_pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{ComponentStore, Position, World};

    fn flat_walkable(pos: BlockPos) -> bool {
        pos.y == 1
    }

    #[test]
    fn idle_transitions_to_wander() {
        let mut world = World::new();
        let mut ai_store = ComponentStore::<AiComponent>::new();

        let id = world.entities.spawn();
        world
            .positions
            .insert(id, Position(Vec3::new(0.0, 1.0, 0.0)));

        let mut ai = AiComponent::new();
        // Start idle with a tiny timer so it transitions quickly.
        ai.current_goal = AiGoal::Idle;
        ai.retarget_timer = 0.1;
        ai_store.insert(id, ai);

        // Tick with dt large enough to expire the timer.
        AiSystem::tick(&mut world, &mut ai_store, 0.2, &flat_walkable);

        let ai = ai_store.get(id).unwrap();
        assert!(
            matches!(ai.current_goal, AiGoal::Wander { .. }),
            "Expected Wander, got {:?}",
            ai.current_goal
        );
    }

    #[test]
    fn walk_along_path_advances_position() {
        let path = vec![
            BlockPos::new(0, 1, 0),
            BlockPos::new(1, 1, 0),
            BlockPos::new(2, 1, 0),
        ];

        let mut pos = Vec3::new(0.5, 1.0, 0.5);
        let mut index = 0usize;

        // Walk with high speed for one large step — should advance past
        // the first waypoint.
        let done = walk_along_path(&mut pos, &path, &mut index, 100.0, 1.0);

        assert!(
            index > 0,
            "Path index should have advanced, but is {}",
            index
        );
        // With speed=100 and dt=1.0, should reach the end.
        assert!(done, "Should have reached end of path");
    }

    #[test]
    fn walk_along_path_returns_true_when_empty() {
        let path: Vec<BlockPos> = Vec::new();
        let mut pos = Vec3::ZERO;
        let mut index = 0usize;

        let done = walk_along_path(&mut pos, &path, &mut index, 1.0, 0.1);
        assert!(done, "Empty path should immediately return true");
    }

    #[test]
    fn walk_along_path_moves_toward_waypoint() {
        let path = vec![BlockPos::new(10, 1, 0)];
        let mut pos = Vec3::new(0.0, 1.0, 0.0);
        let mut index = 0usize;

        // Small step — should move toward the waypoint but not reach it.
        let done = walk_along_path(&mut pos, &path, &mut index, 1.0, 0.1);

        assert!(!done, "Should not have reached the waypoint yet");
        assert!(pos.x > 0.0, "Should have moved in +x direction");
        assert_eq!(index, 0, "Index should still be 0");
    }

    #[test]
    fn ai_component_default_is_idle() {
        let ai = AiComponent::new();
        assert!(matches!(ai.current_goal, AiGoal::Idle));
        assert!(ai.path.is_empty());
        assert_eq!(ai.path_index, 0);
    }

    #[test]
    fn wander_eventually_returns_to_idle() {
        let mut world = World::new();
        let mut ai_store = ComponentStore::<AiComponent>::new();

        let id = world.entities.spawn();
        world
            .positions
            .insert(id, Position(Vec3::new(5.0, 1.0, 5.0)));

        let mut ai = AiComponent::new();
        ai.current_goal = AiGoal::Wander { timer: 0.05 };
        ai_store.insert(id, ai);

        // Tick with dt large enough to expire the wander timer.
        AiSystem::tick(&mut world, &mut ai_store, 0.1, &flat_walkable);

        let ai = ai_store.get(id).unwrap();
        assert!(
            matches!(ai.current_goal, AiGoal::Idle),
            "Expected Idle after wander timer expired, got {:?}",
            ai.current_goal
        );
    }

    #[test]
    fn follow_target_moves_entity() {
        let mut world = World::new();
        let mut ai_store = ComponentStore::<AiComponent>::new();

        let id = world.entities.spawn();
        let start_pos = Vec3::new(0.5, 1.0, 0.5);
        world.positions.insert(id, Position(start_pos));

        let mut ai = AiComponent::new();
        ai.current_goal = AiGoal::FollowTarget {
            target_pos: Vec3::new(5.5, 1.0, 0.5),
            speed: 4.0,
        };
        ai_store.insert(id, ai);

        // Tick several times.
        for _ in 0..10 {
            AiSystem::tick(&mut world, &mut ai_store, 0.1, &flat_walkable);
        }

        let final_pos = world.positions.get(id).unwrap().0;
        // Entity should have moved toward the target (x > start).
        assert!(
            final_pos.x > start_pos.x,
            "Entity should have moved toward target: start={}, end={}",
            start_pos.x,
            final_pos.x
        );
    }
}
