//! Biome color blending for smooth grass, foliage, and water color transitions.
//!
//! Samples neighboring biome colors in a grid around a position and averages
//! them to produce smooth visual transitions at biome boundaries.

use mc_core::biome::BiomeId;

/// Default blend radius — samples a 5x5 grid (2*2+1 = 5).
pub const DEFAULT_BLEND_RADIUS: u8 = 2;

/// Per-biome color triplet for grass, foliage, and water tinting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeColors {
    pub grass: [f32; 3],
    pub foliage: [f32; 3],
    pub water: [f32; 3],
}

/// Return the hardcoded base colors for a given biome.
pub fn biome_base_colors(biome: BiomeId) -> BiomeColors {
    match biome {
        BiomeId::Plains => BiomeColors {
            grass: [0.56, 0.73, 0.35],
            foliage: [0.45, 0.60, 0.28],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::Forest => BiomeColors {
            grass: [0.34, 0.65, 0.22],
            foliage: [0.28, 0.55, 0.18],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::Desert => BiomeColors {
            grass: [0.75, 0.72, 0.36],
            foliage: [0.68, 0.65, 0.30],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::Ocean => BiomeColors {
            grass: [0.56, 0.73, 0.35],
            foliage: [0.45, 0.60, 0.28],
            water: [0.24, 0.35, 0.76],
        },
        BiomeId::Mountains => BiomeColors {
            grass: [0.50, 0.68, 0.40],
            foliage: [0.42, 0.58, 0.34],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::Taiga => BiomeColors {
            grass: [0.42, 0.63, 0.38],
            foliage: [0.35, 0.55, 0.32],
            water: [0.20, 0.40, 0.60],
        },
        BiomeId::Swamp => BiomeColors {
            grass: [0.41, 0.55, 0.24],
            foliage: [0.38, 0.50, 0.20],
            water: [0.38, 0.53, 0.30],
        },
        BiomeId::Jungle => BiomeColors {
            grass: [0.23, 0.68, 0.12],
            foliage: [0.18, 0.58, 0.10],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::Savanna => BiomeColors {
            grass: [0.68, 0.66, 0.20],
            foliage: [0.60, 0.58, 0.18],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::Tundra => BiomeColors {
            grass: [0.50, 0.70, 0.58],
            foliage: [0.42, 0.62, 0.50],
            water: [0.18, 0.38, 0.58],
        },
        BiomeId::BirchForest => BiomeColors {
            grass: [0.38, 0.70, 0.30],
            foliage: [0.32, 0.60, 0.25],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::DarkForest => BiomeColors {
            grass: [0.25, 0.47, 0.16],
            foliage: [0.20, 0.40, 0.14],
            water: [0.20, 0.40, 0.58],
        },
        BiomeId::Beach => BiomeColors {
            grass: [0.56, 0.73, 0.35],
            foliage: [0.45, 0.60, 0.28],
            water: [0.24, 0.45, 0.65],
        },
        BiomeId::River => BiomeColors {
            grass: [0.56, 0.73, 0.35],
            foliage: [0.45, 0.60, 0.28],
            water: [0.22, 0.42, 0.70],
        },
        BiomeId::MushroomIsland => BiomeColors {
            grass: [0.33, 0.65, 0.29],
            foliage: [0.28, 0.55, 0.24],
            water: [0.24, 0.45, 0.65],
        },
    }
}

/// Blend biome colors by averaging all biome colors in a `(2*radius+1)^2` grid
/// centered on `(x, z)`.
///
/// `get_biome` is called for every sample position to look up the biome at that
/// coordinate. The returned colors are the component-wise average of all sampled
/// biome base colors.
pub fn blend_biome_colors(
    x: i32,
    z: i32,
    radius: u8,
    get_biome: &impl Fn(i32, i32) -> BiomeId,
) -> BiomeColors {
    let r = i32::from(radius);
    let count = (2 * r + 1) * (2 * r + 1);

    let mut grass = [0.0_f32; 3];
    let mut foliage = [0.0_f32; 3];
    let mut water = [0.0_f32; 3];

    for dz in -r..=r {
        for dx in -r..=r {
            let biome = get_biome(x + dx, z + dz);
            let colors = biome_base_colors(biome);

            for i in 0..3 {
                grass[i] += colors.grass[i];
                foliage[i] += colors.foliage[i];
                water[i] += colors.water[i];
            }
        }
    }

    let inv = 1.0 / count as f32;
    for i in 0..3 {
        grass[i] *= inv;
        foliage[i] *= inv;
        water[i] *= inv;
    }

    BiomeColors {
        grass,
        foliage,
        water,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: assert two `[f32; 3]` arrays are approximately equal.
    fn assert_color_approx(actual: [f32; 3], expected: [f32; 3], label: &str) {
        for i in 0..3 {
            assert!(
                (actual[i] - expected[i]).abs() < 1e-5,
                "{label}[{i}]: expected {}, got {}",
                expected[i],
                actual[i],
            );
        }
    }

    #[test]
    fn single_biome_returns_exact_colors() {
        let colors = biome_base_colors(BiomeId::Plains);
        assert_eq!(colors.grass, [0.56, 0.73, 0.35]);
    }

    #[test]
    fn desert_has_expected_grass_color() {
        let colors = biome_base_colors(BiomeId::Desert);
        assert_eq!(colors.grass, [0.75, 0.72, 0.36]);
    }

    #[test]
    fn forest_has_expected_grass_color() {
        let colors = biome_base_colors(BiomeId::Forest);
        assert_eq!(colors.grass, [0.34, 0.65, 0.22]);
    }

    #[test]
    fn ocean_has_expected_water_color() {
        let colors = biome_base_colors(BiomeId::Ocean);
        assert_eq!(colors.water, [0.24, 0.35, 0.76]);
    }

    #[test]
    fn swamp_has_expected_water_color() {
        let colors = biome_base_colors(BiomeId::Swamp);
        assert_eq!(colors.water, [0.38, 0.53, 0.30]);
    }

    #[test]
    fn radius_zero_returns_exact_colors() {
        let plains = biome_base_colors(BiomeId::Plains);
        let blended = blend_biome_colors(0, 0, 0, &|_, _| BiomeId::Plains);

        assert_color_approx(blended.grass, plains.grass, "grass");
        assert_color_approx(blended.foliage, plains.foliage, "foliage");
        assert_color_approx(blended.water, plains.water, "water");
    }

    #[test]
    fn uniform_biome_blend_returns_exact_colors() {
        let forest = biome_base_colors(BiomeId::Forest);
        let blended = blend_biome_colors(5, 5, DEFAULT_BLEND_RADIUS, &|_, _| BiomeId::Forest);

        assert_color_approx(blended.grass, forest.grass, "grass");
        assert_color_approx(blended.foliage, forest.foliage, "foliage");
        assert_color_approx(blended.water, forest.water, "water");
    }

    #[test]
    fn blend_of_two_biomes_averages_colors() {
        // With radius=0 and a single point, just test that two calls average.
        // Use radius=0 with a custom closure that returns Plains for the center.
        let plains = biome_base_colors(BiomeId::Plains);
        let desert = biome_base_colors(BiomeId::Desert);

        // Create a 3x1 strip: radius=1 in z only isn't possible with a square
        // grid, so use radius=0 for one biome and manually verify averaging.
        // Instead: use a 3x3 grid (radius=1) where all 9 cells alternate.
        // Simpler: half plains, half desert. With radius=1, 3x3=9 cells.
        // Place the boundary at x=0: x<0 => Plains, x>=0 => Desert.
        // dx in [-1,0,1], dz in [-1,0,1]: dx=-1 => Plains (3 cells), dx=0,1 => Desert (6 cells)
        let blended = blend_biome_colors(0, 0, 1, &|bx, _| {
            if bx < 0 {
                BiomeId::Plains
            } else {
                BiomeId::Desert
            }
        });

        // 3 Plains + 6 Desert out of 9
        let expected_grass = [
            (plains.grass[0] * 3.0 + desert.grass[0] * 6.0) / 9.0,
            (plains.grass[1] * 3.0 + desert.grass[1] * 6.0) / 9.0,
            (plains.grass[2] * 3.0 + desert.grass[2] * 6.0) / 9.0,
        ];
        assert_color_approx(blended.grass, expected_grass, "grass");
    }

    #[test]
    fn blend_two_biomes_even_split() {
        // 2x2 grid impossible with odd side lengths, so use a 1x1 grid (radius=0)
        // where each cell is a single biome — just verifying the math with a
        // symmetric split. Use a 3x3 grid where the center column is different.
        let plains = biome_base_colors(BiomeId::Plains);
        let ocean = biome_base_colors(BiomeId::Ocean);

        // 5x5 grid (radius=2), center column (dx=0) => Ocean (5 cells),
        // rest => Plains (20 cells).
        let blended = blend_biome_colors(0, 0, 2, &|bx, _| {
            if bx == 0 {
                BiomeId::Ocean
            } else {
                BiomeId::Plains
            }
        });

        let expected_water = [
            (plains.water[0] * 20.0 + ocean.water[0] * 5.0) / 25.0,
            (plains.water[1] * 20.0 + ocean.water[1] * 5.0) / 25.0,
            (plains.water[2] * 20.0 + ocean.water[2] * 5.0) / 25.0,
        ];
        assert_color_approx(blended.water, expected_water, "water");
    }

    #[test]
    fn default_blend_radius_is_two() {
        assert_eq!(DEFAULT_BLEND_RADIUS, 2);
    }

    #[test]
    fn all_biomes_have_valid_color_range() {
        let biomes = [
            BiomeId::Plains,
            BiomeId::Forest,
            BiomeId::Desert,
            BiomeId::Ocean,
            BiomeId::Mountains,
            BiomeId::Taiga,
            BiomeId::Swamp,
            BiomeId::Jungle,
            BiomeId::Savanna,
            BiomeId::Tundra,
            BiomeId::BirchForest,
            BiomeId::DarkForest,
            BiomeId::Beach,
            BiomeId::River,
            BiomeId::MushroomIsland,
        ];
        for biome in biomes {
            let colors = biome_base_colors(biome);
            for (label, channel) in [
                ("grass", colors.grass),
                ("foliage", colors.foliage),
                ("water", colors.water),
            ] {
                for (i, val) in channel.iter().enumerate() {
                    assert!(
                        (0.0..=1.0).contains(val),
                        "{biome:?} {label}[{i}] = {val} is outside [0.0, 1.0]",
                    );
                }
            }
        }
    }

    #[test]
    fn blend_respects_position_offset() {
        use std::cell::RefCell;
        // Ensure the closure receives offset positions, not just (0,0).
        let called_positions = RefCell::new(Vec::new());
        let _ = blend_biome_colors(10, 20, 0, &|bx, bz| {
            called_positions.borrow_mut().push((bx, bz));
            BiomeId::Plains
        });
        assert_eq!(called_positions.into_inner(), vec![(10, 20)]);
    }

    #[test]
    fn blend_radius_one_samples_nine_positions() {
        use std::cell::Cell;
        let count = Cell::new(0u32);
        let _ = blend_biome_colors(0, 0, 1, &|_, _| {
            count.set(count.get() + 1);
            BiomeId::Plains
        });
        assert_eq!(count.get(), 9);
    }

    #[test]
    fn blend_default_radius_samples_25_positions() {
        use std::cell::Cell;
        let count = Cell::new(0u32);
        let _ = blend_biome_colors(0, 0, DEFAULT_BLEND_RADIUS, &|_, _| {
            count.set(count.get() + 1);
            BiomeId::Plains
        });
        assert_eq!(count.get(), 25);
    }
}
