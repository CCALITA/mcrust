//! Drowning and air meter system.
//!
//! Tracks a player's or mob's remaining air supply while submerged,
//! applies drowning damage when air is depleted, and restores air
//! when above water.

/// Damage dealt per tick once air is fully depleted.
const DROWNING_DAMAGE: f32 = 2.0;

/// Default maximum air in ticks (15 seconds at 20 TPS).
const DEFAULT_MAX_AIR: u32 = 300;

/// Extra air ticks granted per Respiration enchantment level (15 seconds each).
const RESPIRATION_BONUS_PER_LEVEL: u32 = 300;

/// Result of an underwater air-meter tick.
#[derive(Debug, Clone, PartialEq)]
pub enum DrownResult {
    /// Entity is above water or otherwise breathing normally.
    Breathing,
    /// Entity is submerged but still has air remaining.
    Bubbling(u32),
    /// Entity is out of air and taking drowning damage.
    Drowning(f32),
}

/// Tracks remaining air supply for an entity.
#[derive(Debug, Clone, PartialEq)]
pub struct AirMeter {
    pub air_ticks: u32,
    pub max_air: u32,
}

impl AirMeter {
    /// Creates a new `AirMeter` at full capacity (300 ticks / 15 seconds).
    pub fn new() -> Self {
        Self {
            air_ticks: DEFAULT_MAX_AIR,
            max_air: DEFAULT_MAX_AIR,
        }
    }

    /// Tick while the entity is submerged.
    ///
    /// Decrements air by 1. Returns [`DrownResult::Bubbling`] while air
    /// remains, or [`DrownResult::Drowning`] with 2.0 HP damage once
    /// air reaches zero.
    pub fn tick_underwater(&mut self) -> DrownResult {
        if self.air_ticks > 0 {
            self.air_ticks -= 1;
        }
        if self.air_ticks > 0 {
            DrownResult::Bubbling(self.air_ticks)
        } else {
            DrownResult::Drowning(DROWNING_DAMAGE)
        }
    }

    /// Tick while the entity is above water.
    ///
    /// Restores 1 air tick per call, up to `max_air`.
    pub fn tick_above_water(&mut self) {
        if self.air_ticks < self.max_air {
            self.air_ticks += 1;
        }
    }

    /// Returns the fraction of remaining air (0.0 to 1.0) for HUD rendering.
    pub fn air_fraction(&self) -> f32 {
        if self.max_air == 0 {
            return 0.0;
        }
        self.air_ticks as f32 / self.max_air as f32
    }

    /// Returns `true` when the air meter is at maximum capacity.
    pub fn is_full(&self) -> bool {
        self.air_ticks == self.max_air
    }

    /// Returns the bonus air ticks granted by the Respiration enchantment.
    ///
    /// Each level adds 300 ticks (15 seconds at 20 TPS).
    pub fn respiration_bonus(level: u8) -> u32 {
        level as u32 * RESPIRATION_BONUS_PER_LEVEL
    }
}

impl Default for AirMeter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_air_meter_starts_full() {
        let meter = AirMeter::new();
        assert_eq!(meter.air_ticks, 300);
        assert_eq!(meter.max_air, 300);
        assert!(meter.is_full());
        assert!((meter.air_fraction() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_underwater_decrements_air() {
        let mut meter = AirMeter::new();
        let result = meter.tick_underwater();
        assert_eq!(result, DrownResult::Bubbling(299));
        assert_eq!(meter.air_ticks, 299);
    }

    #[test]
    fn full_drain_cycle_then_drowning_then_restore() {
        let mut meter = AirMeter::new();

        // Drain all 300 ticks — each returns Bubbling until the last
        for i in (1..=299).rev() {
            let result = meter.tick_underwater();
            assert_eq!(result, DrownResult::Bubbling(i));
        }

        // The 300th tick drops air to 0 and returns Drowning
        let result = meter.tick_underwater();
        assert_eq!(result, DrownResult::Drowning(2.0));
        assert_eq!(meter.air_ticks, 0);

        // Subsequent underwater ticks keep returning Drowning
        let result = meter.tick_underwater();
        assert_eq!(result, DrownResult::Drowning(2.0));
        assert_eq!(meter.air_ticks, 0);

        // Surface and restore air
        for i in 1..=300 {
            meter.tick_above_water();
            assert_eq!(meter.air_ticks, i);
        }
        assert!(meter.is_full());

        // Extra above-water ticks don't exceed max
        meter.tick_above_water();
        assert_eq!(meter.air_ticks, 300);
    }

    #[test]
    fn damage_at_zero_air() {
        let mut meter = AirMeter { air_ticks: 1, max_air: 300 };
        let result = meter.tick_underwater();
        // Air dropped from 1 to 0 — drowning starts
        assert_eq!(result, DrownResult::Drowning(2.0));
        assert_eq!(meter.air_ticks, 0);
    }

    #[test]
    fn air_fraction_calculation() {
        let meter = AirMeter { air_ticks: 150, max_air: 300 };
        assert!((meter.air_fraction() - 0.5).abs() < f32::EPSILON);

        let empty = AirMeter { air_ticks: 0, max_air: 300 };
        assert!((empty.air_fraction()).abs() < f32::EPSILON);

        let full = AirMeter::new();
        assert!((full.air_fraction() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn air_fraction_zero_max_air() {
        let meter = AirMeter { air_ticks: 0, max_air: 0 };
        assert!((meter.air_fraction()).abs() < f32::EPSILON);
    }

    #[test]
    fn respiration_bonus_per_level() {
        assert_eq!(AirMeter::respiration_bonus(0), 0);
        assert_eq!(AirMeter::respiration_bonus(1), 300);
        assert_eq!(AirMeter::respiration_bonus(2), 600);
        assert_eq!(AirMeter::respiration_bonus(3), 900);
    }

    #[test]
    fn tick_above_water_restores_incrementally() {
        let mut meter = AirMeter { air_ticks: 295, max_air: 300 };
        for _ in 0..5 {
            meter.tick_above_water();
        }
        assert!(meter.is_full());
    }

    #[test]
    fn default_is_same_as_new() {
        assert_eq!(AirMeter::default(), AirMeter::new());
    }
}
