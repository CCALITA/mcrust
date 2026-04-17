use std::collections::{HashMap, HashSet};

use mc_core::block::BlockId;
use mc_core::pos::{BlockPos, CHUNK_SIZE, ChunkPos, WORLD_BOTTOM, WORLD_TOP};

use crate::caves::CaveCarver;
use crate::chunk::Chunk;
use crate::end::EndTerrainGen;
use crate::nether::{DimensionId, NetherTerrainGen};
use crate::noise_terrain::NoiseTerrainGen;
use crate::ores::OreGenerator;
use crate::trees;

pub struct ChunkManager {
    chunks: HashMap<ChunkPos, Chunk>,
    render_distance: i32,
    dirty_chunks: HashSet<ChunkPos>,
    dimension: DimensionId,
    terrain_gen: NoiseTerrainGen,
    cave_carver: CaveCarver,
    ore_gen: OreGenerator,
    nether_gen: NetherTerrainGen,
    end_gen: EndTerrainGen,
    seed: u64,
}

impl ChunkManager {
    pub fn new(render_distance: i32) -> Self {
        let seed = 42;
        Self {
            chunks: HashMap::new(),
            render_distance,
            dirty_chunks: HashSet::new(),
            dimension: DimensionId::Overworld,
            terrain_gen: NoiseTerrainGen::new(seed),
            cave_carver: CaveCarver::new(seed),
            ore_gen: OreGenerator::new(seed),
            nether_gen: NetherTerrainGen::new(seed),
            end_gen: EndTerrainGen::new(seed),
            seed,
        }
    }

    /// Returns the current dimension.
    pub fn current_dimension(&self) -> DimensionId {
        self.dimension
    }

    /// Switches to the given dimension, clearing all loaded chunks and dirty
    /// state so the world is regenerated on the next update.
    pub fn switch_dimension(&mut self, dim: DimensionId) {
        self.chunks.clear();
        self.dirty_chunks.clear();
        self.dimension = dim;
    }

    /// Generates a chunk using the pipeline for the current dimension.
    fn generate_chunk(&mut self, pos: ChunkPos) -> Chunk {
        match self.dimension {
            DimensionId::Overworld => {
                let mut chunk = self.terrain_gen.generate(pos.x, pos.z);
                self.cave_carver.carve(&mut chunk, pos.x, pos.z);
                self.ore_gen.generate_ores(&mut chunk, pos.x, pos.z);
                trees::place_trees(&mut chunk, pos.x, pos.z, self.seed);
                trees::place_vegetation(&mut chunk, pos.x, pos.z, self.seed);
                chunk
            }
            DimensionId::Nether => self.nether_gen.generate(pos.x, pos.z),
            DimensionId::End => self.end_gen.generate(pos.x, pos.z),
        }
    }

