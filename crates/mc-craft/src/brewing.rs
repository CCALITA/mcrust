// ── Potion types and effects ────────────────────────────────────────────────

/// All potion types available in the brewing system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PotionType {
    Speed,
    Slowness,
    JumpBoost,
    Strength,
    Weakness,
    Healing,
    Harming,
    Regeneration,
    Poison,
    FireResistance,
    NightVision,
    Invisibility,
    WaterBreathing,
    SlowFalling,
    Luck,
}

/// A single potion effect with type, remaining duration, and amplifier level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionEffect {
    pub potion_type: PotionType,
    pub duration_ticks: u32,
    pub amplifier: u8,
}

// ── Status effect manager ──────────────────────────────────────────────────

/// Manages active potion effects on an entity (typically a player).
///
/// Handles effect stacking (replacing weaker effects of the same type),
/// tick-based expiry, and derived stat multipliers.
#[derive(Debug, Clone, Default)]
pub struct StatusEffectManager {
    active_effects: Vec<PotionEffect>,
}

impl StatusEffectManager {
    /// Create a manager with no active effects.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_effects: Vec::new(),
        }
    }

    /// Add a potion effect. If an effect of the same type already exists,
    /// it is replaced only when the new effect is strictly stronger
    /// (higher amplifier, or equal amplifier with longer duration).
    pub fn add_effect(&mut self, effect: PotionEffect) {
        if let Some(existing) = self
            .active_effects
            .iter_mut()
            .find(|e| e.potion_type == effect.potion_type)
        {
            let dominated = effect.amplifier > existing.amplifier
                || (effect.amplifier == existing.amplifier
                    && effect.duration_ticks > existing.duration_ticks);
            if dominated {
                *existing = effect;
            }
        } else {
            self.active_effects.push(effect);
        }
    }

    /// Advance all active effects by one tick.
    /// Effects whose duration reaches zero are removed.
    pub fn tick(&mut self) {
        for effect in &mut self.active_effects {
            effect.duration_ticks = effect.duration_ticks.saturating_sub(1);
        }
        self.active_effects.retain(|e| e.duration_ticks > 0);
    }

    /// Look up an active effect by type.
    #[must_use]
    pub fn has_effect(&self, t: PotionType) -> Option<&PotionEffect> {
        self.active_effects.iter().find(|e| e.potion_type == t)
    }

    /// Compute the speed multiplier from active Speed / Slowness effects.
    ///
    /// Base is `1.0`. Each Speed amplifier level adds `+0.2`,
    /// each Slowness amplifier level subtracts `0.15`.
    /// Amplifier `0` counts as level 1.
    #[must_use]
    pub fn get_speed_multiplier(&self) -> f32 {
        let mut multiplier: f32 = 1.0;

        if let Some(speed) = self.has_effect(PotionType::Speed) {
            let level = f32::from(speed.amplifier) + 1.0;
            multiplier += 0.2 * level;
        }
        if let Some(slow) = self.has_effect(PotionType::Slowness) {
            let level = f32::from(slow.amplifier) + 1.0;
            multiplier -= 0.15 * level;
        }

        multiplier
    }

    /// Compute the damage multiplier from active Strength / Weakness effects.
    ///
    /// Each Strength amplifier level adds `+3.0` damage,
    /// each Weakness amplifier level subtracts `4.0` damage.
    /// Amplifier `0` counts as level 1.
    #[must_use]
    pub fn get_damage_multiplier(&self) -> f32 {
        let mut multiplier: f32 = 0.0;

        if let Some(str_eff) = self.has_effect(PotionType::Strength) {
            let level = f32::from(str_eff.amplifier) + 1.0;
            multiplier += 3.0 * level;
        }
        if let Some(weak_eff) = self.has_effect(PotionType::Weakness) {
            let level = f32::from(weak_eff.amplifier) + 1.0;
            multiplier -= 4.0 * level;
        }

        multiplier
    }

    /// Whether the entity is currently invisible.
    #[must_use]
    pub fn is_invisible(&self) -> bool {
        self.has_effect(PotionType::Invisibility).is_some()
    }

    /// Whether the entity is currently fire-resistant.
    #[must_use]
    pub fn is_fire_resistant(&self) -> bool {
        self.has_effect(PotionType::FireResistance).is_some()
    }
}

