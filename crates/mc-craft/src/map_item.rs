// ── Map item & exploration system ────────────────────────────────────────
//
// Implements map data representation and world-to-pixel mapping for the
// in-game map item. Each map covers a 128x128 pixel grid at a configurable
// scale level (0-4), mapping world blocks to colour indices.

/// Side length of a map in pixels.
const MAP_SIZE: usize = 128;

/// Maximum allowed scale level.
const MAX_SCALE: u8 = 4;

// ── Data types ──────────────────────────────────────────────────────────

/// Represents the pixel data and metadata for a single map item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapData {
    /// 128x128 colour-index pixels stored in row-major order.
    pub pixels: Vec<u8>,
    /// Zoom level (0 = closest / 1:1, 4 = furthest / 1:16).
    pub scale: u8,
    /// World X coordinate of the map centre.
    pub center_x: i32,
    /// World Z coordinate of the map centre.
    pub center_z: i32,
    /// Dimension index (0 = overworld, 1 = nether, 2 = end).
    pub dimension: u8,
    /// Whether the map is locked (no further pixel updates).
    pub locked: bool,
}

impl MapData {
    /// Create a new map centred at (`center_x`, `center_z`) with the given
    /// `scale` (clamped to 0..=4) and `dimension`.
    ///
    /// All 128x128 pixels are initialised to 0 (transparent).
    #[must_use]
    pub fn new(center_x: i32, center_z: i32, scale: u8, dimension: u8) -> Self {
        Self {
            pixels: vec![0u8; MAP_SIZE * MAP_SIZE],
            scale: scale.min(MAX_SCALE),
            center_x,
            center_z,
            dimension,
            locked: false,
        }
    }

    /// Set a single pixel colour. Does nothing if (`px`, `pz`) is out of
    /// bounds.
    pub fn set_pixel(&mut self, px: usize, pz: usize, color: u8) {
        if px < MAP_SIZE && pz < MAP_SIZE {
            self.pixels[pz * MAP_SIZE + px] = color;
        }
    }

    /// Read a single pixel colour. Returns 0 if (`px`, `pz`) is out of bounds.
    #[must_use]
    pub fn get_pixel(&self, px: usize, pz: usize) -> u8 {
        if px < MAP_SIZE && pz < MAP_SIZE {
            self.pixels[pz * MAP_SIZE + px]
        } else {
            0
        }
    }

