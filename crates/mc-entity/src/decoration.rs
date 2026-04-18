/// Painting variants and banner decorations for in-world entity placement.

// ---------------------------------------------------------------------------
// Painting
// ---------------------------------------------------------------------------

/// The 20 canonical Minecraft painting variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaintingVariant {
    Kebab,
    Aztec,
    Alban,
    Aztec2,
    Bomb,
    Plant,
    Wasteland,
    Wanderer,
    Graham,
    Pool,
    Courbet,
    Sunset,
    Sea,
    Creebet,
    Match,
    Bust,
    Stage,
    Void,
    SkullAndRoses,
    Fighters,
}

/// All painting variants in declaration order.
const ALL_PAINTINGS: [PaintingVariant; 20] = [
    PaintingVariant::Kebab,
    PaintingVariant::Aztec,
    PaintingVariant::Alban,
    PaintingVariant::Aztec2,
    PaintingVariant::Bomb,
    PaintingVariant::Plant,
    PaintingVariant::Wasteland,
    PaintingVariant::Wanderer,
    PaintingVariant::Graham,
    PaintingVariant::Pool,
    PaintingVariant::Courbet,
    PaintingVariant::Sunset,
    PaintingVariant::Sea,
    PaintingVariant::Creebet,
    PaintingVariant::Match,
    PaintingVariant::Bust,
    PaintingVariant::Stage,
    PaintingVariant::Void,
    PaintingVariant::SkullAndRoses,
    PaintingVariant::Fighters,
];

impl PaintingVariant {
    /// Returns the dimensions `(width, height)` of this painting in blocks.
    pub fn size(&self) -> (u8, u8) {
        match self {
            // 1x1
            Self::Kebab
            | Self::Aztec
            | Self::Alban
            | Self::Aztec2
            | Self::Bomb
            | Self::Plant
            | Self::Wasteland => (1, 1),
            // 1x2
            Self::Wanderer | Self::Graham => (1, 2),
            // 2x1
            Self::Pool | Self::Courbet => (2, 1),
            // 2x2
            Self::Sunset
            | Self::Sea
            | Self::Creebet
            | Self::Match
            | Self::Bust
            | Self::Stage
            | Self::Void
            | Self::SkullAndRoses => (2, 2),
            // 4x2
            Self::Fighters => (4, 2),
        }
    }
}

/// Simple hash-based pseudo-random number derived from `seed` and `index`.
fn pseudo_random(seed: u64, index: u64) -> u64 {
    let mut h = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(index);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// Choose a random painting that fits within the given `available_width` and
/// `available_height` (in blocks). Uses a deterministic `seed` for selection.
///
/// Falls back to `Kebab` (1x1) if no variant would fit — which in practice
/// cannot happen because multiple 1x1 paintings always fit for width >= 1 and
/// height >= 1.
pub fn choose_painting(available_width: u8, available_height: u8, seed: u64) -> PaintingVariant {
    let candidates: Vec<PaintingVariant> = ALL_PAINTINGS
        .iter()
        .copied()
        .filter(|p| {
            let (w, h) = p.size();
            w <= available_width && h <= available_height
        })
        .collect();

    if candidates.is_empty() {
        return PaintingVariant::Kebab;
    }

    let idx = pseudo_random(seed, candidates.len() as u64) as usize % candidates.len();
    candidates[idx]
}

// ---------------------------------------------------------------------------
// Banner
// ---------------------------------------------------------------------------

/// Maximum number of pattern layers on a single banner.
const MAX_BANNER_PATTERNS: usize = 6;

/// The 15 banner pattern types available for decoration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BannerPattern {
    Base,
    Stripe,
    Cross,
    Diagonal,
    HalfHorizontal,
    HalfVertical,
    Triangle,
    Rhombus,
    Circle,
    Border,
    Bricks,
    Gradient,
    Skull,
    Creeper,
    Flower,
}

/// A banner with a base colour and up to 6 pattern layers.
///
/// Each layer is a `(BannerPattern, colour)` pair where `colour` is a dye
/// colour index (0-15 in vanilla Minecraft).
#[derive(Debug, Clone, PartialEq)]
pub struct Banner {
    pub base_color: u8,
    pub patterns: Vec<(BannerPattern, u8)>,
}

impl Banner {
    /// Creates a new banner with the given base colour and no pattern layers.
    pub fn new(color: u8) -> Self {
        Self {
            base_color: color,
            patterns: Vec::new(),
        }
    }

