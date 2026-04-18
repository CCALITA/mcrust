use crate::recipe;

// ── Item constants (furnace-specific) ──────────────────────────────────────
// Items already defined in recipe.rs are imported via `recipe::ITEM_*`.
// New items below are unique to smelting inputs/outputs/fuels.

const ITEM_IRON_ORE: u16 = 300;
const ITEM_GOLD_ORE: u16 = 301;
const ITEM_SAND: u16 = 302;
const ITEM_GLASS: u16 = 303;
const ITEM_STONE: u16 = 304;
const ITEM_CLAY: u16 = 305;
const ITEM_BRICK: u16 = 306;
const ITEM_RAW_BEEF: u16 = 307;
const ITEM_STEAK: u16 = 308;
const ITEM_RAW_PORKCHOP: u16 = 309;
const ITEM_COOKED_PORKCHOP: u16 = 310;
const ITEM_RAW_CHICKEN: u16 = 311;
const ITEM_COOKED_CHICKEN: u16 = 312;
const ITEM_RAW_MUTTON: u16 = 313;
const ITEM_COOKED_MUTTON: u16 = 314;
const ITEM_POTATO: u16 = 315;
const ITEM_BAKED_POTATO: u16 = 316;
const ITEM_WET_SPONGE: u16 = 317;
const ITEM_SPONGE: u16 = 318;
const ITEM_NETHERRACK: u16 = 319;
const ITEM_NETHER_BRICK: u16 = 320;
const ITEM_CACTUS: u16 = 321;
const ITEM_GREEN_DYE: u16 = 322;
const ITEM_CHARCOAL: u16 = 323;
const ITEM_SMOOTH_STONE: u16 = 324;
const ITEM_BLAZE_ROD: u16 = 325;
const ITEM_WOODEN_TOOL: u16 = 326;

/// Standard cook time in ticks for a furnace recipe.
const COOK_TOTAL_DEFAULT: u32 = 200;

// ── Smelting recipe ────────────────────────────────────────────────────────

/// A smelting recipe: input item is consumed and output item is produced.
/// `xp_reward` is granted each time an item finishes smelting.
#[derive(Debug, Clone, PartialEq)]
pub struct SmeltingRecipe {
    pub input: u16,
    pub output: u16,
    pub xp_reward: f32,
}

// ── Fuel value ─────────────────────────────────────────────────────────────

/// How many ticks a single unit of a fuel item burns in a furnace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelValue {
    pub item: u16,
    pub burn_ticks: u32,
}

// ── Default data ───────────────────────────────────────────────────────────

/// Return 15 default smelting recipes covering ores, food, and utility items.
#[must_use]
pub fn default_smelting_recipes() -> Vec<SmeltingRecipe> {
    vec![
        SmeltingRecipe {
            input: ITEM_IRON_ORE,
            output: recipe::ITEM_IRON_INGOT,
            xp_reward: 0.7,
        },
        SmeltingRecipe {
            input: ITEM_GOLD_ORE,
            output: recipe::ITEM_GOLD_INGOT,
            xp_reward: 1.0,
        },
        SmeltingRecipe {
            input: ITEM_SAND,
            output: ITEM_GLASS,
            xp_reward: 0.1,
        },
        SmeltingRecipe {
            input: recipe::ITEM_COBBLESTONE,
            output: ITEM_STONE,
            xp_reward: 0.1,
        },
        SmeltingRecipe {
            input: recipe::ITEM_OAK_LOG,
            output: ITEM_CHARCOAL,
            xp_reward: 0.15,
        },
        SmeltingRecipe {
            input: ITEM_CLAY,
            output: ITEM_BRICK,
            xp_reward: 0.3,
        },
        SmeltingRecipe {
            input: ITEM_RAW_BEEF,
            output: ITEM_STEAK,
            xp_reward: 0.35,
        },
        SmeltingRecipe {
            input: ITEM_RAW_PORKCHOP,
            output: ITEM_COOKED_PORKCHOP,
            xp_reward: 0.35,
        },
        SmeltingRecipe {
            input: ITEM_RAW_CHICKEN,
            output: ITEM_COOKED_CHICKEN,
            xp_reward: 0.35,
        },
        SmeltingRecipe {
            input: ITEM_RAW_MUTTON,
            output: ITEM_COOKED_MUTTON,
            xp_reward: 0.35,
        },
        SmeltingRecipe {
            input: ITEM_POTATO,
            output: ITEM_BAKED_POTATO,
            xp_reward: 0.35,
        },
        SmeltingRecipe {
            input: ITEM_WET_SPONGE,
            output: ITEM_SPONGE,
            xp_reward: 0.15,
        },
        SmeltingRecipe {
            input: ITEM_NETHERRACK,
            output: ITEM_NETHER_BRICK,
            xp_reward: 0.1,
        },
        SmeltingRecipe {
            input: ITEM_CACTUS,
            output: ITEM_GREEN_DYE,
            xp_reward: 1.0,
        },
        SmeltingRecipe {
            input: ITEM_STONE,
            output: ITEM_SMOOTH_STONE,
            xp_reward: 0.1,
        },
    ]
}

