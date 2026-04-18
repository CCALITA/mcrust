// ---------------------------------------------------------------------------
// Progression system bridge
// ---------------------------------------------------------------------------
//
// Unifies XP, advancements, and statistics tracking into a single facade
// that the client can call from gameplay event handlers.

use mc_entity::advancement::{AdvancementTracker, AdvancementTrigger, ADVANCEMENT_REGISTRY};
use mc_entity::experience::{self, ExperienceComponent};
use mc_entity::statistics::{StatisticId, StatisticsTracker};

/// Aggregated progression state for a single player, combining XP tracking,
/// advancement progress, and game statistics.
pub struct ProgressionState {
    pub xp: ExperienceComponent,
    pub advancements: AdvancementTracker,
    pub stats: StatisticsTracker,
}

impl ProgressionState {
    /// Create a fresh progression state with zero XP, no advancements, and
    /// empty statistics.
    pub fn new() -> Self {
        Self {
            xp: ExperienceComponent::new(),
            advancements: AdvancementTracker::new(),
            stats: StatisticsTracker::new(),
        }
    }

    // -- Gameplay event handlers ---------------------------------------------

    /// Called when the player mines a block.
    ///
    /// Awards XP (if the block yields any), increments the `BlocksMined`
    /// statistic, and pushes a `BlockMined` advancement trigger.
    pub fn on_block_mined(&mut self, block_id: u16) {
        let xp = experience::xp_from_block(block_id);
        if xp > 0 {
            experience::add_xp(&mut self.xp, xp);
        }
        self.stats.increment(StatisticId::BlocksMined, 1);
        self.advancements
            .push_trigger(AdvancementTrigger::BlockMined(block_id));
    }

    /// Called when the player kills a mob.
    ///
    /// Awards XP, increments `MobsKilled`, and pushes a `MobKilled` trigger.
    pub fn on_mob_killed(&mut self, mob_kind: u8) {
        let xp = experience::xp_from_mob(mob_kind);
        if xp > 0 {
            experience::add_xp(&mut self.xp, xp);
        }
        self.stats.increment(StatisticId::MobsKilled, 1);
        self.advancements
            .push_trigger(AdvancementTrigger::MobKilled(mob_kind));
    }

    /// Called when the player crafts an item.
    ///
    /// Increments `ItemsCrafted` and pushes an `ItemCrafted` trigger.
    pub fn on_item_crafted(&mut self, item_id: u16) {
        self.stats.increment(StatisticId::ItemsCrafted, 1);
        self.advancements
            .push_trigger(AdvancementTrigger::ItemCrafted(item_id));
    }

    /// Called when the player walks a distance (in blocks).
    ///
    /// Increments `DistanceWalked` by the ceiled integer value of `distance`.
    pub fn on_distance_walked(&mut self, distance: f32) {
        self.stats
            .increment(StatisticId::DistanceWalked, distance.ceil() as u64);
    }

    /// Called when the player jumps.
    ///
    /// Increments the `Jumps` statistic.
    pub fn on_jump(&mut self) {
        self.stats.increment(StatisticId::Jumps, 1);
    }

    // -- Advancement processing ----------------------------------------------

    /// Process all pending advancement triggers and return the display names
    /// of any newly unlocked advancements.
    pub fn check_advancements(&mut self) -> Vec<String> {
        self.advancements
            .check_triggers()
            .into_iter()
            .map(|id| ADVANCEMENT_REGISTRY[id as u8 as usize].name.to_string())
            .collect()
    }

    // -- HUD helpers ---------------------------------------------------------

    /// The player's current experience level (for HUD display).
    pub fn hud_level(&self) -> u32 {
        self.xp.level
    }

    /// Progress toward the next level as a value in `[0.0, 1.0)`.
    pub fn hud_xp_progress(&self) -> f32 {
        self.xp.progress
    }
}

impl Default for ProgressionState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_starts_at_zero() {
        let state = ProgressionState::new();
        assert_eq!(state.hud_level(), 0);
        assert!((state.hud_xp_progress()).abs() < f32::EPSILON);
        assert_eq!(state.stats.get(StatisticId::BlocksMined), 0);
    }

    #[test]
    fn on_block_mined_awards_xp_and_increments_stat() {
        let mut state = ProgressionState::new();
        // Diamond ore (block_id 17) gives 7 XP
        state.on_block_mined(17);
        assert_eq!(state.xp.total_xp, 7);
        assert_eq!(state.stats.get(StatisticId::BlocksMined), 1);
    }

    #[test]
    fn on_block_mined_no_xp_block_still_increments_stat() {
        let mut state = ProgressionState::new();
        // Block 0 gives 0 XP
        state.on_block_mined(0);
        assert_eq!(state.xp.total_xp, 0);
        assert_eq!(state.stats.get(StatisticId::BlocksMined), 1);
    }

    #[test]
    fn on_mob_killed_awards_xp_and_increments_stat() {
        let mut state = ProgressionState::new();
        // Zombie (mob_kind 0) gives 5 XP
        state.on_mob_killed(0);
        assert_eq!(state.xp.total_xp, 5);
        assert_eq!(state.stats.get(StatisticId::MobsKilled), 1);
    }

    #[test]
    fn on_mob_killed_unknown_mob_no_xp() {
        let mut state = ProgressionState::new();
        state.on_mob_killed(255);
        assert_eq!(state.xp.total_xp, 0);
        assert_eq!(state.stats.get(StatisticId::MobsKilled), 1);
    }

    #[test]
    fn on_item_crafted_increments_stat() {
        let mut state = ProgressionState::new();
        state.on_item_crafted(10);
        state.on_item_crafted(20);
        assert_eq!(state.stats.get(StatisticId::ItemsCrafted), 2);
    }

    #[test]
    fn on_distance_walked_accumulates() {
        let mut state = ProgressionState::new();
        state.on_distance_walked(3.5);
        state.on_distance_walked(2.1);
        // ceil(3.5) + ceil(2.1) = 4 + 3 = 7
        assert_eq!(state.stats.get(StatisticId::DistanceWalked), 7);
    }

    #[test]
    fn on_jump_increments_stat() {
        let mut state = ProgressionState::new();
        state.on_jump();
        state.on_jump();
        state.on_jump();
        assert_eq!(state.stats.get(StatisticId::Jumps), 3);
    }

    #[test]
    fn check_advancements_returns_newly_unlocked_names() {
        let mut state = ProgressionState::new();
        // Manually unlock the root advancement so child triggers can fire.
        state
            .advancements
            .push_trigger(AdvancementTrigger::BlockMined(8)); // oak log -> MineWood
        // OpenInventory is the root and has no parent, but MineWood requires
        // OpenInventory as parent. Since we haven't unlocked it, nothing
        // should unlock here.
        let unlocked = state.check_advancements();
        assert!(unlocked.is_empty());
    }

    #[test]
    fn hud_level_and_progress_reflect_xp() {
        let mut state = ProgressionState::new();
        // Add 7 XP -> level 1, progress 0
        experience::add_xp(&mut state.xp, 7);
        assert_eq!(state.hud_level(), 1);
        assert!((state.hud_xp_progress()).abs() < f32::EPSILON);

        // Add 3 more -> level 1, progress 3/9
        experience::add_xp(&mut state.xp, 3);
        assert_eq!(state.hud_level(), 1);
        let expected = 3.0 / 9.0;
        assert!((state.hud_xp_progress() - expected).abs() < f32::EPSILON);
    }
}