    /// Lock the map, preventing further pixel updates.
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Check whether the map is locked.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

// ── Scale helpers ───────────────────────────────────────────────────────

/// Total world-block range covered by a map at the given scale.
///
/// | Scale | Range  |
/// |-------|--------|
/// | 0     | 128    |
/// | 1     | 256    |
/// | 2     | 512    |
/// | 3     | 1024   |
/// | 4     | 2048   |
#[must_use]
pub fn map_range(scale: u8) -> i32 {
    (MAP_SIZE as i32) * blocks_per_pixel(scale)
}

/// Number of world blocks represented by a single map pixel at the given
/// scale level.
#[must_use]
pub fn blocks_per_pixel(scale: u8) -> i32 {
    1 << scale.min(MAX_SCALE)
}

// ── Block-to-colour mapping ─────────────────────────────────────────────

/// Map a block ID to its representative map colour index.
///
/// Returns 0 (transparent) for unknown blocks.
#[must_use]
pub fn map_pixel_for_block(block_id: u16) -> u8 {
    match block_id {
        1 => 11,  // stone
        2 => 34,  // grass
        3 => 10,  // dirt
        12 => 18, // sand
        17 => 22, // wood (oak log)
        18 => 30, // leaves
        79 => 50, // ice
        80 => 8,  // snow
        // Water IDs — flowing (8) and still (9)
        8 | 9 => 48,
        _ => 0,
    }
}

// ── World-to-pixel coordinate conversion ────────────────────────────────

/// Convert a world coordinate to the corresponding pixel coordinate on the
/// given map.
///
/// Returns `None` if the world position falls outside the map's coverage
/// area.
#[must_use]
pub fn world_to_pixel(world_x: i32, world_z: i32, map: &MapData) -> Option<(usize, usize)> {
    let bpp = blocks_per_pixel(map.scale);
    let half_range = (MAP_SIZE as i32 * bpp) / 2;

    let dx = world_x - map.center_x;
    let dz = world_z - map.center_z;

    if dx < -half_range || dx >= half_range || dz < -half_range || dz >= half_range {
        return None;
    }

    let px = ((dx + half_range) / bpp) as usize;
    let pz = ((dz + half_range) / bpp) as usize;

    if px < MAP_SIZE && pz < MAP_SIZE {
        Some((px, pz))
    } else {
        None
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction ────────────────────────────────────────────────────

    #[test]
    fn new_map_has_correct_dimensions() {
        let map = MapData::new(0, 0, 0, 0);
        assert_eq!(map.pixels.len(), 128 * 128);
        assert!(map.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn scale_is_clamped_to_max() {
        let map = MapData::new(0, 0, 10, 0);
        assert_eq!(map.scale, MAX_SCALE);
    }

    #[test]
    fn new_map_is_not_locked() {
        let map = MapData::new(100, -200, 2, 1);
        assert!(!map.is_locked());
    }

    // ── Pixel access ────────────────────────────────────────────────────

    #[test]
    fn set_and_get_pixel_within_bounds() {
        let mut map = MapData::new(0, 0, 0, 0);
        map.set_pixel(10, 20, 34);
        assert_eq!(map.get_pixel(10, 20), 34);
    }

    #[test]
    fn set_pixel_out_of_bounds_is_noop() {
        let mut map = MapData::new(0, 0, 0, 0);
        map.set_pixel(128, 0, 99);
        map.set_pixel(0, 128, 99);
        map.set_pixel(200, 200, 99);
        // Should not panic and pixels remain at 0
        assert_eq!(map.get_pixel(127, 0), 0);
    }

    #[test]
    fn get_pixel_out_of_bounds_returns_zero() {
        let map = MapData::new(0, 0, 0, 0);
        assert_eq!(map.get_pixel(128, 0), 0);
        assert_eq!(map.get_pixel(0, 200), 0);
    }

    #[test]
    fn pixel_corner_access() {
        let mut map = MapData::new(0, 0, 0, 0);
        map.set_pixel(0, 0, 1);
        map.set_pixel(127, 0, 2);
        map.set_pixel(0, 127, 3);
        map.set_pixel(127, 127, 4);
        assert_eq!(map.get_pixel(0, 0), 1);
        assert_eq!(map.get_pixel(127, 0), 2);
        assert_eq!(map.get_pixel(0, 127), 3);
        assert_eq!(map.get_pixel(127, 127), 4);
    }

    // ── Scale ranges ────────────────────────────────────────────────────

    #[test]
    fn map_range_values() {
        assert_eq!(map_range(0), 128);
        assert_eq!(map_range(1), 256);
        assert_eq!(map_range(2), 512);
        assert_eq!(map_range(3), 1024);
        assert_eq!(map_range(4), 2048);
    }

    #[test]
    fn blocks_per_pixel_values() {
        assert_eq!(blocks_per_pixel(0), 1);
        assert_eq!(blocks_per_pixel(1), 2);
        assert_eq!(blocks_per_pixel(2), 4);
        assert_eq!(blocks_per_pixel(3), 8);
        assert_eq!(blocks_per_pixel(4), 16);
    }

    #[test]
    fn blocks_per_pixel_clamps_above_max() {
        assert_eq!(blocks_per_pixel(5), blocks_per_pixel(4));
        assert_eq!(blocks_per_pixel(255), blocks_per_pixel(4));
    }

    // ── World-to-pixel conversion ───────────────────────────────────────

    #[test]
    fn world_to_pixel_center() {
        let map = MapData::new(0, 0, 0, 0);
        // Centre of the map at scale 0: world (0,0) -> pixel (64, 64)
        let result = world_to_pixel(0, 0, &map);
        assert_eq!(result, Some((64, 64)));
    }

    #[test]
    fn world_to_pixel_at_edges() {
        let map = MapData::new(0, 0, 0, 0);
        // Top-left corner: world (-64, -64) -> pixel (0, 0)
        assert_eq!(world_to_pixel(-64, -64, &map), Some((0, 0)));
        // Just inside bottom-right: world (63, 63) -> pixel (127, 127)
        assert_eq!(world_to_pixel(63, 63, &map), Some((127, 127)));
    }

    #[test]
    fn world_to_pixel_out_of_bounds() {
        let map = MapData::new(0, 0, 0, 0);
        assert_eq!(world_to_pixel(64, 0, &map), None);
        assert_eq!(world_to_pixel(0, 64, &map), None);
        assert_eq!(world_to_pixel(-65, 0, &map), None);
        assert_eq!(world_to_pixel(0, -65, &map), None);
    }

    #[test]
    fn world_to_pixel_with_offset_center() {
        let map = MapData::new(1000, -500, 0, 0);
        // Centre pixel
        assert_eq!(world_to_pixel(1000, -500, &map), Some((64, 64)));
        // Out of range
        assert_eq!(world_to_pixel(0, 0, &map), None);
    }

    #[test]
    fn world_to_pixel_at_scale_2() {
        // Scale 2 => blocks_per_pixel = 4, range = 512, half = 256
        let map = MapData::new(0, 0, 2, 0);
        assert_eq!(world_to_pixel(0, 0, &map), Some((64, 64)));
        // Edge: world (-256, -256) -> pixel (0, 0)
        assert_eq!(world_to_pixel(-256, -256, &map), Some((0, 0)));
        // Just outside
        assert_eq!(world_to_pixel(256, 0, &map), None);
    }

    // ── Block colour mapping ────────────────────────────────────────────

    #[test]
    fn known_block_colours() {
        assert_eq!(map_pixel_for_block(2), 34);  // grass
        assert_eq!(map_pixel_for_block(8), 48);  // water (flowing)
        assert_eq!(map_pixel_for_block(9), 48);  // water (still)
        assert_eq!(map_pixel_for_block(12), 18); // sand
        assert_eq!(map_pixel_for_block(1), 11);  // stone
        assert_eq!(map_pixel_for_block(3), 10);  // dirt
        assert_eq!(map_pixel_for_block(80), 8);  // snow
        assert_eq!(map_pixel_for_block(17), 22); // wood
        assert_eq!(map_pixel_for_block(18), 30); // leaves
        assert_eq!(map_pixel_for_block(79), 50); // ice
    }

    #[test]
    fn unknown_block_returns_transparent() {
        assert_eq!(map_pixel_for_block(0), 0);
        assert_eq!(map_pixel_for_block(9999), 0);
    }

    // ── Lock behaviour ──────────────────────────────────────────────────

    #[test]
    fn lock_sets_locked_flag() {
        let mut map = MapData::new(0, 0, 0, 0);
        assert!(!map.is_locked());
        map.lock();
        assert!(map.is_locked());
    }

    #[test]
    fn locked_map_retains_pixel_data() {
        let mut map = MapData::new(0, 0, 0, 0);
        map.set_pixel(5, 5, 42);
        map.lock();
        assert_eq!(map.get_pixel(5, 5), 42);
    }
}
