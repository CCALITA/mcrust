use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

pub struct EntityManager {
    next_id: u64,
    alive: HashSet<EntityId>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            alive: HashSet::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.alive.insert(id);
        id
    }

    pub fn despawn(&mut self, id: EntityId) {
        self.alive.remove(&id);
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.alive.contains(&id)
    }

    pub fn count(&self) -> usize {
        self.alive.len()
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_entity_is_alive() {
        let mut mgr = EntityManager::new();
        let id = mgr.spawn();
        assert!(mgr.is_alive(id));
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn despawn_entity_is_dead() {
        let mut mgr = EntityManager::new();
        let id = mgr.spawn();
        mgr.despawn(id);
        assert!(!mgr.is_alive(id));
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn spawn_increments_ids() {
        let mut mgr = EntityManager::new();
        let a = mgr.spawn();
        let b = mgr.spawn();
        assert_ne!(a, b);
        assert_eq!(a.0 + 1, b.0);
    }

    #[test]
    fn despawn_nonexistent_is_noop() {
        let mut mgr = EntityManager::new();
        mgr.despawn(EntityId(999));
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn count_tracks_alive_entities() {
        let mut mgr = EntityManager::new();
        let a = mgr.spawn();
        let _b = mgr.spawn();
        let _c = mgr.spawn();
        assert_eq!(mgr.count(), 3);
        mgr.despawn(a);
        assert_eq!(mgr.count(), 2);
    }
}
