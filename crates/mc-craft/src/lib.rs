pub mod enchanting;
pub mod inventory;
pub mod recipe;

/// Minimal item representation for crafting.
/// Will be replaced by `mc_core::item::ItemId` later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotItem(pub u16);

/// A stack of items with a count.
#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item: SlotItem,
    pub count: u8,
}

pub use enchanting::{
    apply_enchantment_effect, calculate_enchantment_cost, generate_enchantment_options,
    EnchantOption, EnchantedItem, Enchantment, EnchantmentCategory, EnchantmentId,
    EnchantmentProperties,
};
pub use inventory::Inventory;
pub use recipe::{CraftingGrid, Recipe, RecipePattern, RecipeRegistry};
