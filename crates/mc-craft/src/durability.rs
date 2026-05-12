/// Item durability tracking, damage, repair, and UI helpers.
///
/// Provides [`ItemDurability`] for per-item durability state, tier-based max
/// durability lookup, the Unbreaking enchantment chance formula, and a color
/// gradient for the durability bar HUD element.

/// Per-item durability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemDurability {
    pub current: u16,
    pub max: u16,
}

impl ItemDurability {
    /// Create a new durability tracker at full durability.
    #[must_use]
    pub fn new(max: u16) -> Self {
        Self { current: max, max }
    }
}

/// Maximum durability for a tool tier.
///
/// | Tier | Material   | Durability |
/// |------|-----------|------------|
/// | 0    | Wood/Gold | 59         |
/// | 1    | Stone     | 131        |
/// | 2    | Iron      | 250        |
/// | 3    | Gold      | 32         |
/// | 4    | Diamond   | 1561       |
/// | 5    | Netherite | 2031       |
#[must_use]
pub fn tool_max_durability(tier: u8) -> u16 {
    match tier {
        0 => 59,
        1 => 131,
        2 => 250,
        3 => 32,
        4 => 1561,
        5 => 2031,
        _ => 0,
    }
}

/// Apply damage to an item. Returns `true` when the item breaks (current
/// reaches zero).
///
/// Damage is clamped so `current` never underflows below zero.
pub fn damage_item(durability: &mut ItemDurability, amount: u16) -> bool {
    durability.current = durability.current.saturating_sub(amount);
    durability.current == 0
}

/// Repair an item by `amount` points, capped at `max`.
pub fn repair_item(durability: &mut ItemDurability, amount: u16) {
    durability.current = durability.current.saturating_add(amount).min(durability.max);
}

/// Returns the fraction of durability remaining, in `[0.0, 1.0]`.
///
/// Returns `0.0` when `max` is zero to avoid division by zero.
#[must_use]
pub fn durability_fraction(durability: &ItemDurability) -> f32 {
    if durability.max == 0 {
        return 0.0;
    }
    durability.current as f32 / durability.max as f32
}

/// RGB color for the durability bar, interpolating from red (0.0) through
/// yellow (0.5) to green (1.0).
///
/// `fraction` is clamped to `[0.0, 1.0]`.
#[must_use]
pub fn durability_bar_color(fraction: f32) -> [f32; 3] {
    let f = fraction.clamp(0.0, 1.0);
    let r = (1.0 - f).min(1.0) * 2.0;
    let g = f.min(1.0) * 2.0;
    [r.min(1.0), g.min(1.0), 0.0]
}

