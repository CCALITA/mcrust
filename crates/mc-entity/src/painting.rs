/// Painting variant definitions and placement utilities.
///
/// Provides all 26 canonical Minecraft painting variants with size queries,
/// filtering by available space, and human-readable name lookup.

/// The 26 canonical Minecraft painting variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaintingVariant {
    // 1x1
    Kebab,
    Aztec,
    Alban,
    Aztec2,
    Bomb,
    Plant,
    Wasteland,
    // 2x1
    Pool,
    Courbet,
    Sea,
    Sunset,
    Creebet,
    // 2x2
    Wanderer,
    Graham,
    Match,
    Bust,
    Stage,
    Void,
    SkullAndRoses,
    Wither,
    // 4x2
    Fighters,
    // 4x3
    Skeleton,
    DonkeyKong,
    // 4x4
    Pointer,
    Pigscene,
    BurningSkull,
}

/// All painting variants in declaration order.
const ALL_PAINTINGS: [PaintingVariant; 26] = [
    PaintingVariant::Kebab,
    PaintingVariant::Aztec,
    PaintingVariant::Alban,
    PaintingVariant::Aztec2,
    PaintingVariant::Bomb,
    PaintingVariant::Plant,
    PaintingVariant::Wasteland,
    PaintingVariant::Pool,
    PaintingVariant::Courbet,
    PaintingVariant::Sea,
    PaintingVariant::Sunset,
    PaintingVariant::Creebet,
    PaintingVariant::Wanderer,
    PaintingVariant::Graham,
    PaintingVariant::Match,
    PaintingVariant::Bust,
    PaintingVariant::Stage,
    PaintingVariant::Void,
    PaintingVariant::SkullAndRoses,
    PaintingVariant::Wither,
    PaintingVariant::Fighters,
    PaintingVariant::Skeleton,
    PaintingVariant::DonkeyKong,
    PaintingVariant::Pointer,
    PaintingVariant::Pigscene,
    PaintingVariant::BurningSkull,
];

/// Returns the dimensions `(width, height)` of the given painting in blocks.
pub fn painting_size(variant: PaintingVariant) -> (u8, u8) {
    match variant {
        // 1x1
        PaintingVariant::Kebab
        | PaintingVariant::Aztec
        | PaintingVariant::Alban
        | PaintingVariant::Aztec2
        | PaintingVariant::Bomb
        | PaintingVariant::Plant
        | PaintingVariant::Wasteland => (1, 1),
        // 2x1
        PaintingVariant::Pool
        | PaintingVariant::Courbet
        | PaintingVariant::Sea
        | PaintingVariant::Sunset
        | PaintingVariant::Creebet => (2, 1),
        // 2x2
        PaintingVariant::Wanderer
        | PaintingVariant::Graham
        | PaintingVariant::Match
        | PaintingVariant::Bust
        | PaintingVariant::Stage
        | PaintingVariant::Void
        | PaintingVariant::SkullAndRoses
        | PaintingVariant::Wither => (2, 2),
        // 4x2
        PaintingVariant::Fighters => (4, 2),
        // 4x3
        PaintingVariant::Skeleton | PaintingVariant::DonkeyKong => (4, 3),
        // 4x4
        PaintingVariant::Pointer
        | PaintingVariant::Pigscene
        | PaintingVariant::BurningSkull => (4, 4),
    }
}

/// Returns all painting variants whose dimensions fit within `max_w` x `max_h`.
pub fn paintings_fitting(max_w: u8, max_h: u8) -> Vec<PaintingVariant> {
    ALL_PAINTINGS
        .iter()
        .copied()
        .filter(|p| {
            let (w, h) = painting_size(*p);
            w <= max_w && h <= max_h
        })
        .collect()
}