/// Return the default fuel values (burn duration per item).
#[must_use]
pub fn default_fuel_values() -> Vec<FuelValue> {
    vec![
        FuelValue {
            item: recipe::ITEM_COAL,
            burn_ticks: 1600,
        },
        FuelValue {
            item: recipe::ITEM_OAK_PLANKS,
            burn_ticks: 300,
        },
        FuelValue {
            item: recipe::ITEM_STICK,
            burn_ticks: 100,
        },
        FuelValue {
            item: recipe::ITEM_OAK_LOG,
            burn_ticks: 300,
        },
        FuelValue {
            item: ITEM_WOODEN_TOOL,
            burn_ticks: 200,
        },
        FuelValue {
            item: ITEM_BLAZE_ROD,
            burn_ticks: 2400,
        },
    ]
}

// ── Furnace ────────────────────────────────────────────────────────────────

/// A furnace block that smelts items using fuel.
///
/// Each slot holds an optional `(item_id, count)` pair.
/// `burn_time_remaining` counts down while fuel is actively burning.
/// `cook_progress` counts up toward `cook_total` (default 200 ticks).
#[derive(Debug, Clone)]
pub struct Furnace {
    pub input: Option<(u16, u8)>,
    pub fuel: Option<(u16, u8)>,
    pub output: Option<(u16, u8)>,
    pub burn_time_remaining: u32,
    pub total_burn_time: u32,
    pub cook_progress: u32,
    pub cook_total: u32,
}

impl Default for Furnace {
    fn default() -> Self {
        Self::new()
    }
}

impl Furnace {
    /// Create an empty, idle furnace.
    #[must_use]
    pub fn new() -> Self {
        Self {
            input: None,
            fuel: None,
            output: None,
            burn_time_remaining: 0,
            total_burn_time: 0,
            cook_progress: 0,
            cook_total: COOK_TOTAL_DEFAULT,
        }
    }

    /// Insert items into the input slot.
    ///
    /// If the slot is empty the items are placed directly. If it already
    /// contains items of the same type the counts are merged (capped at 64).
    /// Returns `false` if the slot contains a different item type.
    pub fn insert_input(&mut self, item: u16, count: u8) -> bool {
        match &mut self.input {
            None => {
                self.input = Some((item, count));
                true
            }
            Some((existing_item, existing_count)) if *existing_item == item => {
                *existing_count = (*existing_count).saturating_add(count).min(64);
                true
            }
            _ => false,
        }
    }

    /// Insert items into the fuel slot.
    ///
    /// Same merging rules as `insert_input`.
    pub fn insert_fuel(&mut self, item: u16, count: u8) -> bool {
        match &mut self.fuel {
            None => {
                self.fuel = Some((item, count));
                true
            }
            Some((existing_item, existing_count)) if *existing_item == item => {
                *existing_count = (*existing_count).saturating_add(count).min(64);
                true
            }
            _ => false,
        }
    }

    /// Take all items from the output slot, returning them if present.
    pub fn take_output(&mut self) -> Option<(u16, u8)> {
        self.output.take()
    }

