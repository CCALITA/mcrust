//! Render target configuration for wgpu render passes.

/// Supported texture formats for render targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderTargetFormat {
    Rgba8,
    Bgra8,
    Depth32,
    Rgba16Float,
}

/// Configuration for a render target (color, depth, HDR, shadow, MSAA).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTargetConfig {
    pub width: u32,
    pub height: u32,
    pub format: RenderTargetFormat,
    pub msaa_samples: u32,
}

/// Returns bytes per pixel for the given format.
pub fn bytes_per_pixel(format: RenderTargetFormat) -> u32 {
    match format {
        RenderTargetFormat::Rgba8 => 4,
        RenderTargetFormat::Bgra8 => 4,
        RenderTargetFormat::Depth32 => 4,
        RenderTargetFormat::Rgba16Float => 8,
    }
}

/// Returns total memory in bytes for a render target configuration.
pub fn total_memory(config: &RenderTargetConfig) -> usize {
    config.width as usize
        * config.height as usize
        * bytes_per_pixel(config.format) as usize
        * config.msaa_samples as usize
}

/// Default RGBA8 color target with no MSAA.
pub fn default_color_target(width: u32, height: u32) -> RenderTargetConfig {
    RenderTargetConfig {
        width,
        height,
        format: RenderTargetFormat::Rgba8,
        msaa_samples: 1,
    }
}

/// Default Depth32 target with no MSAA.
pub fn default_depth_target(width: u32, height: u32) -> RenderTargetConfig {
    RenderTargetConfig {
        width,
        height,
        format: RenderTargetFormat::Depth32,
        msaa_samples: 1,
    }
}

/// HDR (Rgba16Float) target with no MSAA.
pub fn hdr_target(width: u32, height: u32) -> RenderTargetConfig {
    RenderTargetConfig {
        width,
        height,
        format: RenderTargetFormat::Rgba16Float,
        msaa_samples: 1,
    }
}

/// Square depth target for shadow mapping.
pub fn shadow_target(resolution: u32) -> RenderTargetConfig {
    RenderTargetConfig {
        width: resolution,
        height: resolution,
        format: RenderTargetFormat::Depth32,
        msaa_samples: 1,
    }
}

/// RGBA8 color target with the given MSAA sample count.
pub fn msaa_target(width: u32, height: u32, samples: u32) -> RenderTargetConfig {
    RenderTargetConfig {
        width,
        height,
        format: RenderTargetFormat::Rgba8,
        msaa_samples: samples,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_per_pixel_rgba8() {
        assert_eq!(bytes_per_pixel(RenderTargetFormat::Rgba8), 4);
    }

    #[test]
    fn bytes_per_pixel_bgra8() {
        assert_eq!(bytes_per_pixel(RenderTargetFormat::Bgra8), 4);
    }

    #[test]
    fn bytes_per_pixel_depth32() {
        assert_eq!(bytes_per_pixel(RenderTargetFormat::Depth32), 4);
    }

    #[test]
    fn bytes_per_pixel_rgba16float() {
        assert_eq!(bytes_per_pixel(RenderTargetFormat::Rgba16Float), 8);
    }

    #[test]
    fn total_memory_basic() {
        let config = default_color_target(1920, 1080);
        assert_eq!(total_memory(&config), 1920 * 1080 * 4);
    }

    #[test]
    fn total_memory_hdr() {
        let config = hdr_target(1920, 1080);
        assert_eq!(total_memory(&config), 1920 * 1080 * 8);
    }

    #[test]
    fn total_memory_msaa() {
        let config = msaa_target(800, 600, 4);
        assert_eq!(total_memory(&config), 800 * 600 * 4 * 4);
    }

    #[test]
    fn default_color_target_fields() {
        let c = default_color_target(640, 480);
        assert_eq!(c.width, 640);
        assert_eq!(c.height, 480);
        assert_eq!(c.format, RenderTargetFormat::Rgba8);
        assert_eq!(c.msaa_samples, 1);
    }

    #[test]
    fn default_depth_target_fields() {
        let d = default_depth_target(640, 480);
        assert_eq!(d.format, RenderTargetFormat::Depth32);
        assert_eq!(d.msaa_samples, 1);
    }

    #[test]
    fn hdr_target_fields() {
        let h = hdr_target(1280, 720);
        assert_eq!(h.format, RenderTargetFormat::Rgba16Float);
    }

    #[test]
    fn shadow_target_is_square() {
        let s = shadow_target(2048);
        assert_eq!(s.width, 2048);
        assert_eq!(s.height, 2048);
        assert_eq!(s.format, RenderTargetFormat::Depth32);
    }

    #[test]
    fn msaa_target_samples() {
        let m = msaa_target(800, 600, 8);
        assert_eq!(m.msaa_samples, 8);
        assert_eq!(m.format, RenderTargetFormat::Rgba8);
    }
}
