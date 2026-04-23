// ---------------------------------------------------------------------------
// Loaded projectile types
// ---------------------------------------------------------------------------

/// The type of projectile loaded into a crossbow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadedProjectile {
    Arrow,
    SpectralArrow,
    /// A tipped arrow carrying the given potion effect id.
    TippedArrow(u8),
    Firework,
}

// ---------------------------------------------------------------------------
// Crossbow state
// ---------------------------------------------------------------------------

/// Tracks the charging and loaded state of a crossbow.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossbowState {
    /// Whether the crossbow is currently being charged.
    pub charging: bool,
    /// Accumulated charge time in seconds.
    pub charge_time: f32,
    /// Projectiles loaded into the crossbow, if any.
    pub loaded: Option<Vec<LoadedProjectile>>,
    /// Quick Charge enchantment level (0 = no enchantment).
    pub quick_charge_level: u8,
}

impl CrossbowState {
    /// Create a new crossbow in its default (uncharged, unloaded) state.
    pub fn new() -> Self {
        Self {
            charging: false,
            charge_time: 0.0,
            loaded: None,
            quick_charge_level: 0,
        }
    }

    /// Begin charging the crossbow with the given projectile.
    ///
    /// Resets any previous charge progress and marks the crossbow as charging.
    pub fn start_charging(&mut self, projectile: LoadedProjectile) {
        self.charging = true;
        self.charge_time = 0.0;
        self.loaded = Some(vec![projectile]);
    }

    /// Advance the charge by `dt` seconds.
    ///
    /// Returns `true` when the crossbow is fully charged.
    pub fn tick_charging(&mut self, dt: f32) -> bool {
        if !self.charging {
            return false;
        }

        self.charge_time += dt;
        let target = charge_duration(self.quick_charge_level);

        if self.charge_time >= target {
            self.charging = false;
            self.charge_time = target;
            return true;
        }

        false
    }

    /// Fire the loaded projectile(s).
    ///
    /// With `has_multishot` the crossbow fires three copies of the loaded
    /// projectile; otherwise it fires one. Returns the projectiles to spawn
    /// and clears the loaded state.
    pub fn fire(&mut self, has_multishot: bool) -> Vec<LoadedProjectile> {
        let projectiles = match self.loaded.take() {
            Some(loaded) if !loaded.is_empty() => {
                let base = loaded[0];
                if has_multishot {
                    vec![base, base, base]
                } else {
                    vec![base]
                }
            }
            _ => Vec::new(),
        };

        self.charge_time = 0.0;
        projectiles
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Calculate the total charge duration in seconds for a given Quick Charge
/// enchantment level.
///
/// Base duration is 1.25 seconds, reduced by 0.25 per level, with a
/// minimum of 0.25 seconds.
pub fn charge_duration(quick_charge_level: u8) -> f32 {
    let reduction = 0.25 * quick_charge_level as f32;
    (1.25 - reduction).max(0.25)
}

/// Return the number of extra targets a piercing projectile can pass
/// through beyond the first.
///
/// The number of additional targets equals the enchantment level.
pub fn piercing_remaining_targets(piercing_level: u8) -> u8 {
    piercing_level
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- charge_duration ------------------------------------------------------

    #[test]
    fn charge_duration_base_is_1_25() {
        let dur = charge_duration(0);
        assert!(
            (dur - 1.25).abs() < f32::EPSILON,
            "base charge duration should be 1.25, got {}",
            dur,
        );
    }

    #[test]
    fn charge_duration_quick_charge_reduces() {
        assert!(
            (charge_duration(1) - 1.0).abs() < f32::EPSILON,
            "quick charge 1 should give 1.0, got {}",
            charge_duration(1),
        );
        assert!(
            (charge_duration(2) - 0.75).abs() < f32::EPSILON,
            "quick charge 2 should give 0.75, got {}",
            charge_duration(2),
        );
        assert!(
            (charge_duration(3) - 0.5).abs() < f32::EPSILON,
            "quick charge 3 should give 0.5, got {}",
            charge_duration(3),
        );
    }

    #[test]
    fn charge_duration_minimum_is_0_25() {
        assert!(
            (charge_duration(5) - 0.25).abs() < f32::EPSILON,
            "charge duration should floor at 0.25, got {}",
            charge_duration(5),
        );
        assert!(
            (charge_duration(10) - 0.25).abs() < f32::EPSILON,
            "high quick charge should still floor at 0.25, got {}",
            charge_duration(10),
        );
    }

    // -- CrossbowState::new ---------------------------------------------------

    #[test]
    fn new_crossbow_is_uncharged_and_unloaded() {
        let cb = CrossbowState::new();
        assert!(!cb.charging);
        assert!((cb.charge_time).abs() < f32::EPSILON);
        assert!(cb.loaded.is_none());
        assert_eq!(cb.quick_charge_level, 0);
    }

    // -- start_charging -------------------------------------------------------

    #[test]
    fn start_charging_sets_state() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::Arrow);

        assert!(cb.charging);
        assert!((cb.charge_time).abs() < f32::EPSILON);
        assert_eq!(cb.loaded, Some(vec![LoadedProjectile::Arrow]));
    }

