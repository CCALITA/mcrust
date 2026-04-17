use mc_core::block::BlockId;
use mc_core::item::{ToolTier, ToolType};
use mc_core::pos::BlockPos;
use rand::Rng;

// ---------------------------------------------------------------------------
// DurabilityComponent
// ---------------------------------------------------------------------------

/// Tracks remaining durability for a tool item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurabilityComponent {
    pub current: u32,
    pub max: u32,
}

impl DurabilityComponent {
    /// Create a new durability component at full durability.
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    /// Returns `true` when the tool has no remaining durability.
    pub fn is_broken(&self) -> bool {
        self.current == 0
    }

    /// Returns the remaining durability as a percentage (0.0..=100.0).
    pub fn remaining_percent(&self) -> f32 {
        if self.max == 0 {
            return 0.0;
        }
        (self.current as f32 / self.max as f32) * 100.0
    }
}

// ---------------------------------------------------------------------------
// use_tool
// ---------------------------------------------------------------------------

/// Consume one durability point, with the Unbreaking enchantment providing a
/// `1 / (unbreaking_level + 1)` chance of actually consuming the point.
///
/// Returns `true` if the tool broke (durability reached zero) as a result.
pub fn use_tool(durability: &mut DurabilityComponent, unbreaking_level: u8) -> bool {
    if durability.is_broken() {
        return true;
    }

    let mut rng = rand::rng();
    let chance = 1.0 / (unbreaking_level as f32 + 1.0);
    if rng.random::<f32>() < chance {
        durability.current = durability.current.saturating_sub(1);
    }

    durability.is_broken()
}

/// Deterministic variant of [`use_tool`] for testing. The caller supplies the
/// random roll directly (0.0..1.0).
#[cfg(test)]
fn use_tool_with_roll(
    durability: &mut DurabilityComponent,
    unbreaking_level: u8,
    roll: f32,
) -> bool {
    if durability.is_broken() {
        return true;
    }

    let chance = 1.0 / (unbreaking_level as f32 + 1.0);
    if roll < chance {
        durability.current = durability.current.saturating_sub(1);
    }

    durability.is_broken()
}

// ---------------------------------------------------------------------------
// preferred_tool
// ---------------------------------------------------------------------------

/// Returns the preferred tool type for mining a given block.
///
/// * **Pickaxe** -- stone, ores, cobblestone, obsidian, bricks, stone bricks,
///   furnace, etc.
/// * **Axe** -- logs, planks, crafting table, chest, bookshelf.
/// * **Shovel** -- dirt, sand, gravel, clay, snow, grass block, mycelium,
///   podzol, soul sand.
/// * **None** -- everything else.
pub fn preferred_tool(block: BlockId) -> ToolType {
    match block {
        // Pickaxe blocks
        BlockId::Stone
        | BlockId::Cobblestone
        | BlockId::MossyCobblestone
        | BlockId::Obsidian
        | BlockId::Bricks
        | BlockId::StoneBricks
        | BlockId::CoalOre
        | BlockId::IronOre
        | BlockId::GoldOre
        | BlockId::DiamondOre
        | BlockId::CopperOre
        | BlockId::LapisOre
        | BlockId::EmeraldOre
        | BlockId::RedstoneOre
        | BlockId::Netherrack
        | BlockId::EndStone
        | BlockId::Furnace
        | BlockId::Glowstone
        | BlockId::Terracotta
        | BlockId::Ice
        | BlockId::PackedIce => ToolType::Pickaxe,

        // Axe blocks
        BlockId::OakLog
        | BlockId::BirchLog
        | BlockId::SpruceLog
        | BlockId::JungleLog
        | BlockId::DarkOakLog
        | BlockId::OakPlanks
        | BlockId::BirchPlanks
        | BlockId::SprucePlanks
        | BlockId::JunglePlanks
        | BlockId::DarkOakPlanks
        | BlockId::CraftingTable
        | BlockId::Chest
        | BlockId::Bookshelf
        | BlockId::NoteBlock
        | BlockId::Pumpkin
        | BlockId::Melon => ToolType::Axe,

        // Shovel blocks
        BlockId::Dirt
        | BlockId::GrassBlock
        | BlockId::Sand
        | BlockId::Gravel
        | BlockId::Clay
        | BlockId::Snow
        | BlockId::SnowBlock
        | BlockId::Mycelium
        | BlockId::Podzol
        | BlockId::SoulSand => ToolType::Shovel,

        _ => ToolType::None,
    }
}

