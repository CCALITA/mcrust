//! Tool effectiveness and mining speed system.
//!
//! Provides the [`ToolType`] / [`ToolTier`] taxonomy plus pure helper functions
//! to compute Minecraft-style mining speed and break time from raw block ids.
//!
//! Unlike [`crate::tool_use`], which integrates with `mc_core::block::BlockId`
//! and the durability/break-progress workflow, this module operates on the
//! lower-level numeric ids exchanged across crate boundaries (network packets,
//! save format, etc.) and exposes the harvest-level/speed-multiplier scalars
//! used by the surrounding gameplay systems.

// ---------------------------------------------------------------------------
// ToolType
// ---------------------------------------------------------------------------

/// The category of a tool. Matches the tool slot used by Minecraft recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    None,
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
}

impl ToolType {
    /// Stable numeric id used for serialisation across crate boundaries.
    pub fn id(&self) -> u8 {
        match self {
            ToolType::None => 0,
            ToolType::Pickaxe => 1,
            ToolType::Axe => 2,
            ToolType::Shovel => 3,
            ToolType::Sword => 4,
            ToolType::Hoe => 5,
        }
    }
}

// ---------------------------------------------------------------------------
// ToolTier
// ---------------------------------------------------------------------------

/// Material tier of a tool. Determines mining speed and harvest level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolTier {
    Wood,
    Stone,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

impl ToolTier {
    /// Mining speed multiplier applied when mining a block this tool is
    /// effective against. Values match the Minecraft wiki:
    /// Wood 2, Stone 4, Iron 6, Gold 12, Diamond 8, Netherite 9.
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            ToolTier::Wood => 2.0,
            ToolTier::Stone => 4.0,
            ToolTier::Iron => 6.0,
            ToolTier::Gold => 12.0,
            ToolTier::Diamond => 8.0,
            ToolTier::Netherite => 9.0,
        }
    }

    /// Harvest tier required to drop the block. Higher = stronger.
    /// Wood 0, Stone 1, Iron 2, Gold 0, Diamond 3, Netherite 4.
    pub fn harvest_level(&self) -> u8 {
        match self {
            ToolTier::Wood => 0,
            ToolTier::Stone => 1,
            ToolTier::Iron => 2,
            ToolTier::Gold => 0,
            ToolTier::Diamond => 3,
            ToolTier::Netherite => 4,
        }
    }
}

// ---------------------------------------------------------------------------
// Block id classification
// ---------------------------------------------------------------------------
//
// The numeric block ids below mirror `mc_core::block::BlockId` discriminants,
// but we accept a raw `u16` so this module can be used at the network/save
// boundary without pulling in the `BlockId` enum. Unknown ids fall back to
// `ToolType::None`.

const STONE: u16 = 1;
const COBBLESTONE: u16 = 4;
const MOSSY_COBBLESTONE: u16 = 48;
const OBSIDIAN: u16 = 49;
const BRICKS: u16 = 45;
const STONE_BRICKS: u16 = 98;
const COAL_ORE: u16 = 16;
const IRON_ORE: u16 = 15;
const GOLD_ORE: u16 = 14;
const DIAMOND_ORE: u16 = 56;
const COPPER_ORE: u16 = 1500;
const LAPIS_ORE: u16 = 21;
const EMERALD_ORE: u16 = 129;
const REDSTONE_ORE: u16 = 73;
const NETHERRACK: u16 = 87;
const END_STONE: u16 = 121;
const FURNACE: u16 = 61;
const GLOWSTONE: u16 = 89;
const TERRACOTTA: u16 = 172;
const ICE: u16 = 79;
const PACKED_ICE: u16 = 174;
const SANDSTONE: u16 = 24;

const OAK_LOG: u16 = 17;
const BIRCH_LOG: u16 = 1701;
const SPRUCE_LOG: u16 = 1702;
const JUNGLE_LOG: u16 = 1703;
const DARK_OAK_LOG: u16 = 1704;
const OAK_PLANKS: u16 = 5;
const BIRCH_PLANKS: u16 = 1705;
const SPRUCE_PLANKS: u16 = 1706;
const JUNGLE_PLANKS: u16 = 1707;
const DARK_OAK_PLANKS: u16 = 1708;
const CRAFTING_TABLE: u16 = 58;
const CHEST: u16 = 54;
const BOOKSHELF: u16 = 47;
const NOTE_BLOCK: u16 = 25;
const PUMPKIN: u16 = 86;
const MELON: u16 = 103;

