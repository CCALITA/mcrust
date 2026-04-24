//! Bamboo block types, properties, and growth mechanics.
//!
//! Implements the bamboo wood family introduced in Minecraft 1.20,
//! including all derivative block types (planks, stairs, slabs, etc.),
//! their hardness values, flammability, and bamboo plant growth stages.

/// All block types in the bamboo wood family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BambooBlockType {
    Block,
    Planks,
    Mosaic,
    Stairs,
    Slab,
    FenceGate,
    Door,
    Trapdoor,
    Button,
    PressurePlate,
    Sign,
    HangingSign,
    Raft,
}

impl BambooBlockType {
    /// Returns the display name of this bamboo block type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Block => "Bamboo Block",
            Self::Planks => "Bamboo Planks",
            Self::Mosaic => "Bamboo Mosaic",
            Self::Stairs => "Bamboo Stairs",
            Self::Slab => "Bamboo Slab",
            Self::FenceGate => "Bamboo Fence Gate",
            Self::Door => "Bamboo Door",
            Self::Trapdoor => "Bamboo Trapdoor",
            Self::Button => "Bamboo Button",
            Self::PressurePlate => "Bamboo Pressure Plate",
            Self::Sign => "Bamboo Sign",
            Self::HangingSign => "Bamboo Hanging Sign",
            Self::Raft => "Bamboo Raft",
        }
    }
}

/// Returns the hardness value for a bamboo block type.
///
/// Structural blocks (Block, Planks, Mosaic, Stairs, Slab) have a hardness of 2.0.
/// All other bamboo blocks have a hardness of 3.0.
pub fn bamboo_block_hardness(block: BambooBlockType) -> f32 {
    match block {
        BambooBlockType::Block
        | BambooBlockType::Planks
        | BambooBlockType::Mosaic
        | BambooBlockType::Stairs
        | BambooBlockType::Slab => 2.0,
        _ => 3.0,
    }
}

/// Returns whether a bamboo block type is flammable.
///
/// All bamboo blocks are wood-based and therefore flammable.
pub fn bamboo_is_flammable(_block: BambooBlockType) -> bool {
    true
}

/// Returns the number of growth stages for a bamboo plant.
pub fn bamboo_growth_stages() -> u8 {
    3
}

/// Returns whether a bamboo shoot should transition to a stalk.
///
/// A bamboo shoot becomes a stalk when its age reaches or exceeds 3.
pub fn bamboo_shoot_to_stalk(age: u8) -> bool {
    age >= 3
}

/// Returns the minimum and maximum height range for bamboo plants.
///
/// Bamboo can grow between 12 and 16 blocks tall.
pub fn bamboo_max_height_range() -> (u8, u8) {
    (12, 16)
}

