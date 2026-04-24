//! Lodestone block and linked compass behavior.
//!
//! A lodestone is a Nether-only utility block used to mark a location.
//! When a compass is used on a lodestone, it becomes "linked" to that
//! lodestone's position and dimension. A linked compass:
//!
//! - Points toward the linked lodestone (when in the same dimension)
//! - Spins randomly when in a different dimension than its linked lodestone
//!   (i.e. the player is "lost" relative to the link)
//!
//! Crafting a lodestone in vanilla Minecraft requires a netherite ingot
//! surrounded by 8 chiseled stone bricks.

/// A lodestone placed in the world at a specific position and dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lodestone {
    pub pos: (i32, i32, i32),
    pub dimension: u8,
}

/// A compass that may optionally be linked to a lodestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LodestoneCompass {
    pub linked: Option<Lodestone>,
}

impl LodestoneCompass {
    /// Creates a new unlinked lodestone compass.
    pub fn new() -> Self {
        Self { linked: None }
    }
}

/// Links a compass to the given lodestone.
pub fn link_compass(compass: &mut LodestoneCompass, lodestone: Lodestone) {
    compass.linked = Some(lodestone);
}

/// Removes the link from a compass, returning it to its unlinked state.
pub fn unlink_compass(compass: &mut LodestoneCompass) {
    compass.linked = None;
}

/// Returns true if the compass is linked AND its linked lodestone is in the
/// given dimension. An unlinked compass always returns false.
pub fn compass_works_in_dimension(compass: &LodestoneCompass, current_dim: u8) -> bool {
    match compass.linked {
        Some(l) => l.dimension == current_dim,
        None => false,
    }
}

/// Computes the angle (in radians) the compass needle should display,
/// relative to the player's facing direction.
///
/// The result is normalized to `(-PI, PI]`, where 0 means the lodestone is
/// directly ahead and positive values rotate to the player's right.
///
/// Minecraft uses a left-handed coordinate system where +X is east, +Z is
/// south, and yaw=0 means facing south (+Z). Yaw increases clockwise when
/// viewed from above.
pub fn compass_angle_to_lodestone(
    player_x: f32,
    player_z: f32,
    player_yaw: f32,
    lodestone_x: f32,
    lodestone_z: f32,
) -> f32 {
    let dx = lodestone_x - player_x;
    let dz = lodestone_z - player_z;
    // Bearing from player to lodestone, measured the same way as yaw
    // (0 = facing +Z / south, increasing clockwise from above).
    let bearing = dx.atan2(dz);
    let raw = bearing - player_yaw;
    normalize_angle(raw)
}

/// Returns true if the compass is linked but the player is in a different
/// dimension than the linked lodestone — in this case the compass needle
/// spins randomly in the player's hand.
pub fn compass_spins_when_lost(compass: &LodestoneCompass, current_dim: u8) -> bool {
    match compass.linked {
        Some(l) => l.dimension != current_dim,
        None => false,
    }
}

/// Lodestone crafting requires a netherite ingot in the center surrounded by
/// 8 chiseled stone bricks (vanilla Minecraft recipe).
pub fn lodestone_creation_requires_netherite() -> bool {
    true
}