// ---------------------------------------------------------------------------
// calculate_break_time
// ---------------------------------------------------------------------------

/// Calculate the time (in seconds) required to break a block.
///
/// Formula (Minecraft-inspired):
/// 1. `base = hardness * 1.5` when using the preferred tool, or
///    `hardness * 5.0` otherwise.
/// 2. `speed_multiplier` = `tool_tier.mining_speed()` when preferred, else 1.
/// 3. `efficiency_bonus` = `efficiency_level^2 + 1` (1 when level == 0).
/// 4. `haste_multiplier` = `1.0 + 0.2 * haste_level`.
/// 5. `time = base / (speed_multiplier * efficiency_bonus * haste_multiplier)`.
/// 6. Minimum return value is `0.05` seconds (one game tick).
pub fn calculate_break_time(
    hardness: f32,
    _tool_type: ToolType,
    tool_tier: ToolTier,
    is_preferred: bool,
    efficiency_level: u8,
    haste_level: u8,
) -> f32 {
    // Instant-break blocks (hardness 0 or less)
    if hardness <= 0.0 {
        return 0.05;
    }

    let base = if is_preferred {
        hardness * 1.5
    } else {
        hardness * 5.0
    };

    let speed_multiplier = if is_preferred {
        tool_tier.mining_speed()
    } else {
        1.0
    };

    let efficiency_bonus = if efficiency_level > 0 {
        (efficiency_level as f32).powi(2) + 1.0
    } else {
        1.0
    };

    let haste_multiplier = 1.0 + 0.2 * haste_level as f32;

    let time = base / (speed_multiplier * efficiency_bonus * haste_multiplier);

    time.max(0.05)
}

// ---------------------------------------------------------------------------
// BreakProgress
// ---------------------------------------------------------------------------

/// Tracks the progress of breaking a single block.
#[derive(Debug, Clone)]
pub struct BreakProgress {
    pub block_pos: BlockPos,
    pub progress: f32,
    pub total_time: f32,
}

impl BreakProgress {
    /// Create a new break-progress tracker.
    pub fn new(block_pos: BlockPos, total_time: f32) -> Self {
        Self {
            block_pos,
            progress: 0.0,
            total_time,
        }
    }

