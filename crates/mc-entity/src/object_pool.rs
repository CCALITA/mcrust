//! Reusable object pool for reducing allocation overhead on frequently
//! created and destroyed entities (projectiles, particles, XP orbs, etc.).

/// A generic object pool that recycles instances via an internal free list.
pub struct ObjectPool<T> {
    free_list: Vec<T>,
    active_count: usize,
}

impl<T> ObjectPool<T> {
    /// Creates an empty pool.
    pub fn new() -> Self {
        Self {
            free_list: Vec::new(),
            active_count: 0,
        }
    }

    /// Takes an item from the pool, returning `None` if the free list is empty.
    pub fn acquire(&mut self) -> Option<T> {
        let item = self.free_list.pop()?;
        self.active_count += 1;
        Some(item)
    }

    /// Returns an item to the pool for future reuse.
    pub fn release(&mut self, item: T) {
        self.active_count = self.active_count.saturating_sub(1);
        self.free_list.push(item);
    }

    /// Pre-populates the pool with `count` items produced by `factory`.
    pub fn pre_allocate(&mut self, count: usize, factory: impl Fn() -> T) {
        self.free_list.reserve(count);
        for _ in 0..count {
            self.free_list.push(factory());
        }
    }

    /// Number of items currently sitting in the free list.
    pub fn pool_size(&self) -> usize {
        self.free_list.len()
    }

    /// Number of items currently acquired (outstanding).
    pub fn active(&self) -> usize {
        self.active_count
    }

    /// Ratio of active items to total items (active + pooled).
    /// Returns `0.0` when the pool is completely empty.
    pub fn utilization(&self) -> f32 {
        let total = self.active_count + self.free_list.len();
        if total == 0 {
            return 0.0;
        }
        self.active_count as f32 / total as f32
    }
}

impl<T> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_is_empty() {
        let pool: ObjectPool<u32> = ObjectPool::new();
        assert_eq!(pool.pool_size(), 0);
        assert_eq!(pool.active(), 0);
        assert_eq!(pool.utilization(), 0.0);
    }

    #[test]
    fn acquire_returns_none_when_empty() {
        let mut pool: ObjectPool<u32> = ObjectPool::new();
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn pre_allocate_fills_pool() {
        let mut pool: ObjectPool<u32> = ObjectPool::new();
        pool.pre_allocate(5, || 42);
        assert_eq!(pool.pool_size(), 5);
        assert_eq!(pool.active(), 0);
    }

    #[test]
    fn acquire_after_pre_allocate() {
        let mut pool = ObjectPool::new();
        pool.pre_allocate(3, || String::from("obj"));

        let item = pool.acquire().unwrap();
        assert_eq!(item, "obj");
        assert_eq!(pool.pool_size(), 2);
        assert_eq!(pool.active(), 1);
    }

    #[test]
    fn release_returns_item_to_pool() {
        let mut pool = ObjectPool::new();
        pool.pre_allocate(1, || 10u32);

        let item = pool.acquire().unwrap();
        assert_eq!(pool.pool_size(), 0);
        assert_eq!(pool.active(), 1);

        pool.release(item);
        assert_eq!(pool.pool_size(), 1);
        assert_eq!(pool.active(), 0);
    }

    #[test]
    fn utilization_reflects_active_ratio() {
        let mut pool = ObjectPool::new();
        pool.pre_allocate(4, || 0u32);

        let _a = pool.acquire().unwrap();
        // 1 active, 3 pooled → 1/4 = 0.25
        assert!((pool.utilization() - 0.25).abs() < f32::EPSILON);

        let _b = pool.acquire().unwrap();
        // 2 active, 2 pooled → 0.5
        assert!((pool.utilization() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn release_without_prior_acquire_saturates_to_zero() {
        let mut pool: ObjectPool<u32> = ObjectPool::new();
        pool.release(99);
        assert_eq!(pool.active(), 0);
        assert_eq!(pool.pool_size(), 1);
    }

    #[test]
    fn multiple_pre_allocate_calls_accumulate() {
        let mut pool = ObjectPool::new();
        pool.pre_allocate(2, || 1u32);
        pool.pre_allocate(3, || 2u32);
        assert_eq!(pool.pool_size(), 5);
    }

    #[test]
    fn default_creates_empty_pool() {
        let pool: ObjectPool<u32> = ObjectPool::default();
        assert_eq!(pool.pool_size(), 0);
        assert_eq!(pool.active(), 0);
    }
}
