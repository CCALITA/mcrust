//! Block breaking animation overlay.
//!
//! Tracks the progress of breaking a block and provides stage-based
//! crack overlay data for rendering.

/// Result of a single tick of the block-breaking animation.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockBreakResult {
    /// No block is being broken.
    None,
    /// Block is actively being broken; `stage` ranges from 0 (just started) to 9 (about to break).
    Breaking { stage: u8 },
    /// Block has been fully broken at the given position.
    Broken { pos: (i32, i32, i32) },
}

/// Overlay state for the block-breaking animation.
#[derive(Debug, Clone)]
pub struct BlockBreakOverlay {
    /// Position of the block currently being broken (if any).
    pub block_pos: Option<(i32, i32, i32)>,
    /// Progress from 0.0 (not started) to 1.0 (fully broken).
    pub progress: f32,
    /// Total time required to break this block, in seconds.
    pub total_time: f32,
    /// Whether a break operation is currently active.
    pub active: bool,
}

impl BlockBreakOverlay {
    /// Create a new inactive overlay.
    pub fn new() -> Self {
        Self {
            block_pos: None,
            progress: 0.0,
            total_time: 0.0,
            active: false,
        }
    }

    /// Begin breaking a block at the given position with the specified break time.
    pub fn start(&mut self, pos: (i32, i32, i32), break_time: f32) {
        self.block_pos = Some(pos);
        self.progress = 0.0;
        self.total_time = break_time;
        self.active = true;
    }

    /// Cancel the current break operation and reset to inactive.
    pub fn cancel(&mut self) {
        self.block_pos = None;
        self.progress = 0.0;
        self.total_time = 0.0;
        self.active = false;
    }

    /// Advance the break animation by `dt` seconds.
    ///
    /// Returns [`BlockBreakResult::None`] if inactive, [`BlockBreakResult::Broken`]
    /// when the block is fully broken (resetting state), or [`BlockBreakResult::Breaking`]
    /// with the current crack stage.
    pub fn tick(&mut self, dt: f32) -> BlockBreakResult {
        if !self.active {
            return BlockBreakResult::None;
        }

        if self.total_time <= 0.0 {
            // Instant break
            let pos = self.block_pos.unwrap_or((0, 0, 0));
            self.cancel();
            return BlockBreakResult::Broken { pos };
        }

        self.progress += dt / self.total_time;

        if self.progress >= 1.0 {
            let pos = self.block_pos.unwrap_or((0, 0, 0));
            self.cancel();
            return BlockBreakResult::Broken { pos };
        }

        BlockBreakResult::Breaking {
            stage: crack_stage(self.progress),
        }
    }
}

