//! Breeding cooldown and baby mob aging system.
//!
//! Provides [`BreedingState`] for tracking love mode, breeding cooldowns,
//! and baby-to-adult growth using a tick-based age system (negative = baby).

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Cooldown in seconds after breeding before the mob can breed again.
pub const BREED_COOLDOWN: f32 = 300.0;

/// Duration in seconds that love mode lasts.
const LOVE_DURATION: f32 = 30.0;

/// Time in seconds for a baby to grow into an adult (24000 ticks at 20 tps).
const BABY_GROW_SECONDS: f32 = 1200.0;

/// Starting age (in ticks) for a baby mob. Negative means baby.
const BABY_START_AGE: i32 = -24000;

// ---------------------------------------------------------------------------
// BreedingState
// ---------------------------------------------------------------------------

/// Unified breeding and aging state for a breedable mob.
///
/// * `age_ticks` — negative values indicate a baby; zero or positive is adult.
/// * `in_love` / `love_timer` — love-mode state with automatic expiry.
/// * `cooldown` — post-breed cooldown preventing immediate re-breeding.
#[derive(Debug, Clone, PartialEq)]
pub struct BreedingState {
    pub in_love: bool,
    pub love_timer: f32,
    pub cooldown: f32,
    pub age_ticks: i32,
}

impl BreedingState {
    /// Create a new adult breeding state (age = 0, no cooldown).
    pub fn new_adult() -> Self {
        Self {
            in_love: false,
            love_timer: 0.0,
            cooldown: 0.0,
            age_ticks: 0,
        }
    }

