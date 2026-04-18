use crate::block::BlockId;

/// Trait for querying block data from the world.
/// Used to decouple entity/physics logic from the concrete world implementation.
pub trait BlockAccess {
    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId;
    fn is_solid(&self, x: i32, y: i32, z: i32) -> bool;
    fn is_air(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(x, y, z).is_air()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock world for testing BlockAccess consumers without a real world.
    struct MockWorld {
        blocks: std::collections::HashMap<(i32, i32, i32), BlockId>,
    }

    impl MockWorld {
        fn new() -> Self {
            Self {
                blocks: std::collections::HashMap::new(),
            }
        }

        fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockId) {
            self.blocks.insert((x, y, z), block);
        }
    }

    impl BlockAccess for MockWorld {
        fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId {
            self.blocks.get(&(x, y, z)).copied().unwrap_or(BlockId::Air)
        }

        fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
            self.get_block(x, y, z).is_solid()
        }
    }

    #[test]
    fn mock_world_returns_air_by_default() {
        let world = MockWorld::new();
        assert_eq!(world.get_block(0, 0, 0), BlockId::Air);
        assert!(world.is_air(0, 0, 0));
        assert!(!world.is_solid(0, 0, 0));
    }

    #[test]
    fn mock_world_returns_set_block() {
        let mut world = MockWorld::new();
        world.set_block(1, 2, 3, BlockId::Stone);

        assert_eq!(world.get_block(1, 2, 3), BlockId::Stone);
        assert!(!world.is_air(1, 2, 3));
        assert!(world.is_solid(1, 2, 3));
    }

    #[test]
    fn is_air_default_impl_delegates_to_get_block() {
        let mut world = MockWorld::new();
        world.set_block(0, 0, 0, BlockId::Water);

        // Water is not air
        assert!(!world.is_air(0, 0, 0));
        // Unset position is air
        assert!(world.is_air(5, 5, 5));
    }

    #[test]
    fn is_solid_distinguishes_solid_from_non_solid() {
        let mut world = MockWorld::new();
        world.set_block(0, 0, 0, BlockId::Stone);
        world.set_block(1, 0, 0, BlockId::Water);
        world.set_block(2, 0, 0, BlockId::Torch);

        assert!(world.is_solid(0, 0, 0)); // Stone is solid
        assert!(!world.is_solid(1, 0, 0)); // Water is not solid
        assert!(!world.is_solid(2, 0, 0)); // Torch is not solid
    }
}