const DIRT: u16 = 3;
const GRASS_BLOCK: u16 = 2;
const SAND: u16 = 12;
const GRAVEL: u16 = 13;
const CLAY: u16 = 82;
const SNOW: u16 = 78;
const SNOW_BLOCK: u16 = 80;
const MYCELIUM: u16 = 110;
const PODZOL: u16 = 1801;
const SOUL_SAND: u16 = 88;

/// Returns the correct tool category for the given block id.
///
/// Stone, ores, bricks, and other masonry blocks → `Pickaxe`.
/// Logs, planks, and wooden furniture → `Axe`.
/// Dirt, sand, gravel, clay, snow → `Shovel`.
/// Anything else → `None` (no preferred tool).
pub fn correct_tool_for_block(block_id: u16) -> ToolType {
    match block_id {
        STONE | COBBLESTONE | MOSSY_COBBLESTONE | OBSIDIAN | BRICKS | STONE_BRICKS | COAL_ORE
        | IRON_ORE | GOLD_ORE | DIAMOND_ORE | COPPER_ORE | LAPIS_ORE | EMERALD_ORE
        | REDSTONE_ORE | NETHERRACK | END_STONE | FURNACE | GLOWSTONE | TERRACOTTA | ICE
        | PACKED_ICE | SANDSTONE => ToolType::Pickaxe,

        OAK_LOG | BIRCH_LOG | SPRUCE_LOG | JUNGLE_LOG | DARK_OAK_LOG | OAK_PLANKS
        | BIRCH_PLANKS | SPRUCE_PLANKS | JUNGLE_PLANKS | DARK_OAK_PLANKS | CRAFTING_TABLE
        | CHEST | BOOKSHELF | NOTE_BLOCK | PUMPKIN | MELON => ToolType::Axe,

        DIRT | GRASS_BLOCK | SAND | GRAVEL | CLAY | SNOW | SNOW_BLOCK | MYCELIUM | PODZOL
        | SOUL_SAND => ToolType::Shovel,

        _ => ToolType::None,
    }
}

// ---------------------------------------------------------------------------
// Mining math
// ---------------------------------------------------------------------------

/// Compute mining speed for a block.
///
/// `base_speed` is `tier.speed_multiplier()` if the supplied tool matches the
/// block's correct tool category, otherwise `1.0` (bare hand / wrong tool).
/// The Efficiency enchantment adds `level^2 + 1` on top when `level > 0`.
///
/// The result is `base_speed + efficiency_bonus`.
pub fn mining_speed(
    hardness: f32,
    tool: ToolType,
    tier: ToolTier,
    efficiency_level: u8,
) -> f32 {
    let _ = hardness; // hardness affects break_time, not raw speed
    let base_speed = if tool == ToolType::None {
        1.0
    } else {
        tier.speed_multiplier()
    };
    let efficiency_bonus = if efficiency_level > 0 {
        (efficiency_level as f32).powi(2) + 1.0
    } else {
        0.0
    };
    base_speed + efficiency_bonus
}

/// Convert `(hardness, speed)` into a break time in seconds, using the
/// canonical Minecraft formula `time = hardness * 1.5 / speed`.
///
/// Returns 0.0 for instant-break blocks (`hardness <= 0`) and never divides
/// by zero (a non-positive speed degenerates to `0.0`).
pub fn break_time(hardness: f32, speed: f32) -> f32 {
    if hardness <= 0.0 || speed <= 0.0 {
        return 0.0;
    }
    hardness * 1.5 / speed
}

/// Returns `true` when the supplied tool is strong enough to drop the block.
///
/// Blocks with no harvest-level requirement (stone, dirt, wood, etc.) can
/// always be broken and dropped with *any* tool or bare hand -- the correct
/// tool just makes it faster.
///
/// Blocks that *do* have a harvest-level gate (ores, obsidian) require the
/// correct tool category at a sufficient tier:
/// * Obsidian needs Diamond+ (level 3).
/// * Diamond/emerald/gold/redstone ore need Iron+ (level 2).
/// * Iron/lapis/copper ore need Stone+ (level 1).
pub fn can_harvest(block_id: u16, tool: ToolType, tier: ToolTier) -> bool {
    let required = required_harvest_level(block_id);

    // Blocks with no special harvest requirement always drop.
    if required == 0 {
        return true;
    }

    // For level-gated blocks the player must use the correct tool type at a
    // sufficient tier.
    let correct = correct_tool_for_block(block_id);
    if tool != correct {
        return false;
    }
    tier.harvest_level() >= required
}

