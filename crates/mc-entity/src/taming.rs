use glam::Vec3;

// ---------------------------------------------------------------------------
// Mob kind / item ID constants
// ---------------------------------------------------------------------------

/// Mob kind identifiers used for taming and breeding logic.
const MOB_WOLF: u8 = 1;
const MOB_CAT: u8 = 2;
const MOB_HORSE: u8 = 3;
const MOB_COW: u8 = 10;
const MOB_SHEEP: u8 = 11;
const MOB_PIG: u8 = 12;
const MOB_CHICKEN: u8 = 13;

/// Item IDs for taming/breeding consumables.
const ITEM_BONE: u16 = 200;
const ITEM_RAW_FISH: u16 = 201;
const ITEM_WHEAT: u16 = 300;
const ITEM_CARROT: u16 = 301;
const ITEM_SEEDS: u16 = 302;

// ---------------------------------------------------------------------------
// Taming
// ---------------------------------------------------------------------------

/// Tracks whether a mob has been tamed, who owns it, and whether it is sitting.
#[derive(Debug, Clone)]
pub struct TameableComponent {
    pub owner: Option<u64>,
    pub tamed: bool,
    pub sitting: bool,
}

impl TameableComponent {
    /// Create an untamed component.
    pub fn new() -> Self {
        Self {
            owner: None,
            tamed: false,
            sitting: false,
        }
    }

    /// Tame this mob, assigning it to `owner_id`.
    pub fn tame(&mut self, owner_id: u64) {
        self.owner = Some(owner_id);
        self.tamed = true;
        self.sitting = false;
    }

    /// Toggle the sitting state (only meaningful when tamed).
    pub fn toggle_sit(&mut self) {
        if self.tamed {
            self.sitting = !self.sitting;
        }
    }
}

impl Default for TameableComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// Attempt to tame a mob given its kind, the item used, and a random value
/// in `0.0..1.0`.
///
/// * Wolf + bone: 33% chance
/// * Cat + raw fish: 33% chance
/// * Horse (any item): 10% chance
///
/// Returns `true` if the taming attempt succeeds.
pub fn try_tame(mob_kind: u8, item_used: u16, random_val: f32) -> bool {
    match (mob_kind, item_used) {
        (MOB_WOLF, ITEM_BONE) => random_val < 1.0 / 3.0,
        (MOB_CAT, ITEM_RAW_FISH) => random_val < 1.0 / 3.0,
        (MOB_HORSE, _) => random_val < 0.1,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Breeding
// ---------------------------------------------------------------------------

/// Tracks love-mode state for a breedable mob.
#[derive(Debug, Clone)]
pub struct BreedingComponent {
    pub love_mode: bool,
    pub love_timer: f32,
    pub breed_cooldown: f32,
}

impl BreedingComponent {
    /// Duration (seconds) that love mode lasts before expiring.
    const LOVE_DURATION: f32 = 30.0;

    pub fn new() -> Self {
        Self {
            love_mode: false,
            love_timer: 0.0,
            breed_cooldown: 0.0,
        }
    }

    /// Advance timers by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if self.love_mode {
            self.love_timer -= dt;
            if self.love_timer <= 0.0 {
                self.love_mode = false;
                self.love_timer = 0.0;
            }
        }
        if self.breed_cooldown > 0.0 {
            self.breed_cooldown = (self.breed_cooldown - dt).max(0.0);
        }
    }

    /// Enter love mode (resets the love timer).
    pub fn enter_love_mode(&mut self) {
        self.love_mode = true;
        self.love_timer = Self::LOVE_DURATION;
    }
}

impl Default for BreedingComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of attempting to feed a breeding item to a mob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedResult {
    EnteredLoveMode,
    AlreadyInLove,
    WrongFood,
    OnCooldown,
}

/// Returns `true` when `item` is the correct breeding food for `mob_kind`.
fn is_breeding_food(mob_kind: u8, item: u16) -> bool {
    matches!(
        (mob_kind, item),
        (MOB_COW, ITEM_WHEAT)
            | (MOB_SHEEP, ITEM_WHEAT)
            | (MOB_PIG, ITEM_CARROT)
            | (MOB_CHICKEN, ITEM_SEEDS)
    )
}

/// Feed an animal with a potential breeding item.
///
/// * Returns `WrongFood` if the item does not match the mob kind.
/// * Returns `OnCooldown` if the breed cooldown has not expired.
/// * Returns `AlreadyInLove` if already in love mode.
/// * Returns `EnteredLoveMode` on success.
pub fn feed_animal(breeding: &mut BreedingComponent, mob_kind: u8, item: u16) -> FeedResult {
    if !is_breeding_food(mob_kind, item) {
        return FeedResult::WrongFood;
    }
    if breeding.breed_cooldown > 0.0 {
        return FeedResult::OnCooldown;
    }
    if breeding.love_mode {
        return FeedResult::AlreadyInLove;
    }
    breeding.enter_love_mode();
    FeedResult::EnteredLoveMode
}

