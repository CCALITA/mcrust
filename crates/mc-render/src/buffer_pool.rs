//! GPU buffer pool for reusing allocated buffer slots.

/// A slot representing a single GPU buffer allocation.
#[derive(Debug, Clone)]
pub struct BufferSlot {
    pub id: u32,
    pub size: usize,
    pub in_use: bool,
}

/// A pool that manages GPU buffer slot allocations.
#[derive(Debug)]
pub struct BufferPool {
    pub buffers: Vec<BufferSlot>,
    pub total_allocated: usize,
    next_id: u32,
}

impl BufferPool {
    /// Creates an empty buffer pool.
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            total_allocated: 0,
            next_id: 0,
        }
    }

    /// Allocates a buffer slot of the given size, reusing a free slot if possible.
    /// Returns the id of the allocated slot.
    pub fn allocate(&mut self, size: usize) -> u32 {
        // Try to reuse a free slot with sufficient size
        for slot in &mut self.buffers {
            if !slot.in_use && slot.size >= size {
                slot.in_use = true;
                return slot.id;
            }
        }

        // Allocate a new slot
        let id = self.next_id;
        self.next_id += 1;
        self.buffers.push(BufferSlot {
            id,
            size,
            in_use: true,
        });
        self.total_allocated += size;
        id
    }

    /// Frees the buffer slot with the given id.
    pub fn free(&mut self, id: u32) {
        for slot in &mut self.buffers {
            if slot.id == id {
                slot.in_use = false;
                return;
            }
        }
    }

    /// Removes all free (unused) buffer slots, reclaiming their memory.
    pub fn defragment(&mut self) {
        let removed_size: usize = self.buffers.iter()
            .filter(|s| !s.in_use)
            .map(|s| s.size)
            .sum();
        self.buffers.retain(|s| s.in_use);
        self.total_allocated -= removed_size;
    }

    /// Returns total memory allocated across all slots.
    pub fn total_memory(&self) -> usize {
        self.total_allocated
    }

    /// Returns the fraction of allocated memory currently in use (0.0 to 1.0).
    pub fn utilization(&self) -> f32 {
        if self.total_allocated == 0 {
            return 0.0;
        }
        let in_use: usize = self.buffers.iter()
            .filter(|s| s.in_use)
            .map(|s| s.size)
            .sum();
        in_use as f32 / self.total_allocated as f32
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_is_empty() {
        let pool = BufferPool::new();
        assert!(pool.buffers.is_empty());
        assert_eq!(pool.total_memory(), 0);
        assert_eq!(pool.utilization(), 0.0);
    }

    #[test]
    fn allocate_creates_slot() {
        let mut pool = BufferPool::new();
        let id = pool.allocate(1024);
        assert_eq!(id, 0);
        assert_eq!(pool.buffers.len(), 1);
        assert_eq!(pool.total_memory(), 1024);
        assert!(pool.buffers[0].in_use);
    }

    #[test]
    fn allocate_returns_unique_ids() {
        let mut pool = BufferPool::new();
        let a = pool.allocate(100);
        let b = pool.allocate(200);
        assert_ne!(a, b);
    }

    #[test]
    fn free_marks_slot_unused() {
        let mut pool = BufferPool::new();
        let id = pool.allocate(512);
        pool.free(id);
        assert!(!pool.buffers[0].in_use);
    }

    #[test]
    fn free_nonexistent_id_is_noop() {
        let mut pool = BufferPool::new();
        pool.allocate(100);
        pool.free(999); // should not panic
        assert_eq!(pool.buffers.len(), 1);
    }

    #[test]
    fn allocate_reuses_free_slot() {
        let mut pool = BufferPool::new();
        let id = pool.allocate(1024);
        pool.free(id);
        let reused = pool.allocate(512);
        assert_eq!(reused, id);
        assert_eq!(pool.buffers.len(), 1);
    }

    #[test]
    fn allocate_does_not_reuse_too_small_slot() {
        let mut pool = BufferPool::new();
        let id = pool.allocate(256);
        pool.free(id);
        let new_id = pool.allocate(512);
        assert_ne!(new_id, id);
        assert_eq!(pool.buffers.len(), 2);
    }

    #[test]
    fn defragment_removes_free_slots() {
        let mut pool = BufferPool::new();
        let a = pool.allocate(100);
        let _b = pool.allocate(200);
        pool.free(a);
        pool.defragment();
        assert_eq!(pool.buffers.len(), 1);
        assert_eq!(pool.total_memory(), 200);
    }

    #[test]
    fn defragment_empty_pool_is_noop() {
        let mut pool = BufferPool::new();
        pool.defragment();
        assert!(pool.buffers.is_empty());
    }

    #[test]
    fn utilization_reflects_usage() {
        let mut pool = BufferPool::new();
        let a = pool.allocate(100);
        let _b = pool.allocate(100);
        assert_eq!(pool.utilization(), 1.0);
        pool.free(a);
        assert!((pool.utilization() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn total_memory_accumulates() {
        let mut pool = BufferPool::new();
        pool.allocate(100);
        pool.allocate(250);
        pool.allocate(150);
        assert_eq!(pool.total_memory(), 500);
    }
}
