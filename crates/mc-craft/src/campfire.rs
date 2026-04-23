//! Campfire cooking system.
//!
//! A campfire has 4 cooking slots. Items placed on a lit campfire cook
//! over time and are returned as their cooked output when done.

// ── Item constants (campfire-specific) ────────────────────────────────────

const ITEM_RAW_BEEF: u16 = 3010;
const ITEM_COOKED_BEEF: u16 = 3007;
const ITEM_RAW_PORK: u16 = 3011;
const ITEM_COOKED_PORK: u16 = 3012;
const ITEM_RAW_CHICKEN: u16 = 3013;
const ITEM_COOKED_CHICKEN: u16 = 3006;
const ITEM_POTATO: u16 = 3014;
const ITEM_BAKED_POTATO: u16 = 3001;
const ITEM_RAW_COD: u16 = 3015;
const ITEM_COOKED_COD: u16 = 3016;
const ITEM_RAW_SALMON: u16 = 3017;
const ITEM_COOKED_SALMON: u16 = 3018;
const ITEM_KELP: u16 = 3019;
const ITEM_DRIED_KELP: u16 = 3020;

// ── Cooking slot ──────────────────────────────────────────────────────────

/// A single item cooking on a campfire.
#[derive(Debug, Clone, PartialEq)]
pub struct CookingSlot {
    pub item_id: u16,
    pub cook_time: f32,
    pub total_time: f32,
}

// ── Campfire state ────────────────────────────────────────────────────────

/// State of a campfire block with up to 4 simultaneous cooking slots.
#[derive(Debug, Clone)]
pub struct CampfireState {
    pub slots: [Option<CookingSlot>; 4],
    pub lit: bool,
}

impl Default for CampfireState {
    fn default() -> Self {
        Self::new()
    }
}

impl CampfireState {
    /// Create a new campfire with 4 empty slots, lit by default.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [None, None, None, None],
            lit: true,
        }
    }

    /// Place an item on the campfire.
    ///
    /// Finds the first empty slot and starts cooking if a matching recipe
    /// exists. Returns `false` if there is no empty slot or no recipe for
    /// the given item.
    pub fn add_item(&mut self, item_id: u16) -> bool {
        let recipe = campfire_recipes().into_iter().find(|r| r.input == item_id);

        let recipe = match recipe {
            Some(r) => r,
            None => return false,
        };

        let empty_slot = self.slots.iter_mut().find(|s| s.is_none());

        match empty_slot {
            Some(slot) => {
                *slot = Some(CookingSlot {
                    item_id,
                    cook_time: 0.0,
                    total_time: recipe.cook_time,
                });
                true
            }
            None => false,
        }
    }

    /// Advance cooking for all active slots by `dt` seconds.
    ///
    /// Returns the output item IDs for any slots that finished cooking.
    /// Completed slots are cleared.
    pub fn tick(&mut self, dt: f32) -> Vec<u16> {
        if !self.lit {
            return Vec::new();
        }

        let recipes = campfire_recipes();
        let mut finished = Vec::new();

        for slot in &mut self.slots {
            let completed = match slot {
                Some(cooking) => {
                    cooking.cook_time += dt;
                    cooking.cook_time >= cooking.total_time
                }
                None => false,
            };

            if completed {
                if let Some(cooking) = slot.take() {
                    if let Some(recipe) = recipes.iter().find(|r| r.input == cooking.item_id) {
                        finished.push(recipe.output);
                    }
                }
            }
        }

        finished
    }

    /// Extinguish the campfire. Cooking pauses until relit.
    pub fn extinguish(&mut self) {
        self.lit = false;
    }

    /// Relight the campfire. Cooking resumes from where it left off.
    pub fn relight(&mut self) {
        self.lit = true;
    }
}

// ── Campfire recipe ───────────────────────────────────────────────────────

/// A campfire cooking recipe mapping an input item to an output item.
#[derive(Debug, Clone, PartialEq)]
pub struct CampfireRecipe {
    pub input: u16,
    pub output: u16,
    pub cook_time: f32,
}

