use mc_core::block::BlockId;

/// Actions a piston can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PistonAction {
    Extend,
    Retract,
}

/// Returns `true` if the given block can be pushed by a piston.
///
/// Most blocks are pushable. Obsidian, bedrock, and a few special blocks
/// cannot be moved.
pub fn piston_can_push(block: BlockId) -> bool {
    !matches!(block, BlockId::Obsidian | BlockId::Bedrock)
}

/// Maximum number of blocks a piston can push in a line.
pub fn piston_push_limit() -> usize {
    12
}

/// Number of game ticks between hopper transfer operations.
pub fn hopper_tick_rate() -> u32 {
    8
}

/// Convert a note block click count (0..=24) to a pitch frequency.
///
/// Minecraft note blocks produce 25 pitches from F#3 to F#5, each a
/// semitone apart. The formula is `2^((clicks - 12) / 12)` which yields
/// a multiplier relative to the base pitch (F#4 at clicks=12).
pub fn noteblock_pitch(clicks: u8) -> f32 {
    let clamped = clicks.min(24);
    2.0_f32.powf((clamped as f32 - 12.0) / 12.0)
}

/// Returns the block id for a redstone lamp given a powered state.
///
/// Currently `RedstoneLamp` is a single variant; powered/unpowered visual
/// state is tracked externally (e.g., via block-state metadata). This
/// function exists so callers have a uniform API that can be extended
/// later without breakage.
pub fn lamp_state(powered: bool) -> BlockId {
    let _ = powered;
    BlockId::RedstoneLamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piston_can_push_normal_blocks() {
        assert!(piston_can_push(BlockId::Stone));
        assert!(piston_can_push(BlockId::Dirt));
        assert!(piston_can_push(BlockId::Sand));
        assert!(piston_can_push(BlockId::GrassBlock));
        assert!(piston_can_push(BlockId::RedstoneDust));
        assert!(piston_can_push(BlockId::NoteBlock));
    }

    #[test]
    fn piston_cannot_push_immovable_blocks() {
        assert!(!piston_can_push(BlockId::Obsidian));
        assert!(!piston_can_push(BlockId::Bedrock));
    }

    #[test]
    fn piston_push_limit_is_twelve() {
        assert_eq!(piston_push_limit(), 12);
    }

    #[test]
    fn hopper_tick_rate_is_eight() {
        assert_eq!(hopper_tick_rate(), 8);
    }

    #[test]
    fn noteblock_pitch_at_zero_clicks() {
        let pitch = noteblock_pitch(0);
        // 2^(-12/12) = 2^(-1) = 0.5
        assert!((pitch - 0.5).abs() < 1e-6);
    }

    #[test]
    fn noteblock_pitch_at_twelve_clicks_is_unity() {
        let pitch = noteblock_pitch(12);
        // 2^(0/12) = 1.0
        assert!((pitch - 1.0).abs() < 1e-6);
    }

    #[test]
    fn noteblock_pitch_at_twenty_four_clicks() {
        let pitch = noteblock_pitch(24);
        // 2^(12/12) = 2.0
        assert!((pitch - 2.0).abs() < 1e-6);
    }

    #[test]
    fn noteblock_pitch_clamps_above_24() {
        // Values above 24 should clamp to 24
        let pitch_24 = noteblock_pitch(24);
        let pitch_30 = noteblock_pitch(30);
        assert!((pitch_24 - pitch_30).abs() < 1e-6);
    }

    #[test]
    fn noteblock_produces_25_distinct_pitches() {
        let pitches: Vec<f32> = (0..=24).map(noteblock_pitch).collect();
        // Each successive pitch should be higher
        for window in pitches.windows(2) {
            assert!(window[1] > window[0], "pitches should be monotonically increasing");
        }
    }

    #[test]
    fn lamp_state_returns_redstone_lamp() {
        assert_eq!(lamp_state(true), BlockId::RedstoneLamp);
        assert_eq!(lamp_state(false), BlockId::RedstoneLamp);
    }

    #[test]
    fn piston_action_variants_are_distinct() {
        assert_ne!(PistonAction::Extend, PistonAction::Retract);
    }
}
