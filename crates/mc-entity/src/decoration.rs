/// Banner decorations and painting placement for in-world entity placement.
///
/// For the full painting variant registry, see [`crate::painting`].

use crate::painting;

pub use painting::PaintingVariant;

// ---------------------------------------------------------------------------
// Painting placement (choose_painting)
// ---------------------------------------------------------------------------

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
    let candidates = painting::paintings_fitting(available_width, available_height);

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

    // -- Painting sizes (delegated to painting module) -------------------------

    #[test]
    fn painting_sizes_are_correct() {
        // 1x1
        assert_eq!(painting::painting_size(PaintingVariant::Kebab), (1, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Aztec), (1, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Alban), (1, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Aztec2), (1, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Bomb), (1, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Plant), (1, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Wasteland), (1, 1));

        // 2x1
        assert_eq!(painting::painting_size(PaintingVariant::Pool), (2, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Courbet), (2, 1));

        // 2x2
        assert_eq!(painting::painting_size(PaintingVariant::Wanderer), (2, 2));
        assert_eq!(painting::painting_size(PaintingVariant::Graham), (2, 2));
        assert_eq!(painting::painting_size(PaintingVariant::Sunset), (2, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Sea), (2, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Creebet), (2, 1));
        assert_eq!(painting::painting_size(PaintingVariant::Match), (2, 2));
        assert_eq!(painting::painting_size(PaintingVariant::Bust), (2, 2));
        assert_eq!(painting::painting_size(PaintingVariant::Stage), (2, 2));
        assert_eq!(painting::painting_size(PaintingVariant::Void), (2, 2));
        assert_eq!(painting::painting_size(PaintingVariant::SkullAndRoses), (2, 2));

        // 4x2
        assert_eq!(painting::painting_size(PaintingVariant::Fighters), (4, 2));
    }

    // -- choose_painting respects space ---------------------------------------

    #[test]
    fn choose_painting_fits_within_1x1() {
        for seed in 0..50 {
            let p = choose_painting(1, 1, seed);
            let (w, h) = painting::painting_size(p);
            assert!(
                w <= 1 && h <= 1,
                "painting {p:?} does not fit in 1x1"
            );
        }
    }

    #[test]
    fn choose_painting_fits_within_2x2() {
        for seed in 0..50 {
            let p = choose_painting(2, 2, seed);
            let (w, h) = painting::painting_size(p);
            assert!(
                w <= 2 && h <= 2,
                "painting {p:?} ({w}x{h}) does not fit in 2x2"
            );
        }
    }

    #[test]
    fn choose_painting_fits_within_4x4() {
        for seed in 0..50 {
            let p = choose_painting(4, 4, seed);
            let (w, h) = painting::painting_size(p);
            assert!(
                w <= 4 && h <= 4,
                "painting {p:?} ({w}x{h}) does not fit in 4x4"
            );
        }
    }

    #[test]
    fn choose_painting_only_1x1_for_narrow_space() {
        // Width 1, height 1 — only 1x1 paintings should be chosen.
        for seed in 0..50 {
            let p = choose_painting(1, 1, seed);
            assert_eq!(painting::painting_size(p), (1, 1));
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
