//! Bundle storage item.
//!
//! A bundle is a storage item that holds multiple item stacks with a
//! weight-based capacity system. Items are added and removed in LIFO
//! (last-in, first-out) order, similar to a stack.

// ── Item IDs for weight lookups ──────────────────────────────────────────

/// Ender pearl item ID (stacks to 16 → weight 4).
const ITEM_ENDER_PEARL: u16 = 4000;
/// Snowball item ID (stacks to 16 → weight 4).
const ITEM_SNOWBALL: u16 = 4001;

// ── Weight helpers ───────────────────────────────────────────────────────

/// Return the per-unit weight of an item inside a bundle.
///
/// Weight rules mirror vanilla Minecraft:
/// - Items that stack to 64 → weight 1
/// - Items that stack to 16 (ender pearls, snowballs) → weight 4
/// - Items that stack to 1 (tools, armor) → weight 64
#[must_use]
pub fn item_weight(item_id: u16) -> u32 {
    match item_id {
        // Items that stack to 16 → weight 4
        ITEM_ENDER_PEARL | ITEM_SNOWBALL => 4,

        // Tools / weapons / armor (stack to 1) → weight 64
        // Item ID ranges: 200–233 = tools, 300–333 = armor, 600 = bow, 604 = shield
        200..=233 | 300..=333 | 600 | 604 => 64,

        // Everything else stacks to 64 → weight 1
        _ => 1,
    }
}

// ── Bundle ───────────────────────────────────────────────────────────────

/// A bundle storage container with weight-based capacity.
///
/// Items are stored as `(item_id, count)` pairs and managed in LIFO order.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    /// Ordered list of item stacks. Last entry is the "top" of the bundle.
    pub items: Vec<(u16, u8)>,
    /// Maximum total weight the bundle can hold (default 64).
    pub max_weight: u32,
}

impl Default for Bundle {
    fn default() -> Self {
        Self::new()
    }
}

