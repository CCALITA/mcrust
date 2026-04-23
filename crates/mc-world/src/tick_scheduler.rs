use std::collections::BTreeMap;

/// Events that can be scheduled to fire on a future tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduledEvent {
    BlockUpdate((i32, i32, i32)),
    RedstoneUpdate((i32, i32, i32)),
    FluidTick((i32, i32, i32)),
    CropGrow((i32, i32, i32)),
    FireTick((i32, i32, i32)),
    EntityTick(u64),
}

impl ScheduledEvent {
    /// Returns the block position associated with this event, if any.
    fn position(&self) -> Option<(i32, i32, i32)> {
        match self {
            ScheduledEvent::BlockUpdate(pos)
            | ScheduledEvent::RedstoneUpdate(pos)
            | ScheduledEvent::FluidTick(pos)
            | ScheduledEvent::CropGrow(pos)
            | ScheduledEvent::FireTick(pos) => Some(*pos),
            ScheduledEvent::EntityTick(_) => None,
        }
    }
}

/// The category of a scheduled block-update tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TickType {
    Block,
    Fluid,
    Redstone,
}

/// A single block-update tick scheduled for a future game tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledTick {
    pub pos: (i32, i32, i32),
    pub tick_type: TickType,
    pub priority: u8,
}

/// Schedules game events to fire on specific future ticks.
///
/// Uses a `BTreeMap` keyed by tick number so events are always processed in
/// chronological order. Supports both legacy [`ScheduledEvent`]s and the newer
/// [`ScheduledTick`] block-update ticks.
pub struct TickScheduler {
    current_tick: u64,
    scheduled: BTreeMap<u64, Vec<ScheduledEvent>>,
    pending: BTreeMap<u64, Vec<ScheduledTick>>,
}

impl TickScheduler {
    /// Creates a new scheduler starting at tick 0.
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            scheduled: BTreeMap::new(),
            pending: BTreeMap::new(),
        }
    }

    // ---- legacy ScheduledEvent API ----

    /// Schedules a legacy event to fire `delay` ticks from now.
    pub fn schedule(&mut self, event: ScheduledEvent, delay: u32) {
        let target_tick = self.current_tick + u64::from(delay);
        self.scheduled.entry(target_tick).or_default().push(event);
    }

    /// Advances the scheduler by one tick, returning all legacy events
    /// scheduled for the current tick.
    pub fn advance(&mut self) -> Vec<ScheduledEvent> {
        let events = self
            .scheduled
            .remove(&self.current_tick)
            .unwrap_or_default();
        self.current_tick += 1;
        events
    }

    /// Schedules `count` copies of the same legacy event, each `interval`
    /// ticks apart, starting `interval` ticks from now.
    pub fn schedule_repeating(&mut self, event: ScheduledEvent, interval: u32, count: u32) {
        for i in 1..=count {
            let delay = interval * i;
            self.schedule(event.clone(), delay);
        }
    }

    /// Removes all scheduled legacy events that target the given block position.
    pub fn cancel_block_events(&mut self, pos: (i32, i32, i32)) {
        let ticks_to_clean: Vec<u64> = self.scheduled.keys().copied().collect();
        for tick in ticks_to_clean {
            if let Some(events) = self.scheduled.get_mut(&tick) {
                events.retain(|e| e.position() != Some(pos));
                if events.is_empty() {
                    self.scheduled.remove(&tick);
                }
            }
        }
    }

    /// Returns the total number of pending legacy events across all future
    /// ticks.
    pub fn pending_count(&self) -> usize {
        self.scheduled.values().map(|v| v.len()).sum()
    }

    // ---- ScheduledTick block-update API ----

    /// Schedules a [`ScheduledTick`] to fire at `current_tick + delay`.
    pub fn schedule_tick(&mut self, tick: ScheduledTick, current_tick: u64, delay: u64) {
        let target = current_tick + delay;
        self.pending.entry(target).or_default().push(tick);
    }

    /// Removes and returns all [`ScheduledTick`]s scheduled for
    /// `<= current_tick`, sorted by ascending priority (lower value = higher
    /// priority, processed first).
    pub fn process(&mut self, current_tick: u64) -> Vec<ScheduledTick> {
        let due_keys: Vec<u64> = self
            .pending
            .range(..=current_tick)
            .map(|(k, _)| *k)
            .collect();

        let mut result: Vec<ScheduledTick> = Vec::new();
        for key in due_keys {
            if let Some(ticks) = self.pending.remove(&key) {
                result.extend(ticks);
            }
        }

        result.sort_by_key(|t| t.priority);
        result
    }

    /// Removes all pending [`ScheduledTick`]s at the given position.
    pub fn cancel_at(&mut self, pos: (i32, i32, i32)) {
        let keys: Vec<u64> = self.pending.keys().copied().collect();
        for key in keys {
            if let Some(ticks) = self.pending.get_mut(&key) {
                ticks.retain(|t| t.pos != pos);
                if ticks.is_empty() {
                    self.pending.remove(&key);
                }
            }
        }
    }

    /// Returns the total number of pending [`ScheduledTick`]s.
    pub fn pending_tick_count(&self) -> usize {
        self.pending.values().map(|v| v.len()).sum()
    }

    /// Returns `true` if there is at least one pending [`ScheduledTick`] at
    /// the given position.
    pub fn has_pending_at(&self, pos: (i32, i32, i32)) -> bool {
        self.pending
            .values()
            .any(|ticks| ticks.iter().any(|t| t.pos == pos))
    }
}