/// Attempt to breed two mobs.
///
/// * `pos1` / `pos2` — positions of the two parents.
/// * `both_in_love` — whether both parents are currently in love mode.
///
/// Returns the midpoint spawn position of the baby if breeding succeeds,
/// or `None` if the precondition is not met.
pub fn try_breed(pos1: Vec3, pos2: Vec3, both_in_love: bool) -> Option<Vec3> {
    if !both_in_love {
        return None;
    }
    Some((pos1 + pos2) * 0.5)
}

// ---------------------------------------------------------------------------
// Baby mobs
// ---------------------------------------------------------------------------

/// Tracks the age of a baby mob.
///
/// A baby becomes an adult after 1200 seconds (20 real-time minutes).
#[derive(Debug, Clone)]
pub struct BabyMob {
    pub age_timer: f32,
}

impl BabyMob {
    /// Time in seconds until a baby becomes an adult.
    pub const GROWTH_TIME: f32 = 1200.0;

    pub fn new() -> Self {
        Self { age_timer: 0.0 }
    }

    /// Advance the age timer by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        self.age_timer = (self.age_timer + dt).min(Self::GROWTH_TIME);
    }

    /// Returns `true` when the baby has reached adulthood.
    pub fn is_adult(&self) -> bool {
        self.age_timer >= Self::GROWTH_TIME
    }
}

impl Default for BabyMob {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Taming -------------------------------------------------------------

    #[test]
    fn wolf_tamed_with_bone_succeeds_below_threshold() {
        assert!(try_tame(MOB_WOLF, ITEM_BONE, 0.0));
        assert!(try_tame(MOB_WOLF, ITEM_BONE, 0.32));
    }

    #[test]
    fn wolf_tamed_with_bone_fails_above_threshold() {
        assert!(!try_tame(MOB_WOLF, ITEM_BONE, 0.34));
        assert!(!try_tame(MOB_WOLF, ITEM_BONE, 0.99));
    }

    #[test]
    fn cat_tamed_with_fish_succeeds_below_threshold() {
        assert!(try_tame(MOB_CAT, ITEM_RAW_FISH, 0.0));
        assert!(try_tame(MOB_CAT, ITEM_RAW_FISH, 0.32));
    }

    #[test]
    fn cat_tamed_with_fish_fails_above_threshold() {
        assert!(!try_tame(MOB_CAT, ITEM_RAW_FISH, 0.34));
        assert!(!try_tame(MOB_CAT, ITEM_RAW_FISH, 0.99));
    }

    #[test]
    fn horse_tamed_with_any_item_ten_percent() {
        assert!(try_tame(MOB_HORSE, 0, 0.05));
        assert!(!try_tame(MOB_HORSE, 0, 0.11));
    }

    #[test]
    fn wrong_item_never_tames() {
        // Wolf with fish should fail
        assert!(!try_tame(MOB_WOLF, ITEM_RAW_FISH, 0.0));
        // Cat with bone should fail
        assert!(!try_tame(MOB_CAT, ITEM_BONE, 0.0));
        // Unknown mob should fail
        assert!(!try_tame(255, ITEM_BONE, 0.0));
    }

    #[test]
    fn tameable_component_tame_and_toggle_sit() {
        let mut tc = TameableComponent::new();
        assert!(!tc.tamed);
        assert!(tc.owner.is_none());

        tc.tame(42);
        assert!(tc.tamed);
        assert_eq!(tc.owner, Some(42));
        assert!(!tc.sitting);

        tc.toggle_sit();
        assert!(tc.sitting);

        tc.toggle_sit();
        assert!(!tc.sitting);
    }

    #[test]
    fn toggle_sit_does_nothing_when_not_tamed() {
        let mut tc = TameableComponent::new();
        tc.toggle_sit();
        assert!(!tc.sitting);
    }

    // -- Breeding / feeding -------------------------------------------------

    #[test]
    fn feed_cow_wheat_enters_love_mode() {
        let mut bc = BreedingComponent::new();
        let result = feed_animal(&mut bc, MOB_COW, ITEM_WHEAT);
        assert_eq!(result, FeedResult::EnteredLoveMode);
        assert!(bc.love_mode);
    }

    #[test]
    fn feed_sheep_wheat_enters_love_mode() {
        let mut bc = BreedingComponent::new();
        let result = feed_animal(&mut bc, MOB_SHEEP, ITEM_WHEAT);
        assert_eq!(result, FeedResult::EnteredLoveMode);
    }

