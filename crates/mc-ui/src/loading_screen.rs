//! Loading screen progress display.
//!
//! Provides [`LoadingProgress`] for tracking world-load state and helpers for
//! rendering a progress bar, phase text, and random gameplay tips.

/// Current phase of the loading process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadPhase {
    TerrainGen,
    MeshBuilding,
    SpawningEntities,
    Ready,
}

/// Progress state for the loading screen.
#[derive(Debug, Clone)]
pub struct LoadingProgress {
    pub chunks_loaded: usize,
    pub chunks_needed: usize,
    pub phase: LoadPhase,
}

/// Return a progress fraction in the range `0.0..=1.0`.
///
/// Computed as `chunks_loaded / chunks_needed`, clamped to `[0, 1]`.
/// Returns `0.0` when `chunks_needed` is zero.
pub fn progress_fraction(p: &LoadingProgress) -> f32 {
    if p.chunks_needed == 0 {
        return 0.0;
    }
    let ratio = p.chunks_loaded as f32 / p.chunks_needed as f32;
    ratio.clamp(0.0, 1.0)
}

/// Return a human-readable loading status string for the current phase.
///
/// Includes a percentage for phases that track chunk progress.
pub fn loading_text(p: &LoadingProgress) -> String {
    let pct = (progress_fraction(p) * 100.0) as u32;
    match p.phase {
        LoadPhase::TerrainGen => format!("Generating terrain... {}%", pct),
        LoadPhase::MeshBuilding => format!("Building meshes... {}%", pct),
        LoadPhase::SpawningEntities => "Spawning entities...".to_string(),
        LoadPhase::Ready => "Ready!".to_string(),
    }
}

/// Compute the rectangle for a centered progress bar.
///
/// The bar is 60% of `screen_w`, 20 px tall, and vertically centered.
/// Returns `(x, y, width, height)`.
pub fn progress_bar_rect(screen_w: f32, screen_h: f32, fraction: f32) -> (f32, f32, f32, f32) {
    let full_width = screen_w * 0.6;
    let width = full_width * fraction.clamp(0.0, 1.0);
    let height = 20.0;
    let x = (screen_w - full_width) / 2.0;
    let y = (screen_h - height) / 2.0;
    (x, y, width, height)
}

/// Gameplay tips shown on the loading screen.
pub const TIPS: &[&str] = &[
    "Press F3 for debug info",
    "Torches prevent mob spawning",
    "Diamonds spawn below Y=16",
    "Crouch to avoid falling off edges",
    "Beds set your spawn point",
    "Water buckets negate fall damage",
    "Creepers are afraid of cats",
    "Iron golems protect villagers",
    "Ender pearls let you teleport",
    "Redstone powers mechanisms",
];

/// Return a tip from [`TIPS`] at the given index, wrapping around.
pub fn tip_for_index(index: usize) -> &'static str {
    TIPS[index % TIPS.len()]
}

