//! Footstep sound system.
//!
//! Maps block types to footstep sounds, controls playback intervals,
//! volume levels, and pitch variation for player movement audio.

use mc_core::BlockId;

/// Category of footstep sound determined by the block being walked on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FootstepSound {
    Stone,
    Wood,
    Grass,
    Sand,
    Gravel,
    Snow,
    Wool,
    Glass,
    Metal,
    Wet,
    Powder,
    Honey,
}

/// Returns the footstep sound category for the given block ID.
///
/// Block-to-sound mapping follows vanilla Minecraft material groupings.
/// Blocks without a specific mapping default to [`FootstepSound::Stone`].
pub fn footstep_for_block(block_id: u16) -> FootstepSound {
    let Some(block) = BlockId::from_raw(block_id) else {
        return FootstepSound::Stone;
    };

    match block {
        // Stone-family blocks
        BlockId::Stone
        | BlockId::Cobblestone
        | BlockId::MossyCobblestone
        | BlockId::StoneBricks
        | BlockId::Bricks
        | BlockId::Obsidian
        | BlockId::Netherrack
        | BlockId::EndStone
        | BlockId::Bedrock
        | BlockId::Terracotta => FootstepSound::Stone,

        // Wood-family blocks
        BlockId::OakPlanks
        | BlockId::BirchPlanks
        | BlockId::SprucePlanks
        | BlockId::JunglePlanks
        | BlockId::DarkOakPlanks
        | BlockId::OakLog
        | BlockId::BirchLog
        | BlockId::SpruceLog
        | BlockId::JungleLog
        | BlockId::DarkOakLog
        | BlockId::Bookshelf
        | BlockId::CraftingTable
        | BlockId::Chest
        | BlockId::NoteBlock => FootstepSound::Wood,

        // Grass / dirt
        BlockId::GrassBlock
        | BlockId::Dirt
        | BlockId::Farmland
        | BlockId::Mycelium
        | BlockId::Podzol => FootstepSound::Grass,

        // Sand
        BlockId::Sand => FootstepSound::Sand,

        // Gravel
        BlockId::Gravel => FootstepSound::Gravel,

        // Snow
        BlockId::Snow | BlockId::SnowBlock => FootstepSound::Snow,

        // Wool
        BlockId::RedWool
        | BlockId::BlueWool
        | BlockId::GreenWool
        | BlockId::YellowWool
        | BlockId::WhiteWool
        | BlockId::BlackWool => FootstepSound::Wool,

        // Glass
        BlockId::Glass => FootstepSound::Glass,

        // Metal (iron ore is the closest iron block available)
        BlockId::IronOre => FootstepSound::Metal,

        // Wet
        BlockId::Clay => FootstepSound::Wet,

        // Everything else defaults to Stone
        _ => FootstepSound::Stone,
    }
}

/// Returns the time interval (in seconds) between consecutive footstep sounds.
///
/// Walking (speed < 5.0) plays at 0.50 s intervals; sprinting plays at 0.35 s.
pub fn footstep_interval(speed: f32, is_sprinting: bool) -> f32 {
    if is_sprinting {
        0.35
    } else if speed < 5.0 {
        0.50
    } else {
        0.35
    }
}

/// Returns the volume level for a footstep sound.
///
/// Sneaking = 0.15, walking (speed < 5.0) = 0.5, sprinting = 0.8.
pub fn footstep_volume(speed: f32, is_sneaking: bool) -> f32 {
    if is_sneaking {
        0.15
    } else if speed < 5.0 {
        0.5
    } else {
        0.8
    }
}

/// Returns a pitch value with pseudo-random variation applied.
///
/// The result varies by +/-20% from `base_pitch`, determined by `seed`.
/// Uses a simple hash to derive variation without requiring a full RNG.
pub fn footstep_pitch_variation(base_pitch: f32, seed: u64) -> f32 {
    // Simple hash-based variation: map seed into [-0.2, +0.2] range
    let hash = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let fraction = (hash % 10_001) as f32 / 10_000.0; // 0.0 ..= 1.0
    let variation = (fraction * 2.0 - 1.0) * 0.2; // -0.2 ..= +0.2
    base_pitch * (1.0 + variation)
}