    /// Advance the furnace by one game tick.
    ///
    /// Returns `Some(xp)` if an item finished smelting this tick,
    /// `None` otherwise.
    pub fn tick(&mut self, recipes: &[SmeltingRecipe], fuels: &[FuelValue]) -> Option<f32> {
        // Determine whether the current input matches a recipe.
        let recipe = self
            .input
            .and_then(|(input_id, _)| recipes.iter().find(|r| r.input == input_id));

        // If there is no valid recipe for the current input, reset progress
        // and do nothing else.
        let recipe = match recipe {
            Some(r) => r.clone(),
            None => {
                self.cook_progress = 0;
                // Fuel still burns out even without a recipe.
                self.burn_time_remaining = self.burn_time_remaining.saturating_sub(1);
                return None;
            }
        };

        // Check that the output slot can accept the recipe result.
        let output_ok = match &self.output {
            None => true,
            Some((out_id, out_count)) => *out_id == recipe.output && *out_count < 64,
        };

        if !output_ok {
            // Output slot full or mismatched; stall.
            self.burn_time_remaining = self.burn_time_remaining.saturating_sub(1);
            return None;
        }

        // If fuel is not burning, try to consume a fuel item.
        if self.burn_time_remaining == 0 {
            if let Some(burn_ticks) = self.try_consume_fuel(fuels) {
                self.burn_time_remaining = burn_ticks;
                self.total_burn_time = burn_ticks;
            } else {
                // No fuel available — reset cook progress and stall.
                self.cook_progress = 0;
                return None;
            }
        }

        // Fuel is burning — advance cooking.
        self.burn_time_remaining = self.burn_time_remaining.saturating_sub(1);
        self.cook_progress += 1;

        if self.cook_progress >= self.cook_total {
            // Item finished smelting.
            self.cook_progress = 0;

            // Consume one input item.
            if let Some((_, ref mut count)) = self.input {
                *count -= 1;
                if *count == 0 {
                    self.input = None;
                }
            }

            // Produce one output item.
            match &mut self.output {
                None => {
                    self.output = Some((recipe.output, 1));
                }
                Some((_, count)) => {
                    *count = (*count).saturating_add(1).min(64);
                }
            }

            return Some(recipe.xp_reward);
        }

        None
    }

    /// Try to consume one fuel item from the fuel slot.
    /// Returns the burn duration if successful.
    fn try_consume_fuel(&mut self, fuels: &[FuelValue]) -> Option<u32> {
        let (fuel_id, fuel_count) = self.fuel.as_mut()?;
        let burn_ticks = fuels.iter().find(|f| f.item == *fuel_id)?.burn_ticks;

        *fuel_count -= 1;
        if *fuel_count == 0 {
            self.fuel = None;
        }

        Some(burn_ticks)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Vec<SmeltingRecipe>, Vec<FuelValue>) {
        (default_smelting_recipes(), default_fuel_values())
    }

    // ── Recipe / fuel defaults ─────────────────────────────────────────

    #[test]
    fn default_recipes_has_fifteen_entries() {
        let recipes = default_smelting_recipes();
        assert_eq!(recipes.len(), 15);
    }

    #[test]
    fn default_fuels_has_six_entries() {
        let fuels = default_fuel_values();
        assert_eq!(fuels.len(), 6);
    }

