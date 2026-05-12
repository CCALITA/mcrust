/// Nether portal linking: coordinate conversion, frame geometry, and portal pairing.

/// Search radius (in blocks) when looking for an existing portal to link to.
pub const PORTAL_SEARCH_RADIUS: i32 = 128;

/// A linked pair of portal positions between the overworld and the nether.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalLink {
    pub overworld_pos: (i32, i32, i32),
    pub nether_pos: (i32, i32, i32),
}

/// Convert overworld coordinates to nether coordinates.
///
/// The x and z axes are divided by 8; y is unchanged.
pub fn overworld_to_nether(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (x / 8, y, z / 8)
}

/// Convert nether coordinates to overworld coordinates.
///
/// The x and z axes are multiplied by 8; y is unchanged.
pub fn nether_to_overworld(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (x * 8, y, z * 8)
}

/// Return the block positions that make up a 4-wide x 5-tall portal frame.
///
/// The frame is oriented along the x-axis with `center` as the bottom-left
/// corner. The result contains 14 obsidian positions: 4 bottom, 4 top, 3 left
/// pillar, and 3 right pillar.
pub fn find_portal_frame_positions(center: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
    let (cx, cy, cz) = center;
    let mut positions = Vec::with_capacity(14);

    // Bottom row (4 blocks)
    for dx in 0..4 {
        positions.push((cx + dx, cy, cz));
    }

    // Top row (4 blocks)
    for dx in 0..4 {
        positions.push((cx + dx, cy + 4, cz));
    }

    // Left pillar (3 blocks, y+1 to y+3)
    for dy in 1..4 {
        positions.push((cx, cy + dy, cz));
    }

    // Right pillar (3 blocks, y+1 to y+3)
    for dy in 1..4 {
        positions.push((cx + 3, cy + dy, cz));
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overworld_to_nether_divides_xz_by_eight() {
        assert_eq!(overworld_to_nether(80, 64, -160), (10, 64, -20));
    }

    #[test]
    fn overworld_to_nether_zero() {
        assert_eq!(overworld_to_nether(0, 0, 0), (0, 0, 0));
    }

    #[test]
    fn overworld_to_nether_truncates_toward_zero() {
        assert_eq!(overworld_to_nether(7, 100, 15), (0, 100, 1));
    }

    #[test]
    fn nether_to_overworld_multiplies_xz_by_eight() {
        assert_eq!(nether_to_overworld(10, 64, -20), (80, 64, -160));
    }

    #[test]
    fn nether_to_overworld_zero() {
        assert_eq!(nether_to_overworld(0, 0, 0), (0, 0, 0));
    }

    #[test]
    fn round_trip_overworld_nether() {
        let (nx, ny, nz) = overworld_to_nether(800, 64, -400);
        let back = nether_to_overworld(nx, ny, nz);
        assert_eq!(back, (800, 64, -400));
    }

    #[test]
    fn portal_link_stores_both_positions() {
        let link = PortalLink {
            overworld_pos: (100, 64, 200),
            nether_pos: (12, 64, 25),
        };
        assert_eq!(link.overworld_pos, (100, 64, 200));
        assert_eq!(link.nether_pos, (12, 64, 25));
    }

    #[test]
    fn portal_link_equality() {
        let a = PortalLink {
            overworld_pos: (0, 64, 0),
            nether_pos: (0, 64, 0),
        };
        let b = PortalLink {
            overworld_pos: (0, 64, 0),
            nether_pos: (0, 64, 0),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn portal_link_inequality() {
        let a = PortalLink {
            overworld_pos: (0, 64, 0),
            nether_pos: (0, 64, 0),
        };
        let b = PortalLink {
            overworld_pos: (8, 64, 8),
            nether_pos: (1, 64, 1),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn frame_positions_count_is_14() {
        let positions = find_portal_frame_positions((0, 0, 0));
        assert_eq!(positions.len(), 14);
    }

    #[test]
    fn frame_positions_contain_bottom_row() {
        let positions = find_portal_frame_positions((10, 20, 30));
        for dx in 0..4 {
            assert!(
                positions.contains(&(10 + dx, 20, 30)),
                "missing bottom block at dx={dx}"
            );
        }
    }

    #[test]
    fn frame_positions_contain_top_row() {
        let positions = find_portal_frame_positions((10, 20, 30));
        for dx in 0..4 {
            assert!(
                positions.contains(&(10 + dx, 24, 30)),
                "missing top block at dx={dx}"
            );
        }
    }

    #[test]
    fn frame_positions_contain_left_pillar() {
        let positions = find_portal_frame_positions((10, 20, 30));
        for dy in 1..4 {
            assert!(
                positions.contains(&(10, 20 + dy, 30)),
                "missing left pillar block at dy={dy}"
            );
        }
    }

    #[test]
    fn frame_positions_contain_right_pillar() {
        let positions = find_portal_frame_positions((10, 20, 30));
        for dy in 1..4 {
            assert!(
                positions.contains(&(13, 20 + dy, 30)),
                "missing right pillar block at dy={dy}"
            );
        }
    }

    #[test]
    fn frame_positions_no_duplicates() {
        let positions = find_portal_frame_positions((0, 0, 0));
        let mut sorted = positions.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), positions.len(), "frame has duplicate positions");
    }

    #[test]
    fn portal_search_radius_is_128() {
        assert_eq!(PORTAL_SEARCH_RADIUS, 128);
    }

    #[test]
    fn negative_coordinate_conversion() {
        assert_eq!(overworld_to_nether(-80, 64, -160), (-10, 64, -20));
        assert_eq!(nether_to_overworld(-10, 64, -20), (-80, 64, -160));
    }
}
