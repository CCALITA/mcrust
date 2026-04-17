use mc_core::block::BlockId;
use mc_core::pos::CHUNK_SIZE;

use crate::chunk::Chunk;

pub struct FlatWorldGen {
    pub sea_level: i32,
}

impl FlatWorldGen {
    pub fn new() -> Self {
        Self { sea_level: 63 }
    }

    pub fn generate(&self, _cx: i32, _cz: i32) -> Chunk {
        let mut chunk = Chunk::new();
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                // Bedrock at y=-64
                chunk.set_block(x, -64, z, BlockId::Bedrock);
                // Stone from -63 to 59
                for y in -63..60 {
                    chunk.set_block(x, y, z, BlockId::Stone);
                }
                // Dirt from 60 to 62
                for y in 60..63 {
                    chunk.set_block(x, y, z, BlockId::Dirt);
                }
                // Grass at 63
                chunk.set_block(x, self.sea_level, z, BlockId::GrassBlock);
            }
        }
        chunk
    }
}

impl Default for FlatWorldGen {
    fn default() -> Self {
        Self::new()
    }
}
