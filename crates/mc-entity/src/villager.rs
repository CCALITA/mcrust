// ---------------------------------------------------------------------------
// Villager trading system
// ---------------------------------------------------------------------------

use crate::villager_trades::default_trades;

// ---------------------------------------------------------------------------
// Profession enum
// ---------------------------------------------------------------------------

/// All villager professions in Minecraft. Each profession determines the
/// pool of trades offered at each experience level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VillagerProfession {
    Farmer,
    Librarian,
    Cleric,
    Armorer,
    Weaponsmith,
    Toolsmith,
    Butcher,
    Leatherworker,
    Fletcher,
    Cartographer,
    Mason,
    Shepherd,
    Nitwit,
}

// ---------------------------------------------------------------------------
// Trade offer
// ---------------------------------------------------------------------------

/// A single trade slot. Items are represented as `(item_id, quantity)`.
///
/// `input2` is `None` for trades that require only one input item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeOffer {
    pub input1: (u16, u8),
    pub input2: Option<(u16, u8)>,
    pub output: (u16, u8),
    pub max_uses: u8,
    pub current_uses: u8,
    pub xp_reward: u8,
}

impl TradeOffer {
    /// Returns `true` when the trade still has uses remaining.
    pub fn is_available(&self) -> bool {
        self.current_uses < self.max_uses
    }

    /// Consume one use of this trade. Panics in debug mode if the trade is
    /// already exhausted; in release mode the counter saturates.
    pub fn use_trade(&mut self) {
        debug_assert!(self.is_available(), "trade is exhausted");
        self.current_uses = self.current_uses.saturating_add(1);
    }
}

// ---------------------------------------------------------------------------
// Trade result
// ---------------------------------------------------------------------------

/// Outcome of attempting a trade with a villager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeResult {
    /// Trade succeeded; contains the output item `(id, qty)`.
    Success { output: (u16, u8) },
    /// Player does not possess the required input items.
    InsufficientItems,
    /// The trade has been used the maximum number of times.
    TradeExhausted,
    /// The supplied trade index does not exist.
    InvalidIndex,
}

// ---------------------------------------------------------------------------
// Villager data
// ---------------------------------------------------------------------------

/// Persistent data for a single villager entity.
#[derive(Debug, Clone)]
pub struct VillagerData {
    pub profession: VillagerProfession,
    pub level: u8,
    pub trades: Vec<TradeOffer>,
    pub xp: u32,
}

impl VillagerData {
    /// Create a new villager of the given profession at level 1 with its
    /// initial set of trades.
    pub fn new(profession: VillagerProfession) -> Self {
        let trades = default_trades(profession, 1);
        Self {
            profession,
            level: 1,
            trades,
            xp: 0,
        }
    }

    /// Grant `amount` experience to this villager. Returns `true` if the
    /// villager levelled up (and new trades were appended).
    pub fn add_xp(&mut self, amount: u32) -> bool {
        if self.level >= 5 {
            return false;
        }

        self.xp += amount;
        let threshold = xp_for_level(self.level + 1);

        if self.xp >= threshold {
            self.level += 1;
            let new_trades = default_trades(self.profession, self.level);
            self.trades.extend(new_trades);
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// XP thresholds
// ---------------------------------------------------------------------------

/// Cumulative XP required to reach the given villager level (2 through 5).
///
/// Levels below 2 or above 5 return `u32::MAX` (unreachable).
pub fn xp_for_level(level: u8) -> u32 {
    match level {
        2 => 10,
        3 => 70,
        4 => 150,
        5 => 250,
        _ => u32::MAX,
    }
}

// ---------------------------------------------------------------------------
// Trade execution
// ---------------------------------------------------------------------------

/// Attempt to execute the trade at `trade_idx` on `villager`.
///
/// `has_input1` / `has_input2` indicate whether the player has the required
/// items in sufficient quantity. On success the trade's use counter is
/// incremented and the villager gains XP from the trade.
pub fn execute_trade(
    villager: &mut VillagerData,
    trade_idx: usize,
    has_input1: bool,
    has_input2: bool,
) -> TradeResult {
    let trade = match villager.trades.get(trade_idx) {
        Some(t) => t,
        None => return TradeResult::InvalidIndex,
    };

    if !trade.is_available() {
        return TradeResult::TradeExhausted;
    }

    let needs_input2 = trade.input2.is_some();
    if !has_input1 || (needs_input2 && !has_input2) {
        return TradeResult::InsufficientItems;
    }

    // Clone output before mutating so we satisfy the borrow checker.
    let output = trade.output;
    let xp_reward = trade.xp_reward as u32;

    // SAFETY of indexing: we already verified the index above via `get`.
    villager.trades[trade_idx].use_trade();
    villager.add_xp(xp_reward);

    TradeResult::Success { output }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- TradeOffer ----------------------------------------------------------

    #[test]
    fn new_trade_is_available() {
        let trade = TradeOffer {
            input1: (1, 20),
            input2: None,
            output: (2, 1),
            max_uses: 16,
            current_uses: 0,
            xp_reward: 2,
        };
        assert!(trade.is_available());
    }

    #[test]
    fn exhausted_trade_is_not_available() {
        let trade = TradeOffer {
            input1: (1, 20),
            input2: None,
            output: (2, 1),
            max_uses: 16,
            current_uses: 16,
            xp_reward: 2,
        };
        assert!(!trade.is_available());
    }

    #[test]
    fn use_trade_increments_current_uses() {
        let mut trade = TradeOffer {
            input1: (1, 20),
            input2: None,
            output: (2, 1),
            max_uses: 4,
            current_uses: 0,
            xp_reward: 2,
        };
        trade.use_trade();
        assert_eq!(trade.current_uses, 1);
        trade.use_trade();
        assert_eq!(trade.current_uses, 2);
    }

    // -- VillagerData --------------------------------------------------------

    #[test]
    fn new_villager_starts_at_level_1() {
        let v = VillagerData::new(VillagerProfession::Farmer);
        assert_eq!(v.level, 1);
        assert_eq!(v.xp, 0);
        assert_eq!(v.profession, VillagerProfession::Farmer);
    }

    #[test]
    fn new_villager_has_level_1_trades() {
        let v = VillagerData::new(VillagerProfession::Farmer);
        assert!(!v.trades.is_empty());
        // Farmer L1 has exactly 2 trades
        assert_eq!(v.trades.len(), 2);
    }

    // -- add_xp and levelling ------------------------------------------------

    #[test]
    fn add_xp_returns_false_when_no_level_up() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        let levelled = v.add_xp(5);
        assert!(!levelled);
        assert_eq!(v.level, 1);
        assert_eq!(v.xp, 5);
    }

    #[test]
    fn add_xp_levels_up_at_threshold() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        let levelled = v.add_xp(10);
        assert!(levelled);
        assert_eq!(v.level, 2);
        // Should have appended L2 trades
        let initial_l1 = default_trades(VillagerProfession::Farmer, 1).len();
        let new_l2 = default_trades(VillagerProfession::Farmer, 2).len();
        assert_eq!(v.trades.len(), initial_l1 + new_l2);
    }

    #[test]
    fn add_xp_does_not_exceed_level_5() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        v.level = 5;
        let levelled = v.add_xp(1000);
        assert!(!levelled);
        assert_eq!(v.level, 5);
    }