    #[test]
    fn recipe_iron_ore_to_iron_ingot() {
        let recipes = default_smelting_recipes();
        let r = recipes.iter().find(|r| r.input == ITEM_IRON_ORE);
        assert!(r.is_some());
        let r = r.expect("iron ore recipe");
        assert_eq!(r.output, recipe::ITEM_IRON_INGOT);
        assert!((r.xp_reward - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn recipe_stone_to_smooth_stone() {
        let recipes = default_smelting_recipes();
        let r = recipes
            .iter()
            .find(|r| r.input == ITEM_STONE && r.output == ITEM_SMOOTH_STONE);
        assert!(r.is_some());
    }

    // ── Complete smelt cycle ───────────────────────────────────────────

    #[test]
    fn complete_smelt_cycle_produces_output_and_xp() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        furnace.insert_input(ITEM_IRON_ORE, 1);
        furnace.insert_fuel(recipe::ITEM_COAL, 1);

        let mut total_xp: f32 = 0.0;
        for _ in 0..COOK_TOTAL_DEFAULT {
            if let Some(xp) = furnace.tick(&recipes, &fuels) {
                total_xp += xp;
            }
        }

        assert!((total_xp - 0.7).abs() < f32::EPSILON, "expected 0.7 xp");
        assert_eq!(furnace.output, Some((recipe::ITEM_IRON_INGOT, 1)));
        assert!(furnace.input.is_none(), "input should be consumed");
    }

    // ── Fuel consumption ──────────────────────────────────────────────

    #[test]
    fn fuel_is_consumed_when_smelting_starts() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        furnace.insert_input(ITEM_IRON_ORE, 2);
        furnace.insert_fuel(recipe::ITEM_COAL, 2);

        // First tick consumes one fuel unit.
        furnace.tick(&recipes, &fuels);

        assert_eq!(furnace.fuel, Some((recipe::ITEM_COAL, 1)));
        assert!(furnace.burn_time_remaining > 0);
    }

    #[test]
    fn fuel_burns_for_correct_duration() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        // Use sticks (100 ticks burn time).
        furnace.insert_input(ITEM_IRON_ORE, 10);
        furnace.insert_fuel(recipe::ITEM_STICK, 1);

        furnace.tick(&recipes, &fuels);
        // After first tick: burn_time_remaining = 100 - 1 = 99
        assert_eq!(furnace.burn_time_remaining, 99);
        assert_eq!(furnace.total_burn_time, 100);
    }

    // ── No-fuel pause ─────────────────────────────────────────────────

    #[test]
    fn smelting_pauses_without_fuel() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        furnace.insert_input(ITEM_IRON_ORE, 1);
        // No fuel inserted.

        for _ in 0..300 {
            let xp = furnace.tick(&recipes, &fuels);
            assert!(xp.is_none());
        }

