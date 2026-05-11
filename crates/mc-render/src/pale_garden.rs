//! Pale garden block rendering properties.

/// Pale garden block types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaleGardenBlock {
    PaleMossCarpet,
    PaleMossBlock,
    PaleHangingMoss,
    PaleOakLeaves,
}

/// Returns the tint color for pale garden blocks.
pub fn pale_block_color(_block: PaleGardenBlock) -> [f32; 3] {
    [0.6, 0.65, 0.55]
}

/// Returns the height of a pale moss carpet block (1 pixel = 1/16).
pub fn pale_moss_carpet_height() -> f32 {
    0.0625
}

/// Returns the visual length of pale hanging moss based on age (1–4 blocks).
pub fn pale_hanging_moss_length(age: u8) -> f32 {
    (age.clamp(1, 4)) as f32
}

/// Returns the ambient light color for the pale garden biome.
pub fn pale_garden_ambient_color() -> [f32; 3] {
    [0.5, 0.55, 0.5]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pale_block_color_all_variants() {
        for block in [
            PaleGardenBlock::PaleMossCarpet,
            PaleGardenBlock::PaleMossBlock,
            PaleGardenBlock::PaleHangingMoss,
            PaleGardenBlock::PaleOakLeaves,
        ] {
            assert_eq!(pale_block_color(block), [0.6, 0.65, 0.55]);
        }
    }

    #[test]
    fn test_pale_moss_carpet_height() {
        assert_eq!(pale_moss_carpet_height(), 0.0625);
    }

    #[test]
    fn test_pale_hanging_moss_length() {
        assert_eq!(pale_hanging_moss_length(0), 1.0);
        assert_eq!(pale_hanging_moss_length(1), 1.0);
        assert_eq!(pale_hanging_moss_length(2), 2.0);
        assert_eq!(pale_hanging_moss_length(3), 3.0);
        assert_eq!(pale_hanging_moss_length(4), 4.0);
        assert_eq!(pale_hanging_moss_length(5), 4.0);
    }

    #[test]
    fn test_pale_garden_ambient_color() {
        assert_eq!(pale_garden_ambient_color(), [0.5, 0.55, 0.5]);
    }
}
