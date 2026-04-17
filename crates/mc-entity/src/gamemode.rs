use glam::Vec3;

// ---------------------------------------------------------------------------
// Game mode enum
// ---------------------------------------------------------------------------

/// The four Minecraft game modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Spectator,
    Adventure,
}

// ---------------------------------------------------------------------------
// Game mode properties
// ---------------------------------------------------------------------------

/// Query-only trait that maps each [`GameMode`] to its property flags.
///
/// Every property is a pure function of the mode -- no mutable state.
pub trait GameModeProperties {
    fn can_fly(&self) -> bool;
    fn instant_break(&self) -> bool;
    fn invincible(&self) -> bool;
    fn has_hunger(&self) -> bool;
    fn can_interact(&self) -> bool;
    fn no_clip(&self) -> bool;
    fn infinite_items(&self) -> bool;
    fn can_take_damage(&self) -> bool;
    fn show_health(&self) -> bool;
    fn natural_regen(&self) -> bool;
}

impl GameModeProperties for GameMode {
    fn can_fly(&self) -> bool {
        matches!(self, GameMode::Creative | GameMode::Spectator)
    }

    fn instant_break(&self) -> bool {
        matches!(self, GameMode::Creative)
    }

    fn invincible(&self) -> bool {
        matches!(self, GameMode::Creative | GameMode::Spectator)
    }

    fn has_hunger(&self) -> bool {
        matches!(self, GameMode::Survival | GameMode::Adventure)
    }

    fn can_interact(&self) -> bool {
        matches!(
            self,
            GameMode::Survival | GameMode::Creative | GameMode::Adventure
        )
    }

    fn no_clip(&self) -> bool {
        matches!(self, GameMode::Spectator)
    }

    fn infinite_items(&self) -> bool {
        matches!(self, GameMode::Creative)
    }

    fn can_take_damage(&self) -> bool {
        matches!(self, GameMode::Survival | GameMode::Adventure)
    }

    fn show_health(&self) -> bool {
        matches!(self, GameMode::Survival | GameMode::Adventure)
    }

    fn natural_regen(&self) -> bool {
        matches!(self, GameMode::Survival)
    }
}

// ---------------------------------------------------------------------------
// Fly state
// ---------------------------------------------------------------------------

/// Tracks whether the player is currently flying and at what speed.
///
/// Creative flying: vertical input (Space = up, Shift = down), horizontal
/// WASD, no gravity.
#[derive(Debug, Clone)]
pub struct FlyState {
    pub flying: bool,
    pub fly_speed: f32,
}

/// Default fly speed (blocks per second) when not sprinting.
pub const DEFAULT_FLY_SPEED: f32 = 0.5;

/// Fly speed (blocks per second) when sprinting.
pub const SPRINT_FLY_SPEED: f32 = 1.0;

impl Default for FlyState {
    fn default() -> Self {
        Self {
            flying: false,
            fly_speed: DEFAULT_FLY_SPEED,
        }
    }
}

impl FlyState {
    /// Toggle flying on/off (double-tap space in Creative).
    pub fn toggle_fly(&mut self) {
        self.flying = !self.flying;
    }

