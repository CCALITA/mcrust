use serde::{Deserialize, Serialize};

use crate::block::BlockId;

// ---------------------------------------------------------------------------
// ToolType & ToolTier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
    Shears,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolTier {
    Wood,
    Stone,
    Iron,
    Gold,
    Diamond,
    None,
}

impl ToolTier {
    pub fn mining_speed(&self) -> f32 {
        match self {
            ToolTier::Wood => 2.0,
            ToolTier::Stone => 4.0,
            ToolTier::Iron => 6.0,
            ToolTier::Gold => 12.0,
            ToolTier::Diamond => 8.0,
            ToolTier::None => 1.0,
        }
    }

    pub fn durability(&self) -> u32 {
        match self {
            ToolTier::Wood => 59,
            ToolTier::Stone => 131,
            ToolTier::Iron => 250,
            ToolTier::Gold => 32,
            ToolTier::Diamond => 1561,
            ToolTier::None => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// ItemId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum ItemId {
    // Block items (mirror placeable BlockId variants) ----------------------
    Stone = 0,
    Dirt,
    GrassBlock,
    Sand,
    Gravel,
    OakLog,
    OakLeaves,
    OakPlanks,
    Cobblestone,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Glass,
    Torch,
    CraftingTable,
    Furnace,
    Chest,

    // Tools ----------------------------------------------------------------
    WoodenPickaxe,
    StonePickaxe,
    IronPickaxe,
    DiamondPickaxe,
    WoodenAxe,
    StoneAxe,
    IronAxe,
    DiamondAxe,
    WoodenShovel,
    StoneShovel,
    IronShovel,
    DiamondShovel,
    WoodenSword,
    StoneSword,
    IronSword,
    DiamondSword,
    WoodenHoe,
    StoneHoe,
    IronHoe,
    DiamondHoe,
    Shears,
    FlintAndSteel,
    FishingRod,
    Bow,

    // Food -----------------------------------------------------------------
    Apple,
    GoldenApple,
    Bread,
    CookedPorkchop,
    CookedBeef,
    CookedChicken,
    CookedMutton,
    Cookie,
    Cake,
    Melon,
    Carrot,
    Potato,
    BakedPotato,

    // Materials ------------------------------------------------------------
    Stick,
    Coal,
    IronIngot,
    GoldIngot,
    Diamond,
    Emerald,
    LapisLazuli,
    Redstone,
    StringItem,
    Leather,
    Feather,
    Bone,
    Gunpowder,
    Arrow,
    Bucket,
    WaterBucket,
    LavaBucket,
}

impl ItemId {
    pub const COUNT: usize = 72;

    pub fn properties(self) -> &'static ItemProperties {
        &ITEM_REGISTRY[self as usize]
    }

    /// Convert a `BlockId` to its corresponding `ItemId`, if one exists.
    pub fn from_block(block: BlockId) -> Option<Self> {
        match block {
            BlockId::Stone => Some(ItemId::Stone),
            BlockId::Dirt => Some(ItemId::Dirt),
            BlockId::GrassBlock => Some(ItemId::GrassBlock),
            BlockId::Sand => Some(ItemId::Sand),
            BlockId::Gravel => Some(ItemId::Gravel),
            BlockId::OakLog => Some(ItemId::OakLog),
            BlockId::OakLeaves => Some(ItemId::OakLeaves),
            BlockId::OakPlanks => Some(ItemId::OakPlanks),
            BlockId::Cobblestone => Some(ItemId::Cobblestone),
            BlockId::CoalOre => Some(ItemId::CoalOre),
            BlockId::IronOre => Some(ItemId::IronOre),
            BlockId::GoldOre => Some(ItemId::GoldOre),
            BlockId::DiamondOre => Some(ItemId::DiamondOre),
            BlockId::Glass => Some(ItemId::Glass),
            BlockId::Torch => Some(ItemId::Torch),
            BlockId::CraftingTable => Some(ItemId::CraftingTable),
            BlockId::Furnace => Some(ItemId::Furnace),
            BlockId::Chest => Some(ItemId::Chest),
            // Air, Bedrock, Water have no item form
            BlockId::Air | BlockId::Bedrock | BlockId::Water => None,
            // New block types — map to themselves if a matching ItemId exists,
            // otherwise None (item variants will be added incrementally)
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// ItemProperties
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ItemProperties {
    pub name: &'static str,
    pub max_stack_size: u8,
    pub tool_type: ToolType,
    pub tool_tier: ToolTier,
    pub food_value: Option<u32>,
}

// ---------------------------------------------------------------------------
// Helper constructors (const)
// ---------------------------------------------------------------------------

const fn block_item(name: &'static str) -> ItemProperties {
    ItemProperties {
        name,
        max_stack_size: 64,
        tool_type: ToolType::None,
        tool_tier: ToolTier::None,
        food_value: None,
    }
}

const fn tool(name: &'static str, tool_type: ToolType, tool_tier: ToolTier) -> ItemProperties {
    ItemProperties {
        name,
        max_stack_size: 1,
        tool_type,
        tool_tier,
        food_value: None,
    }
}

const fn food(name: &'static str, hunger: u32) -> ItemProperties {
    ItemProperties {
        name,
        max_stack_size: 64,
        tool_type: ToolType::None,
        tool_tier: ToolTier::None,
        food_value: Some(hunger),
    }
}

const fn material(name: &'static str) -> ItemProperties {
    ItemProperties {
        name,
        max_stack_size: 64,
        tool_type: ToolType::None,
        tool_tier: ToolTier::None,
        food_value: None,
    }
}

const fn material_stack16(name: &'static str) -> ItemProperties {
    ItemProperties {
        name,
        max_stack_size: 16,
        tool_type: ToolType::None,
        tool_tier: ToolTier::None,
        food_value: None,
    }
}

// ---------------------------------------------------------------------------
// Static registry
// ---------------------------------------------------------------------------

static ITEM_REGISTRY: [ItemProperties; ItemId::COUNT] = [
    // Block items (18) -----------------------------------------------------
    block_item("stone"),
    block_item("dirt"),
    block_item("grass_block"),
    block_item("sand"),
    block_item("gravel"),
    block_item("oak_log"),
    block_item("oak_leaves"),
    block_item("oak_planks"),
    block_item("cobblestone"),
    block_item("coal_ore"),
    block_item("iron_ore"),
    block_item("gold_ore"),
    block_item("diamond_ore"),
    block_item("glass"),
    block_item("torch"),
    block_item("crafting_table"),
    block_item("furnace"),
    block_item("chest"),
    // Tools (24) -----------------------------------------------------------
    tool("wooden_pickaxe", ToolType::Pickaxe, ToolTier::Wood),
    tool("stone_pickaxe", ToolType::Pickaxe, ToolTier::Stone),
    tool("iron_pickaxe", ToolType::Pickaxe, ToolTier::Iron),
    tool("diamond_pickaxe", ToolType::Pickaxe, ToolTier::Diamond),
    tool("wooden_axe", ToolType::Axe, ToolTier::Wood),
    tool("stone_axe", ToolType::Axe, ToolTier::Stone),
    tool("iron_axe", ToolType::Axe, ToolTier::Iron),
    tool("diamond_axe", ToolType::Axe, ToolTier::Diamond),
    tool("wooden_shovel", ToolType::Shovel, ToolTier::Wood),
    tool("stone_shovel", ToolType::Shovel, ToolTier::Stone),
    tool("iron_shovel", ToolType::Shovel, ToolTier::Iron),
    tool("diamond_shovel", ToolType::Shovel, ToolTier::Diamond),
    tool("wooden_sword", ToolType::Sword, ToolTier::Wood),
    tool("stone_sword", ToolType::Sword, ToolTier::Stone),
    tool("iron_sword", ToolType::Sword, ToolTier::Iron),
    tool("diamond_sword", ToolType::Sword, ToolTier::Diamond),
    tool("wooden_hoe", ToolType::Hoe, ToolTier::Wood),
    tool("stone_hoe", ToolType::Hoe, ToolTier::Stone),
    tool("iron_hoe", ToolType::Hoe, ToolTier::Iron),
    tool("diamond_hoe", ToolType::Hoe, ToolTier::Diamond),
    tool("shears", ToolType::Shears, ToolTier::Iron),
    tool("flint_and_steel", ToolType::None, ToolTier::None),
    tool("fishing_rod", ToolType::None, ToolTier::None),
    tool("bow", ToolType::None, ToolTier::None),
    // Food (13) ------------------------------------------------------------
    food("apple", 4),
    food("golden_apple", 4),
    food("bread", 5),
    food("cooked_porkchop", 8),
    food("cooked_beef", 8),
    food("cooked_chicken", 6),
    food("cooked_mutton", 6),
    food("cookie", 2),
    food("cake", 14),
    food("melon_slice", 2),
    food("carrot", 3),
    food("potato", 1),
    food("baked_potato", 5),
    // Materials (17) -------------------------------------------------------
    material("stick"),
    material("coal"),
    material("iron_ingot"),
    material("gold_ingot"),
    material("diamond"),
    material("emerald"),
    material("lapis_lazuli"),
    material("redstone"),
    material("string"),
    material("leather"),
    material("feather"),
    material("bone"),
    material("gunpowder"),
    material("arrow"),
    material_stack16("bucket"),
    material_stack16("water_bucket"),
    material_stack16("lava_bucket"),
];

// ---------------------------------------------------------------------------
// ItemStack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: ItemId,
    pub count: u8,
}

impl ItemStack {
    pub fn new(item: ItemId, count: u8) -> Self {
        let max = item.properties().max_stack_size;
        let clamped = count.min(max);
        Self {
            item,
            count: clamped,
        }
    }

    pub fn is_full(&self) -> bool {
        self.count >= self.item.properties().max_stack_size
    }

    pub fn can_merge(&self, other: &ItemStack) -> bool {
        self.item == other.item && !self.is_full()
    }

    /// Merge `other` into `self`, transferring as many items as possible.
    /// After merging, `other.count` holds the leftover that did not fit.
    pub fn merge(&mut self, other: &mut ItemStack) {
        if self.item != other.item {
            return;
        }
        let max = self.item.properties().max_stack_size;
        let space = max.saturating_sub(self.count);
        let transfer = space.min(other.count);
        self.count += transfer;
        other.count -= transfer;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_max_stack_is_one() {
        assert_eq!(ItemId::WoodenPickaxe.properties().max_stack_size, 1);
        assert_eq!(ItemId::DiamondSword.properties().max_stack_size, 1);
        assert_eq!(ItemId::Bow.properties().max_stack_size, 1);
        assert_eq!(ItemId::Shears.properties().max_stack_size, 1);
    }

    #[test]
    fn block_item_max_stack_is_64() {
        assert_eq!(ItemId::Stone.properties().max_stack_size, 64);
        assert_eq!(ItemId::Dirt.properties().max_stack_size, 64);
        assert_eq!(ItemId::OakLog.properties().max_stack_size, 64);
    }

    #[test]
    fn bucket_max_stack_is_16() {
        assert_eq!(ItemId::Bucket.properties().max_stack_size, 16);
        assert_eq!(ItemId::WaterBucket.properties().max_stack_size, 16);
        assert_eq!(ItemId::LavaBucket.properties().max_stack_size, 16);
    }

    #[test]
    fn food_has_food_value() {
        assert_eq!(ItemId::Apple.properties().food_value, Some(4));
        assert_eq!(ItemId::CookedBeef.properties().food_value, Some(8));
        assert_eq!(ItemId::Bread.properties().food_value, Some(5));
    }

    #[test]
    fn non_food_has_no_food_value() {
        assert_eq!(ItemId::Stone.properties().food_value, None);
        assert_eq!(ItemId::WoodenPickaxe.properties().food_value, None);
        assert_eq!(ItemId::Stick.properties().food_value, None);
    }

    #[test]
    fn merge_combines_stacks() {
        let mut a = ItemStack::new(ItemId::Dirt, 40);
        let mut b = ItemStack::new(ItemId::Dirt, 30);
        a.merge(&mut b);
        assert_eq!(a.count, 64);
        assert_eq!(b.count, 6);
    }

    #[test]
    fn merge_exact_fill() {
        let mut a = ItemStack::new(ItemId::Dirt, 32);
        let mut b = ItemStack::new(ItemId::Dirt, 32);
        a.merge(&mut b);
        assert_eq!(a.count, 64);
        assert_eq!(b.count, 0);
    }

    #[test]
    fn merge_full_stack_no_transfer() {
        let mut a = ItemStack::new(ItemId::Dirt, 64);
        let mut b = ItemStack::new(ItemId::Dirt, 10);
        a.merge(&mut b);
        assert_eq!(a.count, 64);
        assert_eq!(b.count, 10);
    }

    #[test]
    fn merge_different_items_no_transfer() {
        let mut a = ItemStack::new(ItemId::Dirt, 32);
        let mut b = ItemStack::new(ItemId::Stone, 32);
        a.merge(&mut b);
        assert_eq!(a.count, 32);
        assert_eq!(b.count, 32);
    }

    #[test]
    fn merge_tool_stacks_max_one() {
        let mut a = ItemStack::new(ItemId::WoodenPickaxe, 1);
        let mut b = ItemStack::new(ItemId::WoodenPickaxe, 1);
        a.merge(&mut b);
        assert_eq!(a.count, 1);
        assert_eq!(b.count, 1);
    }

    #[test]
    fn can_merge_same_item_not_full() {
        let a = ItemStack::new(ItemId::Dirt, 32);
        let b = ItemStack::new(ItemId::Dirt, 10);
        assert!(a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_different_items() {
        let a = ItemStack::new(ItemId::Dirt, 32);
        let b = ItemStack::new(ItemId::Stone, 10);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_when_full() {
        let a = ItemStack::new(ItemId::Dirt, 64);
        let b = ItemStack::new(ItemId::Dirt, 10);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn is_full_works() {
        let full = ItemStack::new(ItemId::Dirt, 64);
        let not_full = ItemStack::new(ItemId::Dirt, 32);
        let tool_full = ItemStack::new(ItemId::WoodenPickaxe, 1);
        assert!(full.is_full());
        assert!(!not_full.is_full());
        assert!(tool_full.is_full());
    }

    #[test]
    fn new_clamps_to_max() {
        let stack = ItemStack::new(ItemId::WoodenPickaxe, 5);
        assert_eq!(stack.count, 1);
        let stack = ItemStack::new(ItemId::Bucket, 64);
        assert_eq!(stack.count, 16);
    }

    #[test]
    fn from_block_maps_placeable_blocks() {
        assert_eq!(ItemId::from_block(BlockId::Stone), Some(ItemId::Stone));
        assert_eq!(ItemId::from_block(BlockId::Dirt), Some(ItemId::Dirt));
        assert_eq!(ItemId::from_block(BlockId::OakLog), Some(ItemId::OakLog));
        assert_eq!(ItemId::from_block(BlockId::Chest), Some(ItemId::Chest));
        assert_eq!(
            ItemId::from_block(BlockId::GrassBlock),
            Some(ItemId::GrassBlock)
        );
    }

    #[test]
    fn from_block_returns_none_for_non_items() {
        assert_eq!(ItemId::from_block(BlockId::Air), None);
        assert_eq!(ItemId::from_block(BlockId::Bedrock), None);
        assert_eq!(ItemId::from_block(BlockId::Water), None);
    }

    #[test]
    fn tool_tier_mining_speed() {
        assert_eq!(ToolTier::Wood.mining_speed(), 2.0);
        assert_eq!(ToolTier::Diamond.mining_speed(), 8.0);
        assert_eq!(ToolTier::Gold.mining_speed(), 12.0);
        assert_eq!(ToolTier::None.mining_speed(), 1.0);
    }

    #[test]
    fn tool_tier_durability() {
        assert_eq!(ToolTier::Wood.durability(), 59);
        assert_eq!(ToolTier::Diamond.durability(), 1561);
        assert_eq!(ToolTier::None.durability(), 0);
    }

    #[test]
    fn item_count_matches_registry() {
        // Verify the COUNT constant matches the actual enum variant count
        assert_eq!(ITEM_REGISTRY.len(), ItemId::COUNT);
    }

    #[test]
    fn tool_properties_correct() {
        let props = ItemId::IronPickaxe.properties();
        assert_eq!(props.name, "iron_pickaxe");
        assert_eq!(props.tool_type, ToolType::Pickaxe);
        assert_eq!(props.tool_tier, ToolTier::Iron);
        assert_eq!(props.max_stack_size, 1);
    }
}