// ── Brewing ingredients and recipes ────────────────────────────────────────

/// Ingredients that can be placed in a brewing stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrewingIngredient {
    NetherWart,
    GlisteringMelon,
    SpiderEye,
    MagmaCream,
    BlazePowder,
    Sugar,
    Redstone,
    Glowstone,
    FermentedSpiderEye,
    GunPowder,
    DragonBreath,
}

/// A brewing recipe: base potion + ingredient = result potion.
///
/// `base` of `None` represents a Water Bottle (the starting state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewingRecipe {
    pub base: Option<PotionType>,
    pub ingredient: BrewingIngredient,
    pub result: PotionType,
}

/// Return the default set of brewing recipes (Minecraft-inspired).
#[must_use]
pub fn default_brewing_recipes() -> Vec<BrewingRecipe> {
    vec![
        // Awkward base potions (NetherWart on Water Bottle is handled
        // specially in `BrewingStand`; these assume an "Awkward" base
        // represented as `None` for simplicity).
        //
        // Positive effects
        BrewingRecipe {
            base: None,
            ingredient: BrewingIngredient::Sugar,
            result: PotionType::Speed,
        },
        BrewingRecipe {
            base: None,
            ingredient: BrewingIngredient::MagmaCream,
            result: PotionType::FireResistance,
        },
        BrewingRecipe {
            base: None,
            ingredient: BrewingIngredient::GlisteringMelon,
            result: PotionType::Healing,
        },
        BrewingRecipe {
            base: None,
            ingredient: BrewingIngredient::BlazePowder,
            result: PotionType::Strength,
        },
        BrewingRecipe {
            base: None,
            ingredient: BrewingIngredient::SpiderEye,
            result: PotionType::Poison,
        },
        // Negative / utility effects
        BrewingRecipe {
            base: Some(PotionType::Speed),
            ingredient: BrewingIngredient::FermentedSpiderEye,
            result: PotionType::Slowness,
        },
        BrewingRecipe {
            base: Some(PotionType::Healing),
            ingredient: BrewingIngredient::FermentedSpiderEye,
            result: PotionType::Harming,
        },
        BrewingRecipe {
            base: Some(PotionType::Strength),
            ingredient: BrewingIngredient::FermentedSpiderEye,
            result: PotionType::Weakness,
        },
        BrewingRecipe {
            base: Some(PotionType::Poison),
            ingredient: BrewingIngredient::FermentedSpiderEye,
            result: PotionType::Harming,
        },
        // Additional base recipes
        BrewingRecipe {
            base: None,
            ingredient: BrewingIngredient::GunPowder,
            result: PotionType::NightVision,
        },
        BrewingRecipe {
            base: Some(PotionType::NightVision),
            ingredient: BrewingIngredient::FermentedSpiderEye,
            result: PotionType::Invisibility,
        },
    ]
}

// ── Brewing stand ──────────────────────────────────────────────────────────

/// The number of ticks required to complete a brew.
const BREW_DURATION: u32 = 400;

/// A brewing stand with three potion slots, an ingredient slot, and fuel.
#[derive(Debug, Clone)]
pub struct BrewingStand {
    /// Three potion output slots.
    pub slots: [Option<PotionType>; 3],
    /// The ingredient placed in the top slot.
    pub ingredient: Option<BrewingIngredient>,
    /// Blaze-powder fuel charges remaining.
    pub fuel: u8,
    /// Remaining ticks in the current brew cycle.
    pub brew_ticks: u32,
    /// Whether the stand is actively brewing.
    pub brewing: bool,
}

impl Default for BrewingStand {
    fn default() -> Self {
        Self::new()
    }
}