    /// Advance the progress by `dt` seconds.
    /// Returns `true` when the block is fully broken.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.progress += dt;
        self.progress >= self.total_time
    }

    /// Returns the current crack animation stage (0..=9).
    pub fn crack_stage(&self) -> u8 {
        if self.total_time <= 0.0 {
            return 9;
        }
        let ratio = (self.progress / self.total_time).clamp(0.0, 1.0);
        let stage = (ratio * 10.0).floor() as u8;
        stage.min(9)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- DurabilityComponent tests --

    #[test]
    fn new_durability_starts_at_max() {
        let d = DurabilityComponent::new(100);
        assert_eq!(d.current, 100);
        assert_eq!(d.max, 100);
        assert!(!d.is_broken());
    }

    #[test]
    fn is_broken_when_zero() {
        let d = DurabilityComponent {
            current: 0,
            max: 100,
        };
        assert!(d.is_broken());
    }

    #[test]
    fn remaining_percent_full() {
        let d = DurabilityComponent::new(200);
        assert!((d.remaining_percent() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn remaining_percent_half() {
        let d = DurabilityComponent {
            current: 50,
            max: 100,
        };
        assert!((d.remaining_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn remaining_percent_zero_max() {
        let d = DurabilityComponent::new(0);
        assert!((d.remaining_percent()).abs() < f32::EPSILON);
    }

    // -- use_tool tests (deterministic) --

    #[test]
    fn use_tool_no_unbreaking_always_consumes() {
        let mut d = DurabilityComponent::new(3);
        // roll=0.0 < chance=1.0 => always consumes
        let broke = use_tool_with_roll(&mut d, 0, 0.0);
        assert!(!broke);
        assert_eq!(d.current, 2);
    }

    #[test]
    fn use_tool_unbreaking_prevents_consumption() {
        let mut d = DurabilityComponent::new(3);
        // unbreaking 2 => chance = 1/3 ~ 0.333
        // roll=0.5 >= 0.333 => no consumption
        let broke = use_tool_with_roll(&mut d, 2, 0.5);
        assert!(!broke);
        assert_eq!(d.current, 3);
    }

    #[test]
    fn use_tool_unbreaking_allows_consumption() {
        let mut d = DurabilityComponent::new(3);
        // unbreaking 2 => chance = 1/3 ~ 0.333
        // roll=0.1 < 0.333 => consumes
        let broke = use_tool_with_roll(&mut d, 2, 0.1);
        assert!(!broke);
        assert_eq!(d.current, 2);
    }

    #[test]
    fn unbreaking_extends_tool_life() {
        // Without unbreaking: 5 uses at roll=0.0 always consume
        let mut d_no_ench = DurabilityComponent::new(5);
        for _ in 0..5 {
            use_tool_with_roll(&mut d_no_ench, 0, 0.0);
        }
        assert!(d_no_ench.is_broken());

        // With unbreaking 3: 5 uses at roll=0.5 (>0.25 threshold) never consume
        let mut d_ench = DurabilityComponent::new(5);
        for _ in 0..5 {
            use_tool_with_roll(&mut d_ench, 3, 0.5);
        }
        assert!(!d_ench.is_broken());
        assert_eq!(d_ench.current, 5);
    }

    #[test]
    fn use_tool_breaks_on_last_point() {
        let mut d = DurabilityComponent::new(1);
        let broke = use_tool_with_roll(&mut d, 0, 0.0);
        assert!(broke);
        assert!(d.is_broken());
    }

    #[test]
    fn use_tool_already_broken_returns_true() {
        let mut d = DurabilityComponent {
            current: 0,
            max: 10,
        };
        let broke = use_tool_with_roll(&mut d, 0, 0.0);
        assert!(broke);
    }

    // -- preferred_tool tests --

    #[test]
    fn pickaxe_preferred_for_stone_and_ores() {
        assert_eq!(preferred_tool(BlockId::Stone), ToolType::Pickaxe);
        assert_eq!(preferred_tool(BlockId::Cobblestone), ToolType::Pickaxe);
        assert_eq!(preferred_tool(BlockId::Obsidian), ToolType::Pickaxe);
        assert_eq!(preferred_tool(BlockId::CoalOre), ToolType::Pickaxe);
        assert_eq!(preferred_tool(BlockId::IronOre), ToolType::Pickaxe);
        assert_eq!(preferred_tool(BlockId::DiamondOre), ToolType::Pickaxe);
        assert_eq!(preferred_tool(BlockId::Bricks), ToolType::Pickaxe);
        assert_eq!(preferred_tool(BlockId::StoneBricks), ToolType::Pickaxe);
    }

    #[test]
    fn axe_preferred_for_wood() {
        assert_eq!(preferred_tool(BlockId::OakLog), ToolType::Axe);
        assert_eq!(preferred_tool(BlockId::BirchLog), ToolType::Axe);
        assert_eq!(preferred_tool(BlockId::OakPlanks), ToolType::Axe);
        assert_eq!(preferred_tool(BlockId::CraftingTable), ToolType::Axe);
        assert_eq!(preferred_tool(BlockId::Chest), ToolType::Axe);
    }

    #[test]
    fn shovel_preferred_for_dirt_sand_gravel() {
        assert_eq!(preferred_tool(BlockId::Dirt), ToolType::Shovel);
        assert_eq!(preferred_tool(BlockId::Sand), ToolType::Shovel);
        assert_eq!(preferred_tool(BlockId::Gravel), ToolType::Shovel);
        assert_eq!(preferred_tool(BlockId::Clay), ToolType::Shovel);
        assert_eq!(preferred_tool(BlockId::Snow), ToolType::Shovel);
    }

    #[test]
    fn none_for_unspecified_blocks() {
        assert_eq!(preferred_tool(BlockId::Air), ToolType::None);
        assert_eq!(preferred_tool(BlockId::Glass), ToolType::None);
        assert_eq!(preferred_tool(BlockId::Torch), ToolType::None);
    }

    // -- calculate_break_time tests --

    #[test]
    fn diamond_pickaxe_breaks_stone_faster_than_wood() {
        let hardness = BlockId::Stone.properties().hardness; // 1.5
        let diamond_time =
            calculate_break_time(hardness, ToolType::Pickaxe, ToolTier::Diamond, true, 0, 0);
        let wood_time =
            calculate_break_time(hardness, ToolType::Pickaxe, ToolTier::Wood, true, 0, 0);
        assert!(
            diamond_time < wood_time,
            "diamond ({diamond_time}) should be faster than wood ({wood_time})"
        );
    }

    #[test]
    fn wrong_tool_is_slow() {
        let hardness = BlockId::Stone.properties().hardness;
        // Right tool (pickaxe, diamond)
        let right =
            calculate_break_time(hardness, ToolType::Pickaxe, ToolTier::Diamond, true, 0, 0);
        // Wrong tool
        let wrong =
            calculate_break_time(hardness, ToolType::Shovel, ToolTier::Diamond, false, 0, 0);
        assert!(
            wrong > right,
            "wrong tool ({wrong}) should be slower than right tool ({right})"
        );
    }

    #[test]
    fn efficiency_speeds_up_breaking() {
        let hardness = BlockId::Stone.properties().hardness;
        let no_eff =
            calculate_break_time(hardness, ToolType::Pickaxe, ToolTier::Iron, true, 0, 0);
        let eff_3 =
            calculate_break_time(hardness, ToolType::Pickaxe, ToolTier::Iron, true, 3, 0);
        assert!(
            eff_3 < no_eff,
            "efficiency 3 ({eff_3}) should be faster than none ({no_eff})"
        );
    }

    #[test]
    fn haste_speeds_up_breaking() {
        let hardness = BlockId::Stone.properties().hardness;
        let no_haste =
            calculate_break_time(hardness, ToolType::Pickaxe, ToolTier::Iron, true, 0, 0);
        let haste_2 =
            calculate_break_time(hardness, ToolType::Pickaxe, ToolTier::Iron, true, 0, 2);
        assert!(
            haste_2 < no_haste,
            "haste 2 ({haste_2}) should be faster than none ({no_haste})"
        );
    }

    #[test]
    fn minimum_break_time_is_one_tick() {
        // hardness 0.0 (instant-break) should still return 0.05
        let time = calculate_break_time(0.0, ToolType::None, ToolTier::None, false, 0, 0);
        assert!((time - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn break_time_values_are_reasonable() {
        // Stone: hardness 1.5, diamond pickaxe => base = 1.5*1.5 = 2.25, speed = 8.0
        // time = 2.25/8.0 = 0.28125
        let time = calculate_break_time(1.5, ToolType::Pickaxe, ToolTier::Diamond, true, 0, 0);
        assert!((time - 0.28125).abs() < 0.001);
    }

    // -- BreakProgress tests --

    #[test]
    fn break_progress_advances_correctly() {
        let pos = BlockPos::new(0, 64, 0);
        let mut bp = BreakProgress::new(pos, 2.0);
        assert!(!bp.tick(0.5));
        assert!((bp.progress - 0.5).abs() < f32::EPSILON);
        assert!(!bp.tick(0.5));
        assert!((bp.progress - 1.0).abs() < f32::EPSILON);
        assert!(!bp.tick(0.5));
        assert!(bp.tick(0.5)); // 2.0 total
    }

    #[test]
    fn break_progress_finishes_exactly() {
        let pos = BlockPos::new(1, 2, 3);
        let mut bp = BreakProgress::new(pos, 1.0);
        assert!(bp.tick(1.0));
    }

    #[test]
    fn crack_stages_map_to_0_through_9() {
        let pos = BlockPos::new(0, 0, 0);
        let mut bp = BreakProgress::new(pos, 10.0);

        // At 0% progress -> stage 0
        assert_eq!(bp.crack_stage(), 0);

        // Advance by 1.0 each time and check stages
        for expected_stage in 1..=9u8 {
            bp.progress += 1.0;
            assert_eq!(
                bp.crack_stage(),
                expected_stage,
                "at progress {}, expected stage {}",
                bp.progress,
                expected_stage,
            );
        }

        // Even past 100%, stage caps at 9
        bp.progress = 15.0;
        assert_eq!(bp.crack_stage(), 9);
    }

    #[test]
    fn crack_stage_zero_total_time() {
        let pos = BlockPos::new(0, 0, 0);
        let bp = BreakProgress::new(pos, 0.0);
        assert_eq!(bp.crack_stage(), 9);
    }
}