    /// Load chunks within render distance of the player, unload those beyond
    /// render_distance + 2.
    pub fn update(&mut self, player_chunk: ChunkPos) {
        let rd = self.render_distance;
        let unload_distance = rd + 2;

        // Load missing chunks within render distance.
        for dx in -rd..=rd {
            for dz in -rd..=rd {
                let pos = ChunkPos::new(player_chunk.x + dx, player_chunk.z + dz);
                if !self.chunks.contains_key(&pos) {
                    let chunk = self.generate_chunk(pos);
                    self.chunks.insert(pos, chunk);
                    self.dirty_chunks.insert(pos);
                }
            }
        }

        // Unload chunks beyond unload_distance.
        let to_unload: Vec<ChunkPos> = self
            .chunks
            .keys()
            .filter(|pos| {
                let dx = (pos.x - player_chunk.x).abs();
                let dz = (pos.z - player_chunk.z).abs();
                dx > unload_distance || dz > unload_distance
            })
            .copied()
            .collect();

        for pos in to_unload {
            self.chunks.remove(&pos);
            self.dirty_chunks.remove(&pos);
        }
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// Look up a block across chunks. Returns `Air` if the chunk is not loaded
    /// or the position is out of vertical bounds.
    pub fn get_block(&self, pos: BlockPos) -> BlockId {
        if pos.y < WORLD_BOTTOM || pos.y >= WORLD_TOP {
            return BlockId::Air;
        }
        let chunk_pos = pos.chunk_pos();
        match self.chunks.get(&chunk_pos) {
            Some(chunk) => {
                let (lx, _ly, lz) = pos.local();
                chunk.get_block(lx, pos.y, lz)
            }
            None => BlockId::Air,
        }
    }

    /// Set a block and mark the containing chunk as dirty.
    pub fn set_block(&mut self, pos: BlockPos, block: BlockId) {
        if pos.y < WORLD_BOTTOM || pos.y >= WORLD_TOP {
            return;
        }
        let chunk_pos = pos.chunk_pos();
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            let (lx, _ly, lz) = pos.local();
            chunk.set_block(lx, pos.y, lz, block);
            self.dirty_chunks.insert(chunk_pos);

            // If the block is on a chunk boundary, also mark the neighbor dirty
            // so its mesh is rebuilt with the correct face culling.
            let local_x = pos.x.rem_euclid(CHUNK_SIZE);
            let local_z = pos.z.rem_euclid(CHUNK_SIZE);
            if local_x == 0 {
                let neighbor = ChunkPos::new(chunk_pos.x - 1, chunk_pos.z);
                if self.chunks.contains_key(&neighbor) {
                    self.dirty_chunks.insert(neighbor);
                }
            }
            if local_x == CHUNK_SIZE - 1 {
                let neighbor = ChunkPos::new(chunk_pos.x + 1, chunk_pos.z);
                if self.chunks.contains_key(&neighbor) {
                    self.dirty_chunks.insert(neighbor);
                }
            }
            if local_z == 0 {
                let neighbor = ChunkPos::new(chunk_pos.x, chunk_pos.z - 1);
                if self.chunks.contains_key(&neighbor) {
                    self.dirty_chunks.insert(neighbor);
                }
            }
            if local_z == CHUNK_SIZE - 1 {
                let neighbor = ChunkPos::new(chunk_pos.x, chunk_pos.z + 1);
                if self.chunks.contains_key(&neighbor) {
                    self.dirty_chunks.insert(neighbor);
                }
            }
        }
    }

    /// Drain and return the set of dirty chunks that need mesh rebuilds.
    pub fn take_dirty(&mut self) -> HashSet<ChunkPos> {
        std::mem::take(&mut self.dirty_chunks)
    }

    pub fn loaded_chunks(&self) -> impl Iterator<Item = (&ChunkPos, &Chunk)> {
        self.chunks.iter()
    }

