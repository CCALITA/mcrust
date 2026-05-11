//! Dragon breath cloud collection mechanics.

/// A lingering dragon breath cloud in the world.
#[derive(Debug, Clone, PartialEq)]
pub struct DragonBreathCloud {
    pub pos: [f32; 3],
    pub radius: f32,
    pub duration: f32,
    pub damage_per_tick: f32,
}

impl DragonBreathCloud {
    /// Creates a new dragon breath cloud at the given position with default values.
    pub fn new(pos: [f32; 3]) -> Self {
        Self {
            pos,
            radius: 3.0,
            duration: 600.0,
            damage_per_tick: 1.0,
        }
    }
}

/// Returns `true` if a player holding a glass bottle at `bottle_pos` is close
/// enough to collect breath from the cloud.
pub fn collect_dragon_breath(cloud: &DragonBreathCloud, bottle_pos: [f32; 3]) -> bool {
    let dx = cloud.pos[0] - bottle_pos[0];
    let dy = cloud.pos[1] - bottle_pos[1];
    let dz = cloud.pos[2] - bottle_pos[2];
    let dist_sq = dx * dx + dy * dy + dz * dz;
    dist_sq <= cloud.radius * cloud.radius
}

/// Item ID for the dragon's breath item.
pub fn dragon_breath_item_id() -> u16 {
    9300
}

/// Rate at which the breath cloud shrinks per tick.
pub fn breath_cloud_shrink_rate() -> f32 {
    0.005
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_default_cloud() {
        let cloud = DragonBreathCloud::new([1.0, 2.0, 3.0]);
        assert_eq!(cloud.pos, [1.0, 2.0, 3.0]);
        assert_eq!(cloud.radius, 3.0);
        assert_eq!(cloud.duration, 600.0);
        assert_eq!(cloud.damage_per_tick, 1.0);
    }

    #[test]
    fn collect_within_radius_returns_true() {
        let cloud = DragonBreathCloud::new([0.0, 0.0, 0.0]);
        assert!(collect_dragon_breath(&cloud, [1.0, 1.0, 0.0]));
    }

    #[test]
    fn collect_outside_radius_returns_false() {
        let cloud = DragonBreathCloud::new([0.0, 0.0, 0.0]);
        assert!(!collect_dragon_breath(&cloud, [10.0, 10.0, 10.0]));
    }

    #[test]
    fn collect_at_exact_boundary() {
        let cloud = DragonBreathCloud::new([0.0, 0.0, 0.0]);
        assert!(collect_dragon_breath(&cloud, [3.0, 0.0, 0.0]));
    }

    #[test]
    fn dragon_breath_item_id_is_9300() {
        assert_eq!(dragon_breath_item_id(), 9300);
    }

    #[test]
    fn shrink_rate_is_correct() {
        assert!((breath_cloud_shrink_rate() - 0.005).abs() < f32::EPSILON);
    }
}
