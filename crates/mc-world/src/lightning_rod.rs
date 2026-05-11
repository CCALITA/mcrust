//! Lightning rod block mechanics: attraction, redirection, and redstone signal.

/// A lightning rod that attracts nearby lightning strikes and emits a redstone signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningRod {
    pub pos: (i32, i32, i32),
    pub powered: bool,
    pub power_ticks: u32,
}

impl LightningRod {
    /// Create a new unpowered lightning rod at the given position.
    #[must_use]
    pub fn new(pos: (i32, i32, i32)) -> Self {
        Self {
            pos,
            powered: false,
            power_ticks: 0,
        }
    }

    /// Whether this rod attracts a lightning strike at the given position.
    /// Returns true if the strike is within 128 blocks horizontally.
    #[must_use]
    pub fn attracts_lightning(&self, strike: (i32, i32, i32)) -> bool {
        let dx = (self.pos.0 - strike.0).unsigned_abs() as u64;
        let dz = (self.pos.2 - strike.2).unsigned_abs() as u64;
        let dist_sq = dx * dx + dz * dz;
        dist_sq <= 128 * 128
    }

    /// Returns the position where a redirected lightning strike should land
    /// (one block above the rod).
    #[must_use]
    pub fn redirect_lightning(&self) -> (i32, i32, i32) {
        (self.pos.0, self.pos.1 + 1, self.pos.2)
    }

    /// Power the rod after a lightning strike. Sets powered to true and
    /// power ticks to 8.
    pub fn power_on_strike(&mut self) {
        self.powered = true;
        self.power_ticks = 8;
    }

    /// Tick the rod, decrementing power ticks and turning off when expired.
    pub fn tick_rod(&mut self) {
        if self.power_ticks > 0 {
            self.power_ticks -= 1;
            if self.power_ticks == 0 {
                self.powered = false;
            }
        }
    }

    /// Redstone signal strength emitted when powered.
    #[must_use]
    pub fn rod_signal_strength(&self) -> u8 {
        if self.powered { 15 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rod_is_unpowered() {
        let rod = LightningRod::new((10, 64, 20));
        assert!(!rod.powered);
        assert_eq!(rod.power_ticks, 0);
    }

    #[test]
    fn attracts_within_range() {
        let rod = LightningRod::new((0, 64, 0));
        assert!(rod.attracts_lightning((100, 80, 0)));
        assert!(rod.attracts_lightning((0, 80, 128)));
        assert!(rod.attracts_lightning((0, 80, -128)));
    }

    #[test]
    fn does_not_attract_beyond_range() {
        let rod = LightningRod::new((0, 64, 0));
        assert!(!rod.attracts_lightning((129, 80, 0)));
        assert!(!rod.attracts_lightning((91, 80, 91)));
    }

    #[test]
    fn attracts_at_exact_boundary() {
        let rod = LightningRod::new((0, 64, 0));
        // 128,0 is exactly on boundary
        assert!(rod.attracts_lightning((128, 80, 0)));
    }

    #[test]
    fn redirect_is_one_above() {
        let rod = LightningRod::new((5, 70, 10));
        assert_eq!(rod.redirect_lightning(), (5, 71, 10));
    }

    #[test]
    fn power_on_strike_sets_state() {
        let mut rod = LightningRod::new((0, 64, 0));
        rod.power_on_strike();
        assert!(rod.powered);
        assert_eq!(rod.power_ticks, 8);
        assert_eq!(rod.rod_signal_strength(), 15);
    }

    #[test]
    fn tick_decrements_and_powers_off() {
        let mut rod = LightningRod::new((0, 64, 0));
        rod.power_on_strike();
        for i in (0..8).rev() {
            assert!(rod.powered);
            rod.tick_rod();
            if i > 0 {
                assert!(rod.powered);
                assert_eq!(rod.power_ticks, i as u32);
            }
        }
        assert!(!rod.powered);
        assert_eq!(rod.power_ticks, 0);
        assert_eq!(rod.rod_signal_strength(), 0);
    }

    #[test]
    fn tick_noop_when_unpowered() {
        let mut rod = LightningRod::new((0, 64, 0));
        rod.tick_rod();
        assert!(!rod.powered);
        assert_eq!(rod.power_ticks, 0);
    }

    #[test]
    fn signal_strength_zero_when_unpowered() {
        let rod = LightningRod::new((0, 64, 0));
        assert_eq!(rod.rod_signal_strength(), 0);
    }
}
