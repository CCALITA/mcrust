//! Cake block state, slice eating, and candle variants.

/// Maximum number of slices a fresh cake has.
pub const MAX_SLICES: u8 = 7;

/// Hunger restored per slice eaten.
const HUNGER_PER_SLICE: u32 = 2;

/// Cake block state tracking remaining slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CakeState {
    pub slices_remaining: u8,
}

impl CakeState {
    /// Create a fresh cake with all slices intact.
    pub fn new() -> Self {
        Self {
            slices_remaining: MAX_SLICES,
        }
    }

    /// Attempt to eat one slice. Returns `Some(hunger)` on success,
    /// or `None` if no slices remain.
    pub fn eat_slice(&mut self) -> Option<u32> {
        if self.slices_remaining == 0 {
            return None;
        }
        self.slices_remaining -= 1;
        Some(HUNGER_PER_SLICE)
    }

    /// Whether the cake has been fully eaten and the block should be removed.
    pub fn is_destroyed(&self) -> bool {
        self.slices_remaining == 0
    }

    /// Attach a candle of the given color to this cake.
    pub fn with_candle(&self, candle_color: u8) -> CakeWithCandle {
        CakeWithCandle {
            slices: self.slices_remaining,
            candle_color,
            candle_lit: false,
        }
    }
}

impl Default for CakeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Visual height of a cake block for rendering, given remaining slices.
///
/// Base height is 0.5 (half a block). Each slice adds 1/14, reaching 1.0 at 7 slices.
pub fn cake_height_for_slices(slices: u8) -> f32 {
    0.5 + (slices as f32) / 14.0
}

/// A cake with a candle on top. Cannot be eaten while the candle is attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CakeWithCandle {
    pub slices: u8,
    pub candle_color: u8,
    pub candle_lit: bool,
}

/// Light the candle on a cake with candle.
pub fn light_candle(cake: &mut CakeWithCandle) {
    cake.candle_lit = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cake_has_seven_slices() {
        let cake = CakeState::new();
        assert_eq!(cake.slices_remaining, MAX_SLICES);
        assert_eq!(cake.slices_remaining, 7);
    }

    #[test]
    fn eat_slice_returns_hunger_and_decrements() {
        let mut cake = CakeState::new();
        let hunger = cake.eat_slice();
        assert_eq!(hunger, Some(2));
        assert_eq!(cake.slices_remaining, 6);
    }

    #[test]
    fn eat_all_slices_then_none() {
        let mut cake = CakeState::new();
        for _ in 0..7 {
            assert!(cake.eat_slice().is_some());
        }
        assert_eq!(cake.slices_remaining, 0);
        assert!(cake.is_destroyed());
        assert_eq!(cake.eat_slice(), None);
    }

    #[test]
    fn fresh_cake_is_not_destroyed() {
        let cake = CakeState::new();
        assert!(!cake.is_destroyed());
    }

    #[test]
    fn height_for_full_cake_is_one() {
        assert!((cake_height_for_slices(7) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn height_for_zero_slices_is_half() {
        assert!((cake_height_for_slices(0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn height_increases_with_slices() {
        let h0 = cake_height_for_slices(0);
        let h3 = cake_height_for_slices(3);
        let h7 = cake_height_for_slices(7);
        assert!(h0 < h3);
        assert!(h3 < h7);
    }

    #[test]
    fn with_candle_preserves_slice_count() {
        let cake = CakeState::new();
        let cwc = cake.with_candle(5);
        assert_eq!(cwc.slices, 7);
        assert_eq!(cwc.candle_color, 5);
        assert!(!cwc.candle_lit);
    }

    #[test]
    fn light_candle_sets_lit_true() {
        let cake = CakeState::new();
        let mut cwc = cake.with_candle(2);
        assert!(!cwc.candle_lit);
        light_candle(&mut cwc);
        assert!(cwc.candle_lit);
    }

    #[test]
    fn partial_cake_with_candle_retains_slices() {
        let mut cake = CakeState::new();
        cake.eat_slice();
        cake.eat_slice();
        let cwc = cake.with_candle(0);
        assert_eq!(cwc.slices, 5);
    }
}
