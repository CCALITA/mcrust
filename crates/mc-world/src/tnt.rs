use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default fuse duration in ticks (4 seconds at 20 tps).
const DEFAULT_FUSE_TICKS: u32 = 80;

/// Gravity applied per tick (blocks/tick^2), matching vanilla TNT.
const GRAVITY_PER_TICK: f32 = 0.04;

/// Small upward velocity given to newly activated TNT.
const INITIAL_UPWARD_VELOCITY: f32 = 0.2;

// ---------------------------------------------------------------------------
// TNT Entity
// ---------------------------------------------------------------------------

/// A primed TNT entity in the world.
#[derive(Debug, Clone, PartialEq)]
pub struct TntEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub fuse_ticks: u32,
}

// ---------------------------------------------------------------------------
// Activation
// ---------------------------------------------------------------------------

/// Create a newly primed TNT entity at `pos` with an 80-tick fuse and a small
/// upward velocity so it hops when ignited.
pub fn activate_tnt(pos: Vec3) -> TntEntity {
    TntEntity {
        position: pos,
        velocity: Vec3::new(0.0, INITIAL_UPWARD_VELOCITY, 0.0),
        fuse_ticks: DEFAULT_FUSE_TICKS,
    }
}

// ---------------------------------------------------------------------------
// Tick action
// ---------------------------------------------------------------------------

/// The result of ticking a TNT entity for one step.
#[derive(Debug, Clone, PartialEq)]
pub enum TntAction {
    /// The TNT is still counting down.
    Ticking,
    /// The fuse has expired; the TNT should explode at this position.
    Explode(Vec3),
}

/// Advance a TNT entity by `dt_ticks` ticks.
///
/// Each tick applies gravity to the velocity, integrates position, and
/// decrements the fuse. Returns [`TntAction::Explode`] once the fuse reaches
/// zero, carrying the final position of the entity.
///
/// Returns a *new* `TntEntity` (via the mutable reference) and the resulting
/// action. The caller keeps owning the entity so it can continue to render the
/// falling TNT until the explosion frame.
pub fn tick_tnt(tnt: &mut TntEntity, dt_ticks: u32) -> TntAction {
    for _ in 0..dt_ticks {
        if tnt.fuse_ticks == 0 {
            return TntAction::Explode(tnt.position);
        }

        // Apply gravity.
        tnt.velocity.y -= GRAVITY_PER_TICK;

        // Integrate position.
        tnt.position += tnt.velocity;

        // Decrement fuse.
        tnt.fuse_ticks -= 1;

        // Check again after decrement so fuse == 0 triggers immediately.
        if tnt.fuse_ticks == 0 {
            return TntAction::Explode(tnt.position);
        }
    }

    TntAction::Ticking
}

// ---------------------------------------------------------------------------
// Chain activation
// ---------------------------------------------------------------------------

/// Given an explosion center and a list of TNT block positions, return those
/// positions that fall within the activation `radius`.
///
/// This models how a TNT explosion can ignite nearby TNT blocks, creating a
/// chain reaction.
pub fn chain_activation(
    explosion_center: Vec3,
    tnt_positions: &[(i32, i32, i32)],
    radius: f32,
) -> Vec<(i32, i32, i32)> {
    let radius_sq = radius * radius;

    tnt_positions
        .iter()
        .filter(|&&(x, y, z)| {
            // Use center of the block (+0.5) for distance check.
            let block_center = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
            explosion_center.distance_squared(block_center) <= radius_sq
        })
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activate_tnt_sets_80_tick_fuse() {
        let tnt = activate_tnt(Vec3::ZERO);
        assert_eq!(tnt.fuse_ticks, 80);
    }

    #[test]
    fn activate_tnt_has_upward_velocity() {
        let tnt = activate_tnt(Vec3::ZERO);
        assert!(tnt.velocity.y > 0.0, "initial velocity should point upward");
    }

    #[test]
    fn fuse_counts_down_each_tick() {
        let mut tnt = activate_tnt(Vec3::ZERO);
        let initial_fuse = tnt.fuse_ticks;

        let action = tick_tnt(&mut tnt, 1);
        assert_eq!(action, TntAction::Ticking);
        assert_eq!(tnt.fuse_ticks, initial_fuse - 1);
    }

    #[test]
    fn fuse_counts_down_multiple_ticks() {
        let mut tnt = activate_tnt(Vec3::ZERO);
        let action = tick_tnt(&mut tnt, 10);
        assert_eq!(action, TntAction::Ticking);
        assert_eq!(tnt.fuse_ticks, 70);
    }

    #[test]
    fn explosion_at_fuse_zero() {
        let mut tnt = activate_tnt(Vec3::ZERO);
        // Tick all 80 ticks; the last tick should trigger the explosion.
        let action = tick_tnt(&mut tnt, 80);
        assert!(
            matches!(action, TntAction::Explode(_)),
            "expected Explode variant, got {action:?}"
        );
    }

    #[test]
    fn explosion_position_reflects_gravity() {
        let mut tnt = activate_tnt(Vec3::new(0.0, 64.0, 0.0));
        let action = tick_tnt(&mut tnt, 80);
        if let TntAction::Explode(pos) = action {
            // TNT should have fallen below its starting Y due to gravity.
            assert!(
                pos.y < 64.0,
                "TNT should fall due to gravity, final y = {}",
                pos.y
            );
        } else {
            panic!("expected Explode");
        }
    }

    #[test]
    fn chain_activation_within_radius() {
        let center = Vec3::new(0.5, 0.5, 0.5);
        let positions = vec![(1, 0, 0), (2, 0, 0), (10, 0, 0)];
        let activated = chain_activation(center, &positions, 3.0);

        assert!(
            activated.contains(&(1, 0, 0)),
            "block at (1,0,0) should be within radius"
        );
        assert!(
            activated.contains(&(2, 0, 0)),
            "block at (2,0,0) should be within radius"
        );
    }

    #[test]
    fn chain_activation_outside_radius_misses() {
        let center = Vec3::new(0.5, 0.5, 0.5);
        let positions = vec![(10, 10, 10)];
        let activated = chain_activation(center, &positions, 3.0);

        assert!(
            activated.is_empty(),
            "block at (10,10,10) should not be activated with radius 3"
        );
    }

    #[test]
    fn chain_activation_empty_list() {
        let activated = chain_activation(Vec3::ZERO, &[], 5.0);
        assert!(activated.is_empty());
    }

    #[test]
    fn already_expired_tnt_explodes_immediately() {
        let mut tnt = TntEntity {
            position: Vec3::new(5.0, 10.0, 5.0),
            velocity: Vec3::ZERO,
            fuse_ticks: 0,
        };
        let action = tick_tnt(&mut tnt, 1);
        assert!(
            matches!(action, TntAction::Explode(pos) if (pos - Vec3::new(5.0, 10.0, 5.0)).length() < f32::EPSILON),
            "already-expired TNT should explode at its current position"
        );
    }
}