/// Background color for the loading screen (dark red, Minecraft style).
pub fn loading_bg_color() -> [f32; 3] {
    [0.54, 0.0, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // progress_fraction
    // ------------------------------------------------------------------

    #[test]
    fn fraction_zero_when_nothing_loaded() {
        let p = LoadingProgress {
            chunks_loaded: 0,
            chunks_needed: 100,
            phase: LoadPhase::TerrainGen,
        };
        assert!((progress_fraction(&p) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_one_when_fully_loaded() {
        let p = LoadingProgress {
            chunks_loaded: 100,
            chunks_needed: 100,
            phase: LoadPhase::MeshBuilding,
        };
        assert!((progress_fraction(&p) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_clamped_above_one() {
        let p = LoadingProgress {
            chunks_loaded: 200,
            chunks_needed: 100,
            phase: LoadPhase::TerrainGen,
        };
        assert!((progress_fraction(&p) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_zero_when_chunks_needed_is_zero() {
        let p = LoadingProgress {
            chunks_loaded: 50,
            chunks_needed: 0,
            phase: LoadPhase::TerrainGen,
        };
        assert!((progress_fraction(&p) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fraction_half() {
        let p = LoadingProgress {
            chunks_loaded: 50,
            chunks_needed: 100,
            phase: LoadPhase::TerrainGen,
        };
        assert!((progress_fraction(&p) - 0.5).abs() < f32::EPSILON);
    }

    // ------------------------------------------------------------------
    // loading_text
    // ------------------------------------------------------------------

    #[test]
    fn text_terrain_gen() {
        let p = LoadingProgress {
            chunks_loaded: 45,
            chunks_needed: 100,
            phase: LoadPhase::TerrainGen,
        };
        assert_eq!(loading_text(&p), "Generating terrain... 45%");
    }

    #[test]
    fn text_mesh_building() {
        let p = LoadingProgress {
            chunks_loaded: 78,
            chunks_needed: 100,
            phase: LoadPhase::MeshBuilding,
        };
        assert_eq!(loading_text(&p), "Building meshes... 78%");
    }

    #[test]
    fn text_spawning_entities() {
        let p = LoadingProgress {
            chunks_loaded: 0,
            chunks_needed: 100,
            phase: LoadPhase::SpawningEntities,
        };
        assert_eq!(loading_text(&p), "Spawning entities...");
    }

    #[test]
    fn text_ready() {
        let p = LoadingProgress {
            chunks_loaded: 100,
            chunks_needed: 100,
            phase: LoadPhase::Ready,
        };
        assert_eq!(loading_text(&p), "Ready!");
    }

    // ------------------------------------------------------------------
    // progress_bar_rect
    // ------------------------------------------------------------------

    #[test]
    fn bar_centered_horizontally() {
        let (x, _y, width, _height) = progress_bar_rect(1000.0, 600.0, 1.0);
        let full_width = 1000.0 * 0.6;
        let expected_x = (1000.0 - full_width) / 2.0;
        assert!((x - expected_x).abs() < f32::EPSILON);
        assert!((width - full_width).abs() < f32::EPSILON);
    }

    #[test]
    fn bar_centered_vertically() {
        let (_x, y, _width, height) = progress_bar_rect(1000.0, 600.0, 1.0);
        let expected_y = (600.0 - 20.0) / 2.0;
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((height - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bar_width_scales_with_fraction() {
        let (_x, _y, width, _height) = progress_bar_rect(1000.0, 600.0, 0.5);
        let expected = 1000.0 * 0.6 * 0.5;
        assert!((width - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn bar_zero_fraction() {
        let (_x, _y, width, _height) = progress_bar_rect(800.0, 600.0, 0.0);
        assert!((width - 0.0).abs() < f32::EPSILON);
    }

    // ------------------------------------------------------------------
    // tip_for_index
    // ------------------------------------------------------------------

    #[test]
    fn tip_first_index() {
        assert_eq!(tip_for_index(0), "Press F3 for debug info");
    }

    #[test]
    fn tip_last_index() {
        assert_eq!(tip_for_index(9), "Redstone powers mechanisms");
    }

    #[test]
    fn tip_wraps_around() {
        assert_eq!(tip_for_index(10), tip_for_index(0));
        assert_eq!(tip_for_index(13), tip_for_index(3));
    }

    #[test]
    fn tip_large_index_wraps() {
        assert_eq!(tip_for_index(1000), tip_for_index(0));
    }

    // ------------------------------------------------------------------
    // loading_bg_color
    // ------------------------------------------------------------------

    #[test]
    fn bg_color_is_dark_red() {
        let c = loading_bg_color();
        assert!((c[0] - 0.54).abs() < f32::EPSILON);
        assert!((c[1] - 0.0).abs() < f32::EPSILON);
        assert!((c[2] - 0.0).abs() < f32::EPSILON);
    }

    // ------------------------------------------------------------------
    // TIPS constant
    // ------------------------------------------------------------------

    #[test]
    fn tips_has_ten_entries() {
        assert_eq!(TIPS.len(), 10);
    }
}
