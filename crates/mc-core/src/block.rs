use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum BlockId {
    Air = 0,
    Stone,
    Dirt,
    GrassBlock,
    Bedrock,
    Water,
    Sand,
    Gravel,
    OakLog,
    OakLeaves,
    OakPlanks,
    Cobblestone,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Glass,
    Torch,
    CraftingTable,
    Furnace,
    Chest,
    // --- New block types ---
    BirchLog,
    BirchLeaves,
    BirchPlanks,
    SpruceLog,
    SpruceLeaves,
    SprucePlanks,
    JungleLog,
    JungleLeaves,
    JunglePlanks,
    DarkOakLog,
    DarkOakLeaves,
    DarkOakPlanks,
    CopperOre,
    LapisOre,
    EmeraldOre,
    RedstoneOre,
    Obsidian,
    Snow,
    SnowBlock,
    Ice,
    PackedIce,
    Clay,
    Terracotta,
    RedWool,
    BlueWool,
    GreenWool,
    YellowWool,
    WhiteWool,
    BlackWool,
    Cactus,
    SugarCane,
    Pumpkin,
    Melon,
    TNT,
    Bookshelf,
    MossyCobblestone,
    Bricks,
    StoneBricks,
    Netherrack,
    SoulSand,
    Glowstone,
    EndStone,
    Mycelium,
    Podzol,
    RedMushroom,
    BrownMushroom,
    TallGrass,
    Dandelion,
    Poppy,
    // --- Redstone block types ---
    RedstoneDust,
    RedstoneTorch,
    Lever,
    StoneButton,
    Repeater,
    Comparator,
    Piston,
    StickyPiston,
    Observer,
    Hopper,
    Dispenser,
    Dropper,
    NoteBlock,
    RedstoneLamp,
    // --- Farming block types ---
    Farmland,
    WheatCrop,
    CarrotCrop,
    PotatoCrop,
    BeetrootCrop,
    MelonStem,
    PumpkinStem,
}

impl BlockId {
    pub const COUNT: usize = 91;

    pub fn from_raw(id: u16) -> Option<Self> {
        if (id as usize) < Self::COUNT {
            // SAFETY: `id` is validated to be within [0, COUNT), and BlockId
            // is #[repr(u16)] with contiguous discriminants starting at 0.
            Some(unsafe { std::mem::transmute::<u16, BlockId>(id) })
        } else {
            None
        }
    }

    pub fn is_air(self) -> bool {
        matches!(self, BlockId::Air)
    }

    pub fn is_solid(self) -> bool {
        !matches!(
            self,
            BlockId::Air
                | BlockId::Water
                | BlockId::Torch
                | BlockId::SugarCane
                | BlockId::TallGrass
                | BlockId::Dandelion
                | BlockId::Poppy
                | BlockId::RedMushroom
                | BlockId::BrownMushroom
                | BlockId::Snow
                | BlockId::RedstoneDust
                | BlockId::RedstoneTorch
                | BlockId::Lever
                | BlockId::StoneButton
                | BlockId::Repeater
                | BlockId::Comparator
                | BlockId::WheatCrop
                | BlockId::CarrotCrop
                | BlockId::PotatoCrop
                | BlockId::BeetrootCrop
                | BlockId::MelonStem
                | BlockId::PumpkinStem
        )
    }

    pub fn is_transparent(self) -> bool {
        matches!(
            self,
            BlockId::Air
                | BlockId::Water
                | BlockId::Glass
                | BlockId::Torch
                | BlockId::OakLeaves
                | BlockId::BirchLeaves
                | BlockId::SpruceLeaves
                | BlockId::JungleLeaves
                | BlockId::DarkOakLeaves
                | BlockId::Ice
                | BlockId::SugarCane
                | BlockId::TallGrass
                | BlockId::Dandelion
                | BlockId::Poppy
                | BlockId::RedMushroom
                | BlockId::BrownMushroom
                | BlockId::Cactus
                | BlockId::Snow
                | BlockId::RedstoneDust
                | BlockId::RedstoneTorch
                | BlockId::Lever
                | BlockId::StoneButton
                | BlockId::Repeater
                | BlockId::Comparator
                | BlockId::WheatCrop
                | BlockId::CarrotCrop
                | BlockId::PotatoCrop
                | BlockId::BeetrootCrop
                | BlockId::MelonStem
                | BlockId::PumpkinStem
        )
    }