    /// Convenience for physics: returns whether the block at the given
    /// world coordinates is solid.
    pub fn is_block_solid(&self, x: i32, y: i32, z: i32) -> bool {
        let block = self.get_block(BlockPos::new(x, y, z));
        block.is_solid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_chunks_around_origin() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));

        // Should have (2*2+1)^2 = 25 chunks loaded.
        assert_eq!(mgr.chunks.len(), 25);
        assert!(mgr.get_chunk(ChunkPos::new(0, 0)).is_some());
        assert!(mgr.get_chunk(ChunkPos::new(2, 2)).is_some());
        assert!(mgr.get_chunk(ChunkPos::new(-2, -2)).is_some());
        // Beyond render distance.
        assert!(mgr.get_chunk(ChunkPos::new(3, 0)).is_none());
    }

    #[test]
    fn unloads_distant_chunks() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));
        assert_eq!(mgr.chunks.len(), 25);

        // Move player far away — old chunks should be unloaded.
        mgr.update(ChunkPos::new(100, 100));
        assert!(mgr.get_chunk(ChunkPos::new(0, 0)).is_none());
        assert!(mgr.get_chunk(ChunkPos::new(100, 100)).is_some());
    }

    #[test]
    fn get_block_returns_air_for_unloaded() {
        let mgr = ChunkManager::new(2);
        let block = mgr.get_block(BlockPos::new(0, 64, 0));
        assert_eq!(block, BlockId::Air);
    }

    #[test]
    fn get_block_returns_air_out_of_bounds() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));

        assert_eq!(mgr.get_block(BlockPos::new(0, WORLD_TOP, 0)), BlockId::Air);
        assert_eq!(
            mgr.get_block(BlockPos::new(0, WORLD_BOTTOM - 1, 0)),
            BlockId::Air
        );
    }

    #[test]
    fn get_block_reads_terrain() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));

        // NoiseTerrainGen always places bedrock at y=-64.
        assert_eq!(mgr.get_block(BlockPos::new(0, -64, 0)), BlockId::Bedrock);
        // High above terrain is always air.
        assert_eq!(mgr.get_block(BlockPos::new(0, 200, 0)), BlockId::Air);
    }

    #[test]
    fn set_block_marks_dirty() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));
        let _ = mgr.take_dirty(); // clear initial dirty set

        mgr.set_block(BlockPos::new(5, 70, 5), BlockId::Stone);
        assert_eq!(mgr.get_block(BlockPos::new(5, 70, 5)), BlockId::Stone);

        let dirty = mgr.take_dirty();
        assert!(dirty.contains(&ChunkPos::new(0, 0)));
    }

    #[test]
    fn take_dirty_drains() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));

        let first = mgr.take_dirty();
        assert!(!first.is_empty());

        let second = mgr.take_dirty();
        assert!(second.is_empty());
    }

    #[test]
    fn is_block_solid_checks_terrain() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));

        // Bedrock at bottom is always solid.
        assert!(mgr.is_block_solid(0, -64, 0));
        // High up is always air.
        assert!(!mgr.is_block_solid(0, 200, 0));
    }

    #[test]
    fn loaded_chunks_iterates_all() {
        let mut mgr = ChunkManager::new(1);
        mgr.update(ChunkPos::new(0, 0));

        let count = mgr.loaded_chunks().count();
        assert_eq!(count, 9); // (2*1+1)^2 = 9
    }

    #[test]
    fn defaults_to_overworld() {
        let mgr = ChunkManager::new(2);
        assert_eq!(mgr.current_dimension(), DimensionId::Overworld);
    }

    #[test]
    fn switch_dimension_clears_chunks() {
        let mut mgr = ChunkManager::new(2);
        mgr.update(ChunkPos::new(0, 0));
        assert!(!mgr.chunks.is_empty());

        mgr.switch_dimension(DimensionId::Nether);
        assert!(mgr.chunks.is_empty());
        assert!(mgr.dirty_chunks.is_empty());
        assert_eq!(mgr.current_dimension(), DimensionId::Nether);
    }

    #[test]
    fn nether_generates_netherrack() {
        let mut mgr = ChunkManager::new(1);
        mgr.switch_dimension(DimensionId::Nether);
        mgr.update(ChunkPos::new(0, 0));

        // Nether bedrock at floor (y=0)
        assert_eq!(mgr.get_block(BlockPos::new(0, 0, 0)), BlockId::Bedrock);
        // High above nether ceiling is air
        assert_eq!(mgr.get_block(BlockPos::new(0, 200, 0)), BlockId::Air);
    }

    #[test]
    fn end_generates_end_stone_at_origin() {
        let mut mgr = ChunkManager::new(1);
        mgr.switch_dimension(DimensionId::End);
        mgr.update(ChunkPos::new(0, 0));

        // End main island has EndStone at y=64 near origin
        assert_eq!(mgr.get_block(BlockPos::new(0, 64, 0)), BlockId::EndStone);
    }

    #[test]
    fn switch_back_to_overworld() {
        let mut mgr = ChunkManager::new(1);
        mgr.switch_dimension(DimensionId::Nether);
        mgr.update(ChunkPos::new(0, 0));

        mgr.switch_dimension(DimensionId::Overworld);
        assert_eq!(mgr.current_dimension(), DimensionId::Overworld);
        assert!(mgr.chunks.is_empty());

        mgr.update(ChunkPos::new(0, 0));
        // Overworld bedrock at y=-64
        assert_eq!(mgr.get_block(BlockPos::new(0, -64, 0)), BlockId::Bedrock);
    }
}
