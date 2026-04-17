use crate::block::BlockId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BiomeId {
    Plains = 0,
    Forest,
    Desert,
    Ocean,
    Mountains,
    Taiga,
    Swamp,
    Jungle,
    Savanna,
    Tundra,
    BirchForest,
    DarkForest,
    Beach,
    River,
    MushroomIsland,
}

impl BiomeId {
    pub const COUNT: usize = 15;

    pub fn properties(self) -> &'static BiomeProperties {
        &BIOME_REGISTRY[self as usize]
    }
}

#[derive(Debug, Clone)]
pub struct BiomeProperties {
    pub name: &'static str,
    /// Temperature range: 0.0 (freezing) to 2.0 (scorching)
    pub temperature: f32,
    /// Humidity range: 0.0 (arid) to 1.0 (saturated)
    pub humidity: f32,
    /// Block used for the top surface layer
    pub surface_block: BlockId,
    /// Block used for the layers below the surface
    pub filler_block: BlockId,
    /// Block used for underwater surfaces
    pub underwater_block: BlockId,
    /// Tree density: 0.0 (none) to 1.0 (dense forest)
    pub tree_density: f32,
    /// Grass tint color as [R, G, B]
    pub grass_color: [u8; 3],
    /// Base terrain height in blocks
    pub base_height: f32,
    /// How much the terrain height varies
    pub height_variation: f32,
}

