//! Crafting, inventory, and item processing systems.
//!
//! Includes [`RecipeRegistry`] for shaped/shapeless crafting, [`Furnace`] smelting,
//! [`Enchantment`] and brewing, anvil/smithing upgrades, and workstation processing.

pub mod anvil;
pub mod book;
pub mod brewing;
pub mod cartography;
pub mod enchant_table;
pub mod enchanting;
pub mod furnace;
pub mod inventory;
pub mod item_ids;
pub mod potion_data;
pub mod recipe;
pub mod recipe_data;
pub mod recipe_data_ext;
pub mod smithing;
pub mod workstations;

/// Minimal item representation for crafting.
/// Will be replaced by `mc_core::item::ItemId` later.
pub type SlotItem = u16;

/// A stack of items with a count.
#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item: u16,
    pub count: u8,
}

pub use book::{
    BookAndQuill, WrittenBook, copy_book, sign_book, MAX_CHARS_PER_PAGE, MAX_PAGES,
    MAX_TITLE_LENGTH,
};
pub use anvil::{
    AnvilResult, anvil_combine, anvil_degrades, anvil_rename, merge_enchantments, repair_cost,
};
pub use brewing::{BrewingIngredient, BrewingRecipe, BrewingStand};
pub use cartography::{
    CartographyAction, SimpleMapData, cartography_table, clone_map, extend_map, lock_map,
};
pub use enchant_table::{
    EnchantTableState, apply_enchant, bookshelf_power, can_enchant, refresh_options,
};
pub use enchanting::{
    EnchantOption, EnchantedItem, Enchantment, EnchantmentCategory, EnchantmentId,
    EnchantmentProperties, apply_enchantment_effect, calculate_enchantment_cost,
    generate_enchantment_options,
};
pub use furnace::{
    FuelValue, Furnace, SmeltingRecipe, default_fuel_values, default_smelting_recipes,
};
pub use inventory::Inventory;
pub use potion_data::{PotionEffect, PotionType, StatusEffectManager};
pub use recipe::{CraftingGrid, Recipe, RecipePattern, RecipeRegistry};
pub use smithing::{
    SmithingRecipe, SmithingTable, default_smithing_recipes, preserve_enchantments, try_smith,
};
pub use workstations::{
    GrindstoneResult, StonecutterRecipe, default_stonecutter_recipes, grindstone_disenchant,
    grindstone_repair, loom_apply,
};
