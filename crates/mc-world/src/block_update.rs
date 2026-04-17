use std::collections::BTreeMap;

use mc_core::pos::BlockPos;

/// A priority queue of scheduled block updates, keyed by the absolute tick
/// at which they should fire.
///
/// Internally uses a `BTreeMap<u32, Vec<BlockPos>>` so that `tick()` can
/// efficiently pop all entries whose target tick has been reached.
pub struct BlockUpdateQueue {
    current_tick: u32,
    scheduled: BTreeMap<u32, Vec<BlockPos>>,
}

impl BlockUpdateQueue {
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            scheduled: BTreeMap::new(),
        }
    }

    /// Schedule a block update at `pos` to fire `delay` ticks from now.
    pub fn schedule(&mut self, pos: BlockPos, delay: u32) {
        let target_tick = self.current_tick + delay;
        self.scheduled.entry(target_tick).or_default().push(pos);
    }

    /// Advance the internal tick counter by one and return every position
    /// whose scheduled tick has been reached (i.e. target_tick <= current_tick).
    pub fn tick(&mut self) -> Vec<BlockPos> {
        self.current_tick += 1;

        let mut due = Vec::new();

        // Collect all entries up to and including the current tick.
        // `split_off(current_tick + 1)` keeps everything > current_tick in the
        // map and returns it; we swap so `self.scheduled` holds the future
        // entries and we iterate the due ones.
        let future = self.scheduled.split_off(&(self.current_tick + 1));
        let ready = std::mem::replace(&mut self.scheduled, future);

        for (_tick, positions) in ready {
            due.extend(positions);
        }

        due
    }

    /// The current tick counter (useful for debugging / tests).
    pub fn current_tick(&self) -> u32 {
        self.current_tick
    }
}

impl Default for BlockUpdateQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_and_retrieve_after_exact_delay() {
        let mut queue = BlockUpdateQueue::new();
        let pos = BlockPos::new(1, 2, 3);

        queue.schedule(pos, 3);

        // Ticks 1 and 2 should yield nothing.
        assert!(queue.tick().is_empty());
        assert!(queue.tick().is_empty());

        // Tick 3 should return the scheduled position.
        let due = queue.tick();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0], pos);
    }

    #[test]
    fn multiple_positions_at_same_tick() {
        let mut queue = BlockUpdateQueue::new();
        let a = BlockPos::new(0, 0, 0);
        let b = BlockPos::new(1, 1, 1);

        queue.schedule(a, 2);
        queue.schedule(b, 2);

        assert!(queue.tick().is_empty()); // tick 1
        let due = queue.tick(); // tick 2
        assert_eq!(due.len(), 2);
        assert!(due.contains(&a));
        assert!(due.contains(&b));
    }

    #[test]
    fn staggered_delays() {
        let mut queue = BlockUpdateQueue::new();
        let early = BlockPos::new(0, 0, 0);
        let late = BlockPos::new(5, 5, 5);

        queue.schedule(early, 1);
        queue.schedule(late, 3);

        let due1 = queue.tick();
        assert_eq!(due1, vec![early]);

        assert!(queue.tick().is_empty()); // tick 2

        let due3 = queue.tick();
        assert_eq!(due3, vec![late]);
    }

    #[test]
    fn no_updates_returns_empty() {
        let mut queue = BlockUpdateQueue::new();
        assert!(queue.tick().is_empty());
        assert!(queue.tick().is_empty());
    }

    #[test]
    fn delay_zero_fires_on_next_tick() {
        let mut queue = BlockUpdateQueue::new();
        let pos = BlockPos::new(10, 20, 30);

        // delay 0 means target_tick = current_tick + 0 = 0.
        // The next call to tick() advances to tick 1, which is past tick 0,
        // so it should fire immediately.
        queue.schedule(pos, 0);
        let due = queue.tick();
        assert_eq!(due, vec![pos]);
    }

    #[test]
    fn schedule_during_later_tick() {
        let mut queue = BlockUpdateQueue::new();
        let first = BlockPos::new(0, 0, 0);
        let second = BlockPos::new(1, 1, 1);

        queue.schedule(first, 2);
        queue.tick(); // tick 1

        // Schedule another update mid-stream with delay 2 (fires at tick 3).
        queue.schedule(second, 2);

        let due2 = queue.tick(); // tick 2
        assert_eq!(due2, vec![first]);

        let due3 = queue.tick(); // tick 3
        assert_eq!(due3, vec![second]);
    }

    #[test]
    fn current_tick_advances() {
        let mut queue = BlockUpdateQueue::new();
        assert_eq!(queue.current_tick(), 0);
        queue.tick();
        assert_eq!(queue.current_tick(), 1);
        queue.tick();
        assert_eq!(queue.current_tick(), 2);
    }

    #[test]
    fn past_due_updates_fire_immediately() {
        let mut queue = BlockUpdateQueue::new();
        let pos = BlockPos::new(0, 0, 0);

        // Advance a few ticks first.
        queue.tick(); // 1
        queue.tick(); // 2
        queue.tick(); // 3

        // Schedule with delay 0 — target tick is 3, next tick is 4, so it
        // fires at tick 4.
        queue.schedule(pos, 0);
        let due = queue.tick(); // tick 4 — target tick 3 is in the past, so it fires
        assert_eq!(due, vec![pos]);
    }
}
