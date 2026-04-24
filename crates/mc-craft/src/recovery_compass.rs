//! Recovery compass item mechanics.
//!
//! The recovery compass points toward the player's last death location.
//! It spins randomly when no death position is recorded or when the player
//! is in a different dimension from where they died. Crafted with 8 echo
//! shards surrounding a compass.

use std::f32::consts::TAU;

use crate::item_ids::ITEM_COMPASS;

/// Number of compass animation frames for a full rotation.
const COMPASS_FRAMES: u8 = 32;

/// Item ID for an echo shard.
const ECHO_SHARD_ID: u16 = 8201;

/// A recovery compass that tracks the player's last death position.
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryCompass {
    /// The coordinates of the player's last death, or `None` if they have not died.
    pub linked_death_pos: Option<(f64, f64, f64)>,
    /// The dimension where the player last died (0 = Overworld, 1 = Nether, 2 = End).
    pub death_dimension: u8,
}

impl RecoveryCompass {
    /// Create a new recovery compass with no linked death position.
    pub fn new() -> Self {
        Self {
            linked_death_pos: None,
            death_dimension: 0,
        }
    }

    /// Record a death position and dimension on this compass.
    pub fn set_death_pos(&mut self, pos: (f64, f64, f64), dim: u8) {
        self.linked_death_pos = Some(pos);
        self.death_dimension = dim;
    }
}

