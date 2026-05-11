use mc_entity::{
    ArmorSet, HungerComponent, HungerSystem,
    armor::calculate_damage_reduction,
    combat::calculate_fall_damage,
    drowning::{AirMeter, DrownResult},
    fall_damage::calculate_fall_damage as calculate_fall_damage_ex,
    food::food_data,
    survival::{EXHAUSTION_SPRINT_PER_METER, EXHAUSTION_WALK_PER_METER},
};
use mc_ui::damage_indicator::DamageIndicator;

// ---------------------------------------------------------------------------
// Survival state (client-side bridge)
// ---------------------------------------------------------------------------

/// Client-side bridge that bundles health, hunger, and armor into a single
/// struct, delegating to the authoritative systems in `mc_entity`.
pub struct SurvivalState {
    pub health: f32,
    pub max_health: f32,
    pub hunger: HungerComponent,
    pub armor: ArmorSet,

    // Timers owned by the caller of `HungerSystem::tick`.
    regen_timer: f32,
    starve_timer: f32,

    // Drowning / air meter.
    air_meter: AirMeter,

    // Fall damage accumulator.
    fall_distance: f32,

    // Damage indicator overlay.
    damage_indicator: DamageIndicator,
}

impl SurvivalState {
    /// Create a new survival state: 20 HP, full hunger, no armor.
    pub fn new() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            hunger: HungerComponent::default(),
            armor: ArmorSet::new(),
            regen_timer: 0.0,
            starve_timer: 0.0,
            air_meter: AirMeter::new(),
            fall_distance: 0.0,
            damage_indicator: DamageIndicator::new(),
        }
    }

    /// Advance one frame: accumulate exhaustion from movement, then run the
    /// hunger/regen/starvation system. Also handles drowning based on submersion.
    pub fn tick(&mut self, dt: f32, is_sprinting: bool, distance_moved: f32, is_underwater: bool) {
        // Exhaustion from movement.
        if distance_moved > 0.0 {
            let cost_per_meter = if is_sprinting {
                EXHAUSTION_SPRINT_PER_METER
            } else {
                EXHAUSTION_WALK_PER_METER
            };
            HungerSystem::add_exhaustion(&mut self.hunger, cost_per_meter * distance_moved);
        }

        // Drowning logic.
        if is_underwater {
            let result = self.air_meter.tick_underwater();
            if let DrownResult::Drowning(damage) = result {
                self.take_damage(damage);
            }
        } else {
            self.air_meter.tick_above_water();
        }

        // Wrap health in an mc_entity::Health so HungerSystem can operate on
        // it, then copy the result back.
        let mut health = mc_entity::Health {
            current: self.health,
            max: self.max_health,
        };

        HungerSystem::tick(
            &mut self.hunger,
            &mut health,
            dt,
            &mut self.regen_timer,
            &mut self.starve_timer,
        );

        self.health = health.current;
    }

    /// Apply armor-reduced damage. Returns the actual damage taken.
    pub fn take_damage(&mut self, raw_damage: f32) -> f32 {
        let defense = self.armor.total_defense();
        let toughness = self.armor.total_toughness();
        let actual = calculate_damage_reduction(defense, toughness, raw_damage);

        self.health = (self.health - actual).max(0.0);
        self.damage_indicator.trigger(actual, 0.0);
        actual
    }

    /// Calculate and apply fall damage. Returns the damage taken (0 if the
    /// fall was within the free 3-block threshold).
    pub fn take_fall_damage(&mut self, fall_distance: f32) -> f32 {
        let raw = calculate_fall_damage(fall_distance);
        if raw <= 0.0 {
            return 0.0;
        }
        self.take_damage(raw)
    }

    /// Consume food, restoring hunger and saturation.
    pub fn eat(&mut self, food_value: u32, saturation: f32) {
        HungerSystem::eat(&mut self.hunger, food_value, saturation);
    }

    /// Attempt to eat an item by ID. Looks up food data and applies hunger +
    /// saturation if the item is a recognized food. Returns `true` on success.
    pub fn eat_item(&mut self, item_id: u16) -> bool {
        if let Some(food) = food_data(item_id) {
            HungerSystem::eat(&mut self.hunger, food.hunger, food.saturation);
            true
        } else {
            false
        }
    }

    /// Update fall distance accumulation and apply fall damage on landing.
    ///
    /// Call each frame with the current vertical velocity, ground state, and dt.
    /// Damage is applied (via `take_damage`) when the player lands after falling.
    pub fn update_fall(&mut self, velocity_y: f32, on_ground: bool, dt: f32) {
        if !on_ground && velocity_y < 0.0 {
            self.fall_distance += -velocity_y * dt;
        } else if on_ground && self.fall_distance > 0.0 {
            let raw = calculate_fall_damage_ex(self.fall_distance, 0) as f32;
            if raw > 0.0 {
                self.take_damage(raw);
            }
            self.fall_distance = 0.0;
        }
    }

    /// Advance the damage indicator overlay by `dt` seconds.
    pub fn tick_indicator(&mut self, dt: f32) {
        self.damage_indicator.tick(dt);
    }

    /// Returns the current red-flash overlay alpha (0.0 = invisible).
    pub fn damage_overlay_alpha(&self) -> f32 {
        self.damage_indicator.red_alpha
    }

    /// Returns the fraction of remaining air (0.0 to 1.0) for HUD rendering.
    pub fn air_fraction(&self) -> f32 {
        self.air_meter.air_fraction()
    }

    /// Heal the player by `amount`, clamped to `max_health`.
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Returns `true` when health has reached zero.
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Reset to full HP and default hunger (respawn).
    pub fn respawn(&mut self) {
        self.health = self.max_health;
        self.hunger = HungerComponent::default();
        self.regen_timer = 0.0;
        self.starve_timer = 0.0;
        self.air_meter = AirMeter::new();
        self.fall_distance = 0.0;
        self.damage_indicator = DamageIndicator::new();
    }

    // -- HUD helpers --------------------------------------------------------

    /// Current health for the HUD (0.0..=20.0).
    pub fn hud_health(&self) -> f32 {
        self.health
    }

    /// Current food level for the HUD (0..=20).
    pub fn hud_hunger(&self) -> u32 {
        self.hunger.food_level
    }

    /// Total armor defense points for the HUD (0..=20).
    pub fn hud_armor(&self) -> u32 {
        self.armor.total_defense()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mc_entity::{ArmorMaterial, ArmorPiece, ArmorSlot};

    #[test]
    fn new_state_has_full_health_and_hunger() {
        let s = SurvivalState::new();
        assert!((s.health - 20.0).abs() < f32::EPSILON);
        assert_eq!(s.hunger.food_level, 20);
        assert!(!s.is_dead());
    }

    #[test]
    fn take_damage_without_armor_deals_full_damage() {
        let mut s = SurvivalState::new();
        let actual = s.take_damage(6.0);
        assert!((actual - 6.0).abs() < f32::EPSILON);
        assert!((s.health - 14.0).abs() < f32::EPSILON);
    }

    #[test]
    fn take_damage_with_armor_reduces_damage() {
        let mut s = SurvivalState::new();
        s.armor.equip(ArmorPiece::new(
            ArmorMaterial::Diamond,
            ArmorSlot::Chestplate,
        ));
        let actual = s.take_damage(10.0);
        assert!(actual < 10.0, "armor should reduce damage, got {actual}");
    }

    #[test]
    fn take_fall_damage_below_threshold_is_zero() {
        let mut s = SurvivalState::new();
        let dmg = s.take_fall_damage(3.0);
        assert!((dmg).abs() < f32::EPSILON);
        assert!((s.health - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn take_fall_damage_above_threshold() {
        let mut s = SurvivalState::new();
        let dmg = s.take_fall_damage(10.0);
        // Raw fall damage = 7.0; no armor so actual = 7.0
        assert!((dmg - 7.0).abs() < f32::EPSILON);
        assert!((s.health - 13.0).abs() < f32::EPSILON);
    }

    #[test]
    fn eat_restores_hunger() {
        let mut s = SurvivalState::new();
        s.hunger.food_level = 10;
        s.eat(5, 3.0);
        assert_eq!(s.hunger.food_level, 15);
    }

    #[test]
    fn heal_clamps_to_max() {
        let mut s = SurvivalState::new();
        s.health = 10.0;
        s.heal(100.0);
        assert!((s.health - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn is_dead_at_zero_health() {
        let mut s = SurvivalState::new();
        s.health = 0.0;
        assert!(s.is_dead());
    }

    #[test]
    fn respawn_resets_state() {
        let mut s = SurvivalState::new();
        s.health = 0.0;
        s.hunger.food_level = 0;
        s.respawn();
        assert!((s.health - 20.0).abs() < f32::EPSILON);
        assert_eq!(s.hunger.food_level, 20);
    }

    #[test]
    fn hud_health_returns_current() {
        let mut s = SurvivalState::new();
        s.health = 12.5;
        assert!((s.hud_health() - 12.5).abs() < f32::EPSILON);
    }

    #[test]
    fn hud_hunger_returns_food_level() {
        let mut s = SurvivalState::new();
        s.hunger.food_level = 8;
        assert_eq!(s.hud_hunger(), 8);
    }

    #[test]
    fn hud_armor_returns_total_defense() {
        let mut s = SurvivalState::new();
        s.armor
            .equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Helmet));
        s.armor
            .equip(ArmorPiece::new(ArmorMaterial::Iron, ArmorSlot::Chestplate));
        // 2 + 6 = 8
        assert_eq!(s.hud_armor(), 8);
    }

    #[test]
    fn tick_with_sprinting_adds_more_exhaustion() {
        let mut walk = SurvivalState::new();
        let mut sprint = SurvivalState::new();

        walk.tick(0.05, false, 10.0, false);
        sprint.tick(0.05, true, 10.0, false);

        assert!(
            sprint.hunger.exhaustion > walk.hunger.exhaustion,
            "sprinting should produce more exhaustion"
        );
    }

    #[test]
    fn tick_starvation_reduces_health() {
        let mut s = SurvivalState::new();
        s.hunger.food_level = 0;
        s.hunger.saturation = 0.0;

        // Tick long enough for starvation (4s interval).
        s.tick(5.0, false, 0.0, false);
        assert!(s.health < 20.0, "starvation should reduce health");
    }

    // -- Food eating (eat_item) ------------------------------------------------

    #[test]
    fn eat_item_known_food_restores_hunger() {
        let mut s = SurvivalState::new();
        s.hunger.food_level = 10;
        // Apple (id 3000): hunger = 4, saturation = 2.4
        assert!(s.eat_item(3000));
        assert_eq!(s.hunger.food_level, 14);
    }

    #[test]
    fn eat_item_unknown_returns_false() {
        let mut s = SurvivalState::new();
        s.hunger.food_level = 10;
        assert!(!s.eat_item(9999));
        assert_eq!(s.hunger.food_level, 10);
    }

    // -- Drowning --------------------------------------------------------------

    #[test]
    fn tick_underwater_decrements_air() {
        let mut s = SurvivalState::new();
        s.tick(0.05, false, 0.0, true);
        assert!(s.air_fraction() < 1.0);
    }

    #[test]
    fn tick_above_water_restores_air() {
        let mut s = SurvivalState::new();
        // Drain some air
        s.tick(0.05, false, 0.0, true);
        let after_drain = s.air_fraction();
        // Restore
        s.tick(0.05, false, 0.0, false);
        assert!(s.air_fraction() > after_drain);
    }

    #[test]
    fn drowning_deals_damage() {
        let mut s = SurvivalState::new();
        // Fully drain the air meter (300 ticks underwater)
        for _ in 0..300 {
            s.tick(0.05, false, 0.0, true);
        }
        // One more underwater tick should trigger drowning damage
        s.tick(0.05, false, 0.0, true);
        assert!(s.health < 20.0, "drowning should reduce health");
    }

    #[test]
    fn air_fraction_starts_at_one() {
        let s = SurvivalState::new();
        assert!((s.air_fraction() - 1.0).abs() < f32::EPSILON);
    }

    // -- Fall damage -----------------------------------------------------------

    #[test]
    fn update_fall_accumulates_while_airborne() {
        let mut s = SurvivalState::new();
        // Falling at -10 m/s for 0.5s = 5 blocks
        s.update_fall(-10.0, false, 0.5);
        assert!((s.health - 20.0).abs() < f32::EPSILON, "no damage mid-air");
    }

    #[test]
    fn update_fall_applies_damage_on_landing() {
        let mut s = SurvivalState::new();
        // Accumulate 10 blocks of fall
        s.update_fall(-20.0, false, 0.5);
        // Land
        s.update_fall(0.0, true, 0.05);
        // 10 blocks -> base damage = floor(10 - 3) = 7
        assert!(s.health < 20.0, "landing should deal fall damage");
    }

    #[test]
    fn update_fall_no_damage_below_threshold() {
        let mut s = SurvivalState::new();
        // Accumulate only 2 blocks of fall (below 3-block threshold)
        s.update_fall(-4.0, false, 0.5); // 2 blocks
        s.update_fall(0.0, true, 0.05);
        assert!((s.health - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn update_fall_resets_distance_on_ground() {
        let mut s = SurvivalState::new();
        s.update_fall(-20.0, false, 0.5);
        s.update_fall(0.0, true, 0.05);
        // Second landing with no new fall should deal no additional damage
        let health_after_first = s.health;
        s.update_fall(0.0, true, 0.05);
        assert!((s.health - health_after_first).abs() < f32::EPSILON);
    }

    // -- Damage indicator ------------------------------------------------------

    #[test]
    fn take_damage_triggers_indicator() {
        let mut s = SurvivalState::new();
        s.take_damage(10.0);
        assert!(s.damage_overlay_alpha() > 0.0);
    }

    #[test]
    fn tick_indicator_fades_overlay() {
        let mut s = SurvivalState::new();
        s.take_damage(10.0);
        let initial = s.damage_overlay_alpha();
        s.tick_indicator(0.25);
        assert!(s.damage_overlay_alpha() < initial);
    }

    #[test]
    fn tick_indicator_eventually_zeroes() {
        let mut s = SurvivalState::new();
        s.take_damage(10.0);
        s.tick_indicator(1.0);
        assert!((s.damage_overlay_alpha()).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_overlay_alpha_starts_at_zero() {
        let s = SurvivalState::new();
        assert!((s.damage_overlay_alpha()).abs() < f32::EPSILON);
    }
}
