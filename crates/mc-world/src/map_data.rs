use mc_core::BlockId;

// ---------------------------------------------------------------------------
// Map color constants (u8 palette indices, inspired by Minecraft's map colors)
// ---------------------------------------------------------------------------

pub mod map_color {
    pub const GRASS: u8 = 0;
    pub const WATER: u8 = 1;
    pub const SAND: u8 = 2;
    pub const STONE: u8 = 3;
    pub const DIRT: u8 = 4;
    pub const WOOD: u8 = 5;
    pub const SNOW: u8 = 6;
    pub const ICE: u8 = 7;
    pub const LAVA: u8 = 8;
    pub const LEAVES: u8 = 9;
    pub const CLAY: u8 = 10;
    pub const OBSIDIAN: u8 = 11;
    pub const NETHER: u8 = 12;
    pub const RED: u8 = 13;
    pub const BLUE: u8 = 14;
    pub const GREEN: u8 = 15;
    pub const YELLOW: u8 = 16;
    pub const WHITE: u8 = 17;
    pub const BLACK: u8 = 18;
    pub const TRANSPARENT: u8 = 19;
}

/// Map width/height in pixels.
pub const MAP_SIZE: usize = 128;

// ---------------------------------------------------------------------------
// Block-to-color mapping
// ---------------------------------------------------------------------------

/// Return the map palette color for a given block type.
#[allow(clippy::too_many_lines)]
pub fn block_to_map_color(block: BlockId) -> u8 {
    match block {
        // Transparent / non-visible
        BlockId::Air => map_color::TRANSPARENT,

        // Stone family
        BlockId::Stone
        | BlockId::Cobblestone
        | BlockId::MossyCobblestone
        | BlockId::StoneBricks
        | BlockId::Bedrock
        | BlockId::Gravel => map_color::STONE,

        // Dirt / grass / mycelium / podzol
        BlockId::Dirt => map_color::DIRT,
        BlockId::GrassBlock | BlockId::Mycelium | BlockId::Podzol => map_color::GRASS,

        // Water
        BlockId::Water => map_color::WATER,

        // Sand
        BlockId::Sand => map_color::SAND,

        // Logs and planks → wood
        BlockId::OakLog
        | BlockId::BirchLog
        | BlockId::SpruceLog
        | BlockId::JungleLog
        | BlockId::DarkOakLog
        | BlockId::OakPlanks
        | BlockId::BirchPlanks
        | BlockId::SprucePlanks
        | BlockId::JunglePlanks
        | BlockId::DarkOakPlanks
        | BlockId::CraftingTable
        | BlockId::Bookshelf
        | BlockId::NoteBlock => map_color::WOOD,

        // Leaves
        BlockId::OakLeaves
        | BlockId::BirchLeaves
        | BlockId::SpruceLeaves
        | BlockId::JungleLeaves
        | BlockId::DarkOakLeaves
        | BlockId::Cactus => map_color::LEAVES,

        // Ores → stone tint
        BlockId::CoalOre
        | BlockId::IronOre
        | BlockId::GoldOre
        | BlockId::DiamondOre
        | BlockId::CopperOre
        | BlockId::LapisOre
        | BlockId::EmeraldOre
        | BlockId::RedstoneOre => map_color::STONE,

        // Glass / torch — mostly transparent
        BlockId::Glass | BlockId::Torch => map_color::TRANSPARENT,

        // Furnace / dispenser / dropper / hopper / observer / piston family → stone
        BlockId::Furnace
        | BlockId::Dispenser
        | BlockId::Dropper
        | BlockId::Hopper
        | BlockId::Observer
        | BlockId::Piston
        | BlockId::StickyPiston => map_color::STONE,

        // Chest → wood
        BlockId::Chest => map_color::WOOD,

        // Obsidian
        BlockId::Obsidian => map_color::OBSIDIAN,

        // Snow / snow block
        BlockId::Snow | BlockId::SnowBlock => map_color::SNOW,

        // Ice / packed ice
        BlockId::Ice | BlockId::PackedIce => map_color::ICE,

        // Clay / terracotta
        BlockId::Clay | BlockId::Terracotta => map_color::CLAY,

        // Bricks → clay-ish
        BlockId::Bricks => map_color::CLAY,

        // Wool colors
        BlockId::RedWool => map_color::RED,
        BlockId::BlueWool => map_color::BLUE,
        BlockId::GreenWool => map_color::GREEN,
        BlockId::YellowWool => map_color::YELLOW,
        BlockId::WhiteWool => map_color::WHITE,
        BlockId::BlackWool => map_color::BLACK,

        // Plants
        BlockId::SugarCane | BlockId::TallGrass => map_color::GRASS,
        BlockId::Pumpkin | BlockId::Melon => map_color::GREEN,
        BlockId::Dandelion => map_color::YELLOW,
        BlockId::Poppy => map_color::RED,
        BlockId::RedMushroom => map_color::RED,
        BlockId::BrownMushroom => map_color::DIRT,

        // TNT → red
        BlockId::TNT => map_color::RED,

        // Nether blocks
        BlockId::Netherrack | BlockId::SoulSand => map_color::NETHER,
        BlockId::Glowstone => map_color::YELLOW,

        // End stone → sand-ish
        BlockId::EndStone => map_color::SAND,

        // Redstone components → red / stone
        BlockId::RedstoneDust | BlockId::RedstoneTorch | BlockId::RedstoneLamp => map_color::RED,
        BlockId::Lever | BlockId::StoneButton | BlockId::Repeater | BlockId::Comparator => {
            map_color::STONE
        }

        // Farming blocks
        BlockId::Farmland => map_color::DIRT,
        BlockId::WheatCrop
        | BlockId::CarrotCrop
        | BlockId::PotatoCrop
        | BlockId::BeetrootCrop
        | BlockId::MelonStem
        | BlockId::PumpkinStem => map_color::GRASS,
    }
}

