//! Multi-pass render manager for ordering and configuring render passes.

/// Identifies each render pass in the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPassId {
    Sky,
    Terrain,
    Water,
    Overlay,
    Particles,
}

/// An ordered sequence of render passes.
#[derive(Debug, Clone)]
pub struct RenderPassOrder {
    pub passes: Vec<RenderPassId>,
}

/// Returns the default render pass ordering.
pub fn default_pass_order() -> RenderPassOrder {
    RenderPassOrder {
        passes: vec![
            RenderPassId::Sky,
            RenderPassId::Terrain,
            RenderPassId::Water,
            RenderPassId::Particles,
            RenderPassId::Overlay,
        ],
    }
}

/// Returns whether the depth buffer should be cleared before this pass.
pub fn should_clear_depth(pass: RenderPassId) -> bool {
    matches!(pass, RenderPassId::Sky)
}

/// Returns whether this pass requires depth testing.
pub fn needs_depth_test(pass: RenderPassId) -> bool {
    matches!(
        pass,
        RenderPassId::Sky | RenderPassId::Terrain | RenderPassId::Water
    )
}

/// Returns a human-readable name for the pass.
pub fn pass_name(pass: RenderPassId) -> &'static str {
    match pass {
        RenderPassId::Sky => "Sky",
        RenderPassId::Terrain => "Terrain",
        RenderPassId::Water => "Water",
        RenderPassId::Overlay => "Overlay",
        RenderPassId::Particles => "Particles",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_order_has_five_passes() {
        let order = default_pass_order();
        assert_eq!(order.passes.len(), 5);
    }

    #[test]
    fn default_order_sequence() {
        let order = default_pass_order();
        assert_eq!(
            order.passes,
            vec![
                RenderPassId::Sky,
                RenderPassId::Terrain,
                RenderPassId::Water,
                RenderPassId::Particles,
                RenderPassId::Overlay,
            ]
        );
    }

    #[test]
    fn only_sky_clears_depth() {
        assert!(should_clear_depth(RenderPassId::Sky));
        assert!(!should_clear_depth(RenderPassId::Terrain));
        assert!(!should_clear_depth(RenderPassId::Water));
        assert!(!should_clear_depth(RenderPassId::Overlay));
        assert!(!should_clear_depth(RenderPassId::Particles));
    }

    #[test]
    fn depth_test_for_geometry_passes() {
        assert!(needs_depth_test(RenderPassId::Sky));
        assert!(needs_depth_test(RenderPassId::Terrain));
        assert!(needs_depth_test(RenderPassId::Water));
        assert!(!needs_depth_test(RenderPassId::Overlay));
        assert!(!needs_depth_test(RenderPassId::Particles));
    }

    #[test]
    fn pass_names_are_correct() {
        assert_eq!(pass_name(RenderPassId::Sky), "Sky");
        assert_eq!(pass_name(RenderPassId::Terrain), "Terrain");
        assert_eq!(pass_name(RenderPassId::Water), "Water");
        assert_eq!(pass_name(RenderPassId::Overlay), "Overlay");
        assert_eq!(pass_name(RenderPassId::Particles), "Particles");
    }
}