    // -- xp_for_level --------------------------------------------------------

    #[test]
    fn xp_thresholds_are_correct() {
        assert_eq!(xp_for_level(2), 10);
        assert_eq!(xp_for_level(3), 70);
        assert_eq!(xp_for_level(4), 150);
        assert_eq!(xp_for_level(5), 250);
    }

    #[test]
    fn xp_for_invalid_level_returns_max() {
        assert_eq!(xp_for_level(0), u32::MAX);
        assert_eq!(xp_for_level(1), u32::MAX);
        assert_eq!(xp_for_level(6), u32::MAX);
    }

    // -- execute_trade -------------------------------------------------------

    #[test]
    fn successful_trade_returns_output() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        let result = execute_trade(&mut v, 0, true, false);
        assert_eq!(
            result,
            TradeResult::Success {
                output: (2, 1) // 1 emerald
            }
        );
    }

    #[test]
    fn trade_deducts_uses() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        execute_trade(&mut v, 0, true, false);
        assert_eq!(v.trades[0].current_uses, 1);
    }

    #[test]
    fn exhausted_trade_fails() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        // Use all 16 uses
        for _ in 0..16 {
            execute_trade(&mut v, 0, true, false);
        }
        let result = execute_trade(&mut v, 0, true, false);
        assert_eq!(result, TradeResult::TradeExhausted);
    }

    #[test]
    fn trade_with_missing_input1_fails() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        let result = execute_trade(&mut v, 0, false, false);
        assert_eq!(result, TradeResult::InsufficientItems);
    }

    #[test]
    fn trade_with_missing_input2_fails() {
        // Librarian L1 trade index 1 requires input2 (book)
        let mut v = VillagerData::new(VillagerProfession::Librarian);
        let result = execute_trade(&mut v, 1, true, false);
        assert_eq!(result, TradeResult::InsufficientItems);
    }

    #[test]
    fn trade_with_both_inputs_succeeds() {
        let mut v = VillagerData::new(VillagerProfession::Librarian);
        let result = execute_trade(&mut v, 1, true, true);
        assert_eq!(
            result,
            TradeResult::Success {
                output: (12, 1) // enchanted book
            }
        );
    }

    #[test]
    fn invalid_trade_index_fails() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        let result = execute_trade(&mut v, 99, true, false);
        assert_eq!(result, TradeResult::InvalidIndex);
    }

    #[test]
    fn trade_grants_villager_xp() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        assert_eq!(v.xp, 0);
        execute_trade(&mut v, 0, true, false); // xp_reward = 2
        assert_eq!(v.xp, 2);
    }

    #[test]
    fn repeated_trades_can_level_up_villager() {
        let mut v = VillagerData::new(VillagerProfession::Farmer);
        let initial_trades = v.trades.len();
        // Each trade at index 0 gives 2 XP; need 10 to level up => 5 trades
        for _ in 0..5 {
            execute_trade(&mut v, 0, true, false);
        }
        assert_eq!(v.level, 2);
        assert!(v.trades.len() > initial_trades);
    }

    // -- TradeResult variants ------------------------------------------------

    #[test]
    fn trade_result_variants_are_distinct() {
        let success = TradeResult::Success { output: (2, 1) };
        let insufficient = TradeResult::InsufficientItems;
        let exhausted = TradeResult::TradeExhausted;
        let invalid = TradeResult::InvalidIndex;

        assert_ne!(success, insufficient);
        assert_ne!(insufficient, exhausted);
        assert_ne!(exhausted, invalid);
    }
}