/// Probability that a durability point is **not** consumed when an item is
/// used, given the Unbreaking enchantment level.
///
/// Formula: `1 / (level + 1)`.
/// Level 0 returns `1.0` (always consumed), matching vanilla behaviour where
/// Unbreaking 0 is equivalent to no enchantment.
#[must_use]
pub fn unbreaking_chance(level: u8) -> f32 {
    1.0 / (level as f32 + 1.0)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ItemDurability::new ──────────────────────────────────────────

    #[test]
    fn new_starts_at_full_durability() {
        let d = ItemDurability::new(250);
        assert_eq!(d.current, 250);
        assert_eq!(d.max, 250);
    }

    #[test]
    fn new_zero_max_is_valid() {
        let d = ItemDurability::new(0);
        assert_eq!(d.current, 0);
        assert_eq!(d.max, 0);
    }

    // ── tool_max_durability ─────────────────────────────────────────

    #[test]
    fn tier_zero_is_wood() {
        assert_eq!(tool_max_durability(0), 59);
    }

    #[test]
    fn tier_one_is_stone() {
        assert_eq!(tool_max_durability(1), 131);
    }

    #[test]
    fn tier_two_is_iron() {
        assert_eq!(tool_max_durability(2), 250);
    }

    #[test]
    fn tier_three_is_gold() {
        assert_eq!(tool_max_durability(3), 32);
    }

    #[test]
    fn tier_four_is_diamond() {
        assert_eq!(tool_max_durability(4), 1561);
    }

    #[test]
    fn tier_five_is_netherite() {
        assert_eq!(tool_max_durability(5), 2031);
    }

    #[test]
    fn unknown_tier_returns_zero() {
        assert_eq!(tool_max_durability(6), 0);
        assert_eq!(tool_max_durability(255), 0);
    }

    // ── damage_item ─────────────────────────────────────────────────

    #[test]
    fn damage_reduces_current() {
        let mut d = ItemDurability::new(100);
        let broken = damage_item(&mut d, 30);
        assert!(!broken);
        assert_eq!(d.current, 70);
    }

    #[test]
    fn damage_to_zero_returns_broken() {
        let mut d = ItemDurability::new(50);
        let broken = damage_item(&mut d, 50);
        assert!(broken);
        assert_eq!(d.current, 0);
    }

    #[test]
    fn damage_past_zero_saturates() {
        let mut d = ItemDurability::new(10);
        let broken = damage_item(&mut d, 999);
        assert!(broken);
        assert_eq!(d.current, 0);
    }

    #[test]
    fn zero_damage_does_not_break() {
        let mut d = ItemDurability::new(100);
        let broken = damage_item(&mut d, 0);
        assert!(!broken);
        assert_eq!(d.current, 100);
    }

    // ── repair_item ─────────────────────────────────────────────────

    #[test]
    fn repair_increases_current() {
        let mut d = ItemDurability::new(100);
        d.current = 40;
        repair_item(&mut d, 30);
        assert_eq!(d.current, 70);
    }

    #[test]
    fn repair_caps_at_max() {
        let mut d = ItemDurability::new(100);
        d.current = 90;
        repair_item(&mut d, 50);
        assert_eq!(d.current, 100);
    }

    #[test]
    fn repair_zero_is_noop() {
        let mut d = ItemDurability::new(100);
        d.current = 60;
        repair_item(&mut d, 0);
        assert_eq!(d.current, 60);
    }

    // ── durability_fraction ─────────────────────────────────────────

    #[test]
    fn fraction_full_is_one() {
        let d = ItemDurability::new(200);
        assert!((durability_fraction(&d) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_half() {
        let mut d = ItemDurability::new(200);
        d.current = 100;
        assert!((durability_fraction(&d) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_zero_max_returns_zero() {
        let d = ItemDurability::new(0);
        assert!((durability_fraction(&d) - 0.0).abs() < f32::EPSILON);
    }

    // ── durability_bar_color ────────────────────────────────────────

    #[test]
    fn color_at_zero_is_red() {
        let [r, g, b] = durability_bar_color(0.0);
        assert!(r > 0.9);
        assert!(g < 0.01);
        assert!(b < 0.01);
    }

    #[test]
    fn color_at_one_is_green() {
        let [r, g, b] = durability_bar_color(1.0);
        assert!(r < 0.01);
        assert!(g > 0.9);
        assert!(b < 0.01);
    }

    #[test]
    fn color_at_half_is_yellow() {
        let [r, g, b] = durability_bar_color(0.5);
        assert!(r > 0.9);
        assert!(g > 0.9);
        assert!(b < 0.01);
    }

    #[test]
    fn color_clamps_negative() {
        let c = durability_bar_color(-1.0);
        assert_eq!(c, durability_bar_color(0.0));
    }

    #[test]
    fn color_clamps_above_one() {
        let c = durability_bar_color(2.0);
        assert_eq!(c, durability_bar_color(1.0));
    }

    // ── unbreaking_chance ───────────────────────────────────────────

    #[test]
    fn unbreaking_zero_always_consumed() {
        assert!((unbreaking_chance(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unbreaking_one_is_half() {
        assert!((unbreaking_chance(1) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn unbreaking_two_is_third() {
        let expected = 1.0 / 3.0;
        assert!((unbreaking_chance(2) - expected).abs() < 0.001);
    }

    #[test]
    fn unbreaking_three_is_quarter() {
        assert!((unbreaking_chance(3) - 0.25).abs() < f32::EPSILON);
    }
}