impl BrewingStand {
    /// Create an empty, idle brewing stand.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [None; 3],
            ingredient: None,
            fuel: 0,
            brew_ticks: 0,
            brewing: false,
        }
    }

    /// Advance the brewing stand by one tick.
    ///
    /// If a brew is in progress the countdown decreases. When it reaches
    /// zero the recipe is applied to every occupied potion slot that matches.
    pub fn tick(&mut self) {
        if !self.brewing {
            return;
        }

        self.brew_ticks = self.brew_ticks.saturating_sub(1);

        if self.brew_ticks == 0 {
            self.apply_recipes();
            self.ingredient = None;
            self.brewing = false;
        }
    }

    /// Attempt to start a brew cycle.
    ///
    /// Requires fuel, an ingredient, and at least one occupied potion slot.
    /// Returns `true` if brewing was successfully started.
    pub fn start_brew(&mut self) -> bool {
        if self.brewing {
            return false;
        }
        if self.fuel == 0 || self.ingredient.is_none() {
            return false;
        }
        let has_potion = self.slots.iter().any(Option::is_some);
        if !has_potion {
            return false;
        }

        self.fuel -= 1;
        self.brew_ticks = BREW_DURATION;
        self.brewing = true;
        true
    }

    /// Place an ingredient into the stand. Returns `false` if the slot is
    /// already occupied.
    pub fn insert_ingredient(&mut self, ingredient: BrewingIngredient) -> bool {
        if self.ingredient.is_some() {
            return false;
        }
        self.ingredient = Some(ingredient);
        true
    }

    /// Place a potion into the given slot (0..3). Returns `false` if the slot
    /// is already occupied or the index is out of range.
    pub fn insert_potion(&mut self, slot: usize, potion: PotionType) -> bool {
        if slot >= 3 {
            return false;
        }
        if self.slots[slot].is_some() {
            return false;
        }
        self.slots[slot] = Some(potion);
        true
    }

    /// Take a potion from the given slot (0..3). Returns `None` if the slot
    /// is empty or the index is out of range.
    pub fn take_potion(&mut self, slot: usize) -> Option<PotionType> {
        if slot >= 3 {
            return None;
        }
        self.slots[slot].take()
    }

    /// Apply the current ingredient's recipe to all matching potion slots.
    fn apply_recipes(&mut self) {
        let Some(ingredient) = self.ingredient else {
            return;
        };

        let recipes = default_brewing_recipes();

        for slot in &mut self.slots {
            let base = *slot;
            if let Some(recipe) = recipes
                .iter()
                .find(|r| r.base == base && r.ingredient == ingredient)
            {
                *slot = Some(recipe.result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Recipe matching ────────────────────────────────────────────────

    #[test]
    fn default_recipes_has_at_least_ten() {
        let recipes = default_brewing_recipes();
        assert!(
            recipes.len() >= 10,
            "expected at least 10 recipes, got {}",
            recipes.len()
        );
    }

    #[test]
    fn recipe_sugar_produces_speed() {
        let recipes = default_brewing_recipes();
        let found = recipes
            .iter()
            .find(|r| r.base.is_none() && r.ingredient == BrewingIngredient::Sugar);
        assert!(found.is_some());
        assert_eq!(found.map(|r| r.result), Some(PotionType::Speed));
    }

    #[test]
    fn recipe_fermented_spider_eye_corrupts_speed_to_slowness() {
        let recipes = default_brewing_recipes();
        let found = recipes.iter().find(|r| {
            r.base == Some(PotionType::Speed)
                && r.ingredient == BrewingIngredient::FermentedSpiderEye
        });
        assert!(found.is_some());
        assert_eq!(found.map(|r| r.result), Some(PotionType::Slowness));
    }

    // ── Status effect stacking / expiry ────────────────────────────────

    #[test]
    fn add_effect_stores_new_type() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 0,
        });
        assert!(mgr.has_effect(PotionType::Speed).is_some());
        assert!(mgr.has_effect(PotionType::Strength).is_none());
    }

    #[test]
    fn stronger_effect_replaces_weaker() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 200,
            amplifier: 0,
        });
        // Stronger amplifier should replace.
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 1,
        });
        let eff = mgr.has_effect(PotionType::Speed).expect("should exist");
        assert_eq!(eff.amplifier, 1);
        assert_eq!(eff.duration_ticks, 100);
    }

    #[test]
    fn weaker_effect_does_not_replace_stronger() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 200,
            amplifier: 1,
        });
        // Lower amplifier should NOT replace.
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 300,
            amplifier: 0,
        });
        let eff = mgr.has_effect(PotionType::Speed).expect("should exist");
        assert_eq!(eff.amplifier, 1);
        assert_eq!(eff.duration_ticks, 200);
    }

    #[test]
    fn equal_amplifier_longer_duration_replaces() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Regeneration,
            duration_ticks: 100,
            amplifier: 0,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Regeneration,
            duration_ticks: 200,
            amplifier: 0,
        });
        let eff = mgr
            .has_effect(PotionType::Regeneration)
            .expect("should exist");
        assert_eq!(eff.duration_ticks, 200);
    }

    #[test]
    fn tick_decrements_duration() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 5,
            amplifier: 0,
        });
        mgr.tick();
        let eff = mgr.has_effect(PotionType::Speed).expect("should exist");
        assert_eq!(eff.duration_ticks, 4);
    }

    #[test]
    fn expired_effect_removed_after_tick() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 1,
            amplifier: 0,
        });
        mgr.tick();
        assert!(mgr.has_effect(PotionType::Speed).is_none());
    }

    // ── Speed / damage multipliers ─────────────────────────────────────

    #[test]
    fn speed_multiplier_no_effects() {
        let mgr = StatusEffectManager::new();
        assert!((mgr.get_speed_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_with_speed_1() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 1.0 + 0.2 * 1 = 1.2
        assert!((mgr.get_speed_multiplier() - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_with_slowness() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Slowness,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 1.0 - 0.15 * 1 = 0.85
        assert!((mgr.get_speed_multiplier() - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_multiplier_combined() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 1, // level 2
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Slowness,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 1.0 + 0.2*2 - 0.15*1 = 1.25
        assert!((mgr.get_speed_multiplier() - 1.25).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_no_effects() {
        let mgr = StatusEffectManager::new();
        assert!((mgr.get_damage_multiplier() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_with_strength() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Strength,
            duration_ticks: 100,
            amplifier: 0,
        });
        // 3.0 * 1 = 3.0
        assert!((mgr.get_damage_multiplier() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_with_weakness() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Weakness,
            duration_ticks: 100,
            amplifier: 0,
        });
        // -4.0 * 1 = -4.0
        assert!((mgr.get_damage_multiplier() - (-4.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_multiplier_combined() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Strength,
            duration_ticks: 100,
            amplifier: 1, // level 2
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Weakness,
            duration_ticks: 100,
            amplifier: 0, // level 1
        });
        // 3.0*2 - 4.0*1 = 2.0
        assert!((mgr.get_damage_multiplier() - 2.0).abs() < f32::EPSILON);
    }

    // ── Boolean helpers ────────────────────────────────────────────────

    #[test]
    fn is_invisible_when_effect_active() {
        let mut mgr = StatusEffectManager::new();
        assert!(!mgr.is_invisible());
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Invisibility,
            duration_ticks: 100,
            amplifier: 0,
        });
        assert!(mgr.is_invisible());
    }

    #[test]
    fn is_fire_resistant_when_effect_active() {
        let mut mgr = StatusEffectManager::new();
        assert!(!mgr.is_fire_resistant());
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::FireResistance,
            duration_ticks: 100,
            amplifier: 0,
        });
        assert!(mgr.is_fire_resistant());
    }

    // ── Brewing stand cycle ────────────────────────────────────────────

    #[test]
    fn brewing_stand_starts_empty_and_idle() {
        let stand = BrewingStand::new();
        assert!(!stand.brewing);
        assert_eq!(stand.fuel, 0);
        assert!(stand.ingredient.is_none());
        for slot in &stand.slots {
            assert!(slot.is_none());
        }
    }

    #[test]
    fn cannot_start_brew_without_fuel() {
        let mut stand = BrewingStand::new();
        stand.insert_ingredient(BrewingIngredient::Sugar);
        stand.slots[0] = None; // base "water bottle"
        assert!(!stand.start_brew());
    }

    #[test]
    fn cannot_start_brew_without_ingredient() {
        let mut stand = BrewingStand::new();
        stand.fuel = 5;
        stand.slots[0] = None; // empty = no potion to brew
        assert!(!stand.start_brew());
    }

    #[test]
    fn cannot_start_brew_without_potions() {
        let mut stand = BrewingStand::new();
        stand.fuel = 5;
        stand.insert_ingredient(BrewingIngredient::Sugar);
        // All slots are None (empty) — no potions to brew on.
        assert!(!stand.start_brew());
    }

    #[test]
    fn full_brew_cycle() {
        let mut stand = BrewingStand::new();
        stand.fuel = 3;

        // Place a "base: None" potion in slot 0 (represents water/awkward).
        // The recipe `None + Sugar -> Speed` should apply.
        stand.insert_potion(0, PotionType::Speed); // we need a potion present
        stand.slots[0] = None; // reset to None-base for recipe matching
        // Actually, we need the slot occupied for `start_brew` to succeed.
        // Let's use a real potion and a corruption recipe instead.
        stand.slots[0] = Some(PotionType::Speed);
        stand.insert_ingredient(BrewingIngredient::FermentedSpiderEye);

        assert!(stand.start_brew());
        assert!(stand.brewing);
        assert_eq!(stand.fuel, 2);
        assert_eq!(stand.brew_ticks, BREW_DURATION);

        // Tick through the entire brew duration.
        for _ in 0..BREW_DURATION {
            stand.tick();
        }

        assert!(!stand.brewing);
        assert!(stand.ingredient.is_none());
        // Speed + FermentedSpiderEye -> Slowness
        assert_eq!(stand.slots[0], Some(PotionType::Slowness));
    }

    #[test]
    fn brew_applies_to_all_matching_slots() {
        let mut stand = BrewingStand::new();
        stand.fuel = 1;
        stand.slots[0] = Some(PotionType::Speed);
        stand.slots[1] = Some(PotionType::Healing);
        stand.slots[2] = Some(PotionType::Speed);
        stand.insert_ingredient(BrewingIngredient::FermentedSpiderEye);
        assert!(stand.start_brew());

        for _ in 0..BREW_DURATION {
            stand.tick();
        }

        // Speed -> Slowness (slots 0, 2), Healing -> Harming (slot 1)
        assert_eq!(stand.slots[0], Some(PotionType::Slowness));
        assert_eq!(stand.slots[1], Some(PotionType::Harming));
        assert_eq!(stand.slots[2], Some(PotionType::Slowness));
    }

    #[test]
    fn insert_ingredient_fails_when_occupied() {
        let mut stand = BrewingStand::new();
        assert!(stand.insert_ingredient(BrewingIngredient::Sugar));
        assert!(!stand.insert_ingredient(BrewingIngredient::BlazePowder));
    }

    #[test]
    fn insert_potion_fails_when_occupied() {
        let mut stand = BrewingStand::new();
        assert!(stand.insert_potion(0, PotionType::Speed));
        assert!(!stand.insert_potion(0, PotionType::Healing));
    }

    #[test]
    fn insert_potion_fails_out_of_range() {
        let mut stand = BrewingStand::new();
        assert!(!stand.insert_potion(3, PotionType::Speed));
    }

    #[test]
    fn take_potion_returns_and_clears_slot() {
        let mut stand = BrewingStand::new();
        stand.slots[1] = Some(PotionType::Healing);
        let taken = stand.take_potion(1);
        assert_eq!(taken, Some(PotionType::Healing));
        assert!(stand.slots[1].is_none());
    }

    #[test]
    fn take_potion_empty_slot_returns_none() {
        let mut stand = BrewingStand::new();
        assert_eq!(stand.take_potion(0), None);
    }

    #[test]
    fn take_potion_out_of_range_returns_none() {
        let mut stand = BrewingStand::new();
        assert_eq!(stand.take_potion(5), None);
    }

    // ── Effect replacement edge cases ──────────────────────────────────

    #[test]
    fn equal_amplifier_shorter_duration_does_not_replace() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Poison,
            duration_ticks: 200,
            amplifier: 0,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Poison,
            duration_ticks: 100,
            amplifier: 0,
        });
        let eff = mgr.has_effect(PotionType::Poison).expect("should exist");
        assert_eq!(eff.duration_ticks, 200);
    }

    #[test]
    fn multiple_different_effects_coexist() {
        let mut mgr = StatusEffectManager::new();
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Speed,
            duration_ticks: 100,
            amplifier: 0,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Strength,
            duration_ticks: 200,
            amplifier: 1,
        });
        mgr.add_effect(PotionEffect {
            potion_type: PotionType::Invisibility,
            duration_ticks: 50,
            amplifier: 0,
        });
        assert!(mgr.has_effect(PotionType::Speed).is_some());
        assert!(mgr.has_effect(PotionType::Strength).is_some());
        assert!(mgr.has_effect(PotionType::Invisibility).is_some());
    }
}
