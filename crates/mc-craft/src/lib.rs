//! Crafting, inventory, and item processing systems.
//!
//! Includes [`RecipeRegistry`] for shaped/shapeless crafting, [`Furnace`] smelting,
//! [`Enchantment`] and brewing, anvil/smithing upgrades, and workstation processing.

pub mod anvil;
pub mod armor_stand;
pub mod beehive_interact;
pub mod book;
pub mod brush_tool;
pub mod brewing;
pub mod bundle;
pub mod cake;
pub mod campfire;
pub mod cartography;
pub mod chiseled_bookshelf;
pub mod compass;
pub mod composter;
pub mod conduit_power;
pub mod copper_bulb;
pub mod crafter;
pub mod decorated_pot_break;
pub mod enchant_table;
pub mod goat_horn;
pub mod hanging_sign;
pub mod heavy_core;
pub mod enchanting;
pub mod firework_star;
pub mod furnace;
pub mod inventory;
pub mod item_ids;
pub mod lectern;
pub mod lodestone;
pub mod map_item;
pub mod pot_crafting;
pub mod potion_data;
pub mod recipe;
pub mod recipe_data;
pub mod recipe_data_ext;
pub mod recovery_compass;
pub mod respawn_anchor;
pub mod smithing;
pub mod smithing_template;
pub mod sniffer;
pub mod specialized_furnace;
pub mod spyglass;
pub mod trial_key;
pub mod vault;
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
pub use campfire::{CampfireRecipe, CampfireState, CookingSlot, campfire_recipes};
pub use crafter::{CrafterState, crafter_cooldown, crafter_eject_direction, trigger_craft};
pub use cartography::{
    CartographyAction, SimpleMapData, cartography_table, clone_map, extend_map, lock_map,
};
pub use compass::{
    clock_frame, clock_time_name, compass_angle, compass_frame, compass_spins_in_dimension,
    lodestone_compass_angle,
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
pub use map_item::{MapData, blocks_per_pixel, map_pixel_for_block, map_range, world_to_pixel};
pub use workstations::{
    GrindstoneResult, StonecutterRecipe, default_stonecutter_recipes, grindstone_disenchant,
    grindstone_repair, loom_apply,
};
pub use goat_horn::{
    GoatHornVariant, horn_cooldown, horn_duration, horn_from_goat_drop, horn_range, horn_sound_id,
    total_variants,
};
