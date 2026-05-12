//! Texture filtering modes for different rendering contexts.

/// Texture filtering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFilter {
    Nearest,
    Bilinear,
    Trilinear,
    Anisotropic(u8),
}

/// Sampler descriptor derived from a texture filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplerDesc {
    pub mag: u8,
    pub min: u8,
    pub mipmap: u8,
    pub aniso: u8,
}

/// Returns the recommended filter for block textures (pixel-perfect).
pub fn filter_for_block_textures() -> TextureFilter {
    TextureFilter::Nearest
}

/// Returns the recommended filter for UI elements.
pub fn filter_for_ui() -> TextureFilter {
    TextureFilter::Bilinear
}

/// Returns the recommended filter for sky rendering.
pub fn filter_for_sky() -> TextureFilter {
    TextureFilter::Trilinear
}

/// Converts a `TextureFilter` into a `SamplerDesc`.
///
/// Encoding: 0 = Nearest, 1 = Linear.
pub fn sampler_descriptor(filter: TextureFilter) -> SamplerDesc {
    match filter {
        TextureFilter::Nearest => SamplerDesc {
            mag: 0,
            min: 0,
            mipmap: 0,
            aniso: 1,
        },
        TextureFilter::Bilinear => SamplerDesc {
            mag: 1,
            min: 1,
            mipmap: 0,
            aniso: 1,
        },
        TextureFilter::Trilinear => SamplerDesc {
            mag: 1,
            min: 1,
            mipmap: 1,
            aniso: 1,
        },
        TextureFilter::Anisotropic(level) => SamplerDesc {
            mag: 1,
            min: 1,
            mipmap: 1,
            aniso: level.max(1),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_textures_use_nearest() {
        assert_eq!(filter_for_block_textures(), TextureFilter::Nearest);
    }

    #[test]
    fn ui_uses_bilinear() {
        assert_eq!(filter_for_ui(), TextureFilter::Bilinear);
    }

    #[test]
    fn sky_uses_trilinear() {
        assert_eq!(filter_for_sky(), TextureFilter::Trilinear);
    }

    #[test]
    fn nearest_sampler_descriptor() {
        let desc = sampler_descriptor(TextureFilter::Nearest);
        assert_eq!(desc, SamplerDesc { mag: 0, min: 0, mipmap: 0, aniso: 1 });
    }

    #[test]
    fn bilinear_sampler_descriptor() {
        let desc = sampler_descriptor(TextureFilter::Bilinear);
        assert_eq!(desc, SamplerDesc { mag: 1, min: 1, mipmap: 0, aniso: 1 });
    }

    #[test]
    fn trilinear_sampler_descriptor() {
        let desc = sampler_descriptor(TextureFilter::Trilinear);
        assert_eq!(desc, SamplerDesc { mag: 1, min: 1, mipmap: 1, aniso: 1 });
    }

    #[test]
    fn anisotropic_sampler_descriptor() {
        let desc = sampler_descriptor(TextureFilter::Anisotropic(8));
        assert_eq!(desc, SamplerDesc { mag: 1, min: 1, mipmap: 1, aniso: 8 });
    }

    #[test]
    fn anisotropic_zero_clamps_to_one() {
        let desc = sampler_descriptor(TextureFilter::Anisotropic(0));
        assert_eq!(desc.aniso, 1);
    }
}