/// Return the 7 default campfire cooking recipes.
#[must_use]
pub fn campfire_recipes() -> Vec<CampfireRecipe> {
    vec![
        CampfireRecipe {
            input: ITEM_RAW_BEEF,
            output: ITEM_COOKED_BEEF,
            cook_time: 30.0,
        },
        CampfireRecipe {
            input: ITEM_RAW_PORK,
            output: ITEM_COOKED_PORK,
            cook_time: 30.0,
        },
        CampfireRecipe {
            input: ITEM_RAW_CHICKEN,
            output: ITEM_COOKED_CHICKEN,
            cook_time: 30.0,
        },
        CampfireRecipe {
            input: ITEM_POTATO,
            output: ITEM_BAKED_POTATO,
            cook_time: 30.0,
        },
        CampfireRecipe {
            input: ITEM_RAW_COD,
            output: ITEM_COOKED_COD,
            cook_time: 30.0,
        },
        CampfireRecipe {
            input: ITEM_RAW_SALMON,
            output: ITEM_COOKED_SALMON,
            cook_time: 30.0,
        },
        CampfireRecipe {
            input: ITEM_KELP,
            output: ITEM_DRIED_KELP,
            cook_time: 30.0,
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Cooking cycle ─────────────────────────────────────────────────

    #[test]
    fn cooking_cycle_produces_output() {
        let mut campfire = CampfireState::new();
        assert!(campfire.add_item(ITEM_RAW_BEEF));

        // Tick just under the cook time — nothing should finish.
        let finished = campfire.tick(29.9);
        assert!(finished.is_empty());

        // Tick the remaining time — item should finish.
        let finished = campfire.tick(0.2);
        assert_eq!(finished, vec![ITEM_COOKED_BEEF]);

        // Slot should now be empty.
        assert!(campfire.slots.iter().all(|s| s.is_none()));
    }

    // ── 4-slot limit ──────────────────────────────────────────────────

    #[test]
    fn four_slot_limit() {
        let mut campfire = CampfireState::new();

        assert!(campfire.add_item(ITEM_RAW_BEEF));
        assert!(campfire.add_item(ITEM_RAW_PORK));
        assert!(campfire.add_item(ITEM_RAW_CHICKEN));
        assert!(campfire.add_item(ITEM_POTATO));

        // Fifth item should be rejected.
        assert!(!campfire.add_item(ITEM_RAW_COD));

        // All four slots occupied.
        assert!(campfire.slots.iter().all(|s| s.is_some()));
    }

    // ── Extinguished campfire does not cook ────────────────────────────

    #[test]
    fn extinguished_does_not_cook() {
        let mut campfire = CampfireState::new();
        assert!(campfire.add_item(ITEM_RAW_BEEF));

        campfire.extinguish();
        assert!(!campfire.lit);

        // Tick past the full cook time while extinguished.
        let finished = campfire.tick(60.0);
        assert!(finished.is_empty());

        // Item should still be in slot with no progress.
        let slot = campfire.slots[0].as_ref().expect("item still in slot");
        assert!((slot.cook_time - 0.0).abs() < f32::EPSILON);
    }

    // ── Relight resumes cooking ───────────────────────────────────────

    #[test]
    fn relight_resumes_cooking() {
        let mut campfire = CampfireState::new();
        assert!(campfire.add_item(ITEM_RAW_BEEF));

        // Cook partially.
        let finished = campfire.tick(15.0);
        assert!(finished.is_empty());

        // Extinguish and try to cook more — should not advance.
        campfire.extinguish();
        let finished = campfire.tick(30.0);
        assert!(finished.is_empty());

        // Relight and finish cooking.
        campfire.relight();
        let finished = campfire.tick(15.0);
        assert_eq!(finished, vec![ITEM_COOKED_BEEF]);
    }

    // ── All 7 recipes ─────────────────────────────────────────────────

    #[test]
    fn all_recipes_present() {
        let recipes = campfire_recipes();
        assert_eq!(recipes.len(), 7);

        let expected = [
            (ITEM_RAW_BEEF, ITEM_COOKED_BEEF),
            (ITEM_RAW_PORK, ITEM_COOKED_PORK),
            (ITEM_RAW_CHICKEN, ITEM_COOKED_CHICKEN),
            (ITEM_POTATO, ITEM_BAKED_POTATO),
            (ITEM_RAW_COD, ITEM_COOKED_COD),
            (ITEM_RAW_SALMON, ITEM_COOKED_SALMON),
            (ITEM_KELP, ITEM_DRIED_KELP),
        ];

        for (input, output) in expected {
            let recipe = recipes.iter().find(|r| r.input == input);
            assert!(recipe.is_some(), "missing recipe for input {input}");
            let recipe = recipe.expect("recipe exists");
            assert_eq!(recipe.output, output);
            assert!((recipe.cook_time - 30.0).abs() < f32::EPSILON);
        }
    }

    // ── Unknown item rejected ─────────────────────────────────────────

    #[test]
    fn unknown_item_rejected() {
        let mut campfire = CampfireState::new();
        assert!(!campfire.add_item(9999));
        assert!(campfire.slots.iter().all(|s| s.is_none()));
    }

    // ── Tick to completion with multiple items ────────────────────────

    #[test]
    fn tick_to_completion_multiple_items() {
        let mut campfire = CampfireState::new();

        assert!(campfire.add_item(ITEM_RAW_BEEF));
        assert!(campfire.add_item(ITEM_RAW_COD));
        assert!(campfire.add_item(ITEM_KELP));

        // Cook all items to completion in one big tick.
        let finished = campfire.tick(30.0);
        assert_eq!(finished.len(), 3);
        assert!(finished.contains(&ITEM_COOKED_BEEF));
        assert!(finished.contains(&ITEM_COOKED_COD));
        assert!(finished.contains(&ITEM_DRIED_KELP));

        // All slots should be empty.
        assert!(campfire.slots.iter().all(|s| s.is_none()));
    }

    // ── Slot freed after completion accepts new item ──────────────────

    #[test]
    fn slot_freed_after_completion() {
        let mut campfire = CampfireState::new();

        assert!(campfire.add_item(ITEM_RAW_BEEF));
        let finished = campfire.tick(30.0);
        assert_eq!(finished.len(), 1);

        // Slot should be free now — add another item.
        assert!(campfire.add_item(ITEM_RAW_SALMON));
        assert!(campfire.slots.iter().any(|s| s.is_some()));
    }

    // ── New campfire defaults ─────────────────────────────────────────

    #[test]
    fn new_campfire_is_lit_with_empty_slots() {
        let campfire = CampfireState::new();
        assert!(campfire.lit);
        assert!(campfire.slots.iter().all(|s| s.is_none()));
    }
}
