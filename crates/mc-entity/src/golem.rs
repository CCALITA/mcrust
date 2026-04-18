use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Placeholder block ID for iron blocks.
const IRON_BLOCK_ID: u16 = 100;

/// Placeholder block ID for pumpkin (carved).
const PUMPKIN_BLOCK_ID: u16 = 62;

/// Placeholder block ID for snow blocks.
const SNOW_BLOCK_ID: u16 = 80;

/// Default health for a newly spawned Iron Golem.
const IRON_GOLEM_HEALTH: f32 = 100.0;

/// Default health for a newly spawned Snow Golem.
const SNOW_GOLEM_HEALTH: f32 = 4.0;

/// Biome temperature at or above which a Snow Golem takes melting damage.
const SNOW_GOLEM_MELT_THRESHOLD: f32 = 1.0;

/// Range within which an Iron Golem will prioritize attacking a hostile.
const IRON_GOLEM_AGGRO_RANGE: f32 = 16.0;

/// Range within which an Iron Golem may offer a flower to a villager.
const IRON_GOLEM_FLOWER_RANGE: f32 = 6.0;

// ---------------------------------------------------------------------------
// Iron Golem
// ---------------------------------------------------------------------------

/// An Iron Golem entity.
#[derive(Debug, Clone, PartialEq)]
pub struct IronGolem {
    pub health: f32,
    pub position: Vec3,
    pub village_center: Option<Vec3>,
}

impl IronGolem {
    /// Create a new Iron Golem at `pos` with 100 HP.
    pub fn new(pos: Vec3) -> Self {
        Self {
            health: IRON_GOLEM_HEALTH,
            position: pos,
            village_center: None,
        }
    }
}

/// Action chosen by the Iron Golem AI each tick.
#[derive(Debug, Clone, PartialEq)]
pub enum GolemAction {
    /// Do nothing.
    Idle,
    /// Attack the nearest hostile mob.
    AttackHostile(Vec3),
    /// Patrol toward a village center.
    PatrolVillage(Vec3),
    /// Offer a flower to a nearby villager.
    OfferFlower,
}

// ---------------------------------------------------------------------------
// Snow Golem
// ---------------------------------------------------------------------------

/// A Snow Golem entity.
#[derive(Debug, Clone, PartialEq)]
pub struct SnowGolem {
    pub health: f32,
    pub position: Vec3,
}

impl SnowGolem {
    /// Create a new Snow Golem at `pos` with 4 HP.
    pub fn new(pos: Vec3) -> Self {
        Self {
            health: SNOW_GOLEM_HEALTH,
            position: pos,
        }
    }
}

/// Effect produced by a Snow Golem each tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GolemEffect {
    /// Place a snow layer at the golem's position.
    PlaceSnow,
    /// The golem is melting due to high biome temperature.
    Melting,
    /// Nothing special this tick.
    Normal,
}

// ---------------------------------------------------------------------------
// Pattern detection — Iron Golem
// ---------------------------------------------------------------------------