/// Convert a palette color index to an RGB triple for rendering.
pub fn map_color_to_rgb(color: u8) -> (u8, u8, u8) {
    match color {
        map_color::GRASS => (127, 178, 56),
        map_color::WATER => (64, 64, 255),
        map_color::SAND => (247, 233, 163),
        map_color::STONE => (112, 112, 112),
        map_color::DIRT => (151, 109, 77),
        map_color::WOOD => (143, 119, 72),
        map_color::SNOW => (255, 255, 255),
        map_color::ICE => (160, 160, 255),
        map_color::LAVA => (255, 0, 0),
        map_color::LEAVES => (0, 124, 0),
        map_color::CLAY => (164, 168, 184),
        map_color::OBSIDIAN => (20, 18, 30),
        map_color::NETHER => (112, 2, 0),
        map_color::RED => (180, 0, 0),
        map_color::BLUE => (64, 64, 255),
        map_color::GREEN => (0, 124, 0),
        map_color::YELLOW => (250, 238, 77),
        map_color::WHITE => (255, 255, 255),
        map_color::BLACK => (25, 25, 25),
        map_color::TRANSPARENT => (0, 0, 0),
        _ => (0, 0, 0), // unknown → black
    }
}

// ---------------------------------------------------------------------------
// MapData
// ---------------------------------------------------------------------------

/// A 128x128 map image stored as palette-indexed pixels.
#[derive(Debug, Clone)]
pub struct MapData {
    /// Row-major palette-indexed pixels (`MAP_SIZE * MAP_SIZE`).
    pixels: Vec<u8>,
    /// World X coordinate at the centre of the map.
    pub center_x: i32,
    /// World Z coordinate at the centre of the map.
    pub center_z: i32,
    /// Blocks per pixel (1, 2, 4, 8, 16).
    pub scale: u8,
    /// 0 = Overworld, 1 = Nether, 2 = End.
    pub dimension: u8,
}

impl MapData {
    /// Create a blank (transparent) map centred at `(center_x, center_z)`.
    pub fn new(center_x: i32, center_z: i32, scale: u8, dimension: u8) -> Self {
        Self {
            pixels: vec![map_color::TRANSPARENT; MAP_SIZE * MAP_SIZE],
            center_x,
            center_z,
            scale,
            dimension,
        }
    }

