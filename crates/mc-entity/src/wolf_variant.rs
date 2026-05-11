//! Wolf variant system for biome-specific wolf appearances.

/// Wolf variants corresponding to different biomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WolfVariant {
    Plains,
    Taiga,
    Snowy,
    Jungle,
    Savanna,
    Forest,
    Birch,
    DarkForest,
    Swamp,
}

/// Returns the wolf variant appropriate for the given biome ID.
pub fn wolf_variant_for_biome(biome_id: u8) -> WolfVariant {
    match biome_id {
        1 => WolfVariant::Plains,
        5 => WolfVariant::Taiga,
        12 => WolfVariant::Snowy,
        21 => WolfVariant::Jungle,
        35 => WolfVariant::Savanna,
        4 => WolfVariant::Forest,
        27 => WolfVariant::Birch,
        29 => WolfVariant::DarkForest,
        6 => WolfVariant::Swamp,
        _ => WolfVariant::Plains,
    }
}

/// Returns the texture ID for a given wolf variant.
pub fn wolf_variant_texture_id(variant: WolfVariant) -> u16 {
    match variant {
        WolfVariant::Plains => 0,
        WolfVariant::Taiga => 1,
        WolfVariant::Snowy => 2,
        WolfVariant::Jungle => 3,
        WolfVariant::Savanna => 4,
        WolfVariant::Forest => 5,
        WolfVariant::Birch => 6,
        WolfVariant::DarkForest => 7,
        WolfVariant::Swamp => 8,
    }
}

/// Returns the default collar color for tamed wolves (red = 14).
pub fn wolf_collar_color_default() -> u8 {
    14
}

/// Returns the RGB color for the wolf anger particle effect.
pub fn wolf_anger_particle_color() -> [f32; 3] {
    [1.0, 0.0, 0.0]
}

/// Returns the total number of wolf variants.
pub fn total_wolf_variants() -> usize {
    9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_for_known_biomes() {
        assert_eq!(wolf_variant_for_biome(1), WolfVariant::Plains);
        assert_eq!(wolf_variant_for_biome(5), WolfVariant::Taiga);
        assert_eq!(wolf_variant_for_biome(12), WolfVariant::Snowy);
        assert_eq!(wolf_variant_for_biome(21), WolfVariant::Jungle);
        assert_eq!(wolf_variant_for_biome(35), WolfVariant::Savanna);
        assert_eq!(wolf_variant_for_biome(4), WolfVariant::Forest);
        assert_eq!(wolf_variant_for_biome(27), WolfVariant::Birch);
        assert_eq!(wolf_variant_for_biome(29), WolfVariant::DarkForest);
        assert_eq!(wolf_variant_for_biome(6), WolfVariant::Swamp);
    }

    #[test]
    fn variant_for_unknown_biome_defaults_to_plains() {
        assert_eq!(wolf_variant_for_biome(255), WolfVariant::Plains);
        assert_eq!(wolf_variant_for_biome(0), WolfVariant::Plains);
    }

    #[test]
    fn texture_ids_are_unique_and_sequential() {
        let variants = [
            WolfVariant::Plains,
            WolfVariant::Taiga,
            WolfVariant::Snowy,
            WolfVariant::Jungle,
            WolfVariant::Savanna,
            WolfVariant::Forest,
            WolfVariant::Birch,
            WolfVariant::DarkForest,
            WolfVariant::Swamp,
        ];
        for (i, variant) in variants.iter().enumerate() {
            assert_eq!(wolf_variant_texture_id(*variant), i as u16);
        }
    }

    #[test]
    fn default_collar_color_is_red() {
        assert_eq!(wolf_collar_color_default(), 14);
    }

    #[test]
    fn anger_particle_color_is_red() {
        assert_eq!(wolf_anger_particle_color(), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn total_variants_is_nine() {
        assert_eq!(total_wolf_variants(), 9);
    }
}