    /// Compute displacement for one frame of creative flight.
    ///
    /// * `wish_dir` -- horizontal wish direction (WASD) in world space,
    ///   expected to be unit-length or zero.
    /// * `up` -- upward input intensity (0.0 or 1.0 for Space).
    /// * `down` -- downward input intensity (0.0 or 1.0 for Shift).
    /// * `dt` -- frame delta time in seconds.
    ///
    /// Returns the displacement vector for this frame. No gravity is applied.
    pub fn apply_movement(&self, wish_dir: Vec3, up: f32, down: f32, dt: f32) -> Vec3 {
        if !self.flying {
            return Vec3::ZERO;
        }

        let vertical = up - down;
        let movement = Vec3::new(wish_dir.x, vertical, wish_dir.z);

        // Scale by fly speed and delta time. Clamp magnitude so diagonal
        // movement is not faster than axis-aligned movement.
        let clamped = if movement.length_squared() > 1.0 {
            movement.normalize()
        } else {
            movement
        };

        clamped * self.fly_speed * dt
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Game mode property flags -------------------------------------------

    #[test]
    fn survival_properties() {
        let mode = GameMode::Survival;
        assert!(!mode.can_fly());
        assert!(!mode.instant_break());
        assert!(!mode.invincible());
        assert!(mode.has_hunger());
        assert!(mode.can_interact());
        assert!(!mode.no_clip());
        assert!(!mode.infinite_items());
        assert!(mode.can_take_damage());
        assert!(mode.show_health());
        assert!(mode.natural_regen());
    }

    #[test]
    fn creative_properties() {
        let mode = GameMode::Creative;
        assert!(mode.can_fly());
        assert!(mode.instant_break());
        assert!(mode.invincible());
        assert!(!mode.has_hunger());
        assert!(mode.can_interact());
        assert!(!mode.no_clip());
        assert!(mode.infinite_items());
        assert!(!mode.can_take_damage());
        assert!(!mode.show_health());
        assert!(!mode.natural_regen());
    }

    #[test]
    fn spectator_properties() {
        let mode = GameMode::Spectator;
        assert!(mode.can_fly());
        assert!(!mode.instant_break());
        assert!(mode.invincible());
        assert!(!mode.has_hunger());
        assert!(!mode.can_interact());
        assert!(mode.no_clip());
        assert!(!mode.infinite_items());
        assert!(!mode.can_take_damage());
        assert!(!mode.show_health());
        assert!(!mode.natural_regen());
    }

    #[test]
    fn adventure_properties() {
        let mode = GameMode::Adventure;
        assert!(!mode.can_fly());
        assert!(!mode.instant_break());
        assert!(!mode.invincible());
        assert!(mode.has_hunger());
        assert!(mode.can_interact());
        assert!(!mode.no_clip());
        assert!(!mode.infinite_items());
        assert!(mode.can_take_damage());
        assert!(mode.show_health());
        assert!(!mode.natural_regen());
    }

    // -- FlyState defaults --------------------------------------------------

    #[test]
    fn fly_state_defaults() {
        let state = FlyState::default();
        assert!(!state.flying);
        assert!((state.fly_speed - DEFAULT_FLY_SPEED).abs() < f32::EPSILON);
    }

    // -- Toggle fly ---------------------------------------------------------

    #[test]
    fn toggle_fly_enables_then_disables() {
        let mut state = FlyState::default();
        assert!(!state.flying);

        state.toggle_fly();
        assert!(state.flying);

        state.toggle_fly();
        assert!(!state.flying);
    }

    // -- Fly movement -------------------------------------------------------

    #[test]
    fn fly_movement_zero_when_not_flying() {
        let state = FlyState::default(); // flying = false
        let displacement = state.apply_movement(Vec3::X, 0.0, 0.0, 1.0);
        assert!(
            displacement.length() < f32::EPSILON,
            "should produce zero displacement when not flying"
        );
    }

    #[test]
    fn fly_movement_horizontal() {
        let state = FlyState {
            flying: true,
            fly_speed: 1.0,
        };
        let displacement = state.apply_movement(Vec3::X, 0.0, 0.0, 1.0);

        assert!(
            (displacement.x - 1.0).abs() < f32::EPSILON,
            "X displacement should be 1.0, got {}",
            displacement.x
        );
        assert!(
            displacement.y.abs() < f32::EPSILON,
            "Y should be zero for horizontal movement"
        );
        assert!(
            displacement.z.abs() < f32::EPSILON,
            "Z should be zero for pure X movement"
        );
    }

    #[test]
    fn fly_movement_vertical_up() {
        let state = FlyState {
            flying: true,
            fly_speed: 1.0,
        };
        let displacement = state.apply_movement(Vec3::ZERO, 1.0, 0.0, 1.0);

        assert!(
            displacement.x.abs() < f32::EPSILON,
            "X should be zero for vertical movement"
        );
        assert!(
            (displacement.y - 1.0).abs() < f32::EPSILON,
            "Y displacement should be 1.0 (up), got {}",
            displacement.y
        );
    }

    #[test]
    fn fly_movement_vertical_down() {
        let state = FlyState {
            flying: true,
            fly_speed: 1.0,
        };
        let displacement = state.apply_movement(Vec3::ZERO, 0.0, 1.0, 1.0);

        assert!(
            (displacement.y - (-1.0)).abs() < f32::EPSILON,
            "Y displacement should be -1.0 (down), got {}",
            displacement.y
        );
    }

    #[test]
    fn fly_movement_diagonal_clamped() {
        let state = FlyState {
            flying: true,
            fly_speed: 1.0,
        };
        // Horizontal + vertical simultaneously should be clamped to unit length.
        let displacement = state.apply_movement(Vec3::X, 1.0, 0.0, 1.0);

        assert!(
            displacement.length() <= 1.0 + f32::EPSILON,
            "diagonal movement should be clamped, got length {}",
            displacement.length()
        );
    }

    #[test]
    fn fly_movement_scales_with_dt() {
        let state = FlyState {
            flying: true,
            fly_speed: 1.0,
        };
        let half = state.apply_movement(Vec3::X, 0.0, 0.0, 0.5);
        let full = state.apply_movement(Vec3::X, 0.0, 0.0, 1.0);

        assert!(
            (full.x - 2.0 * half.x).abs() < f32::EPSILON,
            "displacement should scale linearly with dt"
        );
    }

    #[test]
    fn fly_movement_scales_with_speed() {
        let slow = FlyState {
            flying: true,
            fly_speed: DEFAULT_FLY_SPEED,
        };
        let fast = FlyState {
            flying: true,
            fly_speed: SPRINT_FLY_SPEED,
        };

        let d_slow = slow.apply_movement(Vec3::X, 0.0, 0.0, 1.0);
        let d_fast = fast.apply_movement(Vec3::X, 0.0, 0.0, 1.0);

        assert!(
            d_fast.x > d_slow.x,
            "sprint speed should produce larger displacement"
        );
        assert!(
            (d_fast.x / d_slow.x - SPRINT_FLY_SPEED / DEFAULT_FLY_SPEED).abs() < f32::EPSILON,
            "speed ratio should match"
        );
    }

    // -- Mode transitions ---------------------------------------------------

    #[test]
    fn mode_transitions_change_properties() {
        let mut mode = GameMode::Survival;
        assert!(mode.has_hunger());
        assert!(mode.can_take_damage());

        mode = GameMode::Creative;
        assert!(!mode.has_hunger());
        assert!(mode.invincible());
        assert!(mode.can_fly());

        mode = GameMode::Spectator;
        assert!(mode.no_clip());
        assert!(!mode.can_interact());

        mode = GameMode::Adventure;
        assert!(mode.has_hunger());
        assert!(mode.can_interact());
        assert!(!mode.can_fly());
    }

    #[test]
    fn all_modes_are_distinct() {
        let modes = [
            GameMode::Survival,
            GameMode::Creative,
            GameMode::Spectator,
            GameMode::Adventure,
        ];
        for (i, a) in modes.iter().enumerate() {
            for (j, b) in modes.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "modes at index {i} and {j} should differ");
                }
            }
        }
    }
}
