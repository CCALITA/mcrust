use mc_core::block::BlockId;
use mc_core::pos::{CHUNK_SIZE, SECTION_HEIGHT, SECTIONS_PER_CHUNK, WORLD_BOTTOM};

pub const SECTION_VOLUME: usize = (CHUNK_SIZE * SECTION_HEIGHT * CHUNK_SIZE) as usize;

#[derive(Clone)]
pub struct Section {
    blocks: Vec<BlockId>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            blocks: vec![BlockId::Air; SECTION_VOLUME],
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        y * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) + z * (CHUNK_SIZE as usize) + x
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        self.blocks[Self::index(x, y, z)] = block;
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|b| b.is_air())
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Chunk {
    pub sections: Vec<Section>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            sections: (0..SECTIONS_PER_CHUNK).map(|_| Section::new()).collect(),
        }
    }

    pub fn get_block(&self, x: usize, world_y: i32, z: usize) -> BlockId {
        let sy = ((world_y - WORLD_BOTTOM) / SECTION_HEIGHT) as usize;
        let local_y = (world_y - WORLD_BOTTOM).rem_euclid(SECTION_HEIGHT) as usize;
        if sy < self.sections.len() {
            self.sections[sy].get(x, local_y, z)
        } else {
            BlockId::Air
        }
    }

    pub fn set_block(&mut self, x: usize, world_y: i32, z: usize, block: BlockId) {
        let sy = ((world_y - WORLD_BOTTOM) / SECTION_HEIGHT) as usize;
        let local_y = (world_y - WORLD_BOTTOM).rem_euclid(SECTION_HEIGHT) as usize;
        if sy < self.sections.len() {
            self.sections[sy].set(x, local_y, z, block);
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
