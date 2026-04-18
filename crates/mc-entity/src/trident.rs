use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base melee damage when used as a hand weapon.
pub const TRIDENT_MELEE_DAMAGE: f32 = 9.0;

/// Base damage when thrown as a projectile.
pub const TRIDENT_THROW_DAMAGE: f32 = 8.0;

// ---------------------------------------------------------------------------
// Trident mode
// ---------------------------------------------------------------------------

/// The active mode of a trident, determined by its enchantments and context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TridentMode {
    /// Standard melee attack.
    Melee,
    /// Thrown as a projectile.
    Thrown,
    /// Thrown with the Loyalty enchantment — returns to the owner.
    Loyalty,
    /// Activated in water/rain with the Riptide enchantment — launches the player.
    Riptide,
    /// Thrown during a thunderstorm with the Channeling enchantment — summons lightning.
    Channeling,
}

// ---------------------------------------------------------------------------
// Throwing
// ---------------------------------------------------------------------------

/// Compute the initial position and velocity for a thrown trident.
///
/// `pos` is the thrower's eye position, `look` is the normalised look direction,
/// and `power` controls the throw strength (vanilla default ~2.5).
///
/// Returns `(spawn_position, velocity)`.
pub fn throw_trident(pos: Vec3, look: Vec3, power: f32) -> (Vec3, Vec3) {
    let direction = look.normalize_or_zero();
    let velocity = direction * power;
    (pos, velocity)
}

// ---------------------------------------------------------------------------
// Loyalty enchantment
// ---------------------------------------------------------------------------

/// Calculate the return velocity for a Loyalty-enchanted trident.
///
/// The trident flies back toward the owner at a speed proportional to the
/// enchantment level: `speed = level * 3`.
///
/// Returns the velocity vector pointing from the trident toward the owner.
pub fn loyalty_return(trident_pos: Vec3, owner_pos: Vec3, level: u8) -> Vec3 {
    let diff = owner_pos - trident_pos;
    let direction = diff.normalize_or_zero();
    let speed = level as f32 * 3.0;
    direction * speed
}

// ---------------------------------------------------------------------------
// Riptide enchantment
// ---------------------------------------------------------------------------

/// Calculate the launch velocity when Riptide is activated.
///
/// The player is launched in the look direction with magnitude
/// `level * 4 + 3`.
pub fn riptide_launch(look: Vec3, level: u8) -> Vec3 {
    let magnitude = level as f32 * 4.0 + 3.0;
    look.normalize_or_zero() * magnitude
}

/// Check whether Riptide can be activated.
///
/// The player must be standing in water or exposed to rain.
pub fn riptide_can_use(in_water: bool, is_raining: bool) -> bool {
    in_water || is_raining
}

// ---------------------------------------------------------------------------
// Channeling enchantment
// ---------------------------------------------------------------------------