        assert!(furnace.output.is_none(), "no output without fuel");
        assert_eq!(furnace.cook_progress, 0, "progress reset without fuel");
    }

    #[test]
    fn smelting_resumes_after_fuel_added() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        furnace.insert_input(ITEM_IRON_ORE, 1);

        // Tick without fuel — nothing happens.
        for _ in 0..50 {
            furnace.tick(&recipes, &fuels);
        }
        assert_eq!(furnace.cook_progress, 0);

        // Add fuel and finish smelting.
        furnace.insert_fuel(recipe::ITEM_COAL, 1);
        let mut total_xp: f32 = 0.0;
        for _ in 0..COOK_TOTAL_DEFAULT {
            if let Some(xp) = furnace.tick(&recipes, &fuels) {
                total_xp += xp;
            }
        }

        assert!((total_xp - 0.7).abs() < f32::EPSILON);
        assert_eq!(furnace.output, Some((recipe::ITEM_IRON_INGOT, 1)));
    }

    // ── Recipe matching ───────────────────────────────────────────────

    #[test]
    fn no_recipe_match_resets_progress() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        // Insert an item with no smelting recipe.
        furnace.insert_input(recipe::ITEM_DIAMOND, 1);
        furnace.insert_fuel(recipe::ITEM_COAL, 1);

        for _ in 0..300 {
            let xp = furnace.tick(&recipes, &fuels);
            assert!(xp.is_none());
        }

        assert!(furnace.output.is_none());
        assert_eq!(furnace.cook_progress, 0);
    }

    // ── XP return ─────────────────────────────────────────────────────

    #[test]
    fn xp_returned_only_on_completion_tick() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        furnace.insert_input(ITEM_SAND, 1);
        furnace.insert_fuel(recipe::ITEM_COAL, 1);

        let mut xp_ticks = 0u32;
        for _ in 0..COOK_TOTAL_DEFAULT {
            if furnace.tick(&recipes, &fuels).is_some() {
                xp_ticks += 1;
            }
        }

        assert_eq!(xp_ticks, 1, "XP should be granted exactly once");
    }

    // ── Slot insertion / extraction ───────────────────────────────────

    #[test]
    fn insert_input_merges_same_item() {
        let mut furnace = Furnace::new();
        assert!(furnace.insert_input(ITEM_IRON_ORE, 10));
        assert!(furnace.insert_input(ITEM_IRON_ORE, 5));
        assert_eq!(furnace.input, Some((ITEM_IRON_ORE, 15)));
    }

    #[test]
    fn insert_input_rejects_different_item() {
        let mut furnace = Furnace::new();
        assert!(furnace.insert_input(ITEM_IRON_ORE, 10));
        assert!(!furnace.insert_input(ITEM_GOLD_ORE, 5));
        assert_eq!(furnace.input, Some((ITEM_IRON_ORE, 10)));
    }

    #[test]
    fn insert_fuel_merges_same_item() {
        let mut furnace = Furnace::new();
        assert!(furnace.insert_fuel(recipe::ITEM_COAL, 3));
        assert!(furnace.insert_fuel(recipe::ITEM_COAL, 2));
        assert_eq!(furnace.fuel, Some((recipe::ITEM_COAL, 5)));
    }

    #[test]
    fn take_output_clears_slot() {
        let mut furnace = Furnace::new();
        furnace.output = Some((recipe::ITEM_IRON_INGOT, 3));
        let taken = furnace.take_output();
        assert_eq!(taken, Some((recipe::ITEM_IRON_INGOT, 3)));
        assert!(furnace.output.is_none());
    }

    #[test]
    fn take_output_empty_returns_none() {
        let mut furnace = Furnace::new();
        assert!(furnace.take_output().is_none());
    }

    // ── Multiple items ────────────────────────────────────────────────

    #[test]
    fn smelt_multiple_items_accumulates_output() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        furnace.insert_input(ITEM_IRON_ORE, 3);
        furnace.insert_fuel(recipe::ITEM_COAL, 3);

        let mut total_xp: f32 = 0.0;
        // 3 items * 200 ticks each = 600 ticks maximum.
        for _ in 0..600 {
            if let Some(xp) = furnace.tick(&recipes, &fuels) {
                total_xp += xp;
            }
        }

        assert!((total_xp - 2.1).abs() < 0.001, "expected 3 * 0.7 = 2.1 xp");
        assert_eq!(furnace.output, Some((recipe::ITEM_IRON_INGOT, 3)));
        assert!(furnace.input.is_none());
    }

    // ── Output slot full stalls smelting ───────────────────────────────

    #[test]
    fn output_full_stalls_smelting() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        furnace.insert_input(ITEM_IRON_ORE, 1);
        furnace.insert_fuel(recipe::ITEM_COAL, 1);
        // Pre-fill output with a different item.
        furnace.output = Some((ITEM_GLASS, 10));

        for _ in 0..COOK_TOTAL_DEFAULT {
            let xp = furnace.tick(&recipes, &fuels);
            assert!(xp.is_none());
        }

        assert_eq!(furnace.output, Some((ITEM_GLASS, 10)));
        assert_eq!(furnace.input, Some((ITEM_IRON_ORE, 1)));
    }

    // ── Fuel consumed even mid-smelt when item finishes ───────────────

    #[test]
    fn short_fuel_requires_multiple_fuel_items() {
        let (recipes, fuels) = setup();
        let mut furnace = Furnace::new();

        // Sticks burn for 100 ticks but cooking takes 200.
        // Need 2 sticks to smelt one item.
        furnace.insert_input(ITEM_IRON_ORE, 1);
        furnace.insert_fuel(recipe::ITEM_STICK, 2);

        let mut total_xp: f32 = 0.0;
        for _ in 0..COOK_TOTAL_DEFAULT {
            if let Some(xp) = furnace.tick(&recipes, &fuels) {
                total_xp += xp;
            }
        }

        assert!((total_xp - 0.7).abs() < f32::EPSILON);
        assert_eq!(furnace.output, Some((recipe::ITEM_IRON_INGOT, 1)));
        assert!(furnace.fuel.is_none(), "both sticks should be consumed");
    }
}