static BIOME_REGISTRY: [BiomeProperties; BiomeId::COUNT] = [
    // Plains
    BiomeProperties {
        name: "plains",
        temperature: 0.8,
        humidity: 0.4,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.005,
        grass_color: [124, 189, 107],
        base_height: 68.0,
        height_variation: 2.0,
    },
    // Forest
    BiomeProperties {
        name: "forest",
        temperature: 0.7,
        humidity: 0.8,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.6,
        grass_color: [89, 174, 67],
        base_height: 68.0,
        height_variation: 4.0,
    },
    // Desert
    BiomeProperties {
        name: "desert",
        temperature: 2.0,
        humidity: 0.0,
        surface_block: BlockId::Sand,
        filler_block: BlockId::Sand,
        underwater_block: BlockId::Sand,
        tree_density: 0.0,
        grass_color: [191, 183, 85],
        base_height: 68.0,
        height_variation: 1.5,
    },
    // Ocean
    BiomeProperties {
        name: "ocean",
        temperature: 0.5,
        humidity: 0.5,
        surface_block: BlockId::Gravel,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Gravel,
        tree_density: 0.0,
        grass_color: [113, 169, 93],
        base_height: 36.0,
        height_variation: 3.0,
    },
    // Mountains
    BiomeProperties {
        name: "mountains",
        temperature: 0.2,
        humidity: 0.3,
        surface_block: BlockId::Stone,
        filler_block: BlockId::Stone,
        underwater_block: BlockId::Gravel,
        tree_density: 0.05,
        grass_color: [140, 180, 130],
        base_height: 100.0,
        height_variation: 30.0,
    },
    // Taiga
    BiomeProperties {
        name: "taiga",
        temperature: 0.25,
        humidity: 0.8,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Gravel,
        tree_density: 0.5,
        grass_color: [104, 164, 101],
        base_height: 70.0,
        height_variation: 5.0,
    },
    // Swamp
    BiomeProperties {
        name: "swamp",
        temperature: 0.8,
        humidity: 0.9,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Dirt,
        tree_density: 0.2,
        grass_color: [106, 112, 57],
        base_height: 62.0,
        height_variation: 1.0,
    },
    // Jungle
    BiomeProperties {
        name: "jungle",
        temperature: 1.2,
        humidity: 0.9,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.9,
        grass_color: [59, 174, 31],
        base_height: 68.0,
        height_variation: 6.0,
    },
    // Savanna
    BiomeProperties {
        name: "savanna",
        temperature: 1.2,
        humidity: 0.0,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.03,
        grass_color: [174, 164, 42],
        base_height: 68.0,
        height_variation: 2.5,
    },
    // Tundra
    BiomeProperties {
        name: "tundra",
        temperature: 0.0,
        humidity: 0.5,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Gravel,
        tree_density: 0.0,
        grass_color: [128, 180, 151],
        base_height: 68.0,
        height_variation: 2.0,
    },
    // BirchForest
    BiomeProperties {
        name: "birch_forest",
        temperature: 0.6,
        humidity: 0.6,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.55,
        grass_color: [99, 179, 80],
        base_height: 68.0,
        height_variation: 3.5,
    },
    // DarkForest
    BiomeProperties {
        name: "dark_forest",
        temperature: 0.7,
        humidity: 0.8,
        surface_block: BlockId::GrassBlock,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.85,
        grass_color: [64, 120, 42],
        base_height: 68.0,
        height_variation: 3.0,
    },
    // Beach
    BiomeProperties {
        name: "beach",
        temperature: 0.8,
        humidity: 0.4,
        surface_block: BlockId::Sand,
        filler_block: BlockId::Sand,
        underwater_block: BlockId::Sand,
        tree_density: 0.0,
        grass_color: [124, 189, 107],
        base_height: 64.0,
        height_variation: 1.0,
    },
    // River
    BiomeProperties {
        name: "river",
        temperature: 0.5,
        humidity: 0.5,
        surface_block: BlockId::Sand,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.01,
        grass_color: [113, 169, 93],
        base_height: 58.0,
        height_variation: 1.5,
    },
    // MushroomIsland
    BiomeProperties {
        name: "mushroom_island",
        temperature: 0.9,
        humidity: 1.0,
        surface_block: BlockId::Dirt,
        filler_block: BlockId::Dirt,
        underwater_block: BlockId::Sand,
        tree_density: 0.0,
        grass_color: [85, 165, 75],
        base_height: 68.0,
        height_variation: 4.0,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_count_matches_variants() {
        // Ensure COUNT matches the number of registry entries
        assert_eq!(BIOME_REGISTRY.len(), BiomeId::COUNT);
    }

    #[test]
    fn every_biome_has_valid_temperature() {
        for props in &BIOME_REGISTRY {
            assert!(
                (0.0..=2.0).contains(&props.temperature),
                "biome '{}' has temperature {} outside [0.0, 2.0]",
                props.name,
                props.temperature,
            );
        }
    }

    #[test]
    fn every_biome_has_valid_humidity() {
        for props in &BIOME_REGISTRY {
            assert!(
                (0.0..=1.0).contains(&props.humidity),
                "biome '{}' has humidity {} outside [0.0, 1.0]",
                props.name,
                props.humidity,
            );
        }
    }

    #[test]
    fn every_biome_has_valid_tree_density() {
        for props in &BIOME_REGISTRY {
            assert!(
                (0.0..=1.0).contains(&props.tree_density),
                "biome '{}' has tree_density {} outside [0.0, 1.0]",
                props.name,
                props.tree_density,
            );
        }
    }

    #[test]
    fn every_biome_has_nonempty_name() {
        for props in &BIOME_REGISTRY {
            assert!(!props.name.is_empty(), "biome has empty name");
        }
    }

    #[test]
    fn desert_has_sand_surface() {
        let props = BiomeId::Desert.properties();
        assert_eq!(props.surface_block, BlockId::Sand);
        assert_eq!(props.filler_block, BlockId::Sand);
        assert_eq!(props.name, "desert");
    }

    #[test]
    fn desert_is_hot_and_dry() {
        let props = BiomeId::Desert.properties();
        assert!(
            props.temperature >= 1.5,
            "desert temperature {} should be >= 1.5",
            props.temperature,
        );
        assert!(
            props.humidity <= 0.1,
            "desert humidity {} should be <= 0.1",
            props.humidity,
        );
    }

    #[test]
    fn ocean_has_low_base_height() {
        let props = BiomeId::Ocean.properties();
        assert!(
            props.base_height < 50.0,
            "ocean base_height {} should be < 50.0",
            props.base_height,
        );
    }

    #[test]
    fn mountains_have_high_base_height() {
        let props = BiomeId::Mountains.properties();
        assert!(
            props.base_height >= 90.0,
            "mountains base_height {} should be >= 90.0",
            props.base_height,
        );
        assert!(
            props.height_variation >= 15.0,
            "mountains height_variation {} should be >= 15.0",
            props.height_variation,
        );
    }

    #[test]
    fn taiga_is_cold_and_wet() {
        let props = BiomeId::Taiga.properties();
        assert!(
            props.temperature <= 0.5,
            "taiga temperature {} should be <= 0.5",
            props.temperature,
        );
        assert!(
            props.humidity >= 0.5,
            "taiga humidity {} should be >= 0.5",
            props.humidity,
        );
    }

    #[test]
    fn forest_biomes_have_high_tree_density() {
        let forest_biomes = [BiomeId::Forest, BiomeId::BirchForest, BiomeId::DarkForest];
        for biome in &forest_biomes {
            let props = biome.properties();
            assert!(
                props.tree_density >= 0.4,
                "biome '{}' tree_density {} should be >= 0.4",
                props.name,
                props.tree_density,
            );
        }
    }

    #[test]
    fn jungle_has_highest_tree_density() {
        let jungle = BiomeId::Jungle.properties();
        for props in &BIOME_REGISTRY {
            assert!(
                jungle.tree_density >= props.tree_density,
                "jungle tree_density {} should be >= '{}' tree_density {}",
                jungle.tree_density,
                props.name,
                props.tree_density,
            );
        }
    }

    #[test]
    fn tundra_is_freezing() {
        let props = BiomeId::Tundra.properties();
        assert!(
            props.temperature <= 0.1,
            "tundra temperature {} should be <= 0.1",
            props.temperature,
        );
    }

    #[test]
    fn river_has_low_base_height() {
        let props = BiomeId::River.properties();
        assert!(
            props.base_height < 64.0,
            "river base_height {} should be < 64.0",
            props.base_height,
        );
    }

    #[test]
    fn beach_has_sand_surface() {
        let props = BiomeId::Beach.properties();
        assert_eq!(props.surface_block, BlockId::Sand);
    }

    #[test]
    fn properties_method_returns_correct_biome() {
        assert_eq!(BiomeId::Plains.properties().name, "plains");
        assert_eq!(BiomeId::Forest.properties().name, "forest");
        assert_eq!(BiomeId::Ocean.properties().name, "ocean");
        assert_eq!(BiomeId::MushroomIsland.properties().name, "mushroom_island");
    }
}
