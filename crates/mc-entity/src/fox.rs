use glam::Vec3;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Item ID for sweet berries.
const SWEET_BERRY_ID: u16 = 1125;

/// Damage taken when a fox eats sweet berries.
const SWEET_BERRY_DAMAGE: f32 = 1.0;

/// Range within which a fox will pick up a ground item.
const PICKUP_RANGE: f32 = 2.0;

/// Range within which a fox will pounce on prey.
const POUNCE_RANGE: f32 = 12.0;

/// Range at which a fox flees from a non-trusted player.
const FLEE_RANGE: f32 = 16.0;

/// Daytime boundary (ticks). Day is [0, 12_000), night is [12_000, 24_000).
const DAY_END_TICK: u32 = 12_000;

// ---------------------------------------------------------------------------
// Fox variant & state
// ---------------------------------------------------------------------------

/// The two fox variants found in Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoxVariant {
    Red,
    Arctic,
}

/// Immutable snapshot of a fox's state.
#[derive(Debug, Clone, PartialEq)]
pub struct FoxState {
    pub variant: FoxVariant,
    pub held_item: Option<u16>,
    pub sleeping: bool,
    pub trusted_players: Vec<u64>,
}

// ---------------------------------------------------------------------------
// Fox action
// ---------------------------------------------------------------------------

/// Action produced by `fox_tick` each game tick.
#[derive(Debug, Clone, PartialEq)]
pub enum FoxAction {
    /// Curl up and sleep.
    Sleep,
    /// Pick up a nearby ground item.
    PickUpItem(u16),
    /// Pounce toward a target position.
    Pounce(Vec3),
    /// Flee from a non-trusted entity.
    Flee,
    /// Eat the held item. For sweet berries this inflicts damage.
    Eat,
}

// ---------------------------------------------------------------------------
// Nearby items / entities passed into fox_tick
// ---------------------------------------------------------------------------

/// A ground item near the fox.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearbyItem {
    pub item_id: u16,
    pub position: Vec3,
}

/// A nearby entity the fox may react to (player, prey, etc.).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearbyEntity {
    pub id: u64,
    pub position: Vec3,
    /// `true` if this entity is something the fox can pounce on (chicken, rabbit, etc.).
    pub is_prey: bool,
}

// ---------------------------------------------------------------------------
// Tick function
// ---------------------------------------------------------------------------

/// Returns `true` when the world time represents daytime.
fn is_daytime(time_of_day: u32) -> bool {
    time_of_day < DAY_END_TICK
}

