use std::collections::HashMap;

/// Identifies a specific game statistic that can be tracked per player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatisticId {
    BlocksMined,
    BlocksPlaced,
    MobsKilled,
    Deaths,
    Jumps,
    DistanceWalked,
    DistanceSprinted,
    DistanceSwum,
    DamageDealt,
    DamageTaken,
    FoodEaten,
    ItemsCrafted,
    ItemsSmelted,
    TimePlayed,
    PlayersKilled,
    AnimalsKilled,
    FishCaught,
    TimeSinceDeath,
    ToolsBroken,
    EnchantmentsApplied,
    TradesCompleted,
    PotionsBrewed,
    XpGained,
    ChestsOpened,
    FurnacesUsed,
}

impl StatisticId {
    /// Returns a human-readable display name for this statistic.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::BlocksMined => "Blocks Mined",
            Self::BlocksPlaced => "Blocks Placed",
            Self::MobsKilled => "Mobs Killed",
            Self::Deaths => "Deaths",
            Self::Jumps => "Jumps",
            Self::DistanceWalked => "Distance Walked",
            Self::DistanceSprinted => "Distance Sprinted",
            Self::DistanceSwum => "Distance Swum",
            Self::DamageDealt => "Damage Dealt",
            Self::DamageTaken => "Damage Taken",
            Self::FoodEaten => "Food Eaten",
            Self::ItemsCrafted => "Items Crafted",
            Self::ItemsSmelted => "Items Smelted",
            Self::TimePlayed => "Time Played",
            Self::PlayersKilled => "Players Killed",
            Self::AnimalsKilled => "Animals Killed",
            Self::FishCaught => "Fish Caught",
            Self::TimeSinceDeath => "Time Since Death",
            Self::ToolsBroken => "Tools Broken",
            Self::EnchantmentsApplied => "Enchantments Applied",
            Self::TradesCompleted => "Trades Completed",
            Self::PotionsBrewed => "Potions Brewed",
            Self::XpGained => "XP Gained",
            Self::ChestsOpened => "Chests Opened",
            Self::FurnacesUsed => "Furnaces Used",
        }
    }
}

/// Tracks per-player game statistics as a map of statistic IDs to accumulated values.
pub struct StatisticsTracker {
    stats: HashMap<StatisticId, u64>,
}

impl StatisticsTracker {
    /// Creates a new tracker with no recorded statistics.
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Adds `amount` to the given statistic. Creates the entry if it does not exist.
    pub fn increment(&mut self, id: StatisticId, amount: u64) {
        let entry = self.stats.entry(id).or_insert(0);
        *entry += amount;
    }

    /// Returns the current value for `id`, or `0` if the statistic has never been recorded.
    pub fn get(&self, id: StatisticId) -> u64 {
        self.stats.get(&id).copied().unwrap_or(0)
    }

    /// Resets a single statistic back to zero (removes it from the map).
    pub fn reset(&mut self, id: StatisticId) {
        self.stats.remove(&id);
    }

    /// Resets all tracked statistics.
    pub fn reset_all(&mut self) {
        self.stats.clear();
    }

    /// Returns all statistics with a non-zero value.
    pub fn all_nonzero(&self) -> Vec<(StatisticId, u64)> {
        self.stats
            .iter()
            .filter(|(_, v)| **v > 0)
            .map(|(id, v)| (*id, *v))
            .collect()
    }
}

impl Default for StatisticsTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_accumulates() {
        let mut tracker = StatisticsTracker::new();
        tracker.increment(StatisticId::BlocksMined, 5);
        tracker.increment(StatisticId::BlocksMined, 3);
        assert_eq!(tracker.get(StatisticId::BlocksMined), 8);
    }

    #[test]
    fn get_returns_zero_for_untracked() {
        let tracker = StatisticsTracker::new();
        assert_eq!(tracker.get(StatisticId::Deaths), 0);
    }

    #[test]
    fn reset_clears_single_statistic() {
        let mut tracker = StatisticsTracker::new();
        tracker.increment(StatisticId::Jumps, 10);
        tracker.increment(StatisticId::Deaths, 2);
        tracker.reset(StatisticId::Jumps);
        assert_eq!(tracker.get(StatisticId::Jumps), 0);
        assert_eq!(tracker.get(StatisticId::Deaths), 2);
    }

    #[test]
    fn reset_all_clears_everything() {
        let mut tracker = StatisticsTracker::new();
        tracker.increment(StatisticId::BlocksMined, 10);
        tracker.increment(StatisticId::Deaths, 3);
        tracker.reset_all();
        assert_eq!(tracker.get(StatisticId::BlocksMined), 0);
        assert_eq!(tracker.get(StatisticId::Deaths), 0);
        assert!(tracker.all_nonzero().is_empty());
    }

    #[test]
    fn all_nonzero_filters_zero_values() {
        let mut tracker = StatisticsTracker::new();
        tracker.increment(StatisticId::FishCaught, 7);
        tracker.increment(StatisticId::XpGained, 0);
        tracker.increment(StatisticId::ToolsBroken, 2);

        let nonzero = tracker.all_nonzero();
        assert_eq!(nonzero.len(), 2);
        assert!(nonzero.contains(&(StatisticId::FishCaught, 7)));
        assert!(nonzero.contains(&(StatisticId::ToolsBroken, 2)));
    }

    #[test]
    fn display_name_returns_human_readable() {
        assert_eq!(StatisticId::BlocksMined.display_name(), "Blocks Mined");
        assert_eq!(StatisticId::TimePlayed.display_name(), "Time Played");
        assert_eq!(
            StatisticId::EnchantmentsApplied.display_name(),
            "Enchantments Applied"
        );
    }
}
