/// Banner pattern system.
///
/// Minecraft banners support up to 6 pattern layers, each with a pattern type
/// and dye color. Some patterns require a special "banner pattern" item and can
/// only be applied via the loom.

/// Maximum number of pattern layers a banner can hold.
pub const MAX_LAYERS: usize = 6;

// ── Dye Colors ───────────────────────────────────────────────────────────

/// The 16 Minecraft dye colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DyeColor {
    White = 0,
    Orange = 1,
    Magenta = 2,
    LightBlue = 3,
    Yellow = 4,
    Lime = 5,
    Pink = 6,
    Gray = 7,
    LightGray = 8,
    Cyan = 9,
    Purple = 10,
    Blue = 11,
    Brown = 12,
    Green = 13,
    Red = 14,
    Black = 15,
}

// ── Banner Pattern Types ─────────────────────────────────────────────────

/// All banner pattern types available in Minecraft.
///
/// Patterns that require a special "banner pattern" item (and therefore the
/// loom) are distinguished by [`pattern_requires_loom_pattern_item`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BannerPatternType {
    Base,
    TopHalf,
    BottomHalf,
    LeftHalf,
    RightHalf,
    Center,
    VerticalStripe,
    HorizontalStripe,
    DiagonalLeft,
    DiagonalRight,
    Cross,
    SaltireCross,
    TriangleBottom,
    TriangleTop,
    TrianglesBottom,
    TrianglesTop,
    LeftTriangle,
    RightTriangle,
    Circle,
    Rhombus,
    Border,
    CurlyBorder,
    Brick,
    Gradient,
    GradientUp,
    Creeper,
    Skull,
    Flower,
    Mojang,
    Globe,
    Piglin,
    Flow,
    Guster,
    SmallStripes,
    SquareBottomLeft,
    SquareBottomRight,
    SquareTopLeft,
    SquareTopRight,
    DiagonalUpRight,
    DiagonalUpLeft,
    PerBend,
    PerBendSinister,
}

/// Returns `true` if the pattern requires a special banner-pattern item to
/// apply (i.e. it can only be used via the loom with the matching item).
#[must_use]
pub fn pattern_requires_loom_pattern_item(pattern: BannerPatternType) -> bool {
    matches!(
        pattern,
        BannerPatternType::Creeper
            | BannerPatternType::Skull
            | BannerPatternType::Flower
            | BannerPatternType::Mojang
            | BannerPatternType::Globe
            | BannerPatternType::Piglin
            | BannerPatternType::Flow
            | BannerPatternType::Guster
    )
}

// ── Banner ───────────────────────────────────────────────────────────────

/// A single pattern layer applied to a banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternLayer {
    pub pattern: BannerPatternType,
    pub color: DyeColor,
}

/// A banner with its base color and up to [`MAX_LAYERS`] pattern layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Banner {
    pub base_color: DyeColor,
    pub layers: Vec<PatternLayer>,
}

impl Banner {
    /// Create a blank banner with the given base color.
    #[must_use]
    pub fn new(base_color: DyeColor) -> Self {
        Self {
            base_color,
            layers: Vec::new(),
        }
    }

    /// Return the number of pattern layers currently applied.
    #[must_use]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

// ── Apply Pattern ────────────────────────────────────────────────────────

/// Error returned when a pattern cannot be applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyPatternError {
    /// The banner already has the maximum number of layers.
    MaxLayersReached,
}

impl core::fmt::Display for ApplyPatternError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ApplyPatternError::MaxLayersReached => {
                write!(f, "banner already has the maximum of {MAX_LAYERS} layers")
            }
        }
    }
}