/// Pick the closest item within `PICKUP_RANGE` of `fox_pos`.
fn closest_item_in_range(items: &[NearbyItem], fox_pos: Vec3) -> Option<NearbyItem> {
    let range_sq = PICKUP_RANGE * PICKUP_RANGE;
    items
        .iter()
        .filter(|i| i.position.distance_squared(fox_pos) <= range_sq)
        .min_by(|a, b| {
            let da = a.position.distance_squared(fox_pos);
            let db = b.position.distance_squared(fox_pos);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

/// Pick the closest prey within `POUNCE_RANGE` of `fox_pos`.
fn closest_prey_in_range(entities: &[NearbyEntity], fox_pos: Vec3) -> Option<NearbyEntity> {
    let range_sq = POUNCE_RANGE * POUNCE_RANGE;
    entities
        .iter()
        .filter(|e| e.is_prey && e.position.distance_squared(fox_pos) <= range_sq)
        .min_by(|a, b| {
            let da = a.position.distance_squared(fox_pos);
            let db = b.position.distance_squared(fox_pos);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

/// Returns `true` when a non-trusted player is within `FLEE_RANGE`.
fn should_flee(state: &FoxState, entities: &[NearbyEntity], fox_pos: Vec3) -> bool {
    let range_sq = FLEE_RANGE * FLEE_RANGE;
    entities.iter().any(|e| {
        !e.is_prey
            && !state.trusted_players.contains(&e.id)
            && e.position.distance_squared(fox_pos) <= range_sq
    })
}

/// Determine the fox's action for this tick.
///
/// Priority order (nocturnal -- sleeps during the day):
/// 1. **Flee** from a nearby non-trusted player.
/// 2. **Sleep** during daytime (nocturnal behavior).
/// 3. **Eat** held sweet berries (takes damage).
/// 4. **Pick up** a nearby ground item (only if not holding one).
/// 5. **Pounce** on nearby prey at night.
/// 6. Fall back to **Sleep** (idle resting).
pub fn fox_tick(
    state: &FoxState,
    fox_pos: Vec3,
    time_of_day: u32,
    nearby_items: &[NearbyItem],
    nearby_entities: &[NearbyEntity],
) -> FoxAction {
    // Priority 1 -- flee from non-trusted players.
    if should_flee(state, nearby_entities, fox_pos) {
        return FoxAction::Flee;
    }

    // Priority 2 -- nocturnal: sleep during the day.
    if is_daytime(time_of_day) {
        return FoxAction::Sleep;
    }

    // Priority 3 -- eat held sweet berries.
    if state.held_item == Some(SWEET_BERRY_ID) {
        return FoxAction::Eat;
    }

    // Priority 4 -- pick up a nearby item (only when mouth is empty).
    if state.held_item.is_none() {
        if let Some(item) = closest_item_in_range(nearby_items, fox_pos) {
            return FoxAction::PickUpItem(item.item_id);
        }
    }

    // Priority 5 -- pounce on prey at night.
    if let Some(prey) = closest_prey_in_range(nearby_entities, fox_pos) {
        return FoxAction::Pounce(prey.position);
    }

    // Default -- rest.
    FoxAction::Sleep
}

/// Calculate the damage a fox takes from eating sweet berries.
/// Returns `Some(damage)` if the item is sweet berries, `None` otherwise.
pub fn sweet_berry_eat_damage(item_id: u16) -> Option<f32> {
    if item_id == SWEET_BERRY_ID {
        Some(SWEET_BERRY_DAMAGE)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn red_fox() -> FoxState {
        FoxState {
            variant: FoxVariant::Red,
            held_item: None,
            sleeping: false,
            trusted_players: vec![],
        }
    }

    fn arctic_fox() -> FoxState {
        FoxState {
            variant: FoxVariant::Arctic,
            held_item: None,
            sleeping: false,
            trusted_players: vec![],
        }
    }

    // -- Variant construction --------------------------------------------------

    #[test]
    fn red_fox_variant() {
        let fox = red_fox();
        assert_eq!(fox.variant, FoxVariant::Red);
    }

    #[test]
    fn arctic_fox_variant() {
        let fox = arctic_fox();
        assert_eq!(fox.variant, FoxVariant::Arctic);
    }

    // -- Nocturnal behavior (sleep during day) ---------------------------------

    #[test]
    fn fox_sleeps_during_daytime() {
        let fox = red_fox();
        let action = fox_tick(&fox, Vec3::ZERO, 6_000, &[], &[]);
        assert_eq!(action, FoxAction::Sleep);
    }

    #[test]
    fn fox_does_not_auto_sleep_at_night() {
        let fox = red_fox();
        // No items, no entities, no held item -- default action is Sleep,
        // but the path through the function skips the daytime branch.
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &[], &[]);
        // With nothing to do at night, the fox falls back to resting.
        assert_eq!(action, FoxAction::Sleep);
    }

    // -- Pick up items ---------------------------------------------------------

    #[test]
    fn fox_picks_up_nearby_item_at_night() {
        let fox = red_fox();
        let items = [NearbyItem {
            item_id: 42,
            position: Vec3::new(1.0, 0.0, 0.0),
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &items, &[]);
        assert_eq!(action, FoxAction::PickUpItem(42));
    }

    #[test]
    fn fox_ignores_items_when_already_holding_one() {
        let fox = FoxState {
            variant: FoxVariant::Red,
            held_item: Some(99),
            sleeping: false,
            trusted_players: vec![],
        };
        let items = [NearbyItem {
            item_id: 42,
            position: Vec3::new(1.0, 0.0, 0.0),
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &items, &[]);
        // Should not pick up another item; falls through to Sleep.
        assert_eq!(action, FoxAction::Sleep);
    }

    #[test]
    fn fox_ignores_items_out_of_range() {
        let fox = red_fox();
        let items = [NearbyItem {
            item_id: 42,
            position: Vec3::new(10.0, 0.0, 0.0), // far away
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &items, &[]);
        assert_eq!(action, FoxAction::Sleep);
    }

    // -- Eat sweet berries -----------------------------------------------------

    #[test]
    fn fox_eats_sweet_berries_and_takes_damage() {
        let fox = FoxState {
            variant: FoxVariant::Red,
            held_item: Some(SWEET_BERRY_ID),
            sleeping: false,
            trusted_players: vec![],
        };
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &[], &[]);
        assert_eq!(action, FoxAction::Eat);

        let damage = sweet_berry_eat_damage(SWEET_BERRY_ID);
        assert_eq!(damage, Some(1.0));
    }

    #[test]
    fn eating_non_berry_item_does_no_damage() {
        let damage = sweet_berry_eat_damage(99);
        assert_eq!(damage, None);
    }

    // -- Pounce on prey --------------------------------------------------------

    #[test]
    fn fox_pounces_on_nearby_prey_at_night() {
        let fox = red_fox();
        let prey_pos = Vec3::new(5.0, 0.0, 0.0);
        let entities = [NearbyEntity {
            id: 1,
            position: prey_pos,
            is_prey: true,
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &[], &entities);
        assert_eq!(action, FoxAction::Pounce(prey_pos));
    }

    #[test]
    fn fox_ignores_prey_out_of_pounce_range() {
        let fox = red_fox();
        let entities = [NearbyEntity {
            id: 1,
            position: Vec3::new(20.0, 0.0, 0.0), // > 12 blocks
            is_prey: true,
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &[], &entities);
        assert_eq!(action, FoxAction::Sleep);
    }

    // -- Flee from non-trusted players -----------------------------------------

    #[test]
    fn fox_flees_from_non_trusted_player() {
        let fox = red_fox();
        let entities = [NearbyEntity {
            id: 100,
            position: Vec3::new(5.0, 0.0, 0.0),
            is_prey: false,
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &[], &entities);
        assert_eq!(action, FoxAction::Flee);
    }

    #[test]
    fn fox_does_not_flee_from_trusted_player() {
        let fox = FoxState {
            variant: FoxVariant::Red,
            held_item: None,
            sleeping: false,
            trusted_players: vec![100],
        };
        let entities = [NearbyEntity {
            id: 100,
            position: Vec3::new(5.0, 0.0, 0.0),
            is_prey: false,
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &[], &entities);
        // Trusted player -- no flee; falls through to Sleep.
        assert_eq!(action, FoxAction::Sleep);
    }

    #[test]
    fn flee_takes_priority_over_sleep_during_day() {
        let fox = red_fox();
        let entities = [NearbyEntity {
            id: 200,
            position: Vec3::new(5.0, 0.0, 0.0),
            is_prey: false,
        }];
        // Daytime -- would normally sleep, but flee overrides.
        let action = fox_tick(&fox, Vec3::ZERO, 6_000, &[], &entities);
        assert_eq!(action, FoxAction::Flee);
    }

    #[test]
    fn fox_does_not_flee_from_distant_player() {
        let fox = red_fox();
        let entities = [NearbyEntity {
            id: 300,
            position: Vec3::new(20.0, 0.0, 0.0), // > 16 blocks
            is_prey: false,
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &[], &entities);
        assert_eq!(action, FoxAction::Sleep);
    }

    // -- Priority ordering -----------------------------------------------------

    #[test]
    fn flee_has_highest_priority() {
        // Fox has sweet berries, nearby prey, nearby item, AND non-trusted player.
        let fox = FoxState {
            variant: FoxVariant::Red,
            held_item: Some(SWEET_BERRY_ID),
            sleeping: false,
            trusted_players: vec![],
        };
        let items = [NearbyItem {
            item_id: 42,
            position: Vec3::new(1.0, 0.0, 0.0),
        }];
        let entities = [
            NearbyEntity {
                id: 1,
                position: Vec3::new(3.0, 0.0, 0.0),
                is_prey: true,
            },
            NearbyEntity {
                id: 500,
                position: Vec3::new(5.0, 0.0, 0.0),
                is_prey: false,
            },
        ];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &items, &entities);
        assert_eq!(action, FoxAction::Flee);
    }

    #[test]
    fn eat_has_priority_over_pickup_and_pounce() {
        let fox = FoxState {
            variant: FoxVariant::Arctic,
            held_item: Some(SWEET_BERRY_ID),
            sleeping: false,
            trusted_players: vec![],
        };
        let items = [NearbyItem {
            item_id: 42,
            position: Vec3::new(1.0, 0.0, 0.0),
        }];
        let entities = [NearbyEntity {
            id: 1,
            position: Vec3::new(3.0, 0.0, 0.0),
            is_prey: true,
        }];
        let action = fox_tick(&fox, Vec3::ZERO, 18_000, &items, &entities);
        assert_eq!(action, FoxAction::Eat);
    }
}
