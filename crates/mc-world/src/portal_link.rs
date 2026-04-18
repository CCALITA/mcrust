//! Nether portal linking: coordinate scaling, portal search, and frame creation.

/// Scale overworld coordinates to nether coordinates (divide x and z by 8).
pub fn nether_coordinate_scale(pos: (i32, i32, i32)) -> (i32, i32, i32) {
    (pos.0 / 8, pos.1, pos.2 / 8)
}

/// Scale nether coordinates to overworld coordinates (multiply x and z by 8).
pub fn overworld_coordinate_scale(pos: (i32, i32, i32)) -> (i32, i32, i32) {
    (pos.0 * 8, pos.1, pos.2 * 8)
}

/// Find the nearest portal within `max_dist` blocks (Euclidean distance).
/// Returns `None` if no portal is within range.
pub fn find_nearest_portal(
    search: (i32, i32, i32),
    portals: &[(i32, i32, i32)],
    max_dist: i32,
) -> Option<(i32, i32, i32)> {
    let max_dist_sq = (max_dist as i64) * (max_dist as i64);

    portals
        .iter()
        .filter_map(|&p| {
            let dx = (p.0 - search.0) as i64;
            let dy = (p.1 - search.1) as i64;
            let dz = (p.2 - search.2) as i64;
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq <= max_dist_sq {
                Some((dist_sq, p))
            } else {
                None
            }
        })
        .min_by_key(|&(dist_sq, _)| dist_sq)
        .map(|(_, p)| p)
}

/// Search radius for portal linking.
/// Overworld: 128 blocks, Nether: 16 blocks.
pub fn search_radius(is_nether: bool) -> i32 {
    if is_nether { 16 } else { 128 }
}

/// Build a 4-wide x 5-tall obsidian portal frame at the given position.
///
/// The frame is oriented along the x-axis. `pos` is the bottom-left corner.
/// Calls `set_block(x, y, z, block_id)` for each obsidian block placed.
/// Block ID 49 is used for obsidian.
pub fn create_portal_frame(pos: (i32, i32, i32), set_block: &mut dyn FnMut(i32, i32, i32, u16)) {
    let (bx, by, bz) = pos;
    let obsidian: u16 = 49;

    for dx in 0..4 {
        // Bottom row
        set_block(bx + dx, by, bz, obsidian);
        // Top row
        set_block(bx + dx, by + 4, bz, obsidian);
    }

    for dy in 1..4 {
        // Left pillar
        set_block(bx, by + dy, bz, obsidian);
        // Right pillar
        set_block(bx + 3, by + dy, bz, obsidian);
    }
}

/// A linked pair of portals between the overworld and the nether.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalLink {
    pub overworld: (i32, i32, i32),
    pub nether: (i32, i32, i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nether_scale_divides_xz_by_eight() {
        assert_eq!(nether_coordinate_scale((80, 64, -160)), (10, 64, -20));
        assert_eq!(nether_coordinate_scale((0, 0, 0)), (0, 0, 0));
        assert_eq!(nether_coordinate_scale((7, 100, 15)), (0, 100, 1));
    }

    #[test]
    fn overworld_scale_multiplies_xz_by_eight() {
        assert_eq!(overworld_coordinate_scale((10, 64, -20)), (80, 64, -160));
        assert_eq!(overworld_coordinate_scale((0, 0, 0)), (0, 0, 0));
    }

    #[test]
    fn round_trip_coordinate_scaling() {
        let overworld = (800, 64, -400);
        let nether = nether_coordinate_scale(overworld);
        let back = overworld_coordinate_scale(nether);
        assert_eq!(back, (800, 64, -400));
    }

    #[test]
    fn find_nearest_portal_returns_closest() {
        let portals = vec![(10, 64, 10), (100, 64, 100), (5, 64, 5)];
        let result = find_nearest_portal((0, 64, 0), &portals, 128);
        assert_eq!(result, Some((5, 64, 5)));
    }

    #[test]
    fn find_nearest_portal_returns_none_when_out_of_range() {
        let portals = vec![(1000, 64, 1000)];
        let result = find_nearest_portal((0, 64, 0), &portals, 128);
        assert_eq!(result, None);
    }

    #[test]
    fn find_nearest_portal_empty_list() {
        let result = find_nearest_portal((0, 64, 0), &[], 128);
        assert_eq!(result, None);
    }

    #[test]
    fn search_radius_overworld_128_nether_16() {
        assert_eq!(search_radius(false), 128);
        assert_eq!(search_radius(true), 16);
    }

    #[test]
    fn create_portal_frame_places_correct_block_count() {
        let mut count = 0u32;
        create_portal_frame((0, 0, 0), &mut |_, _, _, _| {
            count += 1;
        });
        // 4 bottom + 4 top + 3 left pillar + 3 right pillar = 14
        assert_eq!(count, 14);
    }

    #[test]
    fn create_portal_frame_uses_obsidian_id() {
        let mut ids = Vec::new();
        create_portal_frame((0, 0, 0), &mut |_, _, _, id| {
            ids.push(id);
        });
        assert!(ids.iter().all(|&id| id == 49));
    }
}