    pub fn properties(self) -> &'static BlockProperties {
        &BLOCK_REGISTRY[self as usize]
    }
}

#[derive(Debug, Clone)]
pub struct BlockProperties {
    pub name: &'static str,
    pub solid: bool,
    pub transparent: bool,
    pub light_emission: u8,
    pub hardness: f32,
    /// Texture indices for [top, bottom, north, south, east, west]
    pub tex_indices: [u16; 6],
}

pub struct BlockRegistry;

impl BlockRegistry {
    pub fn get(id: BlockId) -> &'static BlockProperties {
        &BLOCK_REGISTRY[id as usize]
    }
}

const fn props(
    name: &'static str,
    solid: bool,
    transparent: bool,
    light: u8,
    hardness: f32,
    tex: [u16; 6],
) -> BlockProperties {
    BlockProperties {
        name,
        solid,
        transparent,
        light_emission: light,
        hardness,
        tex_indices: tex,
    }
}

/// Uniform texture on all 6 faces
const fn uniform(
    name: &'static str,
    solid: bool,
    transparent: bool,
    light: u8,
    hardness: f32,
    tex: u16,
) -> BlockProperties {
    props(name, solid, transparent, light, hardness, [tex; 6])
}

static BLOCK_REGISTRY: [BlockProperties; BlockId::COUNT] = [
    // Air
    uniform("air", false, true, 0, 0.0, 0),
    // Stone
    uniform("stone", true, false, 0, 1.5, 1),
    // Dirt
    uniform("dirt", true, false, 0, 0.5, 2),
    // GrassBlock: top=3, bottom=2(dirt), sides=4
    props("grass_block", true, false, 0, 0.6, [3, 2, 4, 4, 4, 4]),
    // Bedrock
    uniform("bedrock", true, false, 0, -1.0, 5),
    // Water
    uniform("water", false, true, 0, 100.0, 6),
    // Sand
    uniform("sand", true, false, 0, 0.5, 7),
    // Gravel
    uniform("gravel", true, false, 0, 0.6, 8),
    // OakLog: top/bottom=9, sides=10
    props("oak_log", true, false, 0, 2.0, [9, 9, 10, 10, 10, 10]),
    // OakLeaves
    uniform("oak_leaves", true, true, 0, 0.2, 11),
    // OakPlanks
    uniform("oak_planks", true, false, 0, 2.0, 12),
    // Cobblestone
    uniform("cobblestone", true, false, 0, 2.0, 13),
    // CoalOre
    uniform("coal_ore", true, false, 0, 3.0, 14),
    // IronOre
    uniform("iron_ore", true, false, 0, 3.0, 15),
    // GoldOre
    uniform("gold_ore", true, false, 0, 3.0, 16),
    // DiamondOre
    uniform("diamond_ore", true, false, 0, 3.0, 17),
    // Glass
    uniform("glass", true, true, 0, 0.3, 18),
    // Torch
    uniform("torch", false, true, 14, 0.0, 19),
    // CraftingTable: top=20, bottom=12(planks), sides=21
    props(
        "crafting_table",
        true,
        false,
        0,
        2.5,
        [20, 12, 21, 21, 21, 21],
    ),
    // Furnace: top=1(stone), bottom=1, front=22, sides=23
    props("furnace", true, false, 0, 3.5, [1, 1, 22, 23, 23, 23]),
    // Chest: top=24, bottom=24, front=25, sides=26
    props("chest", true, false, 0, 2.5, [24, 24, 25, 26, 26, 26]),
    // --- New block types ---
    // BirchLog: top/bottom=27, sides=28
    props("birch_log", true, false, 0, 2.0, [27, 27, 28, 28, 28, 28]),
    // BirchLeaves
    uniform("birch_leaves", true, true, 0, 0.2, 29),
    // BirchPlanks
    uniform("birch_planks", true, false, 0, 2.0, 30),
    // SpruceLog: top/bottom=31, sides=32
    props("spruce_log", true, false, 0, 2.0, [31, 31, 32, 32, 32, 32]),
    // SpruceLeaves
    uniform("spruce_leaves", true, true, 0, 0.2, 33),
    // SprucePlanks
    uniform("spruce_planks", true, false, 0, 2.0, 34),
    // JungleLog: top/bottom=35, sides=36
    props("jungle_log", true, false, 0, 2.0, [35, 35, 36, 36, 36, 36]),
    // JungleLeaves
    uniform("jungle_leaves", true, true, 0, 0.2, 37),
    // JunglePlanks
    uniform("jungle_planks", true, false, 0, 2.0, 38),
    // DarkOakLog: top/bottom=39, sides=40
    props(
        "dark_oak_log",
        true,
        false,
        0,
        2.0,
        [39, 39, 40, 40, 40, 40],
    ),
    // DarkOakLeaves
    uniform("dark_oak_leaves", true, true, 0, 0.2, 41),
    // DarkOakPlanks
    uniform("dark_oak_planks", true, false, 0, 2.0, 42),
    // CopperOre
    uniform("copper_ore", true, false, 0, 3.0, 43),
    // LapisOre
    uniform("lapis_ore", true, false, 0, 3.0, 44),
    // EmeraldOre
    uniform("emerald_ore", true, false, 0, 3.0, 45),
    // RedstoneOre
    uniform("redstone_ore", true, false, 0, 3.0, 46),
    // Obsidian
    uniform("obsidian", true, false, 0, 50.0, 47),
    // Snow (non-solid layer)
    uniform("snow", false, true, 0, 0.1, 48),
    // SnowBlock
    uniform("snow_block", true, false, 0, 0.2, 49),
    // Ice
    uniform("ice", true, true, 0, 0.5, 50),
    // PackedIce
    uniform("packed_ice", true, false, 0, 0.5, 51),
    // Clay
    uniform("clay", true, false, 0, 0.6, 52),
    // Terracotta
    uniform("terracotta", true, false, 0, 1.25, 53),
    // RedWool
    uniform("red_wool", true, false, 0, 0.8, 54),
    // BlueWool
    uniform("blue_wool", true, false, 0, 0.8, 55),
    // GreenWool
    uniform("green_wool", true, false, 0, 0.8, 56),
    // YellowWool
    uniform("yellow_wool", true, false, 0, 0.8, 57),
    // WhiteWool
    uniform("white_wool", true, false, 0, 0.8, 58),
    // BlackWool
    uniform("black_wool", true, false, 0, 0.8, 59),
    // Cactus
    uniform("cactus", true, true, 0, 0.4, 60),
    // SugarCane
    uniform("sugar_cane", false, true, 0, 0.0, 61),
    // Pumpkin: top=62, bottom=62, sides=63
    props("pumpkin", true, false, 0, 1.0, [62, 62, 63, 63, 63, 63]),
    // Melon: top=64, bottom=64, sides=65
    props("melon", true, false, 0, 1.0, [64, 64, 65, 65, 65, 65]),
    // TNT: top=66, bottom=67, sides=68
    props("tnt", true, false, 0, 0.0, [66, 67, 68, 68, 68, 68]),
    // Bookshelf: top=12(planks), bottom=12, sides=69
    props("bookshelf", true, false, 0, 1.5, [12, 12, 69, 69, 69, 69]),
    // MossyCobblestone
    uniform("mossy_cobblestone", true, false, 0, 2.0, 70),
    // Bricks
    uniform("bricks", true, false, 0, 2.0, 71),
    // StoneBricks
    uniform("stone_bricks", true, false, 0, 1.5, 72),
    // Netherrack
    uniform("netherrack", true, false, 0, 0.4, 73),
    // SoulSand
    uniform("soul_sand", true, false, 0, 0.5, 74),
    // Glowstone
    uniform("glowstone", true, false, 15, 0.3, 75),
    // EndStone
    uniform("end_stone", true, false, 0, 3.0, 76),
    // Mycelium: top=77, bottom=2(dirt), sides=78
    props("mycelium", true, false, 0, 0.6, [77, 2, 78, 78, 78, 78]),
    // Podzol: top=79, bottom=2(dirt), sides=80
    props("podzol", true, false, 0, 0.5, [79, 2, 80, 80, 80, 80]),
    // RedMushroom
    uniform("red_mushroom", false, true, 0, 0.0, 81),
    // BrownMushroom
    uniform("brown_mushroom", false, true, 0, 0.0, 82),
    // TallGrass
    uniform("tall_grass", false, true, 0, 0.0, 83),
    // Dandelion
    uniform("dandelion", false, true, 0, 0.0, 84),
    // Poppy
    uniform("poppy", false, true, 0, 0.0, 85),
    // --- Redstone block types ---
    // RedstoneDust (non-solid, transparent, flat on ground)
    uniform("redstone_dust", false, true, 0, 0.0, 86),
    // RedstoneTorch (non-solid, transparent, emits light)
    uniform("redstone_torch", false, true, 7, 0.0, 87),
    // Lever (non-solid, transparent)
    uniform("lever", false, true, 0, 0.5, 88),
    // StoneButton (non-solid, transparent)
    uniform("stone_button", false, true, 0, 0.5, 89),
    // Repeater (non-solid, transparent, flat component)
    uniform("repeater", false, true, 0, 0.0, 90),
    // Comparator (non-solid, transparent, flat component)
    uniform("comparator", false, true, 0, 0.0, 91),
    // Piston (solid, opaque)
    props("piston", true, false, 0, 1.5, [92, 1, 93, 93, 93, 93]),
    // StickyPiston (solid, opaque)
    props(
        "sticky_piston",
        true,
        false,
        0,
        1.5,
        [94, 1, 93, 93, 93, 93],
    ),
    // Observer (solid, opaque)
    uniform("observer", true, false, 0, 3.5, 95),
    // Hopper (solid, opaque)
    uniform("hopper", true, false, 0, 3.0, 96),
    // Dispenser: top=1(stone), bottom=1, front=97, sides=13(cobblestone)
    props("dispenser", true, false, 0, 3.5, [1, 1, 97, 13, 13, 13]),
    // Dropper: top=1(stone), bottom=1, front=98, sides=13(cobblestone)
    props("dropper", true, false, 0, 3.5, [1, 1, 98, 13, 13, 13]),
    // NoteBlock (solid, opaque)
    uniform("note_block", true, false, 0, 0.8, 99),
    // RedstoneLamp (solid, opaque, no light by default — powered state tracked separately)
    uniform("redstone_lamp", true, false, 0, 0.3, 100),
    // --- Farming block types ---
    // Farmland: top=101, bottom=2(dirt), sides=2(dirt)
    props("farmland", true, false, 0, 0.6, [101, 2, 2, 2, 2, 2]),
    // WheatCrop (non-solid, transparent, plant)
    uniform("wheat_crop", false, true, 0, 0.0, 102),
    // CarrotCrop (non-solid, transparent, plant)
    uniform("carrot_crop", false, true, 0, 0.0, 103),
    // PotatoCrop (non-solid, transparent, plant)
    uniform("potato_crop", false, true, 0, 0.0, 104),
    // BeetrootCrop (non-solid, transparent, plant)
    uniform("beetroot_crop", false, true, 0, 0.0, 105),
    // MelonStem (non-solid, transparent, plant)
    uniform("melon_stem", false, true, 0, 0.0, 106),
    // PumpkinStem (non-solid, transparent, plant)
    uniform("pumpkin_stem", false, true, 0, 0.0, 107),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_matches_enum_variants() {
        // RedstoneLamp is the last non-farming variant with value 83,
        // plus 7 farming block types: Farmland..PumpkinStem => COUNT = 91
        assert_eq!(BlockId::COUNT, 91);
        // Verify the last variant can be constructed from raw
        assert_eq!(BlockId::from_raw(90), Some(BlockId::PumpkinStem));
        // One past the end should return None
        assert_eq!(BlockId::from_raw(91), None);
    }

    #[test]
    fn all_properties_accessible() {
        for id in 0..BlockId::COUNT as u16 {
            let block = BlockId::from_raw(id).unwrap();
            let props = block.properties();
            assert!(!props.name.is_empty(), "Block {id} has empty name");
        }
    }

    #[test]
    fn air_is_transparent_and_not_solid() {
        assert!(BlockId::Air.is_transparent());
        assert!(!BlockId::Air.is_solid());
        assert!(BlockId::Air.is_air());
    }

    #[test]
    fn stone_is_solid_and_opaque() {
        assert!(BlockId::Stone.is_solid());
        assert!(!BlockId::Stone.is_transparent());
    }

    #[test]
    fn new_leaves_are_transparent() {
        let leaves = [
            BlockId::BirchLeaves,
            BlockId::SpruceLeaves,
            BlockId::JungleLeaves,
            BlockId::DarkOakLeaves,
        ];
        for leaf in leaves {
            assert!(leaf.is_transparent(), "{:?} should be transparent", leaf);
        }
    }

    #[test]
    fn glowstone_emits_light() {
        let props = BlockId::Glowstone.properties();
        assert_eq!(props.light_emission, 15);
    }

    #[test]
    fn log_blocks_have_different_top_and_side_textures() {
        let logs = [
            BlockId::BirchLog,
            BlockId::SpruceLog,
            BlockId::JungleLog,
            BlockId::DarkOakLog,
        ];
        for log in logs {
            let props = log.properties();
            // top and bottom should be the same
            assert_eq!(props.tex_indices[0], props.tex_indices[1]);
            // top should differ from sides
            assert_ne!(
                props.tex_indices[0], props.tex_indices[2],
                "{:?} top and side textures should differ",
                log
            );
        }
    }

    #[test]
    fn non_solid_blocks() {
        let non_solid = [
            BlockId::Air,
            BlockId::Water,
            BlockId::Torch,
            BlockId::SugarCane,
            BlockId::TallGrass,
            BlockId::Dandelion,
            BlockId::Poppy,
            BlockId::RedMushroom,
            BlockId::BrownMushroom,
            BlockId::Snow,
            BlockId::RedstoneDust,
            BlockId::RedstoneTorch,
            BlockId::Lever,
            BlockId::StoneButton,
            BlockId::Repeater,
            BlockId::Comparator,
        ];
        for block in non_solid {
            assert!(!block.is_solid(), "{:?} should not be solid", block);
        }
    }

    #[test]
    fn obsidian_has_high_hardness() {
        let props = BlockId::Obsidian.properties();
        assert_eq!(props.hardness, 50.0);
    }

    #[test]
    fn registry_get_matches_properties() {
        let block = BlockId::Bricks;
        assert_eq!(BlockRegistry::get(block).name, block.properties().name,);
    }

    #[test]
    fn redstone_torch_emits_light() {
        let props = BlockId::RedstoneTorch.properties();
        assert_eq!(props.light_emission, 7);
    }

    #[test]
    fn redstone_dust_is_non_solid_and_transparent() {
        assert!(!BlockId::RedstoneDust.is_solid());
        assert!(BlockId::RedstoneDust.is_transparent());
    }

    #[test]
    fn piston_blocks_are_solid() {
        assert!(BlockId::Piston.is_solid());
        assert!(BlockId::StickyPiston.is_solid());
        assert!(!BlockId::Piston.is_transparent());
        assert!(!BlockId::StickyPiston.is_transparent());
    }

    #[test]
    fn redstone_lamp_is_solid_and_opaque() {
        assert!(BlockId::RedstoneLamp.is_solid());
        assert!(!BlockId::RedstoneLamp.is_transparent());
    }

    #[test]
    fn hopper_and_dispenser_are_solid() {
        assert!(BlockId::Hopper.is_solid());
        assert!(BlockId::Dispenser.is_solid());
        assert!(BlockId::Dropper.is_solid());
    }

    #[test]
    fn redstone_components_transparent() {
        let transparent = [
            BlockId::RedstoneDust,
            BlockId::RedstoneTorch,
            BlockId::Lever,
            BlockId::StoneButton,
            BlockId::Repeater,
            BlockId::Comparator,
        ];
        for block in transparent {
            assert!(block.is_transparent(), "{:?} should be transparent", block);
        }
    }
}