    /// Return the palette color at pixel `(x, z)`, or `None` if out of bounds.
    pub fn get_pixel(&self, x: usize, z: usize) -> Option<u8> {
        if x < MAP_SIZE && z < MAP_SIZE {
            Some(self.pixels[z * MAP_SIZE + x])
        } else {
            None
        }
    }

    /// Set the palette color at pixel `(x, z)`. Returns `true` on success.
    pub fn set_pixel(&mut self, x: usize, z: usize, color: u8) -> bool {
        if x < MAP_SIZE && z < MAP_SIZE {
            self.pixels[z * MAP_SIZE + x] = color;
            true
        } else {
            false
        }
    }

    /// Read-only access to the full pixel buffer.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

// ---------------------------------------------------------------------------
// Map generation
// ---------------------------------------------------------------------------

/// Generate a map by sampling surface blocks from the world.
///
/// `get_surface_block` is called with world `(x, z)` coordinates and must
/// return the topmost visible `BlockId` at that column.
pub fn generate_map(
    center_x: i32,
    center_z: i32,
    scale: u8,
    dimension: u8,
    get_surface_block: &dyn Fn(i32, i32) -> BlockId,
) -> MapData {
    let mut map = MapData::new(center_x, center_z, scale, dimension);
    let blocks_per_pixel = (scale as i32).max(1);
    let half = (MAP_SIZE as i32) / 2;

    for pz in 0..MAP_SIZE {
        for px in 0..MAP_SIZE {
            let world_x = center_x + (px as i32 - half) * blocks_per_pixel;
            let world_z = center_z + (pz as i32 - half) * blocks_per_pixel;
            let block = get_surface_block(world_x, world_z);
            map.pixels[pz * MAP_SIZE + px] = block_to_map_color(block);
        }
    }

    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Color mapping -------------------------------------------------------

    #[test]
    fn air_maps_to_transparent() {
        assert_eq!(block_to_map_color(BlockId::Air), map_color::TRANSPARENT);
    }

    #[test]
    fn grass_block_maps_to_grass() {
        assert_eq!(block_to_map_color(BlockId::GrassBlock), map_color::GRASS);
    }

    #[test]
    fn water_maps_to_water() {
        assert_eq!(block_to_map_color(BlockId::Water), map_color::WATER);
    }

    #[test]
    fn stone_maps_to_stone() {
        assert_eq!(block_to_map_color(BlockId::Stone), map_color::STONE);
    }

    #[test]
    fn oak_log_maps_to_wood() {
        assert_eq!(block_to_map_color(BlockId::OakLog), map_color::WOOD);
    }

    #[test]
    fn oak_leaves_maps_to_leaves() {
        assert_eq!(block_to_map_color(BlockId::OakLeaves), map_color::LEAVES);
    }

    #[test]
    fn snow_maps_to_snow_color() {
        assert_eq!(block_to_map_color(BlockId::SnowBlock), map_color::SNOW);
    }

    #[test]
    fn ice_maps_to_ice_color() {
        assert_eq!(block_to_map_color(BlockId::Ice), map_color::ICE);
    }

    #[test]
    fn obsidian_maps_to_obsidian_color() {
        assert_eq!(block_to_map_color(BlockId::Obsidian), map_color::OBSIDIAN);
    }

    #[test]
    fn all_84_blocks_have_a_color_mapping() {
        for id in 0..BlockId::COUNT as u16 {
            let block = BlockId::from_raw(id).unwrap();
            // Should not panic; every variant is handled.
            let _color = block_to_map_color(block);
        }
    }

    #[test]
    fn wool_colors_map_correctly() {
        assert_eq!(block_to_map_color(BlockId::RedWool), map_color::RED);
        assert_eq!(block_to_map_color(BlockId::BlueWool), map_color::BLUE);
        assert_eq!(block_to_map_color(BlockId::GreenWool), map_color::GREEN);
        assert_eq!(block_to_map_color(BlockId::YellowWool), map_color::YELLOW);
        assert_eq!(block_to_map_color(BlockId::WhiteWool), map_color::WHITE);
        assert_eq!(block_to_map_color(BlockId::BlackWool), map_color::BLACK);
    }

    #[test]
    fn nether_blocks_map_to_nether_color() {
        assert_eq!(block_to_map_color(BlockId::Netherrack), map_color::NETHER);
        assert_eq!(block_to_map_color(BlockId::SoulSand), map_color::NETHER);
    }

    // -- RGB conversion ------------------------------------------------------

    #[test]
    fn grass_rgb_is_green_tinted() {
        let (r, g, b) = map_color_to_rgb(map_color::GRASS);
        assert!(g > r && g > b, "grass should be predominantly green");
    }

    #[test]
    fn snow_rgb_is_white() {
        assert_eq!(map_color_to_rgb(map_color::SNOW), (255, 255, 255));
    }

    #[test]
    fn unknown_color_returns_black() {
        assert_eq!(map_color_to_rgb(255), (0, 0, 0));
    }

    // -- MapData pixel access ------------------------------------------------

    #[test]
    fn new_map_is_all_transparent() {
        let map = MapData::new(0, 0, 1, 0);
        assert_eq!(map.pixels().len(), MAP_SIZE * MAP_SIZE);
        assert!(map.pixels().iter().all(|&c| c == map_color::TRANSPARENT));
    }

    #[test]
    fn get_set_pixel_roundtrip() {
        let mut map = MapData::new(0, 0, 1, 0);
        assert!(map.set_pixel(10, 20, map_color::WATER));
        assert_eq!(map.get_pixel(10, 20), Some(map_color::WATER));
    }

    #[test]
    fn out_of_bounds_pixel_access() {
        let mut map = MapData::new(0, 0, 1, 0);
        assert_eq!(map.get_pixel(MAP_SIZE, 0), None);
        assert_eq!(map.get_pixel(0, MAP_SIZE), None);
        assert!(!map.set_pixel(MAP_SIZE, 0, map_color::STONE));
    }

    // -- Map generation ------------------------------------------------------

    #[test]
    fn generate_map_fills_all_pixels() {
        let map = generate_map(0, 0, 1, 0, &|_x, _z| BlockId::GrassBlock);
        assert!(map.pixels().iter().all(|&c| c == map_color::GRASS));
    }

    #[test]
    fn generate_map_respects_scale() {
        // With scale=2, each pixel covers 2 blocks.
        // The callback records which world coordinates are queried.
        let mut visited = std::collections::HashSet::new();
        let visited_ptr = &mut visited as *mut std::collections::HashSet<(i32, i32)>;

        let map = generate_map(100, 200, 2, 0, &|x, z| {
            // SAFETY: single-threaded test, pointer is valid for the closure's
            // lifetime.
            unsafe {
                (*visited_ptr).insert((x, z));
            }
            BlockId::Sand
        });

        // Total pixels sampled must equal 128*128
        assert_eq!(visited.len(), MAP_SIZE * MAP_SIZE);
        assert!(map.pixels().iter().all(|&c| c == map_color::SAND));

        // Check that the world coordinates are spaced by scale=2
        let half = (MAP_SIZE as i32) / 2;
        let expected_min_x = 100 - half * 2;
        let expected_max_x = 100 + (MAP_SIZE as i32 - 1 - half) * 2;
        let min_x = visited.iter().map(|&(x, _)| x).min().unwrap();
        let max_x = visited.iter().map(|&(x, _)| x).max().unwrap();
        assert_eq!(min_x, expected_min_x);
        assert_eq!(max_x, expected_max_x);
    }

    #[test]
    fn generate_map_stores_metadata() {
        let map = generate_map(500, -300, 4, 1, &|_, _| BlockId::Netherrack);
        assert_eq!(map.center_x, 500);
        assert_eq!(map.center_z, -300);
        assert_eq!(map.scale, 4);
        assert_eq!(map.dimension, 1);
    }
}