impl Default for BlockBreakOverlay {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a progress value (0.0 -- 1.0) to a crack stage (0 -- 9).
///
/// Stage 0 means the break just started; stage 9 means it is about to break.
pub fn crack_stage(progress: f32) -> u8 {
    let clamped = progress.clamp(0.0, 1.0);
    // Multiply by 10 and floor, but cap at 9 so 1.0 maps to 9 (not 10).
    (clamped * 10.0).min(9.0) as u8
}

/// Return an RGBA overlay color for the given crack stage.
///
/// The color is black with increasing opacity:
/// stage 0 = `[0, 0, 0, 0.05]`, stage 9 = `[0, 0, 0, 0.8]`.
pub fn crack_color(stage: u8) -> [f32; 4] {
    let t = (stage.min(9) as f32) / 9.0;
    let alpha = 0.05 + t * 0.75; // 0.05 at stage 0, 0.80 at stage 9
    [0.0, 0.0, 0.0, alpha]
}

/// Generate 6-face quad vertices slightly offset from the block surface,
/// suitable for rendering a crack overlay later.
///
/// Each face is a quad (4 vertices, 2 triangles). Returns 24 vertices total.
/// The offset prevents z-fighting with the underlying block face.
pub fn crack_vertices(bx: f32, by: f32, bz: f32, stage: u8) -> Vec<[f32; 3]> {
    let _ = stage; // Stage may influence vertex data in the future; included for API stability.
    let offset: f32 = 0.001; // Slight outward push to avoid z-fighting.

    let x0 = bx - offset;
    let y0 = by - offset;
    let z0 = bz - offset;
    let x1 = bx + 1.0 + offset;
    let y1 = by + 1.0 + offset;
    let z1 = bz + 1.0 + offset;

    vec![
        // Top face (+Y)
        [x0, y1, z0],
        [x0, y1, z1],
        [x1, y1, z1],
        [x1, y1, z0],
        // Bottom face (-Y)
        [x0, y0, z1],
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y0, z1],
        // North face (-Z)
        [x1, y1, z0],
        [x1, y0, z0],
        [x0, y0, z0],
        [x0, y1, z0],
        // South face (+Z)
        [x0, y1, z1],
        [x0, y0, z1],
        [x1, y0, z1],
        [x1, y1, z1],
        // East face (+X)
        [x1, y1, z1],
        [x1, y0, z1],
        [x1, y0, z0],
        [x1, y1, z0],
        // West face (-X)
        [x0, y1, z0],
        [x0, y0, z0],
        [x0, y0, z1],
        [x0, y1, z1],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_inactive() {
        let overlay = BlockBreakOverlay::new();
        assert!(!overlay.active);
        assert!(overlay.block_pos.is_none());
        assert!((overlay.progress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn start_sets_active() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((1, 2, 3), 2.0);
        assert!(overlay.active);
        assert_eq!(overlay.block_pos, Some((1, 2, 3)));
        assert!((overlay.total_time - 2.0).abs() < f32::EPSILON);
        assert!((overlay.progress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cancel_resets_to_inactive() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((5, 10, 5), 3.0);
        overlay.cancel();
        assert!(!overlay.active);
        assert!(overlay.block_pos.is_none());
        assert!((overlay.progress - 0.0).abs() < f32::EPSILON);
        assert!((overlay.total_time - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_when_inactive_returns_none() {
        let mut overlay = BlockBreakOverlay::new();
        let result = overlay.tick(0.1);
        assert_eq!(result, BlockBreakResult::None);
    }

    #[test]
    fn tick_advances_progress() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((0, 0, 0), 2.0);
        let result = overlay.tick(0.5); // 0.5 / 2.0 = 0.25
        assert!(overlay.active);
        assert!((overlay.progress - 0.25).abs() < 1e-5);
        assert!(matches!(result, BlockBreakResult::Breaking { .. }));
    }

    #[test]
    fn tick_returns_breaking_with_correct_stage() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((0, 0, 0), 1.0);
        let result = overlay.tick(0.55); // progress = 0.55 -> stage = 5
        assert_eq!(result, BlockBreakResult::Breaking { stage: 5 });
    }

    #[test]
    fn tick_returns_broken_at_100_percent() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((3, 4, 5), 1.0);
        let result = overlay.tick(1.0); // progress >= 1.0
        assert_eq!(result, BlockBreakResult::Broken { pos: (3, 4, 5) });
        assert!(!overlay.active);
    }

    #[test]
    fn tick_returns_broken_when_exceeding_total_time() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((1, 1, 1), 0.5);
        let result = overlay.tick(1.0); // way past total
        assert_eq!(result, BlockBreakResult::Broken { pos: (1, 1, 1) });
        assert!(!overlay.active);
    }

    #[test]
    fn tick_with_zero_total_time_breaks_instantly() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((7, 8, 9), 0.0);
        let result = overlay.tick(0.01);
        assert_eq!(result, BlockBreakResult::Broken { pos: (7, 8, 9) });
        assert!(!overlay.active);
    }

    #[test]
    fn crack_stage_maps_0_to_9() {
        assert_eq!(crack_stage(0.0), 0);
        assert_eq!(crack_stage(0.05), 0);
        assert_eq!(crack_stage(0.1), 1);
        assert_eq!(crack_stage(0.5), 5);
        assert_eq!(crack_stage(0.9), 9);
        assert_eq!(crack_stage(0.99), 9);
        assert_eq!(crack_stage(1.0), 9);
    }

    #[test]
    fn crack_stage_clamps_out_of_range() {
        assert_eq!(crack_stage(-0.5), 0);
        assert_eq!(crack_stage(1.5), 9);
    }

    #[test]
    fn crack_color_darkens_with_stage() {
        let c0 = crack_color(0);
        let c9 = crack_color(9);
        // RGB is always 0
        assert!((c0[0]).abs() < f32::EPSILON);
        assert!((c0[1]).abs() < f32::EPSILON);
        assert!((c0[2]).abs() < f32::EPSILON);
        assert!((c9[0]).abs() < f32::EPSILON);
        assert!((c9[1]).abs() < f32::EPSILON);
        assert!((c9[2]).abs() < f32::EPSILON);
        // Alpha increases
        assert!(
            (c0[3] - 0.05).abs() < 1e-5,
            "stage 0 alpha should be 0.05, got {}",
            c0[3]
        );
        assert!(
            (c9[3] - 0.80).abs() < 1e-5,
            "stage 9 alpha should be 0.80, got {}",
            c9[3]
        );
        // Monotonically increasing alpha
        for stage in 1..=9u8 {
            let prev = crack_color(stage - 1)[3];
            let curr = crack_color(stage)[3];
            assert!(
                curr > prev,
                "alpha should increase: stage {} ({}) > stage {} ({})",
                stage,
                curr,
                stage - 1,
                prev
            );
        }
    }

    #[test]
    fn crack_vertices_produces_24_vertices_for_6_faces() {
        let verts = crack_vertices(0.0, 0.0, 0.0, 5);
        assert_eq!(verts.len(), 24, "6 faces x 4 vertices = 24");
    }

    #[test]
    fn crack_vertices_are_slightly_offset() {
        let verts = crack_vertices(10.0, 20.0, 30.0, 0);
        // Top face first vertex should be slightly outside (10, 21, 30)
        let top0 = verts[0];
        assert!(top0[0] < 10.0, "x0 should be offset below 10.0");
        assert!(top0[1] > 21.0, "y1 should be offset above 21.0");
        assert!(top0[2] < 30.0, "z0 should be offset below 30.0");
    }

    #[test]
    fn default_is_same_as_new() {
        let a = BlockBreakOverlay::new();
        let b = BlockBreakOverlay::default();
        assert_eq!(a.active, b.active);
        assert_eq!(a.block_pos, b.block_pos);
        assert!((a.progress - b.progress).abs() < f32::EPSILON);
        assert!((a.total_time - b.total_time).abs() < f32::EPSILON);
    }

    #[test]
    fn multiple_ticks_accumulate_progress() {
        let mut overlay = BlockBreakOverlay::new();
        overlay.start((0, 0, 0), 4.0);

        overlay.tick(1.0); // progress = 0.25
        assert!((overlay.progress - 0.25).abs() < 1e-5);

        overlay.tick(1.0); // progress = 0.50
        assert!((overlay.progress - 0.50).abs() < 1e-5);

        overlay.tick(1.0); // progress = 0.75
        assert!((overlay.progress - 0.75).abs() < 1e-5);

        let result = overlay.tick(1.0); // progress = 1.0 -> Broken
        assert_eq!(result, BlockBreakResult::Broken { pos: (0, 0, 0) });
    }
}
