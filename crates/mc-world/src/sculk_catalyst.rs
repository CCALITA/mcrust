//! Sculk catalyst block mechanics — spreads sculk on nearby mob deaths.

/// A sculk catalyst block that spreads sculk when mobs die nearby.
#[derive(Debug, Clone, PartialEq)]
pub struct SculkCatalyst {
    pub pos: (i32, i32, i32),
    pub active: bool,
    pub bloom_timer: f32,
}

impl SculkCatalyst {
    /// Creates a new inactive sculk catalyst at the given position.
    pub fn new(pos: (i32, i32, i32)) -> Self {
        Self {
            pos,
            active: false,
            bloom_timer: 0.0,
        }
    }
}

/// Returns the detection range of a sculk catalyst (8 blocks).
pub fn catalyst_detection_range() -> u8 {
    8
}

/// Returns the bloom animation duration in seconds.
pub fn bloom_duration() -> f32 {
    2.0
}

/// Calculates the sculk spread radius based on XP dropped.
/// Formula: min(xp / 5 + 1, 8)
pub fn sculk_spread_radius(xp: u32) -> u8 {
    ((xp / 5 + 1) as u8).min(8)
}

/// Handles a mob death near a sculk catalyst.
/// Returns positions where sculk should spread.
pub fn on_mob_death(catalyst: &mut SculkCatalyst, death_pos: (i32, i32, i32), xp: u32) -> Vec<(i32, i32, i32)> {
    let dx = (catalyst.pos.0 - death_pos.0).unsigned_abs();
    let dy = (catalyst.pos.1 - death_pos.1).unsigned_abs();
    let dz = (catalyst.pos.2 - death_pos.2).unsigned_abs();
    let distance = dx.max(dy).max(dz);

    if distance > catalyst_detection_range() as u32 {
        return Vec::new();
    }

    catalyst.active = true;
    catalyst.bloom_timer = bloom_duration();

    let radius = sculk_spread_radius(xp) as i32;
    let mut positions = Vec::new();

    for x in -radius..=radius {
        for z in -radius..=radius {
            if x.unsigned_abs() + z.unsigned_abs() <= radius as u32 {
                positions.push((
                    death_pos.0 + x,
                    death_pos.1,
                    death_pos.2 + z,
                ));
            }
        }
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_catalyst_is_inactive() {
        let catalyst = SculkCatalyst::new((0, 0, 0));
        assert!(!catalyst.active);
        assert_eq!(catalyst.bloom_timer, 0.0);
    }

    #[test]
    fn detection_range_is_8() {
        assert_eq!(catalyst_detection_range(), 8);
    }

    #[test]
    fn bloom_duration_is_2() {
        assert_eq!(bloom_duration(), 2.0);
    }

    #[test]
    fn spread_radius_formula() {
        assert_eq!(sculk_spread_radius(0), 1);
        assert_eq!(sculk_spread_radius(5), 2);
        assert_eq!(sculk_spread_radius(10), 3);
        assert_eq!(sculk_spread_radius(35), 8);
        assert_eq!(sculk_spread_radius(100), 8); // capped at 8
    }

    #[test]
    fn mob_death_within_range_activates_catalyst() {
        let mut catalyst = SculkCatalyst::new((0, 0, 0));
        let positions = on_mob_death(&mut catalyst, (3, 0, 3), 10);
        assert!(catalyst.active);
        assert_eq!(catalyst.bloom_timer, 2.0);
        assert!(!positions.is_empty());
    }

    #[test]
    fn mob_death_outside_range_does_nothing() {
        let mut catalyst = SculkCatalyst::new((0, 0, 0));
        let positions = on_mob_death(&mut catalyst, (20, 0, 20), 10);
        assert!(!catalyst.active);
        assert!(positions.is_empty());
    }

    #[test]
    fn spread_positions_centered_on_death() {
        let mut catalyst = SculkCatalyst::new((0, 0, 0));
        let positions = on_mob_death(&mut catalyst, (2, 0, 2), 0);
        // radius=1, so spread around death_pos
        assert!(positions.contains(&(2, 0, 2)));
    }
}
