//! Suspicious sand and gravel blocks that can be brushed to reveal items.

/// The type of suspicious block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspiciousType {
    Sand,
    Gravel,
}

/// A suspicious block that may contain a hidden item.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspiciousBlock {
    pub block_type: SuspiciousType,
    pub brush_progress: f32,
    pub item: Option<u16>,
}

impl SuspiciousBlock {
    /// Creates a new suspicious block with the given type and optional hidden item.
    pub fn new(block_type: SuspiciousType, item: Option<u16>) -> Self {
        Self {
            block_type,
            brush_progress: 0.0,
            item,
        }
    }
}

/// The result of a brush tick on a suspicious block.
#[derive(Debug, Clone, PartialEq)]
pub enum BrushResult {
    /// Brushing is still in progress with the current progress value.
    InProgress(f32),
    /// Brushing is complete, yielding an item ID.
    Complete(u16),
    /// Brushing is complete but no item was found.
    Empty,
    /// The block collapsed because brushing stopped.
    Collapsed,
}

/// The total duration in seconds required to fully brush a suspicious block.
pub fn brush_duration() -> f32 {
    4.0
}

/// Advances the brush progress on a suspicious block by `dt` seconds.
///
/// Returns the result of this brush tick.
pub fn brush_tick(block: &mut SuspiciousBlock, dt: f32) -> BrushResult {
    block.brush_progress += dt;
    if block.brush_progress >= brush_duration() {
        block.brush_progress = brush_duration();
        match block.item.take() {
            Some(item_id) => BrushResult::Complete(item_id),
            None => BrushResult::Empty,
        }
    } else {
        BrushResult::InProgress(block.brush_progress)
    }
}

/// Collapses a suspicious block if it is not being brushed.
///
/// Returns `true` if the block had progress and collapsed, `false` otherwise.
pub fn collapse_without_brush(block: &mut SuspiciousBlock) -> bool {
    if block.brush_progress > 0.0 {
        block.brush_progress = 0.0;
        block.item = None;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_block_with_zero_progress() {
        let block = SuspiciousBlock::new(SuspiciousType::Sand, Some(42));
        assert_eq!(block.block_type, SuspiciousType::Sand);
        assert_eq!(block.brush_progress, 0.0);
        assert_eq!(block.item, Some(42));
    }

    #[test]
    fn new_creates_block_without_item() {
        let block = SuspiciousBlock::new(SuspiciousType::Gravel, None);
        assert_eq!(block.block_type, SuspiciousType::Gravel);
        assert_eq!(block.item, None);
    }

    #[test]
    fn brush_duration_is_four_seconds() {
        assert_eq!(brush_duration(), 4.0);
    }

    #[test]
    fn brush_tick_in_progress() {
        let mut block = SuspiciousBlock::new(SuspiciousType::Sand, Some(10));
        let result = brush_tick(&mut block, 1.0);
        assert_eq!(result, BrushResult::InProgress(1.0));
        assert_eq!(block.brush_progress, 1.0);
    }

    #[test]
    fn brush_tick_complete_with_item() {
        let mut block = SuspiciousBlock::new(SuspiciousType::Sand, Some(99));
        let result = brush_tick(&mut block, 4.0);
        assert_eq!(result, BrushResult::Complete(99));
        assert_eq!(block.item, None);
    }

    #[test]
    fn brush_tick_complete_without_item() {
        let mut block = SuspiciousBlock::new(SuspiciousType::Gravel, None);
        let result = brush_tick(&mut block, 5.0);
        assert_eq!(result, BrushResult::Empty);
    }

    #[test]
    fn brush_tick_incremental_completion() {
        let mut block = SuspiciousBlock::new(SuspiciousType::Sand, Some(7));
        assert_eq!(brush_tick(&mut block, 2.0), BrushResult::InProgress(2.0));
        assert_eq!(brush_tick(&mut block, 2.0), BrushResult::Complete(7));
    }

    #[test]
    fn collapse_without_brush_with_progress() {
        let mut block = SuspiciousBlock::new(SuspiciousType::Sand, Some(5));
        block.brush_progress = 2.0;
        assert!(collapse_without_brush(&mut block));
        assert_eq!(block.brush_progress, 0.0);
        assert_eq!(block.item, None);
    }

    #[test]
    fn collapse_without_brush_no_progress() {
        let mut block = SuspiciousBlock::new(SuspiciousType::Gravel, Some(5));
        assert!(!collapse_without_brush(&mut block));
        assert_eq!(block.item, Some(5));
    }
}