/// Returns the total number of bamboo block types.
pub fn total_bamboo_block_types() -> usize {
    13
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- BambooBlockType::name -------------------------------------------------

    #[test]
    fn block_name_is_bamboo_block() {
        assert_eq!(BambooBlockType::Block.name(), "Bamboo Block");
    }

    #[test]
    fn planks_name_is_bamboo_planks() {
        assert_eq!(BambooBlockType::Planks.name(), "Bamboo Planks");
    }

    #[test]
    fn mosaic_name_is_bamboo_mosaic() {
        assert_eq!(BambooBlockType::Mosaic.name(), "Bamboo Mosaic");
    }

    #[test]
    fn stairs_name_is_bamboo_stairs() {
        assert_eq!(BambooBlockType::Stairs.name(), "Bamboo Stairs");
    }

    #[test]
    fn slab_name_is_bamboo_slab() {
        assert_eq!(BambooBlockType::Slab.name(), "Bamboo Slab");
    }

    #[test]
    fn fence_gate_name_is_bamboo_fence_gate() {
        assert_eq!(BambooBlockType::FenceGate.name(), "Bamboo Fence Gate");
    }

    #[test]
    fn door_name_is_bamboo_door() {
        assert_eq!(BambooBlockType::Door.name(), "Bamboo Door");
    }

    #[test]
    fn trapdoor_name_is_bamboo_trapdoor() {
        assert_eq!(BambooBlockType::Trapdoor.name(), "Bamboo Trapdoor");
    }

    #[test]
    fn button_name_is_bamboo_button() {
        assert_eq!(BambooBlockType::Button.name(), "Bamboo Button");
    }

    #[test]
    fn pressure_plate_name_is_bamboo_pressure_plate() {
        assert_eq!(BambooBlockType::PressurePlate.name(), "Bamboo Pressure Plate");
    }

    #[test]
    fn sign_name_is_bamboo_sign() {
        assert_eq!(BambooBlockType::Sign.name(), "Bamboo Sign");
    }

    #[test]
    fn hanging_sign_name_is_bamboo_hanging_sign() {
        assert_eq!(BambooBlockType::HangingSign.name(), "Bamboo Hanging Sign");
    }

    #[test]
    fn raft_name_is_bamboo_raft() {
        assert_eq!(BambooBlockType::Raft.name(), "Bamboo Raft");
    }

    // ---- bamboo_block_hardness -------------------------------------------------

    #[test]
    fn structural_blocks_have_hardness_two() {
        assert_eq!(bamboo_block_hardness(BambooBlockType::Block), 2.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Planks), 2.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Mosaic), 2.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Stairs), 2.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Slab), 2.0);
    }

    #[test]
    fn non_structural_blocks_have_hardness_three() {
        assert_eq!(bamboo_block_hardness(BambooBlockType::FenceGate), 3.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Door), 3.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Trapdoor), 3.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Button), 3.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::PressurePlate), 3.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Sign), 3.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::HangingSign), 3.0);
        assert_eq!(bamboo_block_hardness(BambooBlockType::Raft), 3.0);
    }

    // ---- bamboo_is_flammable ---------------------------------------------------

    #[test]
    fn all_bamboo_blocks_are_flammable() {
        let all_types = [
            BambooBlockType::Block,
            BambooBlockType::Planks,
            BambooBlockType::Mosaic,
            BambooBlockType::Stairs,
            BambooBlockType::Slab,
            BambooBlockType::FenceGate,
            BambooBlockType::Door,
            BambooBlockType::Trapdoor,
            BambooBlockType::Button,
            BambooBlockType::PressurePlate,
            BambooBlockType::Sign,
            BambooBlockType::HangingSign,
            BambooBlockType::Raft,
        ];
        for block_type in all_types {
            assert!(
                bamboo_is_flammable(block_type),
                "{:?} should be flammable",
                block_type,
            );
        }
    }

    // ---- bamboo_growth_stages --------------------------------------------------

    #[test]
    fn growth_stages_is_three() {
        assert_eq!(bamboo_growth_stages(), 3);
    }

    // ---- bamboo_shoot_to_stalk -------------------------------------------------

    #[test]
    fn shoot_does_not_become_stalk_at_age_zero() {
        assert!(!bamboo_shoot_to_stalk(0));
    }

    #[test]
    fn shoot_does_not_become_stalk_at_age_two() {
        assert!(!bamboo_shoot_to_stalk(2));
    }

    #[test]
    fn shoot_becomes_stalk_at_age_three() {
        assert!(bamboo_shoot_to_stalk(3));
    }

    #[test]
    fn shoot_becomes_stalk_above_age_three() {
        assert!(bamboo_shoot_to_stalk(4));
        assert!(bamboo_shoot_to_stalk(255));
    }

    // ---- bamboo_max_height_range -----------------------------------------------

    #[test]
    fn max_height_range_is_twelve_to_sixteen() {
        assert_eq!(bamboo_max_height_range(), (12, 16));
    }

    // ---- total_bamboo_block_types ----------------------------------------------

    #[test]
    fn total_block_types_is_thirteen() {
        assert_eq!(total_bamboo_block_types(), 13);
    }
}