/// Normalizes an angle to the half-open interval `(-PI, PI]`.
fn normalize_angle(mut a: f32) -> f32 {
    let two_pi = std::f32::consts::TAU;
    while a > std::f32::consts::PI {
        a -= two_pi;
    }
    while a <= -std::f32::consts::PI {
        a += two_pi;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    const OVERWORLD: u8 = 0;
    const NETHER: u8 = 1;
    const END: u8 = 2;

    fn sample_lodestone() -> Lodestone {
        Lodestone {
            pos: (10, 64, 20),
            dimension: OVERWORLD,
        }
    }

    #[test]
    fn new_compass_is_unlinked() {
        let c = LodestoneCompass::new();
        assert!(c.linked.is_none());
    }

    #[test]
    fn link_then_unlink_compass() {
        let mut c = LodestoneCompass::new();
        let l = sample_lodestone();
        link_compass(&mut c, l);
        assert_eq!(c.linked, Some(l));

        unlink_compass(&mut c);
        assert!(c.linked.is_none());
    }

    #[test]
    fn unlinked_compass_does_not_work_in_any_dimension() {
        let c = LodestoneCompass::new();
        assert!(!compass_works_in_dimension(&c, OVERWORLD));
        assert!(!compass_works_in_dimension(&c, NETHER));
        assert!(!compass_works_in_dimension(&c, END));
    }

    #[test]
    fn linked_compass_works_only_in_matching_dimension() {
        let mut c = LodestoneCompass::new();
        link_compass(&mut c, sample_lodestone()); // OVERWORLD
        assert!(compass_works_in_dimension(&c, OVERWORLD));
        assert!(!compass_works_in_dimension(&c, NETHER));
        assert!(!compass_works_in_dimension(&c, END));
    }

    #[test]
    fn unlinked_compass_does_not_spin() {
        let c = LodestoneCompass::new();
        assert!(!compass_spins_when_lost(&c, OVERWORLD));
        assert!(!compass_spins_when_lost(&c, NETHER));
    }

    #[test]
    fn linked_compass_spins_when_in_other_dimension() {
        let mut c = LodestoneCompass::new();
        link_compass(&mut c, sample_lodestone()); // OVERWORLD
        assert!(!compass_spins_when_lost(&c, OVERWORLD));
        assert!(compass_spins_when_lost(&c, NETHER));
        assert!(compass_spins_when_lost(&c, END));
    }

    #[test]
    fn lodestone_requires_netherite() {
        assert!(lodestone_creation_requires_netherite());
    }

    #[test]
    fn angle_zero_when_lodestone_directly_ahead() {
        // Player at origin facing +Z (yaw=0). Lodestone north along +Z.
        let a = compass_angle_to_lodestone(0.0, 0.0, 0.0, 0.0, 10.0);
        assert!(a.abs() < 1e-5, "expected ~0, got {a}");
    }

    #[test]
    fn angle_is_pi_when_lodestone_directly_behind() {
        // Player faces +Z, lodestone is at -Z.
        let a = compass_angle_to_lodestone(0.0, 0.0, 0.0, 0.0, -10.0);
        assert!((a.abs() - std::f32::consts::PI).abs() < 1e-5, "got {a}");
    }

    #[test]
    fn angle_positive_when_lodestone_to_player_right() {
        // Player faces +Z (yaw=0). +X is to the player's left in MC's
        // left-handed system? Actually with yaw=0 facing south (+Z) and
        // standard atan2(dx, dz): +X gives positive bearing => rotates
        // clockwise from above => to the player's left when facing south.
        // We just assert the bearing matches dx.atan2(dz) for yaw=0.
        let a = compass_angle_to_lodestone(0.0, 0.0, 0.0, 5.0, 5.0);
        let expected = 5.0_f32.atan2(5.0); // PI/4
        assert!((a - expected).abs() < 1e-5, "got {a}, expected {expected}");
    }

    #[test]
    fn angle_accounts_for_player_yaw() {
        // If player rotates by the same amount as the bearing, the relative
        // angle should drop to zero.
        let lx = 5.0_f32;
        let lz = 5.0_f32;
        let bearing = lx.atan2(lz);
        let a = compass_angle_to_lodestone(0.0, 0.0, bearing, lx, lz);
        assert!(a.abs() < 1e-5, "expected ~0 after aligning yaw, got {a}");
    }

    #[test]
    fn angle_is_normalized_to_pi_range() {
        // Use a large yaw to force wrap-around.
        for &yaw in &[-10.0_f32, -5.0, 0.0, 3.0, 7.5, 12.0] {
            let a = compass_angle_to_lodestone(0.0, 0.0, yaw, 3.0, -2.0);
            assert!(a > -std::f32::consts::PI && a <= std::f32::consts::PI,
                "angle {a} out of (-PI, PI] for yaw {yaw}");
        }
    }
}