/// Returns the human-readable name of the painting variant.
pub fn painting_name(variant: PaintingVariant) -> &'static str {
    match variant {
        PaintingVariant::Kebab => "Kebab",
        PaintingVariant::Aztec => "Aztec",
        PaintingVariant::Alban => "Alban",
        PaintingVariant::Aztec2 => "Aztec2",
        PaintingVariant::Bomb => "Bomb",
        PaintingVariant::Plant => "Plant",
        PaintingVariant::Wasteland => "Wasteland",
        PaintingVariant::Pool => "Pool",
        PaintingVariant::Courbet => "Courbet",
        PaintingVariant::Sea => "Sea",
        PaintingVariant::Sunset => "Sunset",
        PaintingVariant::Creebet => "Creebet",
        PaintingVariant::Wanderer => "Wanderer",
        PaintingVariant::Graham => "Graham",
        PaintingVariant::Match => "Match",
        PaintingVariant::Bust => "Bust",
        PaintingVariant::Stage => "Stage",
        PaintingVariant::Void => "Void",
        PaintingVariant::SkullAndRoses => "SkullAndRoses",
        PaintingVariant::Wither => "Wither",
        PaintingVariant::Fighters => "Fighters",
        PaintingVariant::Skeleton => "Skeleton",
        PaintingVariant::DonkeyKong => "DonkeyKong",
        PaintingVariant::Pointer => "Pointer",
        PaintingVariant::Pigscene => "Pigscene",
        PaintingVariant::BurningSkull => "BurningSkull",
    }
}

