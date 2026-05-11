//! Pointed dripstone mechanics: stalactites, stalagmites, dripping, and fall damage.

/// Direction a pointed dripstone block faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DripstoneDirection {
    Up,
    Down,
}

/// Thickness of a pointed dripstone segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DripstoneThickness {
    Tip,
    Frustum,
    Middle,
    Base,
    Merged,
}

/// Calculate fall damage from landing on a pointed dripstone.
///
/// Damage is `2 + 2 * height`, capped at 40.
pub fn dripstone_fall_damage(height: f32) -> f32 {
    (2.0 + 2.0 * height).min(40.0)
}

/// Whether a stalactite drips water this tick (1 in 6 chance).
pub fn dripstone_drip_water(has_water: bool, seed: u64) -> bool {
    has_water && seed % 6 == 0
}

/// Whether a stalactite drips lava this tick (1 in 6 chance).
pub fn dripstone_drip_lava(has_lava: bool, seed: u64) -> bool {
    has_lava && seed % 6 == 0
}

/// Maximum length of a stalactite in blocks.
pub const fn max_stalactite_length() -> u8 {
    7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fall_damage_scales_with_height() {
        assert_eq!(dripstone_fall_damage(0.0), 2.0);
        assert_eq!(dripstone_fall_damage(5.0), 12.0);
        assert_eq!(dripstone_fall_damage(10.0), 22.0);
    }

    #[test]
    fn fall_damage_caps_at_40() {
        assert_eq!(dripstone_fall_damage(19.0), 40.0);
        assert_eq!(dripstone_fall_damage(100.0), 40.0);
    }

    #[test]
    fn drip_water_requires_water_source() {
        assert!(!dripstone_drip_water(false, 0));
        assert!(!dripstone_drip_water(false, 6));
    }

    #[test]
    fn drip_water_one_in_six() {
        assert!(dripstone_drip_water(true, 0));
        assert!(dripstone_drip_water(true, 6));
        assert!(!dripstone_drip_water(true, 1));
        assert!(!dripstone_drip_water(true, 5));
    }

    #[test]
    fn drip_lava_requires_lava_source() {
        assert!(!dripstone_drip_lava(false, 0));
    }

    #[test]
    fn drip_lava_one_in_six() {
        assert!(dripstone_drip_lava(true, 12));
        assert!(!dripstone_drip_lava(true, 7));
    }

    #[test]
    fn max_stalactite_length_is_seven() {
        assert_eq!(max_stalactite_length(), 7);
    }

    #[test]
    fn direction_variants() {
        let up = DripstoneDirection::Up;
        let down = DripstoneDirection::Down;
        assert_ne!(up, down);
    }

    #[test]
    fn thickness_variants() {
        let variants = [
            DripstoneThickness::Tip,
            DripstoneThickness::Frustum,
            DripstoneThickness::Middle,
            DripstoneThickness::Base,
            DripstoneThickness::Merged,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                assert_eq!(i == j, a == b);
            }
        }
    }
}