/// Determine whether a Channeling strike should summon lightning.
///
/// Lightning is only summoned during a thunderstorm.
pub fn channeling_strike(is_thunderstorm: bool) -> bool {
    is_thunderstorm
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Throw velocity -------------------------------------------------------

    #[test]
    fn throw_trident_returns_correct_velocity() {
        let pos = Vec3::new(0.0, 64.0, 0.0);
        let look = Vec3::new(1.0, 0.0, 0.0);
        let power = 2.5;

        let (spawn_pos, velocity) = throw_trident(pos, look, power);

        assert_eq!(spawn_pos, pos);
        assert!((velocity.x - 2.5).abs() < f32::EPSILON);
        assert!(velocity.y.abs() < f32::EPSILON);
        assert!(velocity.z.abs() < f32::EPSILON);
    }

    #[test]
    fn throw_trident_normalises_look_direction() {
        let pos = Vec3::ZERO;
        let look = Vec3::new(3.0, 0.0, 4.0); // magnitude 5
        let power = 10.0;

        let (_spawn_pos, velocity) = throw_trident(pos, look, power);

        let expected_speed = velocity.length();
        assert!(
            (expected_speed - 10.0).abs() < 1e-5,
            "velocity magnitude should equal power, got {}",
            expected_speed,
        );
    }

    #[test]
    fn throw_trident_zero_look_returns_zero_velocity() {
        let pos = Vec3::new(5.0, 10.0, 5.0);
        let look = Vec3::ZERO;
        let power = 2.5;

        let (_spawn_pos, velocity) = throw_trident(pos, look, power);

        assert_eq!(velocity, Vec3::ZERO);
    }

    // -- Loyalty return -------------------------------------------------------

    #[test]
    fn loyalty_return_points_toward_owner() {
        let trident_pos = Vec3::new(10.0, 64.0, 0.0);
        let owner_pos = Vec3::new(0.0, 64.0, 0.0);
        let level = 1;

        let vel = loyalty_return(trident_pos, owner_pos, level);

        // Should point in the negative-X direction toward the owner.
        assert!(vel.x < 0.0, "expected negative x, got {}", vel.x);
        assert!(vel.y.abs() < f32::EPSILON);
        assert!(vel.z.abs() < f32::EPSILON);
    }

    #[test]
    fn loyalty_return_speed_scales_with_level() {
        let trident_pos = Vec3::new(10.0, 64.0, 0.0);
        let owner_pos = Vec3::new(0.0, 64.0, 0.0);

        let vel1 = loyalty_return(trident_pos, owner_pos, 1);
        let vel2 = loyalty_return(trident_pos, owner_pos, 2);
        let vel3 = loyalty_return(trident_pos, owner_pos, 3);

        assert!(
            (vel1.length() - 3.0).abs() < 1e-5,
            "level 1 speed should be 3.0, got {}",
            vel1.length(),
        );
        assert!(
            (vel2.length() - 6.0).abs() < 1e-5,
            "level 2 speed should be 6.0, got {}",
            vel2.length(),
        );
        assert!(
            (vel3.length() - 9.0).abs() < 1e-5,
            "level 3 speed should be 9.0, got {}",
            vel3.length(),
        );
    }

    #[test]
    fn loyalty_return_zero_distance_returns_zero() {
        let pos = Vec3::new(5.0, 5.0, 5.0);
        let vel = loyalty_return(pos, pos, 3);

        assert_eq!(vel, Vec3::ZERO);
    }

    // -- Riptide conditions ---------------------------------------------------

    #[test]
    fn riptide_can_use_in_water() {
        assert!(riptide_can_use(true, false));
    }

    #[test]
    fn riptide_can_use_in_rain() {
        assert!(riptide_can_use(false, true));
    }

    #[test]
    fn riptide_cannot_use_on_dry_land() {
        assert!(!riptide_can_use(false, false));
    }

    #[test]
    fn riptide_can_use_in_water_and_rain() {
        assert!(riptide_can_use(true, true));
    }

    #[test]
    fn riptide_launch_velocity_scales_with_level() {
        let look = Vec3::new(0.0, 1.0, 0.0);

        let v1 = riptide_launch(look, 1);
        let v2 = riptide_launch(look, 2);
        let v3 = riptide_launch(look, 3);

        assert!(
            (v1.length() - 7.0).abs() < 1e-5,
            "level 1 magnitude should be 7.0, got {}",
            v1.length(),
        );
        assert!(
            (v2.length() - 11.0).abs() < 1e-5,
            "level 2 magnitude should be 11.0, got {}",
            v2.length(),
        );
        assert!(
            (v3.length() - 15.0).abs() < 1e-5,
            "level 3 magnitude should be 15.0, got {}",
            v3.length(),
        );
    }

    // -- Channeling weather ---------------------------------------------------

    #[test]
    fn channeling_strike_during_thunderstorm() {
        assert!(channeling_strike(true));
    }

    #[test]
    fn channeling_no_strike_without_thunderstorm() {
        assert!(!channeling_strike(false));
    }

    // -- Damage constants -----------------------------------------------------

    #[test]
    fn melee_damage_is_nine() {
        assert!((TRIDENT_MELEE_DAMAGE - 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn throw_damage_is_eight() {
        assert!((TRIDENT_THROW_DAMAGE - 8.0).abs() < f32::EPSILON);
    }
}
