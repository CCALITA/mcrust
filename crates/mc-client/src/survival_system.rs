use mc_entity::{
    ArmorSet, HungerComponent, HungerSystem,
    armor::calculate_damage_reduction,
    combat::calculate_fall_damage,
    survival::{EXHAUSTION_SPRINT_PER_METER, EXHAUSTION_WALK_PER_METER},
};

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
        }
    }

    /// Advance one frame: accumulate exhaustion from movement, then run the
    /// hunger/regen/starvation system.
    pub fn tick(&mut self, dt: f32, is_sprinting: bool, distance_moved: f32) {
        // Exhaustion from movement.
        if distance_moved > 0.0 {
            let cost_per_meter = if is_sprinting {
                EXHAUSTION_SPRINT_PER_METER
            } else {
                EXHAUSTION_WALK_PER_METER
            };
            HungerSystem::add_exhaustion(&mut self.hunger, cost_per_meter * distance_moved);
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

        walk.tick(0.05, false, 10.0);
        sprint.tick(0.05, true, 10.0);

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
        s.tick(5.0, false, 0.0);
        assert!(s.health < 20.0, "starvation should reduce health");
    }
}
