// ---------------------------------------------------------------------------
// Villager job / workstation binding system
// ---------------------------------------------------------------------------

use glam::Vec3;

// ---------------------------------------------------------------------------
// Workstation types
// ---------------------------------------------------------------------------

/// All workstation block types that can assign a profession to a villager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkstationType {
    Barrel,
    BlastFurnace,
    BrewingStand,
    CartographyTable,
    Composter,
    FletchingTable,
    Grindstone,
    Lectern,
    Loom,
    MasonTable,
    Smoker,
    SmithingTable,
    Stonecutter,
}

/// Maps a workstation type to the corresponding `VillagerProfession` ordinal.
///
/// Ordinals follow the declaration order in [`super::villager::VillagerProfession`]:
///
/// | Ordinal | Profession     |
/// |---------|----------------|
/// | 0       | Farmer         |
/// | 1       | Librarian      |
/// | 2       | Cleric         |
/// | 3       | Armorer        |
/// | 4       | Weaponsmith    |
/// | 5       | Toolsmith      |
/// | 6       | Butcher        |
/// | 7       | Leatherworker  |
/// | 8       | Fletcher       |
/// | 9       | Cartographer   |
/// | 10      | Mason          |
/// | 11      | Shepherd       |
/// | 12      | Nitwit         |
pub fn workstation_profession(ws: WorkstationType) -> u8 {
    match ws {
        WorkstationType::Composter => 0,        // Farmer
        WorkstationType::Lectern => 1,          // Librarian
        WorkstationType::BrewingStand => 2,     // Cleric
        WorkstationType::BlastFurnace => 3,     // Armorer
        WorkstationType::Grindstone => 4,       // Weaponsmith
        WorkstationType::SmithingTable => 5,    // Toolsmith
        WorkstationType::Smoker => 6,           // Butcher
        WorkstationType::Barrel => 0, // Fisherman — mapped to Farmer (no Fisherman variant)
        WorkstationType::FletchingTable => 8, // Fletcher
        WorkstationType::CartographyTable => 9, // Cartographer
        WorkstationType::MasonTable => 10, // Mason
        WorkstationType::Loom => 11,  // Shepherd
        WorkstationType::Stonecutter => 10, // Mason
    }
}

// ---------------------------------------------------------------------------
// Job binding
// ---------------------------------------------------------------------------

/// Represents the binding between a villager and its workstation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VillagerJobBinding {
    /// `VillagerProfession` ordinal for this binding.
    pub profession: u8,
    /// Block position of the claimed workstation.
    pub workstation_pos: (i32, i32, i32),
    /// Number of restocks the villager has performed today (max 2).
    pub restocks_today: u8,
}

/// Create a new job binding for a villager at the given workstation position.
pub fn bind_villager(profession: u8, pos: (i32, i32, i32)) -> VillagerJobBinding {
    VillagerJobBinding {
        profession,
        workstation_pos: pos,
        restocks_today: 0,
    }
}

// ---------------------------------------------------------------------------
// Workstation search
// ---------------------------------------------------------------------------

