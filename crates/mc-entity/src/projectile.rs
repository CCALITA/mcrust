use glam::Vec3;

// ---------------------------------------------------------------------------
// Projectile types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectileType {
    Arrow,
    SpectralArrow,
    TippedArrow,
    Trident,
    Snowball,
    Egg,
    EnderPearl,
    Fireball,
    WitherSkull,
}

// ---------------------------------------------------------------------------
// Projectile
// ---------------------------------------------------------------------------

/// A projectile entity moving through the world.
#[derive(Debug, Clone)]
pub struct Projectile {
    pub proj_type: ProjectileType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub shooter: Option<u64>,
    pub gravity: f32,
    pub damage: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl Projectile {
    /// Create a new projectile with physics properties set per type.
    ///
    /// `shooter` is the entity id of whoever launched this projectile.
    pub fn new(ptype: ProjectileType, pos: Vec3, vel: Vec3, shooter: Option<u64>) -> Self {
        let (gravity, damage, max_lifetime) = match ptype {
            ProjectileType::Arrow | ProjectileType::SpectralArrow | ProjectileType::TippedArrow => {
                (0.05, 2.0, 60.0)
            }
            ProjectileType::Trident => (0.05, 8.0, 60.0),
            ProjectileType::Snowball => (0.03, 0.0, 15.0),
            ProjectileType::Egg => (0.03, 0.0, 15.0),
            ProjectileType::EnderPearl => (0.03, 0.0, 15.0),
            ProjectileType::Fireball => (0.0, 6.0, 30.0),
            ProjectileType::WitherSkull => (0.0, 8.0, 30.0),
        };

        Self {
            proj_type: ptype,
            position: pos,
            velocity: vel,
            shooter,
            gravity,
            damage,
            lifetime: 0.0,
            max_lifetime,
        }
    }
}

// ---------------------------------------------------------------------------
// Projectile event
// ---------------------------------------------------------------------------

/// Outcome of a single projectile tick.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectileEvent {
    /// Projectile is still airborne.
    Flying,
    /// Projectile struck a solid block at these coordinates.
    HitBlock((i32, i32, i32)),
    /// Projectile struck an entity with this id.
    HitEntity(u64),
    /// Projectile exceeded its maximum lifetime and despawned.
    Expired,
}

// ---------------------------------------------------------------------------
// Tick
// ---------------------------------------------------------------------------

/// Advance a projectile by one tick of `dt` seconds.
///
/// `is_solid` is a callback that returns `true` when the block at
/// (x, y, z) is solid. Entity collision is not handled here — callers
/// should check entity overlap separately and pass `HitEntity` events.
pub fn tick_projectile(
    proj: &mut Projectile,
    dt: f32,
    is_solid: &dyn Fn(i32, i32, i32) -> bool,
) -> ProjectileEvent {
    // Lifetime check
    proj.lifetime += dt;
    if proj.lifetime >= proj.max_lifetime {
        return ProjectileEvent::Expired;
    }

    // Apply gravity (downward)
    proj.velocity.y -= proj.gravity * dt;

    // Move
    let new_pos = proj.position + proj.velocity * dt;

    // Block collision: test the block the projectile is entering.
    let bx = new_pos.x.floor() as i32;
    let by = new_pos.y.floor() as i32;
    let bz = new_pos.z.floor() as i32;

    if is_solid(bx, by, bz) {
        // Snap position to contact point (approximate: stop at new_pos).
        proj.position = new_pos;
        proj.velocity = Vec3::ZERO;
        return ProjectileEvent::HitBlock((bx, by, bz));
    }

    proj.position = new_pos;
    ProjectileEvent::Flying
}

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

/// Calculate arrow damage from flight speed.
///
/// Returns `2.0 * speed`, floored at a minimum of `0.5`.
pub fn arrow_damage(speed: f32) -> f32 {
    (2.0 * speed).max(0.5)
}

/// Fixed knockback magnitude inflicted by a snowball hit.
pub fn snowball_knockback() -> f32 {
    0.5
}

