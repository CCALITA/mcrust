//! Chunk mesh cache for tracking which chunks need mesh rebuilds.

use std::collections::HashMap;

/// A cached mesh entry for a single chunk.
#[derive(Debug, Clone)]
pub struct MeshCacheEntry {
    pub chunk_pos: (i32, i32),
    pub generation: u64,
    pub vertex_count: u32,
    pub dirty: bool,
}

/// Cache that tracks chunk mesh state to avoid unnecessary rebuilds.
pub struct MeshCache {
    entries: HashMap<(i32, i32), MeshCacheEntry>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

impl MeshCache {
    /// Create a new mesh cache with the given maximum entry count.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    /// Returns true if the chunk at `pos` needs a mesh rebuild.
    pub fn should_rebuild(&mut self, pos: (i32, i32), current_gen: u64) -> bool {
        match self.entries.get(&pos) {
            Some(entry) if !entry.dirty && entry.generation == current_gen => {
                self.hits += 1;
                false
            }
            _ => {
                self.misses += 1;
                true
            }
        }
    }

    /// Mark a chunk's mesh as clean (up-to-date) with the given generation and vertex count.
    pub fn mark_clean(&mut self, pos: (i32, i32), generation: u64, vertex_count: u32) {
        self.entries.insert(pos, MeshCacheEntry {
            chunk_pos: pos,
            generation,
            vertex_count,
            dirty: false,
        });
    }

    /// Mark a chunk's mesh as dirty (needs rebuild).
    pub fn mark_dirty(&mut self, pos: (i32, i32)) {
        if let Some(entry) = self.entries.get_mut(&pos) {
            entry.dirty = true;
        }
    }

    /// Evict the cache entry farthest from the player position.
    pub fn evict_farthest(&mut self, player_pos: (i32, i32)) {
        if self.entries.len() <= self.max_entries {
            return;
        }

        let farthest = self.entries.keys().max_by_key(|pos| {
            let dx = (pos.0 - player_pos.0) as i64;
            let dz = (pos.1 - player_pos.1) as i64;
            dx * dx + dz * dz
        }).copied();

        if let Some(pos) = farthest {
            self.entries.remove(&pos);
        }
    }

    /// Returns the cache hit rate as a value between 0.0 and 1.0.
    pub fn cache_hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f32 / total as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache_is_empty() {
        let cache = MeshCache::new(16);
        assert_eq!(cache.cache_hit_rate(), 0.0);
    }

    #[test]
    fn should_rebuild_unknown_chunk() {
        let mut cache = MeshCache::new(16);
        assert!(cache.should_rebuild((0, 0), 1));
    }

    #[test]
    fn should_not_rebuild_clean_chunk() {
        let mut cache = MeshCache::new(16);
        cache.mark_clean((0, 0), 1, 100);
        assert!(!cache.should_rebuild((0, 0), 1));
    }

    #[test]
    fn should_rebuild_dirty_chunk() {
        let mut cache = MeshCache::new(16);
        cache.mark_clean((0, 0), 1, 100);
        cache.mark_dirty((0, 0));
        assert!(cache.should_rebuild((0, 0), 1));
    }

    #[test]
    fn should_rebuild_stale_generation() {
        let mut cache = MeshCache::new(16);
        cache.mark_clean((0, 0), 1, 100);
        assert!(cache.should_rebuild((0, 0), 2));
    }

    #[test]
    fn mark_dirty_nonexistent_is_noop() {
        let mut cache = MeshCache::new(16);
        cache.mark_dirty((5, 5)); // should not panic
    }

    #[test]
    fn evict_farthest_removes_distant_chunk() {
        let mut cache = MeshCache::new(2);
        cache.mark_clean((0, 0), 1, 10);
        cache.mark_clean((10, 10), 1, 10);
        cache.mark_clean((1, 1), 1, 10);
        // 3 entries, max is 2, evict farthest from (0,0)
        cache.evict_farthest((0, 0));
        assert_eq!(cache.entries.len(), 2);
        assert!(!cache.entries.contains_key(&(10, 10)));
    }

    #[test]
    fn evict_farthest_noop_when_under_limit() {
        let mut cache = MeshCache::new(10);
        cache.mark_clean((0, 0), 1, 10);
        cache.evict_farthest((0, 0));
        assert_eq!(cache.entries.len(), 1);
    }

    #[test]
    fn cache_hit_rate_tracks_correctly() {
        let mut cache = MeshCache::new(16);
        cache.mark_clean((0, 0), 1, 100);
        cache.should_rebuild((0, 0), 1); // hit
        cache.should_rebuild((0, 0), 1); // hit
        cache.should_rebuild((1, 1), 1); // miss
        let rate = cache.cache_hit_rate();
        assert!((rate - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn mark_clean_overwrites_existing() {
        let mut cache = MeshCache::new(16);
        cache.mark_clean((0, 0), 1, 100);
        cache.mark_clean((0, 0), 2, 200);
        assert!(!cache.should_rebuild((0, 0), 2));
        assert_eq!(cache.entries[&(0, 0)].vertex_count, 200);
    }
}