/// Find the closest available workstation within `max_dist` blocks of the
/// villager. Returns the index into `available` of the nearest workstation, or
/// `None` if none is in range.
pub fn find_workstation(
    villager_pos: Vec3,
    available: &[((i32, i32, i32), WorkstationType)],
    max_dist: f32,
) -> Option<usize> {
    let max_dist_sq = max_dist * max_dist;

    available
        .iter()
        .enumerate()
        .filter_map(|(i, &((x, y, z), _ws))| {
            let ws_pos = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
            let dist_sq = villager_pos.distance_squared(ws_pos);
            if dist_sq <= max_dist_sq {
                Some((i, dist_sq))
            } else {
                None
            }
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
}

// ---------------------------------------------------------------------------
// Restock
// ---------------------------------------------------------------------------

/// Attempt to restock the villager's trades. Villagers may restock up to 2
/// times per day. Returns `true` if the restock was allowed, `false` if the
/// daily limit has been reached.
///
/// Returns a new `VillagerJobBinding` with the updated restock count on
/// success, or `None` when the daily limit (2) has been reached.
pub fn try_restock(binding: &VillagerJobBinding) -> Option<VillagerJobBinding> {
    if binding.restocks_today >= 2 {
        return None;
    }
    Some(VillagerJobBinding {
        profession: binding.profession,
        workstation_pos: binding.workstation_pos,
        restocks_today: binding.restocks_today + 1,
    })
}

/// Remove the villager's workstation binding, returning a binding with
/// profession reset to `Nitwit` ordinal (12) and position zeroed.
pub fn unbind(binding: &VillagerJobBinding) -> VillagerJobBinding {
    let _ = binding; // acknowledge the old binding
    VillagerJobBinding {
        profession: 12, // Nitwit
        workstation_pos: (0, 0, 0),
        restocks_today: 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- bind_villager -------------------------------------------------------

    #[test]
    fn bind_villager_creates_correct_binding() {
        let binding = bind_villager(0, (10, 64, -20));
        assert_eq!(binding.profession, 0);
        assert_eq!(binding.workstation_pos, (10, 64, -20));
        assert_eq!(binding.restocks_today, 0);
    }

    #[test]
    fn bind_villager_preserves_negative_coords() {
        let binding = bind_villager(3, (-100, -64, -200));
        assert_eq!(binding.workstation_pos, (-100, -64, -200));
    }

    // -- find_workstation (distance) -----------------------------------------

    #[test]
    fn find_workstation_returns_nearest_within_range() {
        let villager_pos = Vec3::new(0.0, 64.0, 0.0);
        let available = vec![
            ((10, 64, 0), WorkstationType::Composter),
            ((3, 64, 0), WorkstationType::Lectern),
            ((20, 64, 0), WorkstationType::BrewingStand),
        ];
        let result = find_workstation(villager_pos, &available, 48.0);
        assert_eq!(result, Some(1)); // Lectern at (3,64,0) is closest
    }

    #[test]
    fn find_workstation_returns_none_when_all_out_of_range() {
        let villager_pos = Vec3::new(0.0, 64.0, 0.0);
        let available = vec![
            ((100, 64, 0), WorkstationType::Composter),
            ((200, 64, 0), WorkstationType::Lectern),
        ];
        let result = find_workstation(villager_pos, &available, 5.0);
        assert_eq!(result, None);
    }

    #[test]
    fn find_workstation_returns_none_for_empty_list() {
        let villager_pos = Vec3::new(0.0, 64.0, 0.0);
        let result = find_workstation(villager_pos, &[], 48.0);
        assert_eq!(result, None);
    }

    #[test]
    fn find_workstation_uses_3d_distance() {
        let villager_pos = Vec3::new(0.0, 64.0, 0.0);
        // (0,74,0) is 10 blocks up — within range of 15
        // (12,64,0) is 12 blocks east — also within range of 15
        let available = vec![
            ((0, 74, 0), WorkstationType::Barrel),
            ((12, 64, 0), WorkstationType::Loom),
        ];
        let result = find_workstation(villager_pos, &available, 15.0);
        // (0,74,0) center = (0.5, 74.5, 0.5), dist ~ 10.5
        // (12,64,0) center = (12.5, 64.5, 0.5), dist ~ 12.5
        assert_eq!(result, Some(0));
    }

    // -- workstation_profession (profession match) ---------------------------

    #[test]
    fn composter_maps_to_farmer() {
        assert_eq!(workstation_profession(WorkstationType::Composter), 0);
    }

    #[test]
    fn lectern_maps_to_librarian() {
        assert_eq!(workstation_profession(WorkstationType::Lectern), 1);
    }

    #[test]
    fn brewing_stand_maps_to_cleric() {
        assert_eq!(workstation_profession(WorkstationType::BrewingStand), 2);
    }

    #[test]
    fn blast_furnace_maps_to_armorer() {
        assert_eq!(workstation_profession(WorkstationType::BlastFurnace), 3);
    }

    #[test]
    fn grindstone_maps_to_weaponsmith() {
        assert_eq!(workstation_profession(WorkstationType::Grindstone), 4);
    }

    #[test]
    fn smithing_table_maps_to_toolsmith() {
        assert_eq!(workstation_profession(WorkstationType::SmithingTable), 5);
    }

    #[test]
    fn smoker_maps_to_butcher() {
        assert_eq!(workstation_profession(WorkstationType::Smoker), 6);
    }

    #[test]
    fn fletching_table_maps_to_fletcher() {
        assert_eq!(workstation_profession(WorkstationType::FletchingTable), 8);
    }

    #[test]
    fn cartography_table_maps_to_cartographer() {
        assert_eq!(workstation_profession(WorkstationType::CartographyTable), 9);
    }

    #[test]
    fn mason_table_maps_to_mason() {
        assert_eq!(workstation_profession(WorkstationType::MasonTable), 10);
    }

    #[test]
    fn loom_maps_to_shepherd() {
        assert_eq!(workstation_profession(WorkstationType::Loom), 11);
    }

    #[test]
    fn stonecutter_maps_to_mason() {
        assert_eq!(workstation_profession(WorkstationType::Stonecutter), 10);
    }

    #[test]
    fn all_workstation_types_map_to_valid_ordinal() {
        let all = [
            WorkstationType::Barrel,
            WorkstationType::BlastFurnace,
            WorkstationType::BrewingStand,
            WorkstationType::CartographyTable,
            WorkstationType::Composter,
            WorkstationType::FletchingTable,
            WorkstationType::Grindstone,
            WorkstationType::Lectern,
            WorkstationType::Loom,
            WorkstationType::MasonTable,
            WorkstationType::Smoker,
            WorkstationType::SmithingTable,
            WorkstationType::Stonecutter,
        ];
        for ws in &all {
            let ordinal = workstation_profession(*ws);
            assert!(ordinal <= 12, "{ws:?} mapped to invalid ordinal {ordinal}");
        }
    }

    // -- try_restock (restock limit) -----------------------------------------

    #[test]
    fn first_restock_succeeds() {
        let binding = bind_villager(0, (10, 64, 0));
        let result = try_restock(&binding);
        assert!(result.is_some());
        assert_eq!(result.unwrap().restocks_today, 1);
    }

    #[test]
    fn second_restock_succeeds() {
        let binding = VillagerJobBinding {
            profession: 0,
            workstation_pos: (10, 64, 0),
            restocks_today: 1,
        };
        let result = try_restock(&binding);
        assert!(result.is_some());
        assert_eq!(result.unwrap().restocks_today, 2);
    }

    #[test]
    fn third_restock_fails() {
        let binding = VillagerJobBinding {
            profession: 0,
            workstation_pos: (10, 64, 0),
            restocks_today: 2,
        };
        let result = try_restock(&binding);
        assert!(result.is_none());
    }

    #[test]
    fn restock_preserves_binding_fields() {
        let binding = bind_villager(5, (42, 70, -3));
        let restocked = try_restock(&binding).unwrap();
        assert_eq!(restocked.profession, 5);
        assert_eq!(restocked.workstation_pos, (42, 70, -3));
    }

    // -- unbind --------------------------------------------------------------

    #[test]
    fn unbind_resets_to_nitwit() {
        let binding = bind_villager(3, (10, 64, 0));
        let unbound = unbind(&binding);
        assert_eq!(unbound.profession, 12); // Nitwit
        assert_eq!(unbound.workstation_pos, (0, 0, 0));
        assert_eq!(unbound.restocks_today, 0);
    }
}
