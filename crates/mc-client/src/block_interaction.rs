use mc_core::pos::BlockPos;
use mc_entity::tool_speed::{ToolType, ToolTier, mining_speed, break_time};
use mc_entity::tool_use::BreakProgress;
use mc_render::block_break::BlockBreakOverlay;

// ---------------------------------------------------------------------------
// BreakResult
// ---------------------------------------------------------------------------

/// Outcome of a single `BlockBreaker::tick` call.
#[derive(Debug, Clone, PartialEq)]
pub enum BreakResult {
    /// The block is still being broken; payload is the crack stage (0..=9).
    Breaking(u8),
    /// The block has been fully broken; payload is the world-space position.
    Broken((i32, i32, i32)),
    /// Breaking was cancelled (player looked away or called `cancel`).
    Cancelled,
    /// No block is currently being broken.
    NotBreaking,
}

// ---------------------------------------------------------------------------
// BlockBreaker
// ---------------------------------------------------------------------------

/// Client-side bridge that coordinates block-breaking progress with the crack
/// overlay rendered by `mc_render`.
pub struct BlockBreaker {
    pub overlay: BlockBreakOverlay,
    pub current_block: Option<(i32, i32, i32)>,
    progress: Option<BreakProgress>,
}

impl BlockBreaker {
    pub fn new() -> Self {
        Self {
            overlay: BlockBreakOverlay::new(),
            current_block: None,
            progress: None,
        }
    }

    /// Begin breaking a block at `block_pos`.
    ///
    /// `tool_type` and `tool_tier` are encoded as `u8` so the caller does not
    /// need to depend on `mc_core::item` directly. They are converted via
    /// [`calculate_break_speed`].
    pub fn start_breaking(
        &mut self,
        block_pos: (i32, i32, i32),
        block_hardness: f32,
        tool_type: u8,
        tool_tier: u8,
    ) {
        let total_time = actual_break_time(block_hardness, tool_type, tool_tier);
        let bp = BlockPos::new(block_pos.0, block_pos.1, block_pos.2);
        self.progress = Some(BreakProgress::new(bp, total_time));
        self.current_block = Some(block_pos);
        self.overlay.set(block_pos, 0.0);
    }

    /// Advance the breaking animation by `dt` seconds.
    ///
    /// If `still_aiming_at_same_block` is `false` the break is cancelled and
    /// the overlay is cleared.
    pub fn tick(&mut self, dt: f32, still_aiming_at_same_block: bool) -> BreakResult {
        let progress = match self.progress.as_mut() {
            Some(p) => p,
            None => return BreakResult::NotBreaking,
        };

        if !still_aiming_at_same_block {
            self.cancel();
            return BreakResult::Cancelled;
        }

        let done = progress.tick(dt);
        let stage = progress.crack_stage();

        if let Some(pos) = self.current_block {
            self.overlay.set(pos, stage as f32 / 10.0);
        }

        if done {
            let pos = self.current_block.unwrap_or((0, 0, 0));
            self.progress = None;
            self.current_block = None;
            self.overlay.clear();
            BreakResult::Broken(pos)
        } else {
            BreakResult::Breaking(stage)
        }
    }

    /// Cancel any in-progress block breaking.
    pub fn cancel(&mut self) {
        self.progress = None;
        self.current_block = None;
        self.overlay.clear();
    }
}

impl Default for BlockBreaker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// calculate_break_speed
// ---------------------------------------------------------------------------

/// Wrapper around [`mc_entity::tool_speed::mining_speed`] that accepts raw `u8`
/// encodings for tool type and tier.
///
/// Encoding for `tool_type`:
///   0 = None, 1 = Pickaxe, 2 = Axe, 3 = Shovel, 4 = Sword, 5 = Hoe
///
/// Encoding for `tool_tier`:
///   0 = Wood, 1 = Stone, 2 = Iron, 3 = Gold, 4 = Diamond, 5 = Netherite
pub fn calculate_break_speed(hardness: f32, tool_type: u8, tool_tier: u8) -> f32 {
    let tool = match tool_type {
        0 => ToolType::None,
        1 => ToolType::Pickaxe,
        2 => ToolType::Axe,
        3 => ToolType::Shovel,
        4 => ToolType::Sword,
        5 => ToolType::Hoe,
        _ => ToolType::None,
    };

    let tier = match tool_tier {
        0 => ToolTier::Wood,
        1 => ToolTier::Stone,
        2 => ToolTier::Iron,
        3 => ToolTier::Gold,
        4 => ToolTier::Diamond,
        5 => ToolTier::Netherite,
        _ => ToolTier::Wood,
    };

    mining_speed(hardness, tool, tier, 0)
}

