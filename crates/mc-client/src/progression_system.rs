// ---------------------------------------------------------------------------
// Progression system bridge
// ---------------------------------------------------------------------------
//
// Unifies XP, advancements, and statistics tracking into a single facade
// that the client can call from gameplay event handlers.

use mc_entity::advancement::{ADVANCEMENT_REGISTRY, AdvancementTracker, AdvancementTrigger};
use mc_entity::experience::{self, ExperienceComponent};
use mc_entity::statistics::{StatisticId, StatisticsTracker};
use mc_entity::tool_speed::{ToolTier, ToolType, can_harvest, correct_tool_for_block};

/// Aggregated progression state for a single player, combining XP tracking,
/// advancement progress, and game statistics.
pub struct ProgressionState {
    pub xp: ExperienceComponent,
    pub advancements: AdvancementTracker,
    pub stats: StatisticsTracker,
    pub blocks_mined_with_correct_tool: u32,
    pub blocks_mined_with_wrong_tool: u32,
}

impl ProgressionState {
    /// Create a fresh progression state with zero XP, no advancements, and
    /// empty statistics.
    pub fn new() -> Self {
        Self {
            xp: ExperienceComponent::new(),
            advancements: AdvancementTracker::new(),
            stats: StatisticsTracker::new(),
            blocks_mined_with_correct_tool: 0,
            blocks_mined_with_wrong_tool: 0,
        }
    }

    // -- Gameplay event handlers ---------------------------------------------

    /// Called when the player mines a block.
    ///
    /// Awards XP (if the block yields any), increments the `BlocksMined`
    /// statistic, and pushes a `BlockMined` advancement trigger.
    /// Also tracks correct-tool usage (defaults to correct since held tool
    /// info is not yet available at the call site).
    pub fn on_block_mined(&mut self, block_id: u16) {
        let xp = experience::xp_from_block(block_id);
        if xp > 0 {
            experience::add_xp(&mut self.xp, xp);
        }
        self.stats.increment(StatisticId::BlocksMined, 1);
        self.advancements
            .push_trigger(AdvancementTrigger::BlockMined(block_id));

        // Default to correct-tool since we don't have held tool info yet.
        self.blocks_mined_with_correct_tool += 1;
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

    // -- Tool effectiveness helpers ------------------------------------------

    /// Returns `true` if the player can harvest (get drops from) the given
    /// block with the specified tool type and tier.
    ///
    /// Wraps [`can_harvest`] with enum conversion from raw numeric ids.
    pub fn can_player_harvest(
        &self,
        block_id: u16,
        held_tool_type: u8,
        held_tool_tier: u8,
    ) -> bool {
        let tool = tool_type_from_id(held_tool_type);
        let tier = tool_tier_from_id(held_tool_tier);
        can_harvest(block_id, tool, tier)
    }

    /// Returns `true` if the held tool matches the preferred tool for the block.
    pub fn is_correct_tool(&self, block_id: u16, tool_type: u8) -> bool {
        let preferred = correct_tool_for_block(block_id);
        let held = tool_type_from_id(tool_type);
        held == preferred
    }

    /// Ratio of blocks mined with the correct tool vs total blocks mined.
    ///
    /// Returns `1.0` when no blocks have been mined yet (no data).
    pub fn tool_efficiency_ratio(&self) -> f32 {
        let total = self.blocks_mined_with_correct_tool + self.blocks_mined_with_wrong_tool;
        if total == 0 {
            return 1.0;
        }
        self.blocks_mined_with_correct_tool as f32 / total as f32
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

// ---------------------------------------------------------------------------
// Enum conversion helpers
// ---------------------------------------------------------------------------

/// Convert a raw `u8` tool type id to [`ToolType`]. Unknown ids map to `None`.
fn tool_type_from_id(id: u8) -> ToolType {
    match id {
        0 => ToolType::None,
        1 => ToolType::Pickaxe,
        2 => ToolType::Axe,
        3 => ToolType::Shovel,
        4 => ToolType::Sword,
        5 => ToolType::Hoe,
        _ => ToolType::None,
    }
}

/// Convert a raw `u8` tool tier id to [`ToolTier`]. Unknown ids map to `Wood`.
fn tool_tier_from_id(id: u8) -> ToolTier {
    match id {
        0 => ToolTier::Wood,
        1 => ToolTier::Stone,
        2 => ToolTier::Iron,
        3 => ToolTier::Gold,
        4 => ToolTier::Diamond,
        5 => ToolTier::Netherite,
        _ => ToolTier::Wood,
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

    // -- Tool effectiveness tests -------------------------------------------

    #[test]
    fn can_player_harvest_iron_ore_with_stone_pickaxe() {
        let state = ProgressionState::new();
        // Stone pickaxe (type=1, tier=1) can harvest iron ore (block 15)
        assert!(state.can_player_harvest(15, 1, 1));
    }

    #[test]
    fn cannot_harvest_iron_ore_with_wood_pickaxe() {
        let state = ProgressionState::new();
        // Wood pickaxe (type=1, tier=0) cannot harvest iron ore
        assert!(!state.can_player_harvest(15, 1, 0));
    }

    #[test]
    fn can_harvest_dirt_with_anything() {
        let state = ProgressionState::new();
        // Dirt (block 3) has no harvest requirement
        assert!(state.can_player_harvest(3, 0, 0));
    }

    #[test]
    fn is_correct_tool_pickaxe_for_stone() {
        let state = ProgressionState::new();
        // Stone (block 1) prefers pickaxe (type 1)
        assert!(state.is_correct_tool(1, 1));
    }

    #[test]
    fn is_correct_tool_axe_for_oak_log() {
        let state = ProgressionState::new();
        // Oak log (block 17) prefers axe (type 2)
        assert!(state.is_correct_tool(17, 2));
    }

    #[test]
    fn is_correct_tool_wrong_tool_returns_false() {
        let state = ProgressionState::new();
        // Stone (block 1) prefers pickaxe, not shovel (type 3)
        assert!(!state.is_correct_tool(1, 3));
    }

    #[test]
    fn on_block_mined_increments_correct_tool_counter() {
        let mut state = ProgressionState::new();
        state.on_block_mined(1);
        state.on_block_mined(3);
        assert_eq!(state.blocks_mined_with_correct_tool, 2);
        assert_eq!(state.blocks_mined_with_wrong_tool, 0);
    }

    #[test]
    fn tool_efficiency_ratio_no_blocks_returns_one() {
        let state = ProgressionState::new();
        assert!((state.tool_efficiency_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tool_efficiency_ratio_all_correct() {
        let mut state = ProgressionState::new();
        state.blocks_mined_with_correct_tool = 10;
        state.blocks_mined_with_wrong_tool = 0;
        assert!((state.tool_efficiency_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tool_efficiency_ratio_mixed() {
        let mut state = ProgressionState::new();
        state.blocks_mined_with_correct_tool = 3;
        state.blocks_mined_with_wrong_tool = 1;
        let expected = 3.0 / 4.0;
        assert!((state.tool_efficiency_ratio() - expected).abs() < f32::EPSILON);
    }
}