/// Minimum harvest level required to drop the block.
fn required_harvest_level(block_id: u16) -> u8 {
    match block_id {
        OBSIDIAN => 3,
        DIAMOND_ORE | EMERALD_ORE | GOLD_ORE | REDSTONE_ORE => 2,
        IRON_ORE | LAPIS_ORE | COPPER_ORE => 1,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_type_ids_are_stable() {
        assert_eq!(ToolType::None.id(), 0);
        assert_eq!(ToolType::Pickaxe.id(), 1);
        assert_eq!(ToolType::Axe.id(), 2);
        assert_eq!(ToolType::Shovel.id(), 3);
        assert_eq!(ToolType::Sword.id(), 4);
        assert_eq!(ToolType::Hoe.id(), 5);
    }

    #[test]
    fn tier_speed_multipliers_match_spec() {
        assert_eq!(ToolTier::Wood.speed_multiplier(), 2.0);
        assert_eq!(ToolTier::Stone.speed_multiplier(), 4.0);
        assert_eq!(ToolTier::Iron.speed_multiplier(), 6.0);
        assert_eq!(ToolTier::Gold.speed_multiplier(), 12.0);
        assert_eq!(ToolTier::Diamond.speed_multiplier(), 8.0);
        assert_eq!(ToolTier::Netherite.speed_multiplier(), 9.0);
    }

    #[test]
    fn tier_harvest_levels_match_spec() {
        assert_eq!(ToolTier::Wood.harvest_level(), 0);
        assert_eq!(ToolTier::Stone.harvest_level(), 1);
        assert_eq!(ToolTier::Iron.harvest_level(), 2);
        assert_eq!(ToolTier::Gold.harvest_level(), 0);
        assert_eq!(ToolTier::Diamond.harvest_level(), 3);
        assert_eq!(ToolTier::Netherite.harvest_level(), 4);
    }

    #[test]
    fn pickaxe_matches_stone_and_ores() {
        assert_eq!(correct_tool_for_block(STONE), ToolType::Pickaxe);
        assert_eq!(correct_tool_for_block(COBBLESTONE), ToolType::Pickaxe);
        assert_eq!(correct_tool_for_block(IRON_ORE), ToolType::Pickaxe);
        assert_eq!(correct_tool_for_block(DIAMOND_ORE), ToolType::Pickaxe);
        assert_eq!(correct_tool_for_block(OBSIDIAN), ToolType::Pickaxe);
    }

    #[test]
    fn axe_matches_wood() {
        assert_eq!(correct_tool_for_block(OAK_LOG), ToolType::Axe);
        assert_eq!(correct_tool_for_block(OAK_PLANKS), ToolType::Axe);
        assert_eq!(correct_tool_for_block(CHEST), ToolType::Axe);
        assert_eq!(correct_tool_for_block(CRAFTING_TABLE), ToolType::Axe);
    }

    #[test]
    fn shovel_matches_dirt_sand_gravel() {
        assert_eq!(correct_tool_for_block(DIRT), ToolType::Shovel);
        assert_eq!(correct_tool_for_block(SAND), ToolType::Shovel);
        assert_eq!(correct_tool_for_block(GRAVEL), ToolType::Shovel);
        assert_eq!(correct_tool_for_block(CLAY), ToolType::Shovel);
        assert_eq!(correct_tool_for_block(SNOW), ToolType::Shovel);
    }

    #[test]
    fn unknown_block_has_no_correct_tool() {
        assert_eq!(correct_tool_for_block(0), ToolType::None);
        assert_eq!(correct_tool_for_block(9999), ToolType::None);
    }

    #[test]
    fn mining_speed_uses_tier_when_correct_tool() {
        // Iron pickaxe, no efficiency: speed = 6.0, bonus = 0
        let s = mining_speed(1.5, ToolType::Pickaxe, ToolTier::Iron, 0);
        assert!((s - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn mining_speed_bare_hand_is_one() {
        // ToolType::None ignores the tier and yields base speed 1.0.
        let s = mining_speed(1.5, ToolType::None, ToolTier::Diamond, 0);
        assert!((s - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn efficiency_bonus_math() {
        // Efficiency adds level^2 + 1 on top of base speed.
        // Iron pickaxe (6.0) + efficiency 3 -> 6 + (9 + 1) = 16
        let s = mining_speed(1.5, ToolType::Pickaxe, ToolTier::Iron, 3);
        assert!((s - 16.0).abs() < f32::EPSILON, "got {s}");

        // Diamond pickaxe (8.0) + efficiency 5 -> 8 + (25 + 1) = 34
        let s = mining_speed(1.5, ToolType::Pickaxe, ToolTier::Diamond, 5);
        assert!((s - 34.0).abs() < f32::EPSILON, "got {s}");
    }

    #[test]
    fn break_time_matches_minecraft_formula() {
        // Stone hardness 1.5, diamond pickaxe speed 8.0
        // time = 1.5 * 1.5 / 8.0 = 0.28125 s
        let speed = mining_speed(1.5, ToolType::Pickaxe, ToolTier::Diamond, 0);
        let t = break_time(1.5, speed);
        assert!((t - 0.28125).abs() < 1e-4, "got {t}");
    }

    #[test]
    fn break_time_obsidian_with_diamond_pickaxe() {
        // Obsidian hardness 50, diamond pickaxe speed 8.0
        // time = 50 * 1.5 / 8 = 9.375 s
        let speed = mining_speed(50.0, ToolType::Pickaxe, ToolTier::Diamond, 0);
        let t = break_time(50.0, speed);
        assert!((t - 9.375).abs() < 1e-3, "got {t}");
    }

    #[test]
    fn break_time_dirt_bare_hand_is_quick() {
        // Dirt hardness 0.5, bare hand speed 1.0
        // time = 0.5 * 1.5 / 1 = 0.75 s
        let speed = mining_speed(0.5, ToolType::None, ToolTier::Wood, 0);
        let t = break_time(0.5, speed);
        assert!((t - 0.75).abs() < 1e-4, "got {t}");
    }

    #[test]
    fn break_time_zero_hardness_is_instant() {
        let t = break_time(0.0, 5.0);
        assert!((t - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn break_time_zero_speed_is_safe() {
        let t = break_time(1.5, 0.0);
        assert!((t - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn can_harvest_dirt_with_anything() {
        // Dirt has no preferred tool -> always harvestable.
        assert!(can_harvest(DIRT, ToolType::None, ToolTier::Wood));
        assert!(can_harvest(DIRT, ToolType::Shovel, ToolTier::Wood));
    }

    #[test]
    fn obsidian_requires_diamond_or_better() {
        assert!(!can_harvest(OBSIDIAN, ToolType::Pickaxe, ToolTier::Wood));
        assert!(!can_harvest(OBSIDIAN, ToolType::Pickaxe, ToolTier::Stone));
        assert!(!can_harvest(OBSIDIAN, ToolType::Pickaxe, ToolTier::Iron));
        assert!(!can_harvest(OBSIDIAN, ToolType::Pickaxe, ToolTier::Gold));
        assert!(can_harvest(OBSIDIAN, ToolType::Pickaxe, ToolTier::Diamond));
        assert!(can_harvest(OBSIDIAN, ToolType::Pickaxe, ToolTier::Netherite));
    }

    #[test]
    fn iron_ore_requires_stone_or_better() {
        assert!(!can_harvest(IRON_ORE, ToolType::Pickaxe, ToolTier::Wood));
        // Gold tier has harvest_level 0 — too weak even though the material is fancy.
        assert!(!can_harvest(IRON_ORE, ToolType::Pickaxe, ToolTier::Gold));
        assert!(can_harvest(IRON_ORE, ToolType::Pickaxe, ToolTier::Stone));
        assert!(can_harvest(IRON_ORE, ToolType::Pickaxe, ToolTier::Iron));
        assert!(can_harvest(IRON_ORE, ToolType::Pickaxe, ToolTier::Diamond));
    }

    #[test]
    fn diamond_ore_requires_iron_or_better() {
        assert!(!can_harvest(DIAMOND_ORE, ToolType::Pickaxe, ToolTier::Wood));
        assert!(!can_harvest(DIAMOND_ORE, ToolType::Pickaxe, ToolTier::Stone));
        assert!(can_harvest(DIAMOND_ORE, ToolType::Pickaxe, ToolTier::Iron));
        assert!(can_harvest(DIAMOND_ORE, ToolType::Pickaxe, ToolTier::Diamond));
    }

    #[test]
    fn wrong_tool_cannot_harvest() {
        // Iron ore needs a pickaxe; a shovel won't drop it.
        assert!(!can_harvest(IRON_ORE, ToolType::Shovel, ToolTier::Diamond));
        // ...with the right tool at sufficient tier, it drops.
        assert!(can_harvest(IRON_ORE, ToolType::Pickaxe, ToolTier::Stone));
    }

    #[test]
    fn no_harvest_requirement_blocks_always_drop() {
        // Stone has harvest level 0 -- any tool or bare hand can harvest it.
        assert!(can_harvest(STONE, ToolType::Shovel, ToolTier::Wood));
        assert!(can_harvest(STONE, ToolType::None, ToolTier::Wood));
        assert!(can_harvest(STONE, ToolType::Pickaxe, ToolTier::Wood));
    }
}
