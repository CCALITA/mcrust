//! Chunk Level-of-Detail (LOD) system for distance-based mesh simplification.
//!
//! Selects a [`LodLevel`] per chunk based on camera distance, computes fade
//! alpha for smooth pop-in at the render horizon, and estimates vertex counts
//! so the mesh allocator can right-size buffers.

/// Discrete level-of-detail tiers for chunk meshes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodLevel {
    /// Full resolution — every block face emitted.
    Full,
    /// Half resolution — sample every 2nd block.
    Half,
    /// Quarter resolution — sample every 4th block.
    Quarter,
}

impl LodLevel {
    /// Sampling step size: 1 for Full, 2 for Half, 4 for Quarter.
    pub fn step(&self) -> u8 {
        match self {
            LodLevel::Full => 1,
            LodLevel::Half => 2,
            LodLevel::Quarter => 4,
        }
    }
}

/// Choose the appropriate LOD level based on chunk distance (in chunks).
///
/// - 0..=6  -> Full
/// - 7..=12 -> Half
/// - 13+    -> Quarter
pub fn select_lod(chunk_distance: i32) -> LodLevel {
    match chunk_distance {
        d if d <= 6 => LodLevel::Full,
        d if d <= 12 => LodLevel::Half,
        _ => LodLevel::Quarter,
    }
}

/// Compute fade-out alpha for a chunk near the render horizon.
///
/// Returns 1.0 when `distance <= max_distance - 2`, linearly fades to 0.0
/// at `max_distance`. If `max_distance <= 2` or `distance <= 0`, returns
/// a clamped value.
pub fn chunk_fade_alpha(distance: i32, max_distance: i32) -> f32 {
    if distance >= max_distance {
        return 0.0;
    }
    let fade_start = max_distance - 2;
    if distance <= fade_start {
        return 1.0;
    }
    // Linear fade over the last 2 chunks.
    let range = (max_distance - fade_start) as f32;
    let progress = (distance - fade_start) as f32;
    1.0 - progress / range
}

/// Vertex sampling step for the given LOD level (alias for [`LodLevel::step`]).
pub fn lod_vertex_step(level: LodLevel) -> u8 {
    level.step()
}

/// Estimate the vertex count after LOD decimation.
///
/// Each step doubles in both X and Z, so the area reduction is step^2:
/// - Full  -> full_count / 1
/// - Half  -> full_count / 4
/// - Quarter -> full_count / 16
pub fn estimate_lod_vertex_count(full_count: usize, level: LodLevel) -> usize {
    let divisor = (level.step() as usize) * (level.step() as usize);
    full_count / divisor
}

/// Returns `true` when the LOD level has changed and the chunk mesh must be rebuilt.
pub fn should_rebuild_lod(old_level: LodLevel, new_level: LodLevel) -> bool {
    old_level != new_level
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- LOD selection boundaries ----

    #[test]
    fn select_lod_full_at_zero() {
        assert_eq!(select_lod(0), LodLevel::Full);
    }

    #[test]
    fn select_lod_full_at_boundary() {
        assert_eq!(select_lod(6), LodLevel::Full);
    }

    #[test]
    fn select_lod_half_just_past_full() {
        assert_eq!(select_lod(7), LodLevel::Half);
    }

    #[test]
    fn select_lod_half_at_boundary() {
        assert_eq!(select_lod(12), LodLevel::Half);
    }

    #[test]
    fn select_lod_quarter_just_past_half() {
        assert_eq!(select_lod(13), LodLevel::Quarter);
    }

    #[test]
    fn select_lod_quarter_far_away() {
        assert_eq!(select_lod(100), LodLevel::Quarter);
    }

    // ---- Step values ----

    #[test]
    fn step_values() {
        assert_eq!(LodLevel::Full.step(), 1);
        assert_eq!(LodLevel::Half.step(), 2);
        assert_eq!(LodLevel::Quarter.step(), 4);
    }

    #[test]
    fn vertex_step_matches_level_step() {
        assert_eq!(lod_vertex_step(LodLevel::Full), LodLevel::Full.step());
        assert_eq!(lod_vertex_step(LodLevel::Half), LodLevel::Half.step());
        assert_eq!(lod_vertex_step(LodLevel::Quarter), LodLevel::Quarter.step());
    }

    // ---- Fade alpha ----

    #[test]
    fn fade_alpha_fully_opaque_inside() {
        assert_eq!(chunk_fade_alpha(5, 16), 1.0);
    }

    #[test]
    fn fade_alpha_fully_opaque_at_fade_start() {
        // fade starts at max_distance - 2 = 14
        assert_eq!(chunk_fade_alpha(14, 16), 1.0);
    }

    #[test]
    fn fade_alpha_midpoint() {
        // distance=15, max=16, fade_start=14, range=2
        // progress=1, alpha=1.0-0.5=0.5
        let alpha = chunk_fade_alpha(15, 16);
        assert!((alpha - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fade_alpha_zero_at_max() {
        assert_eq!(chunk_fade_alpha(16, 16), 0.0);
    }

    #[test]
    fn fade_alpha_zero_beyond_max() {
        assert_eq!(chunk_fade_alpha(20, 16), 0.0);
    }

    #[test]
    fn fade_alpha_small_max_distance() {
        // max_distance=2, fade_start=0
        // distance=1 -> progress=1, range=2, alpha=0.5
        let alpha = chunk_fade_alpha(1, 2);
        assert!((alpha - 0.5).abs() < f32::EPSILON);
    }

    // ---- Vertex count estimates ----

    #[test]
    fn vertex_estimate_full() {
        assert_eq!(estimate_lod_vertex_count(1600, LodLevel::Full), 1600);
    }

    #[test]
    fn vertex_estimate_half() {
        assert_eq!(estimate_lod_vertex_count(1600, LodLevel::Half), 400);
    }

    #[test]
    fn vertex_estimate_quarter() {
        assert_eq!(estimate_lod_vertex_count(1600, LodLevel::Quarter), 100);
    }

    #[test]
    fn vertex_estimate_zero() {
        assert_eq!(estimate_lod_vertex_count(0, LodLevel::Quarter), 0);
    }

    // ---- Rebuild trigger ----

    #[test]
    fn rebuild_when_level_changes() {
        assert!(should_rebuild_lod(LodLevel::Full, LodLevel::Half));
        assert!(should_rebuild_lod(LodLevel::Half, LodLevel::Quarter));
        assert!(should_rebuild_lod(LodLevel::Quarter, LodLevel::Full));
    }

    #[test]
    fn no_rebuild_when_same() {
        assert!(!should_rebuild_lod(LodLevel::Full, LodLevel::Full));
        assert!(!should_rebuild_lod(LodLevel::Half, LodLevel::Half));
        assert!(!should_rebuild_lod(LodLevel::Quarter, LodLevel::Quarter));
    }
}