// ---------------------------------------------------------------------------
// actual_break_time
// ---------------------------------------------------------------------------

/// Compute the actual break time (in seconds) for a block given its hardness
/// and the player's current tool.
///
/// Uses the same `tool_type`/`tool_tier` encoding as [`calculate_break_speed`].
pub fn actual_break_time(hardness: f32, tool_type: u8, tool_tier: u8) -> f32 {
    let speed = calculate_break_speed(hardness, tool_type, tool_tier);
    break_time(hardness, speed)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_block_breaker_is_idle() {
        let bb = BlockBreaker::new();
        assert!(bb.current_block.is_none());
        assert!(bb.overlay.stage.is_none());
    }

    #[test]
    fn start_breaking_sets_current_block() {
        let mut bb = BlockBreaker::new();
        bb.start_breaking((10, 64, 20), 1.5, 1, 2);
        assert_eq!(bb.current_block, Some((10, 64, 20)));
        assert_eq!(bb.overlay.stage, Some(0));
        assert_eq!(bb.overlay.block_pos, Some((10, 64, 20)));
    }

    #[test]
    fn tick_advances_breaking() {
        let mut bb = BlockBreaker::new();
        bb.start_breaking((0, 0, 0), 1.5, 1, 2);

        let result = bb.tick(0.01, true);
        assert!(matches!(result, BreakResult::Breaking(_)));
    }

    #[test]
    fn tick_returns_not_breaking_when_idle() {
        let mut bb = BlockBreaker::new();
        let result = bb.tick(0.1, true);
        assert_eq!(result, BreakResult::NotBreaking);
    }

    #[test]
    fn tick_cancels_when_not_aiming() {
        let mut bb = BlockBreaker::new();
        bb.start_breaking((5, 5, 5), 1.0, 1, 2);

        let result = bb.tick(0.01, false);
        assert_eq!(result, BreakResult::Cancelled);
        assert!(bb.current_block.is_none());
        assert!(bb.overlay.stage.is_none());
    }

    #[test]
    fn tick_returns_broken_when_complete() {
        let mut bb = BlockBreaker::new();
        // Torch has hardness 0.0 -> instant break (0.05s)
        bb.start_breaking((1, 2, 3), 0.0, 255, 255);

        let result = bb.tick(0.05, true);
        assert_eq!(result, BreakResult::Broken((1, 2, 3)));
        assert!(bb.current_block.is_none());
    }

    #[test]
    fn cancel_clears_state() {
        let mut bb = BlockBreaker::new();
        bb.start_breaking((7, 8, 9), 3.0, 1, 4);
        bb.cancel();
        assert!(bb.current_block.is_none());
        assert!(bb.overlay.stage.is_none());
    }

    #[test]
    fn calculate_break_speed_wraps_tool_speed() {
        // Stone hardness 1.5, iron pickaxe (type=1, tier=2)
        // mining_speed ignores hardness; Iron tier speed_multiplier = 6.0
        let speed = calculate_break_speed(1.5, 1, 2);
        assert!((speed - 6.0).abs() < 0.001);
    }

    #[test]
    fn calculate_break_speed_no_tool() {
        // No tool (type=0 maps to None) => base speed 1.0
        let speed = calculate_break_speed(1.5, 0, 0);
        assert!((speed - 1.0).abs() < 0.001);
    }

    #[test]
    fn actual_break_time_iron_pickaxe_on_stone() {
        // Stone hardness 1.5, iron pickaxe (type=1, tier=2)
        // speed = 6.0, break_time = 1.5 * 1.5 / 6.0 = 0.375
        let time = actual_break_time(1.5, 1, 2);
        assert!((time - 0.375).abs() < 0.001);
    }

    #[test]
    fn actual_break_time_bare_hand() {
        // Stone hardness 1.5, bare hand (type=0)
        // speed = 1.0, break_time = 1.5 * 1.5 / 1.0 = 2.25
        let time = actual_break_time(1.5, 0, 0);
        assert!((time - 2.25).abs() < 0.001);
    }

    #[test]
    fn default_trait_works() {
        let bb = BlockBreaker::default();
        assert!(bb.current_block.is_none());
    }
}
