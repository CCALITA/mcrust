//! Async chunk generation queue with priority-based scheduling.

/// Priority levels for chunk generation requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkGenPriority {
    Immediate,
    High,
    Normal,
    Low,
}

impl ChunkGenPriority {
    fn rank(self) -> u8 {
        match self {
            Self::Immediate => 3,
            Self::High => 2,
            Self::Normal => 1,
            Self::Low => 0,
        }
    }
}

impl Ord for ChunkGenPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for ChunkGenPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A request to generate a chunk at a given position.
#[derive(Debug, Clone)]
pub struct ChunkGenRequest {
    pub pos: (i32, i32),
    pub priority: ChunkGenPriority,
    pub queued_at: u64,
}

/// Priority queue for pending chunk generation work.
pub struct ChunkQueue {
    queue: Vec<ChunkGenRequest>,
    max_per_tick: u8,
}

impl ChunkQueue {
    pub fn new(max_per_tick: u8) -> Self {
        Self {
            queue: Vec::new(),
            max_per_tick,
        }
    }

    /// Add a chunk generation request. If already queued, upgrades priority if higher.
    pub fn enqueue(&mut self, pos: (i32, i32), priority: ChunkGenPriority) {
        if let Some(existing) = self.queue.iter_mut().find(|r| r.pos == pos) {
            if priority > existing.priority {
                existing.priority = priority;
            }
            return;
        }
        self.queue.push(ChunkGenRequest {
            pos,
            priority,
            queued_at: 0,
        });
    }

    /// Remove and return up to `count` highest-priority requests.
    pub fn dequeue_batch(&mut self, count: usize) -> Vec<ChunkGenRequest> {
        self.queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        let take = count.min(self.queue.len());
        self.queue.drain(..take).collect()
    }

    /// Remove all requests whose chunk position is beyond `max_dist` from `player_pos`.
    pub fn cancel_out_of_range(&mut self, player_pos: (i32, i32), max_dist: i32) {
        self.queue.retain(|r| {
            let dx = (r.pos.0 - player_pos.0).abs();
            let dz = (r.pos.1 - player_pos.1).abs();
            dx <= max_dist && dz <= max_dist
        });
    }

    /// Number of pending requests.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Maximum chunks to process per tick.
    pub fn max_per_tick(&self) -> u8 {
        self.max_per_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_and_pending_count() {
        let mut q = ChunkQueue::new(4);
        assert_eq!(q.pending_count(), 0);
        q.enqueue((0, 0), ChunkGenPriority::Normal);
        q.enqueue((1, 1), ChunkGenPriority::High);
        assert_eq!(q.pending_count(), 2);
    }

    #[test]
    fn dequeue_batch_returns_highest_priority_first() {
        let mut q = ChunkQueue::new(4);
        q.enqueue((0, 0), ChunkGenPriority::Low);
        q.enqueue((1, 1), ChunkGenPriority::Immediate);
        q.enqueue((2, 2), ChunkGenPriority::Normal);

        let batch = q.dequeue_batch(2);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].priority, ChunkGenPriority::Immediate);
        assert_eq!(batch[1].priority, ChunkGenPriority::Normal);
        assert_eq!(q.pending_count(), 1);
    }

    #[test]
    fn dequeue_batch_with_count_exceeding_queue() {
        let mut q = ChunkQueue::new(4);
        q.enqueue((0, 0), ChunkGenPriority::Normal);
        let batch = q.dequeue_batch(10);
        assert_eq!(batch.len(), 1);
        assert_eq!(q.pending_count(), 0);
    }

    #[test]
    fn cancel_out_of_range_removes_distant_chunks() {
        let mut q = ChunkQueue::new(4);
        q.enqueue((0, 0), ChunkGenPriority::Normal);
        q.enqueue((100, 100), ChunkGenPriority::Normal);
        q.enqueue((5, 5), ChunkGenPriority::Normal);

        q.cancel_out_of_range((0, 0), 10);
        assert_eq!(q.pending_count(), 2);
        // (100,100) should be gone
        assert!(q.queue.iter().all(|r| r.pos != (100, 100)));
    }

    #[test]
    fn enqueue_deduplicates_and_upgrades_priority() {
        let mut q = ChunkQueue::new(4);
        q.enqueue((0, 0), ChunkGenPriority::Low);
        q.enqueue((0, 0), ChunkGenPriority::High);
        assert_eq!(q.pending_count(), 1);

        let batch = q.dequeue_batch(1);
        assert_eq!(batch[0].priority, ChunkGenPriority::High);
    }

    #[test]
    fn enqueue_does_not_downgrade_priority() {
        let mut q = ChunkQueue::new(4);
        q.enqueue((0, 0), ChunkGenPriority::Immediate);
        q.enqueue((0, 0), ChunkGenPriority::Low);
        assert_eq!(q.pending_count(), 1);

        let batch = q.dequeue_batch(1);
        assert_eq!(batch[0].priority, ChunkGenPriority::Immediate);
    }

    #[test]
    fn priority_ordering() {
        assert!(ChunkGenPriority::Immediate > ChunkGenPriority::High);
        assert!(ChunkGenPriority::High > ChunkGenPriority::Normal);
        assert!(ChunkGenPriority::Normal > ChunkGenPriority::Low);
    }
}