impl Bundle {
    /// Create an empty bundle with the default capacity of 64.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_weight: 64,
        }
    }

    /// Total weight of all items currently in the bundle.
    #[must_use]
    pub fn current_weight(&self) -> u32 {
        self.items
            .iter()
            .map(|&(id, count)| u32::from(count) * item_weight(id))
            .sum()
    }

    /// Add items to the bundle, respecting the weight limit.
    ///
    /// Returns the number of items actually added (may be less than
    /// `count` if the bundle does not have enough remaining capacity).
    pub fn add(&mut self, item_id: u16, count: u8) -> u8 {
        let remaining = self.max_weight.saturating_sub(self.current_weight());
        let weight_per = item_weight(item_id);

        // How many items fit in the remaining capacity?
        let can_fit = remaining / weight_per;
        let actual = (u32::from(count)).min(can_fit) as u8;

        if actual > 0 {
            self.items.push((item_id, actual));
        }

        actual
    }

    /// Remove the most recently added item stack (LIFO).
    ///
    /// Returns the `(item_id, count)` pair, or `None` if the bundle is empty.
    pub fn remove_last(&mut self) -> Option<(u16, u8)> {
        self.items.pop()
    }

    /// Whether the bundle has reached its maximum weight.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.current_weight() >= self.max_weight
    }

    /// Whether the bundle contains no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Fraction of the bundle's capacity currently used (0.0–1.0).
    ///
    /// Used by the UI to render the bundle fullness bar.
    #[must_use]
    pub fn fill_ratio(&self) -> f32 {
        if self.max_weight == 0 {
            return 0.0;
        }
        self.current_weight() as f32 / self.max_weight as f32
    }

    /// Number of distinct item types (stacks) in the bundle.
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction ─────────────────────────────────────────────────

    #[test]
    fn new_bundle_is_empty() {
        let bundle = Bundle::new();
        assert!(bundle.is_empty());
        assert!(!bundle.is_full());
        assert_eq!(bundle.max_weight, 64);
        assert_eq!(bundle.current_weight(), 0);
        assert_eq!(bundle.item_count(), 0);
        assert!((bundle.fill_ratio() - 0.0).abs() < f32::EPSILON);
    }

    // ── Add / remove cycle ───────────────────────────────────────────

    #[test]
    fn add_and_remove_cycle() {
        let mut bundle = Bundle::new();

        // Add 10 cobblestone (weight 1 each → 10 total).
        let added = bundle.add(104, 10);
        assert_eq!(added, 10);
        assert_eq!(bundle.current_weight(), 10);
        assert_eq!(bundle.item_count(), 1);

        // Remove the stack.
        let removed = bundle.remove_last();
        assert_eq!(removed, Some((104, 10)));
        assert!(bundle.is_empty());
    }

    // ── Weight limits ────────────────────────────────────────────────

    #[test]
    fn weight_limits_enforced() {
        let mut bundle = Bundle::new();

        // Fill the bundle with 64 cobblestone (weight 1 each).
        let added = bundle.add(104, 64);
        assert_eq!(added, 64);
        assert!(bundle.is_full());

        // Cannot add more.
        let added = bundle.add(104, 1);
        assert_eq!(added, 0);
    }

    // ── Partial adds ─────────────────────────────────────────────────

    #[test]
    fn partial_add_when_near_capacity() {
        let mut bundle = Bundle::new();

        // Add 60 cobblestone (weight 1 each → 60 total, 4 remaining).
        bundle.add(104, 60);
        assert_eq!(bundle.current_weight(), 60);

        // Try to add 10 more — only 4 should fit.
        let added = bundle.add(104, 10);
        assert_eq!(added, 4);
        assert!(bundle.is_full());
    }

    #[test]
    fn partial_add_heavy_items() {
        let mut bundle = Bundle::new();

        // Ender pearls weigh 4 each. 64 / 4 = 16 max.
        let added = bundle.add(ITEM_ENDER_PEARL, 20);
        assert_eq!(added, 16);
        assert!(bundle.is_full());
    }

    // ── LIFO ordering ────────────────────────────────────────────────

    #[test]
    fn lifo_ordering() {
        let mut bundle = Bundle::new();

        bundle.add(100, 5); // oak log
        bundle.add(101, 3); // oak planks
        bundle.add(102, 2); // stick

        // Remove in reverse order.
        assert_eq!(bundle.remove_last(), Some((102, 2)));
        assert_eq!(bundle.remove_last(), Some((101, 3)));
        assert_eq!(bundle.remove_last(), Some((100, 5)));
        assert_eq!(bundle.remove_last(), None);
    }

    // ── Fill ratio ───────────────────────────────────────────────────

    #[test]
    fn fill_ratio_calculation() {
        let mut bundle = Bundle::new();

        bundle.add(104, 32);
        // 32/64 = 0.5
        assert!((bundle.fill_ratio() - 0.5).abs() < f32::EPSILON);

        bundle.add(104, 32);
        // 64/64 = 1.0
        assert!((bundle.fill_ratio() - 1.0).abs() < f32::EPSILON);
    }

    // ── Item weight values ───────────────────────────────────────────

    #[test]
    fn item_weight_values() {
        // Normal items (stack to 64) → weight 1
        assert_eq!(item_weight(104), 1); // cobblestone
        assert_eq!(item_weight(107), 1); // coal

        // Items that stack to 16 → weight 4
        assert_eq!(item_weight(ITEM_ENDER_PEARL), 4);
        assert_eq!(item_weight(ITEM_SNOWBALL), 4);

        // Tools / armor (stack to 1) → weight 64
        assert_eq!(item_weight(200), 64); // wooden pickaxe
        assert_eq!(item_weight(300), 64); // leather helmet
        assert_eq!(item_weight(600), 64); // bow
        assert_eq!(item_weight(604), 64); // shield
    }

    // ── Unstackable items fill bundle immediately ────────────────────

    #[test]
    fn unstackable_item_fills_bundle() {
        let mut bundle = Bundle::new();

        // A single tool weighs 64 → fills the entire bundle.
        let added = bundle.add(200, 1);
        assert_eq!(added, 1);
        assert!(bundle.is_full());

        // Cannot add anything else.
        let added = bundle.add(104, 1);
        assert_eq!(added, 0);
    }

    // ── Remove from empty bundle ─────────────────────────────────────

    #[test]
    fn remove_from_empty_returns_none() {
        let mut bundle = Bundle::new();
        assert_eq!(bundle.remove_last(), None);
    }

    // ── Multiple distinct stacks ─────────────────────────────────────

    #[test]
    fn multiple_distinct_stacks() {
        let mut bundle = Bundle::new();

        bundle.add(104, 10); // 10 weight
        bundle.add(107, 10); // 10 weight
        bundle.add(108, 10); // 10 weight

        assert_eq!(bundle.item_count(), 3);
        assert_eq!(bundle.current_weight(), 30);
        assert!(!bundle.is_full());
    }

    // ── Mixed weight items ───────────────────────────────────────────

    #[test]
    fn mixed_weight_items() {
        let mut bundle = Bundle::new();

        // Add 8 ender pearls (weight 4 each → 32 total).
        bundle.add(ITEM_ENDER_PEARL, 8);
        assert_eq!(bundle.current_weight(), 32);

        // Add 32 cobblestone (weight 1 each → 32 total, fills to 64).
        let added = bundle.add(104, 32);
        assert_eq!(added, 32);
        assert!(bundle.is_full());
    }

    // ── Zero count add ───────────────────────────────────────────────

    #[test]
    fn zero_count_add_does_nothing() {
        let mut bundle = Bundle::new();
        let added = bundle.add(104, 0);
        assert_eq!(added, 0);
        assert!(bundle.is_empty());
    }

    // ── Default trait ────────────────────────────────────────────────

    #[test]
    fn default_matches_new() {
        let a = Bundle::new();
        let b = Bundle::default();
        assert_eq!(a, b);
    }
}
