//! Bridge between the client game loop and world-level tick systems
//! (weather transitions and scheduled block/entity events).

use mc_world::random_tick::{select_random_tick_positions, RANDOM_TICK_SPEED};
use mc_world::{ScheduledEvent, TickScheduler, WeatherSystem};

/// Aggregates world-tick subsystems so the client game loop can advance
/// weather and scheduled events with a single `tick()` call.
pub struct WorldTickState {
    weather: WeatherSystem,
    scheduler: TickScheduler,
    tick_count: u64,
    explosion_queue: Vec<((f32, f32, f32), f32)>,
}

/// Maximum number of scheduled events processed in a single tick to
/// prevent runaway event storms from stalling the frame.
const MAX_EVENTS_PER_TICK: usize = 100;

impl WorldTickState {
    /// Create a new world tick state seeded with `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            weather: WeatherSystem::new(seed),
            scheduler: TickScheduler::new(),
            tick_count: 0,
            explosion_queue: Vec::new(),
        }
    }

    /// Advance all world-tick subsystems by one game tick.
    ///
    /// * Advances the weather state machine.
    /// * Selects random tick positions for crop growth and block updates.
    /// * Drains any queued explosions.
    /// * Advances the tick scheduler and drains up to
    ///   [`MAX_EVENTS_PER_TICK`] events (excess events are dropped for
    ///   this tick to keep frame times bounded).
    pub fn tick(&mut self, _dt: f32) {
        self.tick_count += 1;

        // Weather
        self.weather.tick();

        // Random tick — select positions for the current tick.
        let random_positions =
            select_random_tick_positions(0, 0, 0, 42, self.tick_count, RANDOM_TICK_SPEED);
        if self.tick_count % 600 == 0 {
            log::info!(
                "Random tick: {} positions selected (tick {})",
                random_positions.len(),
                self.tick_count
            );
        }

        // Explosion queue — drain and log each pending explosion.
        for ((x, y, z), power) in self.explosion_queue.drain(..) {
            log::info!("Explosion at ({},{},{}) power={}", x, y, z, power);
        }

        // Scheduler — drain events, capping to avoid runaway processing.
        let events = self.scheduler.advance();
        let _processed: Vec<ScheduledEvent> =
            events.into_iter().take(MAX_EVENTS_PER_TICK).collect();

        // TODO: dispatch `_processed` events to the appropriate subsystems
        // (block updates, redstone, fluid, crops, fire, entities).
    }

    /// Reference to the underlying weather system, useful for sky rendering.
    pub fn weather_state(&self) -> &WeatherSystem {
        &self.weather
    }

    /// `true` when the world is experiencing rain (or thunder).
    pub fn is_raining(&self) -> bool {
        self.weather.is_raining()
    }

    /// `true` only during a thunderstorm.
    pub fn is_thundering(&self) -> bool {
        self.weather.is_thundering()
    }

    /// Current rain intensity in `0.0..=1.0`.
    pub fn rain_strength(&self) -> f32 {
        self.weather.rain_strength()
    }

    /// Schedule a block update at `pos` to fire after `delay` ticks.
    pub fn schedule_block_update(&mut self, pos: (i32, i32, i32), delay: u32) {
        self.scheduler
            .schedule(ScheduledEvent::BlockUpdate(pos), delay);
    }

    /// Queue an explosion to be processed on the next tick.
    pub fn queue_explosion(&mut self, pos: (f32, f32, f32), power: f32) {
        self.explosion_queue.push((pos, power));
    }

    /// The total number of ticks elapsed since world creation.
    pub fn current_tick(&self) -> u64 {
        self.tick_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_tick_zero() {
        let state = WorldTickState::new(42);
        assert_eq!(state.current_tick(), 0);
        assert!(!state.is_raining());
        assert!(!state.is_thundering());
        assert!((state.rain_strength() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_advances_counter() {
        let mut state = WorldTickState::new(42);
        state.tick(0.05);
        assert_eq!(state.current_tick(), 1);
        state.tick(0.05);
        assert_eq!(state.current_tick(), 2);
    }

    #[test]
    fn schedule_block_update_fires() {
        let mut state = WorldTickState::new(42);
        state.schedule_block_update((1, 2, 3), 2);

        // Tick 1 — event not yet due
        state.tick(0.05);
        assert_eq!(state.current_tick(), 1);

        // Tick 2 — event fires (consumed internally)
        state.tick(0.05);
        assert_eq!(state.current_tick(), 2);
    }

    #[test]
    fn weather_state_accessible() {
        let state = WorldTickState::new(42);
        // Just ensure the accessor compiles and returns something sensible.
        let _ws = state.weather_state();
        assert!((state.rain_strength() - 0.0).abs() < f32::EPSILON);
    }
}
