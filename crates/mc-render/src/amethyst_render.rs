//! Amethyst cluster rendering: growth stages, dimensions, and light levels.

/// Growth stages for amethyst clusters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmethystGrowth {
    Small,
    Medium,
    Large,
    Cluster,
}

/// Returns the model height for a given amethyst growth stage.
pub fn amethyst_model_height(growth: AmethystGrowth) -> f32 {
    match growth {
        AmethystGrowth::Small => 0.1875,
        AmethystGrowth::Medium => 0.25,
        AmethystGrowth::Large => 0.3125,
        AmethystGrowth::Cluster => 0.4375,
    }
}

/// Returns the model width for a given amethyst growth stage.
pub fn amethyst_model_width(growth: AmethystGrowth) -> f32 {
    match growth {
        AmethystGrowth::Small => 0.1875,
        AmethystGrowth::Medium => 0.3125,
        AmethystGrowth::Large => 0.3125,
        AmethystGrowth::Cluster => 0.4375,
    }
}

/// Returns the light level emitted by a given amethyst growth stage.
pub fn amethyst_light_level(growth: AmethystGrowth) -> u8 {
    match growth {
        AmethystGrowth::Small => 1,
        AmethystGrowth::Medium => 2,
        AmethystGrowth::Large => 4,
        AmethystGrowth::Cluster => 5,
    }
}

/// Returns the amethyst crystal color as an RGB array.
pub fn amethyst_color() -> [f32; 3] {
    [0.6, 0.3, 0.9]
}

/// Returns the chance per tick that a budding amethyst block will grow a cluster.
pub fn budding_amethyst_tick_chance() -> f32 {
    0.2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amethyst_model_height() {
        assert_eq!(amethyst_model_height(AmethystGrowth::Small), 0.1875);
        assert_eq!(amethyst_model_height(AmethystGrowth::Medium), 0.25);
        assert_eq!(amethyst_model_height(AmethystGrowth::Large), 0.3125);
        assert_eq!(amethyst_model_height(AmethystGrowth::Cluster), 0.4375);
    }

    #[test]
    fn test_amethyst_model_width() {
        assert_eq!(amethyst_model_width(AmethystGrowth::Small), 0.1875);
        assert_eq!(amethyst_model_width(AmethystGrowth::Medium), 0.3125);
        assert_eq!(amethyst_model_width(AmethystGrowth::Large), 0.3125);
        assert_eq!(amethyst_model_width(AmethystGrowth::Cluster), 0.4375);
    }

    #[test]
    fn test_amethyst_light_level() {
        assert_eq!(amethyst_light_level(AmethystGrowth::Small), 1);
        assert_eq!(amethyst_light_level(AmethystGrowth::Medium), 2);
        assert_eq!(amethyst_light_level(AmethystGrowth::Large), 4);
        assert_eq!(amethyst_light_level(AmethystGrowth::Cluster), 5);
    }

    #[test]
    fn test_amethyst_color() {
        assert_eq!(amethyst_color(), [0.6, 0.3, 0.9]);
    }

    #[test]
    fn test_budding_amethyst_tick_chance() {
        assert_eq!(budding_amethyst_tick_chance(), 0.2);
    }
}
