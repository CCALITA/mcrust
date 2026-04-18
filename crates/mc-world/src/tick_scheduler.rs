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

/// Schedules game events to fire on specific future ticks.
///
/// Uses a `BTreeMap` keyed by tick number so events are always processed in
/// chronological order.
pub struct TickScheduler {
    current_tick: u64,
    scheduled: BTreeMap<u64, Vec<ScheduledEvent>>,
}

impl TickScheduler {
    /// Creates a new scheduler starting at tick 0.
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            scheduled: BTreeMap::new(),
        }
    }

    /// Schedules an event to fire `delay` ticks from now.
    pub fn schedule(&mut self, event: ScheduledEvent, delay: u32) {
        let target_tick = self.current_tick + u64::from(delay);
        self.scheduled.entry(target_tick).or_default().push(event);
    }

    /// Advances the scheduler by one tick, returning all events scheduled for
    /// the current tick.
    pub fn advance(&mut self) -> Vec<ScheduledEvent> {
        let events = self
            .scheduled
            .remove(&self.current_tick)
            .unwrap_or_default();
        self.current_tick += 1;
        events
    }

    /// Schedules `count` copies of the same event, each `interval` ticks apart,
    /// starting `interval` ticks from now.
    pub fn schedule_repeating(&mut self, event: ScheduledEvent, interval: u32, count: u32) {
        for i in 1..=count {
            let delay = interval * i;
            self.schedule(event.clone(), delay);
        }
    }

    /// Removes all scheduled events that target the given block position.
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

    /// Returns the total number of pending events across all future ticks.
    pub fn pending_count(&self) -> usize {
        self.scheduled.values().map(|v| v.len()).sum()
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
}