    /// Adds a pattern layer to the banner.
    ///
    /// Returns `true` if the pattern was added, or `false` if the banner
    /// already has the maximum number of layers ([`MAX_BANNER_PATTERNS`]).
    pub fn add_pattern(&mut self, pattern: BannerPattern, color: u8) -> bool {
        if self.patterns.len() >= MAX_BANNER_PATTERNS {
            return false;
        }
        self.patterns.push((pattern, color));
        true
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Painting sizes -------------------------------------------------------

    #[test]
    fn painting_sizes_are_correct() {
        // 1x1
        assert_eq!(PaintingVariant::Kebab.size(), (1, 1));
        assert_eq!(PaintingVariant::Aztec.size(), (1, 1));
        assert_eq!(PaintingVariant::Alban.size(), (1, 1));
        assert_eq!(PaintingVariant::Aztec2.size(), (1, 1));
        assert_eq!(PaintingVariant::Bomb.size(), (1, 1));
        assert_eq!(PaintingVariant::Plant.size(), (1, 1));
        assert_eq!(PaintingVariant::Wasteland.size(), (1, 1));

        // 1x2
        assert_eq!(PaintingVariant::Wanderer.size(), (1, 2));
        assert_eq!(PaintingVariant::Graham.size(), (1, 2));

        // 2x1
        assert_eq!(PaintingVariant::Pool.size(), (2, 1));
        assert_eq!(PaintingVariant::Courbet.size(), (2, 1));

        // 2x2
        assert_eq!(PaintingVariant::Sunset.size(), (2, 2));
        assert_eq!(PaintingVariant::Sea.size(), (2, 2));
        assert_eq!(PaintingVariant::Creebet.size(), (2, 2));
        assert_eq!(PaintingVariant::Match.size(), (2, 2));
        assert_eq!(PaintingVariant::Bust.size(), (2, 2));
        assert_eq!(PaintingVariant::Stage.size(), (2, 2));
        assert_eq!(PaintingVariant::Void.size(), (2, 2));
        assert_eq!(PaintingVariant::SkullAndRoses.size(), (2, 2));

        // 4x2
        assert_eq!(PaintingVariant::Fighters.size(), (4, 2));
    }

    // -- choose_painting respects space ---------------------------------------

    #[test]
    fn choose_painting_fits_within_1x1() {
        for seed in 0..50 {
            let painting = choose_painting(1, 1, seed);
            let (w, h) = painting.size();
            assert!(
                w <= 1 && h <= 1,
                "painting {painting:?} does not fit in 1x1"
            );
        }
    }

    #[test]
    fn choose_painting_fits_within_2x2() {
        for seed in 0..50 {
            let painting = choose_painting(2, 2, seed);
            let (w, h) = painting.size();
            assert!(
                w <= 2 && h <= 2,
                "painting {painting:?} ({w}x{h}) does not fit in 2x2"
            );
        }
    }

    #[test]
    fn choose_painting_fits_within_4x4() {
        for seed in 0..50 {
            let painting = choose_painting(4, 4, seed);
            let (w, h) = painting.size();
            assert!(
                w <= 4 && h <= 4,
                "painting {painting:?} ({w}x{h}) does not fit in 4x4"
            );
        }
    }

    #[test]
    fn choose_painting_only_1x1_for_narrow_space() {
        // Width 1, height 1 — only 1x1 paintings should be chosen.
        for seed in 0..50 {
            let painting = choose_painting(1, 1, seed);
            assert_eq!(painting.size(), (1, 1));
        }
    }

    #[test]
    fn choose_painting_is_deterministic() {
        let a = choose_painting(4, 4, 42);
        let b = choose_painting(4, 4, 42);
        assert_eq!(a, b);
    }

    // -- Banner creation ------------------------------------------------------

    #[test]
    fn banner_creation_has_no_patterns() {
        let banner = Banner::new(0);
        assert_eq!(banner.base_color, 0);
        assert!(banner.patterns.is_empty());
    }

    #[test]
    fn banner_add_pattern_succeeds_within_limit() {
        let mut banner = Banner::new(1);
        for i in 0..MAX_BANNER_PATTERNS {
            assert!(
                banner.add_pattern(BannerPattern::Stripe, i as u8),
                "should accept pattern {i}"
            );
        }
        assert_eq!(banner.patterns.len(), MAX_BANNER_PATTERNS);
    }

    #[test]
    fn banner_rejects_pattern_beyond_limit() {
        let mut banner = Banner::new(14);
        for _ in 0..MAX_BANNER_PATTERNS {
            banner.add_pattern(BannerPattern::Base, 0);
        }
        assert!(
            !banner.add_pattern(BannerPattern::Skull, 15),
            "should reject 7th pattern"
        );
        assert_eq!(banner.patterns.len(), MAX_BANNER_PATTERNS);
    }

    #[test]
    fn banner_stores_patterns_in_order() {
        let mut banner = Banner::new(0);
        banner.add_pattern(BannerPattern::Cross, 1);
        banner.add_pattern(BannerPattern::Flower, 5);
        banner.add_pattern(BannerPattern::Gradient, 10);

        assert_eq!(
            banner.patterns,
            vec![
                (BannerPattern::Cross, 1),
                (BannerPattern::Flower, 5),
                (BannerPattern::Gradient, 10),
            ]
        );
    }
}