    // -- tick_charging --------------------------------------------------------

    #[test]
    fn tick_charging_accumulates_time() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::Arrow);

        let done = cb.tick_charging(0.5);
        assert!(!done, "should not be fully charged after 0.5s");
        assert!((cb.charge_time - 0.5).abs() < f32::EPSILON);
        assert!(cb.charging);
    }

    #[test]
    fn tick_charging_completes_at_threshold() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::Arrow);

        let done = cb.tick_charging(1.25);
        assert!(done, "should be fully charged after 1.25s");
        assert!(!cb.charging);
    }

    #[test]
    fn tick_charging_completes_when_exceeding_threshold() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::SpectralArrow);

        let done = cb.tick_charging(2.0);
        assert!(done, "should be fully charged after exceeding threshold");
        assert!(!cb.charging);
    }

    #[test]
    fn tick_charging_with_quick_charge() {
        let mut cb = CrossbowState::new();
        cb.quick_charge_level = 2;
        cb.start_charging(LoadedProjectile::Arrow);

        // Quick charge 2: 1.25 - 0.5 = 0.75s
        let done = cb.tick_charging(0.5);
        assert!(!done, "should not be done after 0.5s with qc2 (need 0.75)");

        let done = cb.tick_charging(0.25);
        assert!(done, "should be done after total 0.75s with qc2");
    }

    #[test]
    fn tick_charging_returns_false_when_not_charging() {
        let mut cb = CrossbowState::new();
        let done = cb.tick_charging(1.0);
        assert!(!done, "should return false when not charging");
    }

    // -- fire -----------------------------------------------------------------

    #[test]
    fn fire_returns_single_arrow_without_multishot() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::Arrow);
        cb.tick_charging(1.25);

        let projectiles = cb.fire(false);
        assert_eq!(projectiles.len(), 1);
        assert_eq!(projectiles[0], LoadedProjectile::Arrow);
    }

    #[test]
    fn fire_returns_three_arrows_with_multishot() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::Arrow);
        cb.tick_charging(1.25);

        let projectiles = cb.fire(true);
        assert_eq!(projectiles.len(), 3);
        assert!(
            projectiles.iter().all(|p| *p == LoadedProjectile::Arrow),
            "all multishot projectiles should be arrows",
        );
    }

    #[test]
    fn fire_empties_loaded_state() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::Firework);
        cb.tick_charging(1.25);

        let _ = cb.fire(false);
        assert!(cb.loaded.is_none(), "loaded should be None after firing");
    }

    #[test]
    fn fire_returns_empty_when_unloaded() {
        let mut cb = CrossbowState::new();
        let projectiles = cb.fire(false);
        assert!(projectiles.is_empty());
    }

    #[test]
    fn fire_multishot_with_tipped_arrow() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::TippedArrow(5));
        cb.tick_charging(1.25);

        let projectiles = cb.fire(true);
        assert_eq!(projectiles.len(), 3);
        assert!(
            projectiles
                .iter()
                .all(|p| *p == LoadedProjectile::TippedArrow(5)),
            "all multishot tipped arrows should carry same effect id",
        );
    }

    #[test]
    fn fire_resets_charge_time() {
        let mut cb = CrossbowState::new();
        cb.start_charging(LoadedProjectile::Arrow);
        cb.tick_charging(1.25);

        let _ = cb.fire(false);
        assert!(
            (cb.charge_time).abs() < f32::EPSILON,
            "charge_time should be reset after firing",
        );
    }

    // -- piercing_remaining_targets -------------------------------------------

    #[test]
    fn piercing_zero_gives_no_extra_targets() {
        assert_eq!(piercing_remaining_targets(0), 0);
    }

    #[test]
    fn piercing_level_equals_extra_targets() {
        assert_eq!(piercing_remaining_targets(1), 1);
        assert_eq!(piercing_remaining_targets(4), 4);
    }

    // -- LoadedProjectile enum ------------------------------------------------

    #[test]
    fn loaded_projectile_variants_are_distinct() {
        let arrow = LoadedProjectile::Arrow;
        let spectral = LoadedProjectile::SpectralArrow;
        let tipped = LoadedProjectile::TippedArrow(0);
        let firework = LoadedProjectile::Firework;

        assert_ne!(arrow, spectral);
        assert_ne!(arrow, tipped);
        assert_ne!(arrow, firework);
        assert_ne!(spectral, tipped);
        assert_ne!(spectral, firework);
        assert_ne!(tipped, firework);
    }

    #[test]
    fn tipped_arrow_distinguishes_by_effect_id() {
        let a = LoadedProjectile::TippedArrow(1);
        let b = LoadedProjectile::TippedArrow(2);
        assert_ne!(a, b);
    }
}
