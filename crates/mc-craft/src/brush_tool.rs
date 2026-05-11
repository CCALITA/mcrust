//! Brush tool for archaeological excavation.

/// Total time in seconds to complete a brushing action.
pub fn brush_total_time() -> f32 {
    4.0
}

/// Durability cost per brush use.
pub fn brush_durability_cost() -> u16 {
    1
}

/// Item ID for the brush tool.
pub fn brush_item_id() -> u16 {
    9100
}

/// State of an active brushing action.
#[derive(Debug, Clone)]
pub struct BrushState {
    pub using: bool,
    pub progress: f32,
    pub target_pos: Option<(i32, i32, i32)>,
}

impl BrushState {
    pub fn new() -> Self {
        Self {
            using: false,
            progress: 0.0,
            target_pos: None,
        }
    }
}

/// Result of a brush tick update.
#[derive(Debug, Clone, PartialEq)]
pub enum BrushTick {
    InProgress(f32),
    Complete,
    NotBrushing,
}

/// Start brushing at the given block position.
pub fn start_brushing(state: &mut BrushState, pos: (i32, i32, i32)) {
    state.using = true;
    state.progress = 0.0;
    state.target_pos = Some(pos);
}

/// Advance the brush by `dt` seconds, returning the tick result.
pub fn tick_brush(state: &mut BrushState, dt: f32) -> BrushTick {
    if !state.using {
        return BrushTick::NotBrushing;
    }

    state.progress += dt;

    if state.progress >= brush_total_time() {
        state.using = false;
        state.progress = brush_total_time();
        BrushTick::Complete
    } else {
        BrushTick::InProgress(state.progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_defaults() {
        let state = BrushState::new();
        assert!(!state.using);
        assert_eq!(state.progress, 0.0);
        assert!(state.target_pos.is_none());
    }

    #[test]
    fn brush_constants() {
        assert_eq!(brush_total_time(), 4.0);
        assert_eq!(brush_durability_cost(), 1);
        assert_eq!(brush_item_id(), 9100);
    }

    #[test]
    fn start_brushing_sets_state() {
        let mut state = BrushState::new();
        start_brushing(&mut state, (1, 2, 3));
        assert!(state.using);
        assert_eq!(state.progress, 0.0);
        assert_eq!(state.target_pos, Some((1, 2, 3)));
    }

    #[test]
    fn tick_not_brushing() {
        let mut state = BrushState::new();
        assert_eq!(tick_brush(&mut state, 1.0), BrushTick::NotBrushing);
    }

    #[test]
    fn tick_in_progress() {
        let mut state = BrushState::new();
        start_brushing(&mut state, (0, 0, 0));
        let result = tick_brush(&mut state, 1.0);
        assert_eq!(result, BrushTick::InProgress(1.0));
        assert!(state.using);
    }

    #[test]
    fn tick_completes() {
        let mut state = BrushState::new();
        start_brushing(&mut state, (0, 0, 0));
        let result = tick_brush(&mut state, 5.0);
        assert_eq!(result, BrushTick::Complete);
        assert!(!state.using);
        assert_eq!(state.progress, 4.0);
    }

    #[test]
    fn tick_incremental_to_completion() {
        let mut state = BrushState::new();
        start_brushing(&mut state, (10, 20, 30));
        assert_eq!(tick_brush(&mut state, 2.0), BrushTick::InProgress(2.0));
        assert_eq!(tick_brush(&mut state, 1.5), BrushTick::InProgress(3.5));
        assert_eq!(tick_brush(&mut state, 1.0), BrushTick::Complete);
    }
}
