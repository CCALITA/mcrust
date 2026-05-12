//! Render pipeline stage definitions and ordering.

/// Represents a stage in the render pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    TerrainOpaque,
    TerrainTransparent,
    Sky,
    Water,
    Particles,
    UI,
    PostProcess,
}

/// Returns the stages in the order they should be rendered.
pub fn stage_order() -> &'static [PipelineStage] {
    &[
        PipelineStage::TerrainOpaque,
        PipelineStage::TerrainTransparent,
        PipelineStage::Sky,
        PipelineStage::Water,
        PipelineStage::Particles,
        PipelineStage::UI,
        PipelineStage::PostProcess,
    ]
}

/// Returns a human-readable name for the given stage.
pub fn stage_name(stage: PipelineStage) -> &'static str {
    match stage {
        PipelineStage::TerrainOpaque => "TerrainOpaque",
        PipelineStage::TerrainTransparent => "TerrainTransparent",
        PipelineStage::Sky => "Sky",
        PipelineStage::Water => "Water",
        PipelineStage::Particles => "Particles",
        PipelineStage::UI => "UI",
        PipelineStage::PostProcess => "PostProcess",
    }
}

/// Returns whether blending is enabled for the given stage.
pub fn stage_blend_enabled(stage: PipelineStage) -> bool {
    match stage {
        PipelineStage::TerrainOpaque => false,
        PipelineStage::TerrainTransparent => true,
        PipelineStage::Sky => false,
        PipelineStage::Water => true,
        PipelineStage::Particles => true,
        PipelineStage::UI => true,
        PipelineStage::PostProcess => false,
    }
}

/// Returns whether depth writing is enabled for the given stage.
pub fn stage_depth_write(stage: PipelineStage) -> bool {
    match stage {
        PipelineStage::TerrainOpaque => true,
        PipelineStage::TerrainTransparent => false,
        PipelineStage::Sky => false,
        PipelineStage::Water => false,
        PipelineStage::Particles => false,
        PipelineStage::UI => false,
        PipelineStage::PostProcess => false,
    }
}

/// Returns the total number of pipeline stages.
pub fn total_stages() -> usize {
    7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_order_has_correct_count() {
        assert_eq!(stage_order().len(), 7);
        assert_eq!(stage_order().len(), total_stages());
    }

    #[test]
    fn stage_order_starts_with_terrain_opaque() {
        assert_eq!(stage_order()[0], PipelineStage::TerrainOpaque);
    }

    #[test]
    fn stage_order_ends_with_post_process() {
        assert_eq!(stage_order()[6], PipelineStage::PostProcess);
    }

    #[test]
    fn stage_names_are_correct() {
        assert_eq!(stage_name(PipelineStage::TerrainOpaque), "TerrainOpaque");
        assert_eq!(stage_name(PipelineStage::Water), "Water");
        assert_eq!(stage_name(PipelineStage::UI), "UI");
        assert_eq!(stage_name(PipelineStage::PostProcess), "PostProcess");
    }

    #[test]
    fn opaque_stages_have_no_blending() {
        assert!(!stage_blend_enabled(PipelineStage::TerrainOpaque));
        assert!(!stage_blend_enabled(PipelineStage::Sky));
        assert!(!stage_blend_enabled(PipelineStage::PostProcess));
    }

    #[test]
    fn transparent_stages_have_blending() {
        assert!(stage_blend_enabled(PipelineStage::TerrainTransparent));
        assert!(stage_blend_enabled(PipelineStage::Water));
        assert!(stage_blend_enabled(PipelineStage::Particles));
        assert!(stage_blend_enabled(PipelineStage::UI));
    }

    #[test]
    fn only_terrain_opaque_writes_depth() {
        assert!(stage_depth_write(PipelineStage::TerrainOpaque));
        for &stage in &stage_order()[1..] {
            assert!(!stage_depth_write(stage));
        }
    }

    #[test]
    fn total_stages_is_seven() {
        assert_eq!(total_stages(), 7);
    }

    #[test]
    fn all_stages_have_names() {
        for &stage in stage_order() {
            assert!(!stage_name(stage).is_empty());
        }
    }
}