impl FootstepSound {
    /// Returns the base pitch multiplier for this footstep sound category.
    ///
    /// Most materials use 1.0. Sand is lower (0.8) and glass is higher (1.2).
    pub fn base_pitch(&self) -> f32 {
        match self {
            FootstepSound::Sand => 0.8,
            FootstepSound::Glass => 1.2,
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Block mapping tests ---

    #[test]
    fn stone_blocks_produce_stone_sound() {
        assert_eq!(footstep_for_block(BlockId::Stone as u16), FootstepSound::Stone);
        assert_eq!(footstep_for_block(BlockId::Cobblestone as u16), FootstepSound::Stone);
        assert_eq!(footstep_for_block(BlockId::Bricks as u16), FootstepSound::Stone);
        assert_eq!(footstep_for_block(BlockId::StoneBricks as u16), FootstepSound::Stone);
    }

    #[test]
    fn wood_blocks_produce_wood_sound() {
        assert_eq!(footstep_for_block(BlockId::OakPlanks as u16), FootstepSound::Wood);
        assert_eq!(footstep_for_block(BlockId::OakLog as u16), FootstepSound::Wood);
        assert_eq!(footstep_for_block(BlockId::BirchPlanks as u16), FootstepSound::Wood);
        assert_eq!(footstep_for_block(BlockId::DarkOakLog as u16), FootstepSound::Wood);
    }

    #[test]
    fn grass_and_dirt_produce_grass_sound() {
        assert_eq!(footstep_for_block(BlockId::GrassBlock as u16), FootstepSound::Grass);
        assert_eq!(footstep_for_block(BlockId::Dirt as u16), FootstepSound::Grass);
    }

    #[test]
    fn sand_produces_sand_sound() {
        assert_eq!(footstep_for_block(BlockId::Sand as u16), FootstepSound::Sand);
    }

    #[test]
    fn gravel_produces_gravel_sound() {
        assert_eq!(footstep_for_block(BlockId::Gravel as u16), FootstepSound::Gravel);
    }

    #[test]
    fn snow_produces_snow_sound() {
        assert_eq!(footstep_for_block(BlockId::Snow as u16), FootstepSound::Snow);
        assert_eq!(footstep_for_block(BlockId::SnowBlock as u16), FootstepSound::Snow);
    }

    #[test]
    fn wool_produces_wool_sound() {
        assert_eq!(footstep_for_block(BlockId::WhiteWool as u16), FootstepSound::Wool);
        assert_eq!(footstep_for_block(BlockId::RedWool as u16), FootstepSound::Wool);
    }

    #[test]
    fn glass_produces_glass_sound() {
        assert_eq!(footstep_for_block(BlockId::Glass as u16), FootstepSound::Glass);
    }

    #[test]
    fn iron_ore_produces_metal_sound() {
        assert_eq!(footstep_for_block(BlockId::IronOre as u16), FootstepSound::Metal);
    }

    #[test]
    fn unknown_block_id_defaults_to_stone() {
        assert_eq!(footstep_for_block(9999), FootstepSound::Stone);
    }

    // --- Interval tests ---

    #[test]
    fn walking_interval_is_half_second() {
        let interval = footstep_interval(4.0, false);
        assert!((interval - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn sprinting_interval_is_0_35() {
        let interval = footstep_interval(6.0, true);
        assert!((interval - 0.35).abs() < f32::EPSILON);
    }

    #[test]
    fn high_speed_non_sprint_uses_sprint_interval() {
        let interval = footstep_interval(6.0, false);
        assert!((interval - 0.35).abs() < f32::EPSILON);
    }

    // --- Volume tests ---

    #[test]
    fn sneaking_volume_is_quiet() {
        let vol = footstep_volume(2.0, true);
        assert!((vol - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn walking_volume_is_medium() {
        let vol = footstep_volume(4.0, false);
        assert!((vol - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn sprinting_volume_is_loud() {
        let vol = footstep_volume(6.0, false);
        assert!((vol - 0.8).abs() < f32::EPSILON);
    }

    // --- Pitch tests ---

    #[test]
    fn base_pitch_sand_is_low() {
        assert!((FootstepSound::Sand.base_pitch() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn base_pitch_glass_is_high() {
        assert!((FootstepSound::Glass.base_pitch() - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn base_pitch_default_is_one() {
        assert!((FootstepSound::Stone.base_pitch() - 1.0).abs() < f32::EPSILON);
        assert!((FootstepSound::Wood.base_pitch() - 1.0).abs() < f32::EPSILON);
        assert!((FootstepSound::Grass.base_pitch() - 1.0).abs() < f32::EPSILON);
        assert!((FootstepSound::Metal.base_pitch() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pitch_variation_stays_within_20_percent() {
        let base = 1.0;
        for seed in 0..1000_u64 {
            let pitch = footstep_pitch_variation(base, seed);
            assert!(
                pitch >= base * 0.8 && pitch <= base * 1.2,
                "seed {seed}: pitch {pitch} out of range [{}, {}]",
                base * 0.8,
                base * 1.2
            );
        }
    }

    #[test]
    fn pitch_variation_differs_across_seeds() {
        let p1 = footstep_pitch_variation(1.0, 42);
        let p2 = footstep_pitch_variation(1.0, 43);
        // Different seeds should (almost certainly) produce different pitches
        assert!(
            (p1 - p2).abs() > f32::EPSILON,
            "expected different pitches for different seeds, got {p1} and {p2}"
        );
    }

    #[test]
    fn pitch_variation_scales_with_base() {
        let base_low = 0.8;
        let base_high = 1.2;
        let seed = 100;
        let pitch_low = footstep_pitch_variation(base_low, seed);
        let pitch_high = footstep_pitch_variation(base_high, seed);
        assert!(pitch_low < pitch_high, "higher base should produce higher pitch");
    }
}