    #[test]
    fn feed_pig_carrot_enters_love_mode() {
        let mut bc = BreedingComponent::new();
        let result = feed_animal(&mut bc, MOB_PIG, ITEM_CARROT);
        assert_eq!(result, FeedResult::EnteredLoveMode);
    }

    #[test]
    fn feed_chicken_seeds_enters_love_mode() {
        let mut bc = BreedingComponent::new();
        let result = feed_animal(&mut bc, MOB_CHICKEN, ITEM_SEEDS);
        assert_eq!(result, FeedResult::EnteredLoveMode);
    }

    #[test]
    fn wrong_food_rejected() {
        let mut bc = BreedingComponent::new();
        // Cow does not eat carrots
        assert_eq!(
            feed_animal(&mut bc, MOB_COW, ITEM_CARROT),
            FeedResult::WrongFood
        );
        // Pig does not eat wheat
        assert_eq!(
            feed_animal(&mut bc, MOB_PIG, ITEM_WHEAT),
            FeedResult::WrongFood
        );
        // Chicken does not eat wheat
        assert_eq!(
            feed_animal(&mut bc, MOB_CHICKEN, ITEM_WHEAT),
            FeedResult::WrongFood
        );
    }

    #[test]
    fn already_in_love_rejected() {
        let mut bc = BreedingComponent::new();
        bc.enter_love_mode();
        let result = feed_animal(&mut bc, MOB_COW, ITEM_WHEAT);
        assert_eq!(result, FeedResult::AlreadyInLove);
    }

    #[test]
    fn on_cooldown_rejected() {
        let mut bc = BreedingComponent::new();
        bc.breed_cooldown = 60.0;
        let result = feed_animal(&mut bc, MOB_COW, ITEM_WHEAT);
        assert_eq!(result, FeedResult::OnCooldown);
    }

    // -- Breeding spawn -----------------------------------------------------

    #[test]
    fn try_breed_returns_midpoint_when_both_in_love() {
        let pos1 = Vec3::new(0.0, 64.0, 0.0);
        let pos2 = Vec3::new(10.0, 64.0, 10.0);
        let baby_pos = try_breed(pos1, pos2, true).expect("should breed");
        assert!((baby_pos.x - 5.0).abs() < f32::EPSILON);
        assert!((baby_pos.y - 64.0).abs() < f32::EPSILON);
        assert!((baby_pos.z - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn try_breed_fails_when_not_both_in_love() {
        let pos1 = Vec3::ZERO;
        let pos2 = Vec3::new(10.0, 0.0, 10.0);
        assert!(try_breed(pos1, pos2, false).is_none());
    }

    // -- Baby growth --------------------------------------------------------

    #[test]
    fn baby_starts_not_adult() {
        let baby = BabyMob::new();
        assert!(!baby.is_adult());
        assert!((baby.age_timer).abs() < f32::EPSILON);
    }

    #[test]
    fn baby_becomes_adult_after_1200_seconds() {
        let mut baby = BabyMob::new();
        baby.tick(1200.0);
        assert!(baby.is_adult());
    }

    #[test]
    fn baby_not_adult_before_1200_seconds() {
        let mut baby = BabyMob::new();
        baby.tick(1199.0);
        assert!(!baby.is_adult());
    }

    #[test]
    fn baby_age_timer_clamped_to_growth_time() {
        let mut baby = BabyMob::new();
        baby.tick(2000.0);
        assert!((baby.age_timer - BabyMob::GROWTH_TIME).abs() < f32::EPSILON);
    }

    #[test]
    fn baby_grows_incrementally() {
        let mut baby = BabyMob::new();
        for _ in 0..120 {
            baby.tick(10.0);
        }
        assert!(baby.is_adult());
    }

    // -- Love mode expiry ---------------------------------------------------

    #[test]
    fn love_mode_expires_after_duration() {
        let mut bc = BreedingComponent::new();
        bc.enter_love_mode();
        assert!(bc.love_mode);

        bc.tick(31.0);
        assert!(!bc.love_mode, "love mode should expire after 30 seconds");
    }

    #[test]
    fn breed_cooldown_decreases_over_time() {
        let mut bc = BreedingComponent::new();
        bc.breed_cooldown = 10.0;
        bc.tick(5.0);
        assert!((bc.breed_cooldown - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn breed_cooldown_clamps_to_zero() {
        let mut bc = BreedingComponent::new();
        bc.breed_cooldown = 3.0;
        bc.tick(10.0);
        assert!((bc.breed_cooldown).abs() < f32::EPSILON);
    }
}