/// Returns the total number of painting variants (26).
pub fn total_paintings() -> usize {
    ALL_PAINTINGS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_paintings_is_26() {
        assert_eq!(total_paintings(), 26);
    }

    // -- Size correctness per category ----------------------------------------

    #[test]
    fn sizes_1x1() {
        let variants_1x1 = [
            PaintingVariant::Kebab,
            PaintingVariant::Aztec,
            PaintingVariant::Alban,
            PaintingVariant::Aztec2,
            PaintingVariant::Bomb,
            PaintingVariant::Plant,
            PaintingVariant::Wasteland,
        ];
        for v in &variants_1x1 {
            assert_eq!(painting_size(*v), (1, 1), "{v:?} should be 1x1");
        }
    }

    #[test]
    fn sizes_2x1() {
        let variants_2x1 = [
            PaintingVariant::Pool,
            PaintingVariant::Courbet,
            PaintingVariant::Sea,
            PaintingVariant::Sunset,
            PaintingVariant::Creebet,
        ];
        for v in &variants_2x1 {
            assert_eq!(painting_size(*v), (2, 1), "{v:?} should be 2x1");
        }
    }

    #[test]
    fn sizes_2x2() {
        let variants_2x2 = [
            PaintingVariant::Wanderer,
            PaintingVariant::Graham,
            PaintingVariant::Match,
            PaintingVariant::Bust,
            PaintingVariant::Stage,
            PaintingVariant::Void,
            PaintingVariant::SkullAndRoses,
            PaintingVariant::Wither,
        ];
        for v in &variants_2x2 {
            assert_eq!(painting_size(*v), (2, 2), "{v:?} should be 2x2");
        }
    }

    #[test]
    fn sizes_4x2() {
        assert_eq!(
            painting_size(PaintingVariant::Fighters),
            (4, 2),
            "Fighters should be 4x2"
        );
    }

    #[test]
    fn sizes_4x3() {
        let variants_4x3 = [PaintingVariant::Skeleton, PaintingVariant::DonkeyKong];
        for v in &variants_4x3 {
            assert_eq!(painting_size(*v), (4, 3), "{v:?} should be 4x3");
        }
    }

    #[test]
    fn sizes_4x4() {
        let variants_4x4 = [
            PaintingVariant::Pointer,
            PaintingVariant::Pigscene,
            PaintingVariant::BurningSkull,
        ];
        for v in &variants_4x4 {
            assert_eq!(painting_size(*v), (4, 4), "{v:?} should be 4x4");
        }
    }

    // -- Fitting filter -------------------------------------------------------

    #[test]
    fn fitting_1x1_returns_only_1x1() {
        let fits = paintings_fitting(1, 1);
        assert_eq!(fits.len(), 7);
        for v in &fits {
            assert_eq!(painting_size(*v), (1, 1));
        }
    }

    #[test]
    fn fitting_2x1_returns_1x1_and_2x1() {
        let fits = paintings_fitting(2, 1);
        // 7 (1x1) + 5 (2x1)
        assert_eq!(fits.len(), 12);
        for v in &fits {
            let (w, h) = painting_size(*v);
            assert!(w <= 2 && h <= 1, "{v:?} ({w}x{h}) does not fit in 2x1");
        }
    }

    #[test]
    fn fitting_2x2_returns_1x1_2x1_and_2x2() {
        let fits = paintings_fitting(2, 2);
        // 7 + 5 + 8 = 20
        assert_eq!(fits.len(), 20);
        for v in &fits {
            let (w, h) = painting_size(*v);
            assert!(w <= 2 && h <= 2, "{v:?} ({w}x{h}) does not fit in 2x2");
        }
    }

    #[test]
    fn fitting_4x4_returns_all() {
        let fits = paintings_fitting(4, 4);
        assert_eq!(fits.len(), 26);
    }

    #[test]
    fn fitting_4x2_excludes_4x3_and_4x4() {
        let fits = paintings_fitting(4, 2);
        // 7 + 5 + 8 + 1 = 21
        assert_eq!(fits.len(), 21);
        for v in &fits {
            let (w, h) = painting_size(*v);
            assert!(w <= 4 && h <= 2, "{v:?} ({w}x{h}) does not fit in 4x2");
        }
    }

    #[test]
    fn fitting_0x0_returns_empty() {
        let fits = paintings_fitting(0, 0);
        assert!(fits.is_empty());
    }

    // -- Name lookup ----------------------------------------------------------

    #[test]
    fn name_lookup_all_variants() {
        assert_eq!(painting_name(PaintingVariant::Kebab), "Kebab");
        assert_eq!(painting_name(PaintingVariant::Aztec), "Aztec");
        assert_eq!(painting_name(PaintingVariant::Alban), "Alban");
        assert_eq!(painting_name(PaintingVariant::Aztec2), "Aztec2");
        assert_eq!(painting_name(PaintingVariant::Bomb), "Bomb");
        assert_eq!(painting_name(PaintingVariant::Plant), "Plant");
        assert_eq!(painting_name(PaintingVariant::Wasteland), "Wasteland");
        assert_eq!(painting_name(PaintingVariant::Pool), "Pool");
        assert_eq!(painting_name(PaintingVariant::Courbet), "Courbet");
        assert_eq!(painting_name(PaintingVariant::Sea), "Sea");
        assert_eq!(painting_name(PaintingVariant::Sunset), "Sunset");
        assert_eq!(painting_name(PaintingVariant::Creebet), "Creebet");
        assert_eq!(painting_name(PaintingVariant::Wanderer), "Wanderer");
        assert_eq!(painting_name(PaintingVariant::Graham), "Graham");
        assert_eq!(painting_name(PaintingVariant::Match), "Match");
        assert_eq!(painting_name(PaintingVariant::Bust), "Bust");
        assert_eq!(painting_name(PaintingVariant::Stage), "Stage");
        assert_eq!(painting_name(PaintingVariant::Void), "Void");
        assert_eq!(painting_name(PaintingVariant::SkullAndRoses), "SkullAndRoses");
        assert_eq!(painting_name(PaintingVariant::Wither), "Wither");
        assert_eq!(painting_name(PaintingVariant::Fighters), "Fighters");
        assert_eq!(painting_name(PaintingVariant::Skeleton), "Skeleton");
        assert_eq!(painting_name(PaintingVariant::DonkeyKong), "DonkeyKong");
        assert_eq!(painting_name(PaintingVariant::Pointer), "Pointer");
        assert_eq!(painting_name(PaintingVariant::Pigscene), "Pigscene");
        assert_eq!(painting_name(PaintingVariant::BurningSkull), "BurningSkull");
    }
}
