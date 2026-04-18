use mc_core::block::BlockId;

/// The block ID used as a placeholder for lava until a dedicated `Lava`
/// variant is added to `BlockId`.  Netherrack is chosen because it is
/// thematically linked to the Nether / lava lakes.
const LAVA_PLACEHOLDER: u16 = BlockId::Netherrack as u16;

/// What a bucket currently holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketContents {
    Empty,
    Water,
    Lava,
    Milk,
    PowderSnow,
}

/// Outcome of using a bucket on a block or entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketResult {
    /// A fluid was placed in the world; carries the block ID that was set.
    PlacedFluid(u16),
    /// A fluid was picked up from the world.
    PickedUpFluid(BucketContents),
    /// A cow (or mooshroom) was milked.
    Milked,
    /// The action had no effect.
    Failed,
}

/// Entity kind constant for cows (used by `use_bucket_on_entity`).
const COW_ENTITY_KIND: u8 = 1;

/// Use a bucket on a target block.
///
/// # Rules
/// - Empty bucket + Water block -> pick up water
/// - Empty bucket + Lava-like block (placeholder) -> pick up lava
/// - Water bucket + Air -> place water
/// - Lava bucket + Air -> place lava (placeholder)
/// - Everything else -> Failed
pub fn use_bucket_on_block(contents: BucketContents, target_block: u16) -> BucketResult {
    match contents {
        BucketContents::Empty => {
            if target_block == BlockId::Water as u16 {
                BucketResult::PickedUpFluid(BucketContents::Water)
            } else if target_block == LAVA_PLACEHOLDER {
                BucketResult::PickedUpFluid(BucketContents::Lava)
            } else {
                BucketResult::Failed
            }
        }
        BucketContents::Water => {
            if target_block == BlockId::Air as u16 {
                BucketResult::PlacedFluid(BlockId::Water as u16)
            } else {
                BucketResult::Failed
            }
        }
        BucketContents::Lava => {
            if target_block == BlockId::Air as u16 {
                BucketResult::PlacedFluid(LAVA_PLACEHOLDER)
            } else {
                BucketResult::Failed
            }
        }
        _ => BucketResult::Failed,
    }
}

/// Use a bucket on an entity.
///
/// Currently only milking is supported: an empty bucket used on a cow
/// (`entity_kind == 1`) returns `Milked`.
pub fn use_bucket_on_entity(contents: BucketContents, entity_kind: u8) -> BucketResult {
    if contents == BucketContents::Empty && entity_kind == COW_ENTITY_KIND {
        BucketResult::Milked
    } else {
        BucketResult::Failed
    }
}

/// Drinking milk clears all status effects.
///
/// Returns `true` to signal that every active effect should be removed.
pub fn milk_effects() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pickup_water_with_empty_bucket() {
        let result = use_bucket_on_block(BucketContents::Empty, BlockId::Water as u16);
        assert_eq!(result, BucketResult::PickedUpFluid(BucketContents::Water));
    }

    #[test]
    fn place_water_on_air() {
        let result = use_bucket_on_block(BucketContents::Water, BlockId::Air as u16);
        assert_eq!(result, BucketResult::PlacedFluid(BlockId::Water as u16));
    }

    #[test]
    fn milk_cow_with_empty_bucket() {
        let result = use_bucket_on_entity(BucketContents::Empty, COW_ENTITY_KIND);
        assert_eq!(result, BucketResult::Milked);
    }

    #[test]
    fn empty_bucket_on_solid_block_fails() {
        let result = use_bucket_on_block(BucketContents::Empty, BlockId::Stone as u16);
        assert_eq!(result, BucketResult::Failed);
    }

    #[test]
    fn pickup_lava_with_empty_bucket() {
        let result = use_bucket_on_block(BucketContents::Empty, LAVA_PLACEHOLDER);
        assert_eq!(result, BucketResult::PickedUpFluid(BucketContents::Lava));
    }

    #[test]
    fn place_lava_on_air() {
        let result = use_bucket_on_block(BucketContents::Lava, BlockId::Air as u16);
        assert_eq!(result, BucketResult::PlacedFluid(LAVA_PLACEHOLDER));
    }

    #[test]
    fn water_bucket_on_solid_block_fails() {
        let result = use_bucket_on_block(BucketContents::Water, BlockId::Stone as u16);
        assert_eq!(result, BucketResult::Failed);
    }

    #[test]
    fn lava_bucket_on_solid_block_fails() {
        let result = use_bucket_on_block(BucketContents::Lava, BlockId::Stone as u16);
        assert_eq!(result, BucketResult::Failed);
    }

    #[test]
    fn milk_bucket_on_block_fails() {
        let result = use_bucket_on_block(BucketContents::Milk, BlockId::Air as u16);
        assert_eq!(result, BucketResult::Failed);
    }

    #[test]
    fn powder_snow_bucket_on_block_fails() {
        let result = use_bucket_on_block(BucketContents::PowderSnow, BlockId::Air as u16);
        assert_eq!(result, BucketResult::Failed);
    }

    #[test]
    fn empty_bucket_on_non_cow_entity_fails() {
        let result = use_bucket_on_entity(BucketContents::Empty, 99);
        assert_eq!(result, BucketResult::Failed);
    }

    #[test]
    fn water_bucket_on_cow_fails() {
        let result = use_bucket_on_entity(BucketContents::Water, COW_ENTITY_KIND);
        assert_eq!(result, BucketResult::Failed);
    }

    #[test]
    fn milk_clears_all_effects() {
        assert!(milk_effects());
    }
}