/// Fall damage the thrower takes when an ender pearl lands.
pub fn ender_pearl_teleport_damage() -> f32 {
    5.0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Gravity ------------------------------------------------------------

    #[test]
    fn gravity_applies_to_arrow() {
        let mut proj = Projectile::new(
            ProjectileType::Arrow,
            Vec3::new(0.0, 50.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
            Some(1),
        );

        let initial_vy = proj.velocity.y;
        let no_solid = |_: i32, _: i32, _: i32| false;
        tick_projectile(&mut proj, 1.0, &no_solid);

        assert!(
            proj.velocity.y < initial_vy,
            "gravity should decrease Y velocity: got {}",
            proj.velocity.y,
        );
    }

    #[test]
    fn no_gravity_on_fireball() {
        let mut proj = Projectile::new(
            ProjectileType::Fireball,
            Vec3::new(0.0, 50.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
            None,
        );

        let initial_vy = proj.velocity.y;
        let no_solid = |_: i32, _: i32, _: i32| false;
        tick_projectile(&mut proj, 1.0, &no_solid);

        assert!(
            (proj.velocity.y - initial_vy).abs() < f32::EPSILON,
            "fireball should have zero gravity, velocity.y changed from {} to {}",
            initial_vy,
            proj.velocity.y,
        );
    }

    // -- Block collision ----------------------------------------------------

    #[test]
    fn block_collision_stops_projectile() {
        let mut proj = Projectile::new(
            ProjectileType::Arrow,
            Vec3::new(0.0, 50.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Some(1),
        );

        // Every block at x >= 5 is solid.
        let wall_at_5 = |x: i32, _y: i32, _z: i32| x >= 5;
        let event = tick_projectile(&mut proj, 1.0, &wall_at_5);

        assert!(
            matches!(event, ProjectileEvent::HitBlock(_)),
            "expected HitBlock, got {:?}",
            event,
        );
        assert_eq!(proj.velocity, Vec3::ZERO);
    }

    #[test]
    fn no_collision_in_open_air() {
        let mut proj = Projectile::new(
            ProjectileType::Snowball,
            Vec3::new(0.0, 50.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            None,
        );

        let no_solid = |_: i32, _: i32, _: i32| false;
        let event = tick_projectile(&mut proj, 0.05, &no_solid);

        assert_eq!(event, ProjectileEvent::Flying);
    }

    // -- Lifetime expiry ----------------------------------------------------

    #[test]
    fn lifetime_expiry_despawns_projectile() {
        let mut proj = Projectile::new(
            ProjectileType::Egg,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            None,
        );

        // Fast-forward close to expiry.
        proj.lifetime = proj.max_lifetime - 0.01;

        let no_solid = |_: i32, _: i32, _: i32| false;
        let event = tick_projectile(&mut proj, 0.02, &no_solid);

        assert_eq!(event, ProjectileEvent::Expired);
    }

    #[test]
    fn lifetime_does_not_expire_early() {
        let mut proj = Projectile::new(
            ProjectileType::Arrow,
            Vec3::new(0.0, 50.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Some(1),
        );

        let no_solid = |_: i32, _: i32, _: i32| false;
        let event = tick_projectile(&mut proj, 0.05, &no_solid);

        assert_eq!(event, ProjectileEvent::Flying);
    }

    // -- Arrow damage -------------------------------------------------------

    #[test]
    fn arrow_damage_scales_with_speed() {
        assert!((arrow_damage(1.0) - 2.0).abs() < f32::EPSILON);
        assert!((arrow_damage(3.0) - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn arrow_damage_has_minimum() {
        assert!((arrow_damage(0.0) - 0.5).abs() < f32::EPSILON);
        assert!((arrow_damage(0.1) - 0.5).abs() < f32::EPSILON);
    }

    // -- Utility helpers ----------------------------------------------------

    #[test]
    fn snowball_knockback_is_half() {
        assert!((snowball_knockback() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn ender_pearl_deals_five_damage() {
        assert!((ender_pearl_teleport_damage() - 5.0).abs() < f32::EPSILON);
    }

    // -- Projectile type properties -----------------------------------------

    #[test]
    fn arrow_types_share_properties() {
        let arrow = Projectile::new(ProjectileType::Arrow, Vec3::ZERO, Vec3::ZERO, None);
        let spectral = Projectile::new(ProjectileType::SpectralArrow, Vec3::ZERO, Vec3::ZERO, None);
        let tipped = Projectile::new(ProjectileType::TippedArrow, Vec3::ZERO, Vec3::ZERO, None);

        assert!((arrow.gravity - spectral.gravity).abs() < f32::EPSILON);
        assert!((arrow.gravity - tipped.gravity).abs() < f32::EPSILON);
        assert!((arrow.damage - spectral.damage).abs() < f32::EPSILON);
        assert!((arrow.damage - tipped.damage).abs() < f32::EPSILON);
    }

    #[test]
    fn trident_deals_more_damage_than_arrow() {
        let arrow = Projectile::new(ProjectileType::Arrow, Vec3::ZERO, Vec3::ZERO, None);
        let trident = Projectile::new(ProjectileType::Trident, Vec3::ZERO, Vec3::ZERO, None);

        assert!(
            trident.damage > arrow.damage,
            "trident damage {} should exceed arrow damage {}",
            trident.damage,
            arrow.damage,
        );
    }

    #[test]
    fn fireball_and_wither_skull_have_no_gravity() {
        let fireball = Projectile::new(ProjectileType::Fireball, Vec3::ZERO, Vec3::ZERO, None);
        let wither = Projectile::new(ProjectileType::WitherSkull, Vec3::ZERO, Vec3::ZERO, None);

        assert!(
            fireball.gravity.abs() < f32::EPSILON,
            "fireball gravity should be 0, got {}",
            fireball.gravity,
        );
        assert!(
            wither.gravity.abs() < f32::EPSILON,
            "wither skull gravity should be 0, got {}",
            wither.gravity,
        );
    }

    #[test]
    fn snowball_and_egg_deal_no_direct_damage() {
        let snowball = Projectile::new(ProjectileType::Snowball, Vec3::ZERO, Vec3::ZERO, None);
        let egg = Projectile::new(ProjectileType::Egg, Vec3::ZERO, Vec3::ZERO, None);

        assert!(
            snowball.damage.abs() < f32::EPSILON,
            "snowball damage should be 0, got {}",
            snowball.damage,
        );
        assert!(
            egg.damage.abs() < f32::EPSILON,
            "egg damage should be 0, got {}",
            egg.damage,
        );
    }

    #[test]
    fn new_projectile_starts_with_zero_lifetime() {
        let proj = Projectile::new(
            ProjectileType::EnderPearl,
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.0, 5.0, 0.0),
            Some(99),
        );

        assert!(proj.lifetime.abs() < f32::EPSILON);
        assert_eq!(proj.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(proj.velocity, Vec3::new(0.0, 5.0, 0.0));
        assert_eq!(proj.shooter, Some(99));
    }
}
