// ---------------------------------------------------------------------------
// Experience & leveling system
// ---------------------------------------------------------------------------

/// Tracks a player's experience points, level, and progress toward the next
/// level, mirroring Minecraft's XP mechanics.
#[derive(Debug, Clone)]
pub struct ExperienceComponent {
    pub total_xp: u32,
    pub level: u32,
    pub progress: f32,
}

impl ExperienceComponent {
    /// Create a new component starting at zero XP, level 0.
    pub fn new() -> Self {
        Self {
            total_xp: 0,
            level: 0,
            progress: 0.0,
        }
    }
}

impl Default for ExperienceComponent {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// XP formulas (Minecraft Java Edition)
// ---------------------------------------------------------------------------

/// XP required to advance from `level` to `level + 1`.
///
/// * Levels  0..=15 : `2 * level + 7`
/// * Levels 16..=30 : `5 * level - 38`
/// * Levels 31+     : `9 * level - 158`
pub fn xp_for_next_level(level: u32) -> u32 {
    if level <= 15 {
        2 * level + 7
    } else if level <= 30 {
        5 * level - 38
    } else {
        9 * level - 158
    }
}

/// Total XP needed to reach `level` from level 0.
///
/// This is the sum of `xp_for_next_level(0)` through `xp_for_next_level(level - 1)`.
pub fn total_xp_for_level(level: u32) -> u32 {
    (0..level).map(xp_for_next_level).sum()
}

// ---------------------------------------------------------------------------
// XP mutation helpers
// ---------------------------------------------------------------------------

/// Add `amount` XP to a component, recalculating level and progress.
pub fn add_xp(comp: &mut ExperienceComponent, amount: u32) {
    comp.total_xp += amount;
    recalculate_level(comp);
}

/// Spend `levels` levels for enchanting. The player's level is reduced by
/// `levels` and total XP is set to match the new level (progress resets to 0).
///
/// If the player has fewer levels than requested, level and total XP are set
/// to zero.
pub fn remove_xp_for_enchanting(comp: &mut ExperienceComponent, levels: u32) {
    if levels >= comp.level {
        comp.level = 0;
        comp.total_xp = 0;
        comp.progress = 0.0;
    } else {
        comp.level -= levels;
        comp.total_xp = total_xp_for_level(comp.level);
        comp.progress = 0.0;
    }
}

// ---------------------------------------------------------------------------
// XP sources
// ---------------------------------------------------------------------------

/// XP dropped when mining a block (by block ID).
///
/// Returns 0 for blocks that do not yield XP.
pub fn xp_from_block(block_id: u16) -> u32 {
    // Block IDs are conventionally defined in mc-core.
    // The mapping here covers ore blocks that drop XP.
    match block_id {
        14 => 1,  // coal_ore
        15 => 1,  // iron_ore
        16 => 1,  // gold_ore
        17 => 7,  // diamond_ore
        18 => 5,  // lapis_ore
        19 => 7,  // emerald_ore
        20 => 3,  // redstone_ore
        21 => 4,  // nether_quartz
        22 => 30, // spawner
        _ => 0,
    }
}

/// XP dropped when a mob is killed (by mob kind discriminant).
///
/// Returns 0 for unknown mob kinds.
pub fn xp_from_mob(mob_kind: u8) -> u32 {
    match mob_kind {
        0 => 5, // zombie
        1 => 5, // skeleton
        2 => 5, // creeper
        3 => 5, // spider
        4 => 1, // pig
        5 => 1, // cow
        6 => 1, // sheep
        7 => 1, // chicken
        _ => 0,
    }
}

/// XP gained from smelting a single item, ceiling the recipe's float value.
pub fn xp_from_smelting(recipe_xp: f32) -> u32 {
    recipe_xp.ceil() as u32
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Recalculate `level` and `progress` from `total_xp`.
fn recalculate_level(comp: &mut ExperienceComponent) {
    let mut level: u32 = 0;
    let mut xp_consumed: u32 = 0;

    loop {
        let needed = xp_for_next_level(level);
        if comp.total_xp < xp_consumed + needed {
            break;
        }
        xp_consumed += needed;
        level += 1;
    }

    comp.level = level;
    let remaining = comp.total_xp - xp_consumed;
    let needed = xp_for_next_level(level);
    comp.progress = if needed > 0 {
        remaining as f32 / needed as f32
    } else {
        0.0
    };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ExperienceComponent default ----------------------------------------

    #[test]
    fn new_component_starts_at_zero() {
        let xp = ExperienceComponent::new();
        assert_eq!(xp.total_xp, 0);
        assert_eq!(xp.level, 0);
        assert!((xp.progress).abs() < f32::EPSILON);
    }

    // -- xp_for_next_level formula ------------------------------------------

    #[test]
    fn level_0_to_1_needs_7_xp() {
        assert_eq!(xp_for_next_level(0), 7);
    }

    #[test]
    fn level_15_to_16_needs_37_xp() {
        // 2*15 + 7 = 37
        assert_eq!(xp_for_next_level(15), 37);
    }

    #[test]
    fn level_16_to_17_needs_42_xp() {
        // 5*16 - 38 = 42
        assert_eq!(xp_for_next_level(16), 42);
    }

    #[test]
    fn level_30_to_31_needs_112_xp() {
        // 5*30 - 38 = 112
        assert_eq!(xp_for_next_level(30), 112);
    }

    #[test]
    fn level_31_to_32_needs_121_xp() {
        // 9*31 - 158 = 121
        assert_eq!(xp_for_next_level(31), 121);
    }

    // -- total_xp_for_level -------------------------------------------------

    #[test]
    fn total_xp_for_level_0_is_zero() {
        assert_eq!(total_xp_for_level(0), 0);
    }

    #[test]
    fn total_xp_for_level_1_is_7() {
        assert_eq!(total_xp_for_level(1), 7);
    }

    #[test]
    fn total_xp_for_level_2_is_16() {
        // 7 + (2*1+7) = 7 + 9 = 16
        assert_eq!(total_xp_for_level(2), 16);
    }

    // -- add_xp and leveling ------------------------------------------------

    #[test]
    fn adding_7_xp_advances_to_level_1() {
        let mut comp = ExperienceComponent::new();
        add_xp(&mut comp, 7);
        assert_eq!(comp.total_xp, 7);
        assert_eq!(comp.level, 1);
        assert!((comp.progress).abs() < f32::EPSILON);
    }

    #[test]
    fn adding_enough_xp_advances_multiple_levels() {
        let mut comp = ExperienceComponent::new();
        // Level 0->1 needs 7, level 1->2 needs 9 => total = 16 for level 2
        add_xp(&mut comp, 16);
        assert_eq!(comp.level, 2);
        assert!((comp.progress).abs() < f32::EPSILON);
    }

    #[test]
    fn partial_progress_is_between_0_and_1() {
        let mut comp = ExperienceComponent::new();
        // 3 XP out of 7 needed for level 1 => progress = 3/7
        add_xp(&mut comp, 3);
        assert_eq!(comp.level, 0);
        assert!(comp.progress > 0.0);
        assert!(comp.progress < 1.0);
        let expected = 3.0 / 7.0;
        assert!((comp.progress - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn progress_is_zero_at_exact_level_boundary() {
        let mut comp = ExperienceComponent::new();
        add_xp(&mut comp, total_xp_for_level(5));
        assert_eq!(comp.level, 5);
        assert!((comp.progress).abs() < f32::EPSILON);
    }

    #[test]
    fn incremental_xp_addition_accumulates() {
        let mut comp = ExperienceComponent::new();
        add_xp(&mut comp, 4);
        add_xp(&mut comp, 3);
        assert_eq!(comp.total_xp, 7);
        assert_eq!(comp.level, 1);
    }

    // -- remove_xp_for_enchanting -------------------------------------------

    #[test]
    fn spending_levels_reduces_correctly() {
        let mut comp = ExperienceComponent::new();
        add_xp(&mut comp, total_xp_for_level(10) + 5); // level 10 + some extra
        remove_xp_for_enchanting(&mut comp, 3);
        assert_eq!(comp.level, 7);
        assert_eq!(comp.total_xp, total_xp_for_level(7));
        assert!((comp.progress).abs() < f32::EPSILON);
    }

    #[test]
    fn spending_all_levels_resets_to_zero() {
        let mut comp = ExperienceComponent::new();
        add_xp(&mut comp, 100);
        let current_level = comp.level;
        remove_xp_for_enchanting(&mut comp, current_level);
        assert_eq!(comp.level, 0);
        assert_eq!(comp.total_xp, 0);
        assert!((comp.progress).abs() < f32::EPSILON);
    }

    #[test]
    fn spending_more_than_current_levels_clamps_to_zero() {
        let mut comp = ExperienceComponent::new();
        add_xp(&mut comp, 10);
        remove_xp_for_enchanting(&mut comp, 999);
        assert_eq!(comp.level, 0);
        assert_eq!(comp.total_xp, 0);
        assert!((comp.progress).abs() < f32::EPSILON);
    }

    // -- xp_from_block ------------------------------------------------------

    #[test]
    fn coal_ore_gives_1_xp() {
        assert_eq!(xp_from_block(14), 1);
    }

    #[test]
    fn diamond_ore_gives_7_xp() {
        assert_eq!(xp_from_block(17), 7);
    }

    #[test]
    fn spawner_gives_30_xp() {
        assert_eq!(xp_from_block(22), 30);
    }

    #[test]
    fn unknown_block_gives_0_xp() {
        assert_eq!(xp_from_block(0), 0);
        assert_eq!(xp_from_block(255), 0);
    }

    // -- xp_from_mob --------------------------------------------------------

    #[test]
    fn hostile_mobs_give_5_xp() {
        assert_eq!(xp_from_mob(0), 5); // zombie
        assert_eq!(xp_from_mob(1), 5); // skeleton
        assert_eq!(xp_from_mob(2), 5); // creeper
        assert_eq!(xp_from_mob(3), 5); // spider
    }

    #[test]
    fn passive_mobs_give_1_xp() {
        assert_eq!(xp_from_mob(4), 1); // pig
        assert_eq!(xp_from_mob(5), 1); // cow
        assert_eq!(xp_from_mob(6), 1); // sheep
        assert_eq!(xp_from_mob(7), 1); // chicken
    }

    #[test]
    fn unknown_mob_gives_0_xp() {
        assert_eq!(xp_from_mob(255), 0);
    }

    // -- xp_from_smelting ---------------------------------------------------

    #[test]
    fn smelting_ceils_float_xp() {
        assert_eq!(xp_from_smelting(0.1), 1);
        assert_eq!(xp_from_smelting(0.7), 1);
        assert_eq!(xp_from_smelting(1.0), 1);
        assert_eq!(xp_from_smelting(1.1), 2);
        assert_eq!(xp_from_smelting(0.0), 0);
    }
}
