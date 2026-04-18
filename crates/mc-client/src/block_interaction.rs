use mc_core::item::{ToolTier, ToolType};
use mc_core::pos::BlockPos;
use mc_entity::tool_use::{self, BreakProgress};
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
        let total_time = calculate_break_speed(block_hardness, tool_type, tool_tier);
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

/// Wrapper around [`mc_entity::tool_use::calculate_break_time`] that accepts
/// raw `u8` encodings for tool type and tier.
///
/// Encoding for `tool_type`:
///   0 = Pickaxe, 1 = Axe, 2 = Shovel, 3 = Sword, 4 = Hoe, 5 = Shears, _ = None
///
/// Encoding for `tool_tier`:
///   0 = Wood, 1 = Stone, 2 = Iron, 3 = Gold, 4 = Diamond, _ = None
pub fn calculate_break_speed(hardness: f32, tool_type: u8, tool_tier: u8) -> f32 {
    let tt = match tool_type {
        0 => ToolType::Pickaxe,
        1 => ToolType::Axe,
        2 => ToolType::Shovel,
        3 => ToolType::Sword,
        4 => ToolType::Hoe,
        5 => ToolType::Shears,
        _ => ToolType::None,
    };

    let tier = match tool_tier {
        0 => ToolTier::Wood,
        1 => ToolTier::Stone,
        2 => ToolTier::Iron,
        3 => ToolTier::Gold,
        4 => ToolTier::Diamond,
        _ => ToolTier::None,
    };

    // For the bridge we treat any non-None tool type as "preferred" for
    // simplicity. A more accurate version would check preferred_tool, but
    // the caller already knows whether the tool matches.
    let is_preferred = !matches!(tt, ToolType::None);

    // No efficiency or haste through this bridge (those come from
    // enchantments / effects which the caller can layer on top).
    tool_use::calculate_break_time(hardness, tt, tier, is_preferred, 0, 0)
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
        bb.start_breaking((10, 64, 20), 1.5, 0, 2);
        assert_eq!(bb.current_block, Some((10, 64, 20)));
        assert_eq!(bb.overlay.stage, Some(0));
        assert_eq!(bb.overlay.block_pos, Some((10, 64, 20)));
    }

    #[test]
    fn tick_advances_breaking() {
        let mut bb = BlockBreaker::new();
        bb.start_breaking((0, 0, 0), 1.5, 0, 2);

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
        bb.start_breaking((5, 5, 5), 1.0, 0, 2);

        let result = bb.tick(0.01, false);
        assert_eq!(result, BreakResult::Cancelled);
        assert!(bb.current_block.is_none());
        assert!(bb.overlay.stage.is_none());
    }

    #[test]
    fn tick_returns_broken_when_complete() {
        let mut bb = BlockBreaker::new();
        // Torch has hardness 0.0 -> instant break (0.05s)
        bb.start_breaking((1, 2, 3), 0.0, 6, 6);

        let result = bb.tick(0.05, true);
        assert_eq!(result, BreakResult::Broken((1, 2, 3)));
        assert!(bb.current_block.is_none());
    }

    #[test]
    fn cancel_clears_state() {
        let mut bb = BlockBreaker::new();
        bb.start_breaking((7, 8, 9), 3.0, 0, 4);
        bb.cancel();
        assert!(bb.current_block.is_none());
        assert!(bb.overlay.stage.is_none());
    }

    #[test]
    fn calculate_break_speed_wraps_tool_use() {
        // Stone hardness 1.5, iron pickaxe (type=0, tier=2)
        let time = calculate_break_speed(1.5, 0, 2);
        // Preferred: base = 1.5*1.5 = 2.25, speed = 6.0 => 2.25/6.0 = 0.375
        assert!((time - 0.375).abs() < 0.001);
    }

    #[test]
    fn calculate_break_speed_no_tool() {
        // Stone hardness 1.5, no tool (type=255, tier=255)
        let time = calculate_break_speed(1.5, 255, 255);
        // Not preferred: base = 1.5*5.0 = 7.5, speed = 1.0 => 7.5
        assert!((time - 7.5).abs() < 0.001);
    }

    #[test]
    fn default_trait_works() {
        let bb = BlockBreaker::default();
        assert!(bb.current_block.is_none());
    }
}
