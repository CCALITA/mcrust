use glam::IVec3;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: i32 = 16;
pub const SECTION_HEIGHT: i32 = 16;
pub const WORLD_BOTTOM: i32 = -64;
pub const WORLD_TOP: i32 = 320;
pub const SECTIONS_PER_CHUNK: usize = ((WORLD_TOP - WORLD_BOTTOM) / SECTION_HEIGHT) as usize;

/// Position of a chunk in the world (XZ only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_block(bx: i32, bz: i32) -> Self {
        Self {
            x: bx.div_euclid(CHUNK_SIZE),
            z: bz.div_euclid(CHUNK_SIZE),
        }
    }

    pub fn block_origin(self) -> IVec3 {
        IVec3::new(self.x * CHUNK_SIZE, WORLD_BOTTOM, self.z * CHUNK_SIZE)
    }
}

/// Absolute block position in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn chunk_pos(self) -> ChunkPos {
        ChunkPos::from_block(self.x, self.z)
    }

    /// Returns the local position within a chunk (0..15, 0..383, 0..15).
    pub fn local(self) -> (usize, usize, usize) {
        (
            self.x.rem_euclid(CHUNK_SIZE) as usize,
            (self.y - WORLD_BOTTOM) as usize,
            self.z.rem_euclid(CHUNK_SIZE) as usize,
        )
    }

    /// Section index (0..SECTIONS_PER_CHUNK) for this block's Y coordinate.
    pub fn section_index(self) -> usize {
        ((self.y - WORLD_BOTTOM) / SECTION_HEIGHT) as usize
    }

    /// Local Y within the section (0..15).
    pub fn section_local_y(self) -> usize {
        (self.y - WORLD_BOTTOM).rem_euclid(SECTION_HEIGHT) as usize
    }

    pub fn to_vec3(self) -> glam::Vec3 {
        glam::Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl From<IVec3> for BlockPos {
    fn from(v: IVec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<BlockPos> for IVec3 {
    fn from(p: BlockPos) -> Self {
        IVec3::new(p.x, p.y, p.z)
    }
}
