//! Heavy core and mace mechanics from the 1.21 update.

/// Item ID for the mace weapon.
pub const MACE_ITEM_ID: u16 = 9200;

/// Item ID for the heavy core component.
pub const HEAVY_CORE_ID: u16 = 9201;

/// Item ID for the breeze rod component.
pub const BREEZE_ROD_ID: u16 = 9202;

/// Calculates mace damage based on fall distance.
/// Base damage is 5.0 plus 3.0 per block fallen.
pub fn mace_damage(fall_distance: f32) -> f32 {
    5.0 + 3.0 * fall_distance
}

/// Calculates the area-of-effect smash radius based on fall distance.
/// Minimum 1.5 blocks, scaling with fall distance, capped at 10.0.
pub fn mace_smash_radius(fall_distance: f32) -> f32 {
    (1.5 + fall_distance * 0.5).min(10.0)
}

/// Calculates wind burst enchantment knockback strength by level.
pub fn mace_wind_burst_knockback(level: u8) -> f32 {
    level as f32 * 1.5
}

/// Returns the crafting recipe for the mace: heavy core + breeze rod.
pub fn heavy_core_crafting() -> Vec<(u16, u8)> {
    vec![(HEAVY_CORE_ID, 1), (BREEZE_ROD_ID, 1)]
}

/// A successful mace attack resets fall damage.
pub fn mace_resets_fall_damage() -> bool {
    true
}

/// Returns the mace item ID.
pub fn mace_item_id() -> u16 {
    MACE_ITEM_ID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mace_damage_zero_fall() {
        assert_eq!(mace_damage(0.0), 5.0);
    }

    #[test]
    fn test_mace_damage_with_fall() {
        assert_eq!(mace_damage(10.0), 35.0);
    }

    #[test]
    fn test_mace_smash_radius_zero() {
        assert_eq!(mace_smash_radius(0.0), 1.5);
    }

    #[test]
    fn test_mace_smash_radius_capped() {
        assert_eq!(mace_smash_radius(100.0), 10.0);
    }

    #[test]
    fn test_mace_smash_radius_mid() {
        assert_eq!(mace_smash_radius(5.0), 4.0);
    }

    #[test]
    fn test_wind_burst_knockback() {
        assert_eq!(mace_wind_burst_knockback(0), 0.0);
        assert_eq!(mace_wind_burst_knockback(1), 1.5);
        assert_eq!(mace_wind_burst_knockback(3), 4.5);
    }

    #[test]
    fn test_heavy_core_crafting() {
        let recipe = heavy_core_crafting();
        assert_eq!(recipe.len(), 2);
        assert_eq!(recipe[0], (HEAVY_CORE_ID, 1));
        assert_eq!(recipe[1], (BREEZE_ROD_ID, 1));
    }

    #[test]
    fn test_mace_resets_fall_damage() {
        assert!(mace_resets_fall_damage());
    }

    #[test]
    fn test_mace_item_id() {
        assert_eq!(mace_item_id(), 9200);
    }
}