impl Default for RecoveryCompass {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate the angle in radians from the player's facing direction to
/// the recorded death position.
///
/// Uses atan2 to determine the direction from the player to the death
/// location, then subtracts the player's yaw to get the relative angle.
pub fn compass_angle_to_death(
    player_x: f64,
    player_z: f64,
    player_yaw: f32,
    death_x: f64,
    death_z: f64,
) -> f32 {
    let dx = (death_x - player_x) as f32;
    let dz = (death_z - player_z) as f32;
    let world_angle = dz.atan2(dx);
    let relative = world_angle - player_yaw;
    normalize_angle(relative)
}

/// Returns `true` if the recovery compass should spin randomly.
///
/// The compass spins when there is no recorded death position or when the
/// player's current dimension differs from the death dimension.
pub fn compass_spins(compass: &RecoveryCompass, current_dim: u8) -> bool {
    match compass.linked_death_pos {
        None => true,
        Some(_) => compass.death_dimension != current_dim,
    }
}

/// Map a compass angle to an animation frame index (0..31).
///
/// Divides the full rotation into 32 equal segments.
pub fn compass_frame(angle: f32) -> u8 {
    let normalized = ((angle % TAU) + TAU) % TAU;
    let raw = normalized / TAU * f32::from(COMPASS_FRAMES);
    (raw.round() as u8) % COMPASS_FRAMES
}

/// Return the item ID for the recovery compass (8200).
pub fn recovery_compass_item_id() -> u16 {
    8200
}

/// Return the crafting ingredients for a recovery compass.
///
/// The recipe requires 8 echo shards and 1 compass arranged in a
/// 3x3 crafting grid (echo shards in all surrounding slots, compass
/// in the center).
///
/// Each entry is `(item_id, quantity)`.
pub fn recipe_ingredients() -> Vec<(u16, u8)> {
    vec![
        (ECHO_SHARD_ID, 8),
        (ITEM_COMPASS, 1),
    ]
}

/// Normalize an angle to the range `[-PI, PI)`.
fn normalize_angle(angle: f32) -> f32 {
    use std::f32::consts::PI;
    let mut a = angle % TAU;
    if a >= PI {
        a -= TAU;
    } else if a < -PI {
        a += TAU;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    const EPSILON: f32 = 1e-5;

    #[test]
    fn new_compass_has_no_death_pos() {
        let compass = RecoveryCompass::new();
        assert!(compass.linked_death_pos.is_none());
        assert_eq!(compass.death_dimension, 0);
    }

    #[test]
    fn default_matches_new() {
        let a = RecoveryCompass::new();
        let b = RecoveryCompass::default();
        assert_eq!(a, b);
    }

    #[test]
    fn set_death_pos_records_position_and_dimension() {
        let mut compass = RecoveryCompass::new();
        compass.set_death_pos((100.0, 64.0, -200.0), 1);
        assert_eq!(compass.linked_death_pos, Some((100.0, 64.0, -200.0)));
        assert_eq!(compass.death_dimension, 1);
    }

    #[test]
    fn spins_when_no_death_pos() {
        let compass = RecoveryCompass::new();
        assert!(compass_spins(&compass, 0));
        assert!(compass_spins(&compass, 1));
        assert!(compass_spins(&compass, 2));
    }

    #[test]
    fn spins_when_different_dimension() {
        let mut compass = RecoveryCompass::new();
        compass.set_death_pos((0.0, 0.0, 0.0), 0);
        assert!(compass_spins(&compass, 1));
        assert!(compass_spins(&compass, 2));
    }

    #[test]
    fn does_not_spin_in_same_dimension() {
        let mut compass = RecoveryCompass::new();
        compass.set_death_pos((0.0, 0.0, 0.0), 0);
        assert!(!compass_spins(&compass, 0));
    }

    #[test]
    fn angle_straight_ahead() {
        // Player at origin facing east (yaw = 0), death at (10, 0).
        let angle = compass_angle_to_death(0.0, 0.0, 0.0, 10.0, 0.0);
        assert!(angle.abs() < EPSILON, "Expected 0, got {angle}");
    }

    #[test]
    fn angle_behind_player() {
        // Player at origin facing east (yaw = 0), death at (-10, 0).
        // world_angle = atan2(0, -10) = PI
        // relative = PI - 0 = PI => normalized to -PI (since PI >= PI => PI - TAU = -PI)
        let angle = compass_angle_to_death(0.0, 0.0, 0.0, -10.0, 0.0);
        assert!(
            (angle.abs() - PI).abs() < EPSILON,
            "Expected +/-PI, got {angle}"
        );
    }

    #[test]
    fn angle_to_the_right() {
        // Player at origin facing east (yaw = 0), death at (0, 10).
        // world_angle = atan2(10, 0) = PI/2
        let angle = compass_angle_to_death(0.0, 0.0, 0.0, 0.0, 10.0);
        assert!(
            (angle - FRAC_PI_2).abs() < EPSILON,
            "Expected PI/2, got {angle}"
        );
    }

    #[test]
    fn frame_at_zero_angle() {
        assert_eq!(compass_frame(0.0), 0);
    }

    #[test]
    fn frame_at_pi() {
        assert_eq!(compass_frame(PI), 16);
    }

    #[test]
    fn frame_wraps_at_tau() {
        assert_eq!(compass_frame(TAU), 0);
    }

    #[test]
    fn frame_quarter_turn() {
        // PI/2 => frame 8
        assert_eq!(compass_frame(FRAC_PI_2), 8);
    }

    #[test]
    fn item_id_is_8200() {
        assert_eq!(recovery_compass_item_id(), 8200);
    }

    #[test]
    fn recipe_has_correct_ingredients() {
        let ingredients = recipe_ingredients();
        assert_eq!(ingredients.len(), 2);
        // 8 echo shards
        assert_eq!(ingredients[0], (8201, 8));
        // 1 compass
        assert_eq!(ingredients[1], (ITEM_COMPASS, 1));
    }

    #[test]
    fn overwrite_death_pos() {
        let mut compass = RecoveryCompass::new();
        compass.set_death_pos((10.0, 20.0, 30.0), 0);
        compass.set_death_pos((50.0, 60.0, 70.0), 2);
        assert_eq!(compass.linked_death_pos, Some((50.0, 60.0, 70.0)));
        assert_eq!(compass.death_dimension, 2);
    }
}
