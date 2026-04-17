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
}

impl BlockId {
    pub const COUNT: usize = 21;

    pub fn from_raw(id: u16) -> Option<Self> {
        if (id as usize) < Self::COUNT {
            Some(unsafe { std::mem::transmute(id) })
        } else {
            None
        }
    }

    pub fn is_air(self) -> bool {
        matches!(self, BlockId::Air)
    }

    pub fn is_solid(self) -> bool {
        !matches!(self, BlockId::Air | BlockId::Water | BlockId::Torch)
    }

    pub fn is_transparent(self) -> bool {
        matches!(
            self,
            BlockId::Air | BlockId::Water | BlockId::Glass | BlockId::Torch | BlockId::OakLeaves
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
    props("crafting_table", true, false, 0, 2.5, [20, 12, 21, 21, 21, 21]),
    // Furnace: top=1(stone), bottom=1, front=22, sides=23
    props("furnace", true, false, 0, 3.5, [1, 1, 22, 23, 23, 23]),
    // Chest: top=24, bottom=24, front=25, sides=26
    props("chest", true, false, 0, 2.5, [24, 24, 25, 26, 26, 26]),
];