/// Apply a pattern layer to a banner, returning an updated banner.
///
/// Returns an error if the banner already has [`MAX_LAYERS`] layers.
/// The original banner is not modified; a new `Banner` is returned.
pub fn apply_pattern(
    banner: &Banner,
    pattern: BannerPatternType,
    color: DyeColor,
) -> Result<Banner, ApplyPatternError> {
    if banner.layers.len() >= MAX_LAYERS {
        return Err(ApplyPatternError::MaxLayersReached);
    }

    let mut new_layers = banner.layers.clone();
    new_layers.push(PatternLayer { pattern, color });

    Ok(Banner {
        base_color: banner.base_color,
        layers: new_layers,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_pattern_to_blank_banner() {
        let banner = Banner::new(DyeColor::White);
        let result = apply_pattern(&banner, BannerPatternType::Cross, DyeColor::Red);
        let updated = result.expect("should succeed on blank banner");

        assert_eq!(updated.base_color, DyeColor::White);
        assert_eq!(updated.layer_count(), 1);
        assert_eq!(updated.layers[0].pattern, BannerPatternType::Cross);
        assert_eq!(updated.layers[0].color, DyeColor::Red);
    }

    #[test]
    fn apply_pattern_does_not_mutate_original() {
        let banner = Banner::new(DyeColor::Blue);
        let _updated = apply_pattern(&banner, BannerPatternType::Gradient, DyeColor::Black)
            .expect("should succeed");

        assert_eq!(banner.layer_count(), 0, "original banner must be unchanged");
    }

    #[test]
    fn apply_up_to_max_layers() {
        let mut banner = Banner::new(DyeColor::Black);
        for i in 0..MAX_LAYERS {
            banner = apply_pattern(&banner, BannerPatternType::Base, DyeColor::White)
                .unwrap_or_else(|_| panic!("layer {i} should be allowed"));
        }
        assert_eq!(banner.layer_count(), MAX_LAYERS);
    }

    #[test]
    fn reject_layer_beyond_max() {
        let mut banner = Banner::new(DyeColor::Red);
        for _ in 0..MAX_LAYERS {
            banner = apply_pattern(&banner, BannerPatternType::Base, DyeColor::White)
                .expect("should succeed within limit");
        }

        let err = apply_pattern(&banner, BannerPatternType::Rhombus, DyeColor::Green);
        assert_eq!(err, Err(ApplyPatternError::MaxLayersReached));
    }

    #[test]
    fn loom_pattern_items_identified_correctly() {
        // Patterns that require a loom pattern item
        let loom_required = [
            BannerPatternType::Creeper,
            BannerPatternType::Skull,
            BannerPatternType::Flower,
            BannerPatternType::Mojang,
            BannerPatternType::Globe,
            BannerPatternType::Piglin,
            BannerPatternType::Flow,
            BannerPatternType::Guster,
        ];
        for p in loom_required {
            assert!(
                pattern_requires_loom_pattern_item(p),
                "{p:?} should require a loom pattern item"
            );
        }

        // Patterns that do NOT require a loom pattern item
        let no_item_needed = [
            BannerPatternType::Base,
            BannerPatternType::Cross,
            BannerPatternType::SaltireCross,
            BannerPatternType::VerticalStripe,
            BannerPatternType::HorizontalStripe,
            BannerPatternType::Border,
            BannerPatternType::Gradient,
            BannerPatternType::Circle,
            BannerPatternType::Rhombus,
            BannerPatternType::Brick,
        ];
        for p in no_item_needed {
            assert!(
                !pattern_requires_loom_pattern_item(p),
                "{p:?} should NOT require a loom pattern item"
            );
        }
    }

    #[test]
    fn multiple_layers_preserve_order() {
        let banner = Banner::new(DyeColor::White);
        let step1 = apply_pattern(&banner, BannerPatternType::DiagonalLeft, DyeColor::Red)
            .expect("layer 1");
        let step2 = apply_pattern(&step1, BannerPatternType::Circle, DyeColor::Blue)
            .expect("layer 2");
        let step3 = apply_pattern(&step2, BannerPatternType::Border, DyeColor::Green)
            .expect("layer 3");

        assert_eq!(step3.layer_count(), 3);
        assert_eq!(step3.layers[0].pattern, BannerPatternType::DiagonalLeft);
        assert_eq!(step3.layers[0].color, DyeColor::Red);
        assert_eq!(step3.layers[1].pattern, BannerPatternType::Circle);
        assert_eq!(step3.layers[1].color, DyeColor::Blue);
        assert_eq!(step3.layers[2].pattern, BannerPatternType::Border);
        assert_eq!(step3.layers[2].color, DyeColor::Green);
    }

    #[test]
    fn banner_new_creates_empty_banner() {
        let banner = Banner::new(DyeColor::Purple);
        assert_eq!(banner.base_color, DyeColor::Purple);
        assert!(banner.layers.is_empty());
        assert_eq!(banner.layer_count(), 0);
    }
}
