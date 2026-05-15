//! Configuration types for 2D overlay render pipelines (crosshair, text, HUD elements).

/// Blending mode for overlay rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Alpha,
    Additive,
    None,
}

/// Primitive topology for overlay meshes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topology {
    TriangleList,
    LineList,
    LineStrip,
}

/// Configuration for a 2D overlay render pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayPipelineConfig {
    pub depth_test: bool,
    pub blend_mode: BlendMode,
    pub topology: Topology,
}

/// Default overlay config: no depth test, alpha blending, triangle list.
pub fn default_overlay_config() -> OverlayPipelineConfig {
    OverlayPipelineConfig {
        depth_test: false,
        blend_mode: BlendMode::Alpha,
        topology: Topology::TriangleList,
    }
}

/// Crosshair overlay config: no depth test, alpha blending, line list.
pub fn crosshair_config() -> OverlayPipelineConfig {
    OverlayPipelineConfig {
        depth_test: false,
        blend_mode: BlendMode::Alpha,
        topology: Topology::LineList,
    }
}

/// Text overlay config: no depth test, alpha blending, triangle list.
pub fn text_overlay_config() -> OverlayPipelineConfig {
    OverlayPipelineConfig {
        depth_test: false,
        blend_mode: BlendMode::Alpha,
        topology: Topology::TriangleList,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_overlay_has_no_depth_test() {
        let config = default_overlay_config();
        assert!(!config.depth_test);
        assert_eq!(config.blend_mode, BlendMode::Alpha);
        assert_eq!(config.topology, Topology::TriangleList);
    }

    #[test]
    fn crosshair_uses_line_list() {
        let config = crosshair_config();
        assert!(!config.depth_test);
        assert_eq!(config.blend_mode, BlendMode::Alpha);
        assert_eq!(config.topology, Topology::LineList);
    }

    #[test]
    fn text_overlay_uses_triangle_list() {
        let config = text_overlay_config();
        assert!(!config.depth_test);
        assert_eq!(config.blend_mode, BlendMode::Alpha);
        assert_eq!(config.topology, Topology::TriangleList);
    }

    #[test]
    fn blend_mode_equality() {
        assert_ne!(BlendMode::Alpha, BlendMode::Additive);
        assert_ne!(BlendMode::Alpha, BlendMode::None);
        assert_ne!(BlendMode::Additive, BlendMode::None);
    }

    #[test]
    fn topology_equality() {
        assert_ne!(Topology::TriangleList, Topology::LineList);
        assert_ne!(Topology::LineList, Topology::LineStrip);
    }

    #[test]
    fn config_clone() {
        let config = default_overlay_config();
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }
}
