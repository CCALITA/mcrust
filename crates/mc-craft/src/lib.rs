pub mod anvil;
pub mod brewing;
pub mod enchant_table;
pub mod enchanting;
pub mod furnace;
pub mod inventory;
pub mod recipe;
pub mod smithing;
pub mod workstations;

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

pub use anvil::{
    anvil_combine, anvil_degrades, anvil_rename, merge_enchantments, repair_cost, AnvilResult,
};
pub use brewing::{
    BrewingIngredient, BrewingRecipe, BrewingStand, PotionEffect, PotionType, StatusEffectManager,
};
pub use enchanting::{
    apply_enchantment_effect, calculate_enchantment_cost, generate_enchantment_options,
    EnchantOption, EnchantedItem, Enchantment, EnchantmentCategory, EnchantmentId,
    EnchantmentProperties,
};
pub use furnace::{
    default_fuel_values, default_smelting_recipes, FuelValue, Furnace, SmeltingRecipe,
};
pub use inventory::Inventory;
pub use recipe::{CraftingGrid, Recipe, RecipePattern, RecipeRegistry};
pub use smithing::{
    SmithingRecipe, SmithingTable, default_smithing_recipes, preserve_enchantments, try_smith,
};
pub use workstations::{
    GrindstoneResult, StonecutterRecipe, default_stonecutter_recipes, grindstone_disenchant,
    grindstone_repair, loom_apply,
};
pub use enchant_table::{
    EnchantTableState, apply_enchant, bookshelf_power, can_enchant, refresh_options,
};