impl Default for TickScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- legacy ScheduledEvent tests ----

    #[test]
    fn single_event_fires_on_correct_tick() {
        let mut scheduler = TickScheduler::new();
        scheduler.schedule(ScheduledEvent::BlockUpdate((1, 2, 3)), 2);

        // Tick 0 -> no events
        let events = scheduler.advance();
        assert!(events.is_empty());

        // Tick 1 -> no events
        let events = scheduler.advance();
        assert!(events.is_empty());

        // Tick 2 -> event fires
        let events = scheduler.advance();
        assert_eq!(events, vec![ScheduledEvent::BlockUpdate((1, 2, 3))]);
    }

    #[test]
    fn events_fire_in_scheduled_order() {
        let mut scheduler = TickScheduler::new();
        scheduler.schedule(ScheduledEvent::RedstoneUpdate((0, 0, 0)), 1);
        scheduler.schedule(ScheduledEvent::FluidTick((1, 1, 1)), 2);
        scheduler.schedule(ScheduledEvent::CropGrow((2, 2, 2)), 3);

        // tick 0 -> nothing
        let empty = scheduler.advance();
        assert!(empty.is_empty());

        // tick 1 -> redstone
        let first = scheduler.advance();
        assert_eq!(first, vec![ScheduledEvent::RedstoneUpdate((0, 0, 0))]);

        // tick 2 -> fluid
        let second = scheduler.advance();
        assert_eq!(second, vec![ScheduledEvent::FluidTick((1, 1, 1))]);

        // tick 3 -> crop
        let third = scheduler.advance();
        assert_eq!(third, vec![ScheduledEvent::CropGrow((2, 2, 2))]);
    }

    #[test]
    fn repeating_events_schedule_at_correct_intervals() {
        let mut scheduler = TickScheduler::new();
        scheduler.schedule_repeating(ScheduledEvent::FireTick((5, 5, 5)), 3, 3);

        // Expect events at ticks 3, 6, and 9
        assert_eq!(scheduler.pending_count(), 3);

        // Advance to tick 3
        for _ in 0..3 {
            let _ = scheduler.advance();
        }
        let events = scheduler.advance(); // tick 3
        assert_eq!(events, vec![ScheduledEvent::FireTick((5, 5, 5))]);

        // Advance to tick 6
        for _ in 0..2 {
            let _ = scheduler.advance();
        }
        let events = scheduler.advance(); // tick 6
        assert_eq!(events, vec![ScheduledEvent::FireTick((5, 5, 5))]);

        // Advance to tick 9
        for _ in 0..2 {
            let _ = scheduler.advance();
        }
        let events = scheduler.advance(); // tick 9
        assert_eq!(events, vec![ScheduledEvent::FireTick((5, 5, 5))]);

        assert_eq!(scheduler.pending_count(), 0);
    }

    #[test]
    fn cancel_removes_events_for_position() {
        let mut scheduler = TickScheduler::new();
        let target_pos = (10, 20, 30);
        scheduler.schedule(ScheduledEvent::BlockUpdate(target_pos), 5);
        scheduler.schedule(ScheduledEvent::RedstoneUpdate(target_pos), 10);
        scheduler.schedule(ScheduledEvent::EntityTick(42), 5);

        assert_eq!(scheduler.pending_count(), 3);

        scheduler.cancel_block_events(target_pos);

        // Only the EntityTick should remain
        assert_eq!(scheduler.pending_count(), 1);
    }

    #[test]
    fn no_events_returns_empty() {
        let mut scheduler = TickScheduler::new();
        let events = scheduler.advance();
        assert!(events.is_empty());

        let events = scheduler.advance();
        assert!(events.is_empty());
    }

    // ---- ScheduledTick block-update tests ----

    #[test]
    fn schedule_and_process_cycle() {
        let mut scheduler = TickScheduler::new();
        let tick = ScheduledTick {
            pos: (1, 2, 3),
            tick_type: TickType::Block,
            priority: 0,
        };
        scheduler.schedule_tick(tick, 10, 5);

        // Nothing due at tick 14
        let result = scheduler.process(14);
        assert!(result.is_empty());

        // Due at tick 15
        let result = scheduler.process(15);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pos, (1, 2, 3));
        assert_eq!(result[0].tick_type, TickType::Block);

        // Already consumed
        let result = scheduler.process(15);
        assert!(result.is_empty());
    }

    #[test]
    fn process_returns_ticks_sorted_by_priority() {
        let mut scheduler = TickScheduler::new();

        // Schedule three ticks at the same target tick with different priorities
        scheduler.schedule_tick(
            ScheduledTick {
                pos: (0, 0, 0),
                tick_type: TickType::Redstone,
                priority: 10,
            },
            0,
            5,
        );
        scheduler.schedule_tick(
            ScheduledTick {
                pos: (1, 1, 1),
                tick_type: TickType::Block,
                priority: 1,
            },
            0,
            5,
        );
        scheduler.schedule_tick(
            ScheduledTick {
                pos: (2, 2, 2),
                tick_type: TickType::Fluid,
                priority: 5,
            },
            0,
            5,
        );

        let result = scheduler.process(5);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].priority, 1);
        assert_eq!(result[1].priority, 5);
        assert_eq!(result[2].priority, 10);
    }

    #[test]
    fn cancel_at_removes_all_ticks_at_position() {
        let mut scheduler = TickScheduler::new();
        let target = (5, 10, 15);

        scheduler.schedule_tick(
            ScheduledTick {
                pos: target,
                tick_type: TickType::Block,
                priority: 0,
            },
            0,
            3,
        );
        scheduler.schedule_tick(
            ScheduledTick {
                pos: target,
                tick_type: TickType::Fluid,
                priority: 1,
            },
            0,
            7,
        );
        scheduler.schedule_tick(
            ScheduledTick {
                pos: (99, 99, 99),
                tick_type: TickType::Redstone,
                priority: 0,
            },
            0,
            3,
        );

        assert_eq!(scheduler.pending_tick_count(), 3);
        assert!(scheduler.has_pending_at(target));

        scheduler.cancel_at(target);

        assert_eq!(scheduler.pending_tick_count(), 1);
        assert!(!scheduler.has_pending_at(target));
        assert!(scheduler.has_pending_at((99, 99, 99)));
    }

    #[test]
    fn multiple_ticks_at_same_time() {
        let mut scheduler = TickScheduler::new();

        scheduler.schedule_tick(
            ScheduledTick {
                pos: (0, 0, 0),
                tick_type: TickType::Block,
                priority: 2,
            },
            10,
            0,
        );
        scheduler.schedule_tick(
            ScheduledTick {
                pos: (1, 1, 1),
                tick_type: TickType::Fluid,
                priority: 1,
            },
            10,
            0,
        );

        let result = scheduler.process(10);
        assert_eq!(result.len(), 2);
        // Lower priority number comes first
        assert_eq!(result[0].priority, 1);
        assert_eq!(result[1].priority, 2);
    }

    #[test]
    fn process_collects_ticks_from_multiple_past_ticks() {
        let mut scheduler = TickScheduler::new();

        scheduler.schedule_tick(
            ScheduledTick {
                pos: (0, 0, 0),
                tick_type: TickType::Block,
                priority: 5,
            },
            0,
            2,
        );
        scheduler.schedule_tick(
            ScheduledTick {
                pos: (1, 1, 1),
                tick_type: TickType::Fluid,
                priority: 1,
            },
            0,
            4,
        );
        scheduler.schedule_tick(
            ScheduledTick {
                pos: (2, 2, 2),
                tick_type: TickType::Redstone,
                priority: 3,
            },
            0,
            6,
        );

        // Process at tick 5 should get ticks scheduled for 2 and 4, but not 6
        let result = scheduler.process(5);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].priority, 1);
        assert_eq!(result[1].priority, 5);

        // The tick at 6 is still pending
        assert_eq!(scheduler.pending_tick_count(), 1);
    }

    #[test]
    fn has_pending_at_returns_false_for_empty_scheduler() {
        let scheduler = TickScheduler::new();
        assert!(!scheduler.has_pending_at((0, 0, 0)));
        assert_eq!(scheduler.pending_tick_count(), 0);
    }
}