    /// Create a new baby breeding state (age = -24000).
    pub fn new_baby() -> Self {
        Self {
            in_love: false,
            love_timer: 0.0,
            cooldown: 0.0,
            age_ticks: BABY_START_AGE,
        }
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Returns `true` when `age_ticks` indicates adulthood (>= 0).
pub fn is_adult(age_ticks: i32) -> bool {
    age_ticks >= 0
}

/// Total growth time in seconds for a baby to become an adult.
pub fn baby_grow_time() -> f32 {
    BABY_GROW_SECONDS
}

/// Feed a breeding food item to a mob.
///
/// Returns `false` if the mob is on cooldown, is already in love, or is a baby.
/// On success sets `in_love = true` and starts the 30-second love timer.
pub fn feed_breeding_food(state: &mut BreedingState) -> bool {
    if state.cooldown > 0.0 || state.in_love || !is_adult(state.age_ticks) {
        return false;
    }
    state.in_love = true;
    state.love_timer = LOVE_DURATION;
    true
}

/// Returns `true` when the mob is eligible to breed (adult, in love, no cooldown).
pub fn can_breed(state: &BreedingState) -> bool {
    is_adult(state.age_ticks) && state.in_love && state.cooldown <= 0.0
}

/// Attempt to breed two mobs that are both in love.
///
/// On success, both parents exit love mode, receive a cooldown, and a new
/// [`BreedingState`] for the baby is returned.
///
/// Returns `None` if either parent cannot breed.
pub fn attempt_breed(
    p1: &mut BreedingState,
    p2: &mut BreedingState,
) -> Option<BreedingState> {
    if !can_breed(p1) || !can_breed(p2) {
        return None;
    }

    // Reset parents
    p1.in_love = false;
    p1.love_timer = 0.0;
    p1.cooldown = BREED_COOLDOWN;

    p2.in_love = false;
    p2.love_timer = 0.0;
    p2.cooldown = BREED_COOLDOWN;

    Some(BreedingState::new_baby())
}

/// Advance breeding timers and baby aging by `dt` seconds.
///
/// * Decrements `love_timer` and exits love mode when it reaches zero.
/// * Decrements `cooldown` toward zero.
/// * Increments `age_ticks` for babies (20 ticks per second), clamping at 0.
pub fn tick_breeding(state: &mut BreedingState, dt: f32) {
    // Age advancement for babies
    if state.age_ticks < 0 {
        let ticks_to_add = (dt * 20.0) as i32;
        state.age_ticks = (state.age_ticks + ticks_to_add).min(0);
    }

    // Love timer
    if state.in_love {
        state.love_timer -= dt;
        if state.love_timer <= 0.0 {
            state.in_love = false;
            state.love_timer = 0.0;
        }
    }

    // Cooldown
    if state.cooldown > 0.0 {
        state.cooldown = (state.cooldown - dt).max(0.0);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction -------------------------------------------------------

    #[test]
    fn new_adult_has_zero_age_and_no_love() {
        let state = BreedingState::new_adult();
        assert_eq!(state.age_ticks, 0);
        assert!(!state.in_love);
        assert!((state.love_timer).abs() < f32::EPSILON);
        assert!((state.cooldown).abs() < f32::EPSILON);
    }

    #[test]
    fn new_baby_has_negative_age() {
        let state = BreedingState::new_baby();
        assert_eq!(state.age_ticks, BABY_START_AGE);
        assert!(!state.in_love);
    }

    // -- is_adult -----------------------------------------------------------

    #[test]
    fn is_adult_returns_true_for_zero() {
        assert!(is_adult(0));
    }

    #[test]
    fn is_adult_returns_true_for_positive() {
        assert!(is_adult(100));
    }

    #[test]
    fn is_adult_returns_false_for_negative() {
        assert!(!is_adult(-1));
        assert!(!is_adult(BABY_START_AGE));
    }

    // -- baby_grow_time -----------------------------------------------------

    #[test]
    fn baby_grow_time_is_1200() {
        assert!((baby_grow_time() - 1200.0).abs() < f32::EPSILON);
    }

    // -- feed_breeding_food -------------------------------------------------

    #[test]
    fn feed_adult_enters_love_mode() {
        let mut state = BreedingState::new_adult();
        assert!(feed_breeding_food(&mut state));
        assert!(state.in_love);
        assert!((state.love_timer - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn feed_fails_when_on_cooldown() {
        let mut state = BreedingState::new_adult();
        state.cooldown = 100.0;
        assert!(!feed_breeding_food(&mut state));
        assert!(!state.in_love);
    }

    #[test]
    fn feed_fails_when_already_in_love() {
        let mut state = BreedingState::new_adult();
        state.in_love = true;
        state.love_timer = 10.0;
        assert!(!feed_breeding_food(&mut state));
    }

    #[test]
    fn feed_fails_for_baby() {
        let mut state = BreedingState::new_baby();
        assert!(!feed_breeding_food(&mut state));
        assert!(!state.in_love);
    }

    // -- can_breed ----------------------------------------------------------

    #[test]
    fn can_breed_when_adult_in_love_no_cooldown() {
        let mut state = BreedingState::new_adult();
        feed_breeding_food(&mut state);
        assert!(can_breed(&state));
    }

    #[test]
    fn cannot_breed_when_not_in_love() {
        let state = BreedingState::new_adult();
        assert!(!can_breed(&state));
    }

    #[test]
    fn cannot_breed_when_baby() {
        let mut state = BreedingState::new_baby();
        state.in_love = true;
        state.love_timer = 10.0;
        assert!(!can_breed(&state));
    }

    #[test]
    fn cannot_breed_when_on_cooldown() {
        let mut state = BreedingState::new_adult();
        state.in_love = true;
        state.love_timer = 10.0;
        state.cooldown = 50.0;
        assert!(!can_breed(&state));
    }

    // -- attempt_breed ------------------------------------------------------

    #[test]
    fn attempt_breed_succeeds_when_both_in_love() {
        let mut p1 = BreedingState::new_adult();
        let mut p2 = BreedingState::new_adult();
        feed_breeding_food(&mut p1);
        feed_breeding_food(&mut p2);

        let baby = attempt_breed(&mut p1, &mut p2);
        assert!(baby.is_some());

        let baby = baby.expect("baby should exist");
        assert_eq!(baby.age_ticks, BABY_START_AGE);
        assert!(!baby.in_love);

        // Parents should be out of love and on cooldown
        assert!(!p1.in_love);
        assert!((p1.cooldown - BREED_COOLDOWN).abs() < f32::EPSILON);
        assert!(!p2.in_love);
        assert!((p2.cooldown - BREED_COOLDOWN).abs() < f32::EPSILON);
    }

    #[test]
    fn attempt_breed_fails_when_one_not_in_love() {
        let mut p1 = BreedingState::new_adult();
        let mut p2 = BreedingState::new_adult();
        feed_breeding_food(&mut p1);
        // p2 not fed

        assert!(attempt_breed(&mut p1, &mut p2).is_none());
        // p1 should remain in love (no side effects on failure)
        assert!(p1.in_love);
    }

    #[test]
    fn attempt_breed_fails_when_neither_in_love() {
        let mut p1 = BreedingState::new_adult();
        let mut p2 = BreedingState::new_adult();
        assert!(attempt_breed(&mut p1, &mut p2).is_none());
    }

    #[test]
    fn attempt_breed_fails_when_baby_is_in_love() {
        let mut p1 = BreedingState::new_adult();
        feed_breeding_food(&mut p1);

        let mut p2 = BreedingState::new_baby();
        p2.in_love = true;
        p2.love_timer = 10.0;

        assert!(attempt_breed(&mut p1, &mut p2).is_none());
    }

    // -- tick_breeding ------------------------------------------------------

    #[test]
    fn tick_advances_baby_age() {
        let mut state = BreedingState::new_baby();
        tick_breeding(&mut state, 1.0); // 1 second = 20 ticks
        assert_eq!(state.age_ticks, BABY_START_AGE + 20);
    }

    #[test]
    fn tick_baby_becomes_adult_after_full_growth() {
        let mut state = BreedingState::new_baby();
        // 24000 ticks / 20 tps = 1200 seconds
        tick_breeding(&mut state, 1200.0);
        assert!(is_adult(state.age_ticks));
        assert_eq!(state.age_ticks, 0);
    }

    #[test]
    fn tick_baby_age_clamps_at_zero() {
        let mut state = BreedingState::new_baby();
        tick_breeding(&mut state, 5000.0); // way more than needed
        assert_eq!(state.age_ticks, 0);
    }

    #[test]
    fn tick_adult_age_does_not_change() {
        let mut state = BreedingState::new_adult();
        tick_breeding(&mut state, 100.0);
        assert_eq!(state.age_ticks, 0);
    }

    #[test]
    fn tick_love_timer_decrements() {
        let mut state = BreedingState::new_adult();
        feed_breeding_food(&mut state);
        tick_breeding(&mut state, 10.0);
        assert!(state.in_love);
        assert!((state.love_timer - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_love_mode_expires_after_30_seconds() {
        let mut state = BreedingState::new_adult();
        feed_breeding_food(&mut state);
        tick_breeding(&mut state, 31.0);
        assert!(!state.in_love);
        assert!((state.love_timer).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_cooldown_decrements() {
        let mut state = BreedingState::new_adult();
        state.cooldown = 100.0;
        tick_breeding(&mut state, 30.0);
        assert!((state.cooldown - 70.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_cooldown_clamps_to_zero() {
        let mut state = BreedingState::new_adult();
        state.cooldown = 10.0;
        tick_breeding(&mut state, 50.0);
        assert!((state.cooldown).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_incremental_baby_growth() {
        let mut state = BreedingState::new_baby();
        // Tick 120 times at 10 seconds each = 1200 seconds total
        for _ in 0..120 {
            tick_breeding(&mut state, 10.0);
        }
        assert!(is_adult(state.age_ticks));
    }

    #[test]
    fn breed_cooldown_constant_is_300() {
        assert!((BREED_COOLDOWN - 300.0).abs() < f32::EPSILON);
    }

    // -- Full lifecycle test ------------------------------------------------

    #[test]
    fn full_breed_lifecycle() {
        // Two adults meet, breed, baby grows up
        let mut p1 = BreedingState::new_adult();
        let mut p2 = BreedingState::new_adult();

        // Feed both
        assert!(feed_breeding_food(&mut p1));
        assert!(feed_breeding_food(&mut p2));
        assert!(can_breed(&p1));
        assert!(can_breed(&p2));

        // Breed
        let mut baby = attempt_breed(&mut p1, &mut p2)
            .expect("breeding should succeed");
        assert!(!is_adult(baby.age_ticks));

        // Baby cannot be fed
        assert!(!feed_breeding_food(&mut baby));

        // Grow the baby to adulthood
        tick_breeding(&mut baby, 1200.0);
        assert!(is_adult(baby.age_ticks));

        // Now the grown baby can be fed
        assert!(feed_breeding_food(&mut baby));
        assert!(can_breed(&baby));

        // Parents still on cooldown
        assert!(!feed_breeding_food(&mut p1));
        tick_breeding(&mut p1, 300.0);
        // Cooldown expired, parent can breed again
        assert!(feed_breeding_food(&mut p1));
    }
}