/// Check whether the world contains a valid Iron Golem T-shape pattern.
///
/// The T-shape is:
/// ```text
///   [iron] [iron] [iron]   (y+1, along either X or Z axis)
///          [iron]           (y+0, center)
///          [pumpkin]        (y+2, on top of center arm)
/// ```
///
/// `get_block` returns the block ID at the given (x, y, z) coordinates.
///
/// Returns the five block positions to clear if a valid pattern is found,
/// or `None` if no pattern exists.
pub fn check_iron_golem_pattern(
    get_block: &dyn Fn(i32, i32, i32) -> u16,
) -> Option<Vec<(i32, i32, i32)>> {
    // Scan a reasonable range. In practice this would be called around a
    // recently-placed block; here we check a fixed volume for simplicity.
    for y in -64..=128 {
        for x in -64..=64 {
            for z in -64..=64 {
                // Look for the body center (bottom-center iron block).
                if get_block(x, y, z) != IRON_BLOCK_ID {
                    continue;
                }

                // Try the T-shape along the X axis:
                //   arm: (x-1,y+1,z), (x,y+1,z), (x+1,y+1,z)
                //   body: (x,y,z)
                //   head: (x,y+2,z)
                if get_block(x - 1, y + 1, z) == IRON_BLOCK_ID
                    && get_block(x, y + 1, z) == IRON_BLOCK_ID
                    && get_block(x + 1, y + 1, z) == IRON_BLOCK_ID
                    && get_block(x, y + 2, z) == PUMPKIN_BLOCK_ID
                {
                    return Some(vec![
                        (x, y, z),
                        (x - 1, y + 1, z),
                        (x, y + 1, z),
                        (x + 1, y + 1, z),
                        (x, y + 2, z),
                    ]);
                }

                // Try the T-shape along the Z axis:
                //   arm: (x,y+1,z-1), (x,y+1,z), (x,y+1,z+1)
                //   body: (x,y,z)
                //   head: (x,y+2,z)
                if get_block(x, y + 1, z - 1) == IRON_BLOCK_ID
                    && get_block(x, y + 1, z) == IRON_BLOCK_ID
                    && get_block(x, y + 1, z + 1) == IRON_BLOCK_ID
                    && get_block(x, y + 2, z) == PUMPKIN_BLOCK_ID
                {
                    return Some(vec![
                        (x, y, z),
                        (x, y + 1, z - 1),
                        (x, y + 1, z),
                        (x, y + 1, z + 1),
                        (x, y + 2, z),
                    ]);
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Pattern detection — Snow Golem
// ---------------------------------------------------------------------------

/// Check whether the world contains a valid Snow Golem pattern.
///
/// The pattern is a vertical stack:
/// ```text
///   [pumpkin]   (y+2)
///   [snow]      (y+1)
///   [snow]      (y+0)
/// ```
///
/// Returns the three block positions to clear if a valid pattern is found,
/// or `None` if no pattern exists.
pub fn check_snow_golem_pattern(
    get_block: &dyn Fn(i32, i32, i32) -> u16,
) -> Option<Vec<(i32, i32, i32)>> {
    for y in -64..=128 {
        for x in -64..=64 {
            for z in -64..=64 {
                if get_block(x, y, z) == SNOW_BLOCK_ID
                    && get_block(x, y + 1, z) == SNOW_BLOCK_ID
                    && get_block(x, y + 2, z) == PUMPKIN_BLOCK_ID
                {
                    return Some(vec![(x, y, z), (x, y + 1, z), (x, y + 2, z)]);
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Tick — Iron Golem
// ---------------------------------------------------------------------------

/// Choose the Iron Golem's action for this tick.
///
/// Priority order:
/// 1. Attack the nearest hostile within 16 blocks.
/// 2. Patrol toward the village center (if known).
/// 3. Offer a flower if a villager is within 6 blocks.
/// 4. Idle.
pub fn iron_golem_tick(golem: &IronGolem, hostiles: &[Vec3], villagers: &[Vec3]) -> GolemAction {
    // Priority 1 — attack nearest hostile within range.
    if let Some(target) = closest_within(hostiles, golem.position, IRON_GOLEM_AGGRO_RANGE) {
        return GolemAction::AttackHostile(target);
    }

    // Priority 2 — patrol village.
    if let Some(center) = golem.village_center {
        return GolemAction::PatrolVillage(center);
    }

    // Priority 3 — offer flower to nearby villager.
    if closest_within(villagers, golem.position, IRON_GOLEM_FLOWER_RANGE).is_some() {
        return GolemAction::OfferFlower;
    }

    // Priority 4 — nothing to do.
    GolemAction::Idle
}

// ---------------------------------------------------------------------------
// Tick — Snow Golem
// ---------------------------------------------------------------------------

/// Determine the Snow Golem's effect for this tick based on biome temperature.
///
/// - Temperature >= 1.0 -> Melting
/// - Temperature < 1.0  -> PlaceSnow
/// - (Normal is not produced under the current rules but exists for extension.)
pub fn snow_golem_tick(golem: &SnowGolem, biome_temp: f32) -> GolemEffect {
    let _ = golem; // position may be used in the future
    if biome_temp >= SNOW_GOLEM_MELT_THRESHOLD {
        GolemEffect::Melting
    } else {
        GolemEffect::PlaceSnow
    }
}

// ---------------------------------------------------------------------------
// Helpers (private)
// ---------------------------------------------------------------------------

/// Find the closest position in `candidates` that is within `max_range` of `origin`.
fn closest_within(candidates: &[Vec3], origin: Vec3, max_range: f32) -> Option<Vec3> {
    let max_range_sq = max_range * max_range;
    candidates
        .iter()
        .filter(|c| c.distance_squared(origin) <= max_range_sq)
        .min_by(|a, b| {
            let da = a.distance_squared(origin);
            let db = b.distance_squared(origin);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Iron Golem construction --------------------------------------------

    #[test]
    fn iron_golem_new_has_100_hp() {
        let g = IronGolem::new(Vec3::ZERO);
        assert!((g.health - 100.0).abs() < f32::EPSILON);
        assert_eq!(g.position, Vec3::ZERO);
        assert_eq!(g.village_center, None);
    }

    // -- Snow Golem construction --------------------------------------------

    #[test]
    fn snow_golem_new_has_4_hp() {
        let g = SnowGolem::new(Vec3::new(1.0, 2.0, 3.0));
        assert!((g.health - 4.0).abs() < f32::EPSILON);
        assert_eq!(g.position, Vec3::new(1.0, 2.0, 3.0));
    }

    // -- Iron Golem pattern detection ---------------------------------------

    fn make_block_getter(blocks: &[(i32, i32, i32, u16)]) -> impl Fn(i32, i32, i32) -> u16 + '_ {
        move |x, y, z| {
            blocks
                .iter()
                .find(|(bx, by, bz, _)| *bx == x && *by == y && *bz == z)
                .map(|(_, _, _, id)| *id)
                .unwrap_or(0)
        }
    }

    #[test]
    fn detects_iron_golem_pattern_along_x_axis() {
        let blocks = vec![
            (0, 0, 0, IRON_BLOCK_ID),    // body center
            (-1, 1, 0, IRON_BLOCK_ID),   // arm left
            (0, 1, 0, IRON_BLOCK_ID),    // arm center
            (1, 1, 0, IRON_BLOCK_ID),    // arm right
            (0, 2, 0, PUMPKIN_BLOCK_ID), // head
        ];
        let getter = make_block_getter(&blocks);
        let result = check_iron_golem_pattern(&getter);
        assert!(result.is_some());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 5);
    }

    #[test]
    fn detects_iron_golem_pattern_along_z_axis() {
        let blocks = vec![
            (0, 0, 0, IRON_BLOCK_ID),    // body center
            (0, 1, -1, IRON_BLOCK_ID),   // arm back
            (0, 1, 0, IRON_BLOCK_ID),    // arm center
            (0, 1, 1, IRON_BLOCK_ID),    // arm front
            (0, 2, 0, PUMPKIN_BLOCK_ID), // head
        ];
        let getter = make_block_getter(&blocks);
        let result = check_iron_golem_pattern(&getter);
        assert!(result.is_some());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 5);
    }

    #[test]
    fn rejects_incomplete_iron_golem_pattern() {
        // Missing one arm block.
        let blocks = vec![
            (0, 0, 0, IRON_BLOCK_ID),
            (-1, 1, 0, IRON_BLOCK_ID),
            (0, 1, 0, IRON_BLOCK_ID),
            // (1, 1, 0) is missing
            (0, 2, 0, PUMPKIN_BLOCK_ID),
        ];
        let getter = make_block_getter(&blocks);
        let result = check_iron_golem_pattern(&getter);
        assert!(result.is_none());
    }

    #[test]
    fn rejects_iron_golem_pattern_without_pumpkin() {
        let blocks = vec![
            (0, 0, 0, IRON_BLOCK_ID),
            (-1, 1, 0, IRON_BLOCK_ID),
            (0, 1, 0, IRON_BLOCK_ID),
            (1, 1, 0, IRON_BLOCK_ID),
            (0, 2, 0, IRON_BLOCK_ID), // iron instead of pumpkin
        ];
        let getter = make_block_getter(&blocks);
        let result = check_iron_golem_pattern(&getter);
        assert!(result.is_none());
    }

    // -- Snow Golem pattern detection ---------------------------------------

    #[test]
    fn detects_snow_golem_pattern() {
        let blocks = vec![
            (0, 0, 0, SNOW_BLOCK_ID),
            (0, 1, 0, SNOW_BLOCK_ID),
            (0, 2, 0, PUMPKIN_BLOCK_ID),
        ];
        let getter = make_block_getter(&blocks);
        let result = check_snow_golem_pattern(&getter);
        assert!(result.is_some());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 3);
    }

    #[test]
    fn rejects_incomplete_snow_golem_pattern() {
        // Only one snow block instead of two.
        let blocks = vec![
            (0, 0, 0, SNOW_BLOCK_ID),
            (0, 1, 0, 0), // air
            (0, 2, 0, PUMPKIN_BLOCK_ID),
        ];
        let getter = make_block_getter(&blocks);
        let result = check_snow_golem_pattern(&getter);
        assert!(result.is_none());
    }

    // -- Iron Golem behavior ------------------------------------------------

    #[test]
    fn iron_golem_attacks_nearest_hostile() {
        let golem = IronGolem::new(Vec3::ZERO);
        let hostiles = vec![Vec3::new(15.0, 0.0, 0.0), Vec3::new(5.0, 0.0, 0.0)];
        let villagers = vec![Vec3::new(3.0, 0.0, 0.0)];

        let action = iron_golem_tick(&golem, &hostiles, &villagers);
        assert_eq!(action, GolemAction::AttackHostile(Vec3::new(5.0, 0.0, 0.0)));
    }

    #[test]
    fn iron_golem_hostile_priority_over_patrol() {
        let mut golem = IronGolem::new(Vec3::ZERO);
        golem.village_center = Some(Vec3::new(50.0, 0.0, 50.0));
        let hostiles = vec![Vec3::new(10.0, 0.0, 0.0)];

        let action = iron_golem_tick(&golem, &hostiles, &[]);
        assert_eq!(
            action,
            GolemAction::AttackHostile(Vec3::new(10.0, 0.0, 0.0))
        );
    }

    #[test]
    fn iron_golem_patrols_village_when_no_hostiles() {
        let mut golem = IronGolem::new(Vec3::ZERO);
        golem.village_center = Some(Vec3::new(50.0, 0.0, 50.0));

        let action = iron_golem_tick(&golem, &[], &[]);
        assert_eq!(
            action,
            GolemAction::PatrolVillage(Vec3::new(50.0, 0.0, 50.0))
        );
    }

    #[test]
    fn iron_golem_offers_flower_to_nearby_villager() {
        let golem = IronGolem::new(Vec3::ZERO);
        let villagers = vec![Vec3::new(4.0, 0.0, 0.0)];

        let action = iron_golem_tick(&golem, &[], &villagers);
        assert_eq!(action, GolemAction::OfferFlower);
    }

    #[test]
    fn iron_golem_idles_when_nothing_to_do() {
        let golem = IronGolem::new(Vec3::ZERO);
        let action = iron_golem_tick(&golem, &[], &[]);
        assert_eq!(action, GolemAction::Idle);
    }

    #[test]
    fn iron_golem_ignores_hostiles_beyond_range() {
        let golem = IronGolem::new(Vec3::ZERO);
        let hostiles = vec![Vec3::new(20.0, 0.0, 0.0)]; // > 16 blocks

        let action = iron_golem_tick(&golem, &hostiles, &[]);
        assert_eq!(action, GolemAction::Idle);
    }

    // -- Snow Golem behavior ------------------------------------------------

    #[test]
    fn snow_golem_melts_in_hot_biome() {
        let golem = SnowGolem::new(Vec3::ZERO);
        let effect = snow_golem_tick(&golem, 1.5);
        assert_eq!(effect, GolemEffect::Melting);
    }

    #[test]
    fn snow_golem_melts_at_threshold() {
        let golem = SnowGolem::new(Vec3::ZERO);
        let effect = snow_golem_tick(&golem, 1.0);
        assert_eq!(effect, GolemEffect::Melting);
    }

    #[test]
    fn snow_golem_places_snow_in_cold_biome() {
        let golem = SnowGolem::new(Vec3::ZERO);
        let effect = snow_golem_tick(&golem, 0.5);
        assert_eq!(effect, GolemEffect::PlaceSnow);
    }

    #[test]
    fn snow_golem_places_snow_in_freezing_biome() {
        let golem = SnowGolem::new(Vec3::ZERO);
        let effect = snow_golem_tick(&golem, -0.5);
        assert_eq!(effect, GolemEffect::PlaceSnow);
    }
}
