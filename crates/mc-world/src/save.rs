use std::fs;
use std::path::Path;

use mc_core::block::BlockId;
use mc_core::pos::ChunkPos;
use serde::{Deserialize, Serialize};

use crate::chunk::{Chunk, SECTION_VOLUME, Section};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Serialize(String),
    Deserialize(String),
}

impl From<std::io::Error> for SaveError {
    fn from(err: std::io::Error) -> Self {
        SaveError::Io(err)
    }
}

impl From<Box<bincode::ErrorKind>> for SaveError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        // Bincode uses the same error type for both serialization and
        // deserialization, so we inspect the kind to pick the right variant.
        match *err {
            bincode::ErrorKind::Io(io_err) => SaveError::Io(io_err),
            _ => {
                // Default to Serialize; call-sites that deserialize should map
                // explicitly if the distinction matters. In practice, the `?`
                // operator at each call-site already makes intent clear.
                SaveError::Serialize(err.to_string())
            }
        }
    }
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(err) => write!(f, "I/O error: {err}"),
            SaveError::Serialize(msg) => write!(f, "serialization error: {msg}"),
            SaveError::Deserialize(msg) => write!(f, "deserialization error: {msg}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaveError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Save data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSave {
    pub seed: u64,
    pub player_pos: [f32; 3],
    pub player_yaw: f32,
    pub player_pitch: f32,
    pub time_of_day: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSave {
    pub pos: (i32, i32),
    pub sections: Vec<SectionSave>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSave {
    pub index: u8,
    pub blocks: Vec<u16>,
}

// ---------------------------------------------------------------------------
// World-level save / load
// ---------------------------------------------------------------------------

pub fn save_world(path: &Path, save: &WorldSave) -> Result<(), SaveError> {
    let bytes = bincode::serialize(save)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

pub fn load_world(path: &Path) -> Result<WorldSave, SaveError> {
    let bytes = fs::read(path)?;
    bincode::deserialize(&bytes).map_err(|e| SaveError::Deserialize(e.to_string()))
}

// ---------------------------------------------------------------------------
// Chunk-level save / load
// ---------------------------------------------------------------------------

pub fn save_chunk(dir: &Path, chunk: &Chunk, pos: ChunkPos) -> Result<(), SaveError> {
    let save_data = chunk_to_save(chunk, pos);
    let bytes = bincode::serialize(&save_data)?;
    fs::create_dir_all(dir)?;
    let file_path = dir.join(format!("r.{}.{}.bin", pos.x, pos.z));
    fs::write(file_path, bytes)?;
    Ok(())
}

pub fn load_chunk(dir: &Path, pos: ChunkPos) -> Result<Option<Chunk>, SaveError> {
    let file_path = dir.join(format!("r.{}.{}.bin", pos.x, pos.z));
    if !file_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&file_path)?;
    let save_data: ChunkSave =
        bincode::deserialize(&bytes).map_err(|e| SaveError::Deserialize(e.to_string()))?;
    Ok(Some(save_to_chunk(&save_data)))
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

pub fn chunk_to_save(chunk: &Chunk, pos: ChunkPos) -> ChunkSave {
    let sections: Vec<SectionSave> = chunk
        .sections
        .iter()
        .enumerate()
        .filter(|(_, section)| !section.is_empty())
        .map(|(i, section)| SectionSave {
            index: i as u8,
            blocks: section.blocks().iter().map(|b| *b as u16).collect(),
        })
        .collect();

    ChunkSave {
        pos: (pos.x, pos.z),
        sections,
    }
}

pub fn save_to_chunk(save: &ChunkSave) -> Chunk {
    let mut chunk = Chunk::new();
    for section_save in &save.sections {
        let idx = section_save.index as usize;
        if idx < chunk.sections.len() {
            let mut section = Section::new();
            for (i, &raw_id) in section_save.blocks.iter().enumerate() {
                if i >= SECTION_VOLUME {
                    break;
                }
                let block = BlockId::from_raw(raw_id).unwrap_or(BlockId::Air);
                let x = i % 16;
                let z = (i / 16) % 16;
                let y = i / (16 * 16);
                section.set(x, y, z, block);
            }
            chunk.sections[idx] = section;
        }
    }
    chunk
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::pos::WORLD_BOTTOM;

    #[test]
    fn round_trip_world_save() {
        let dir = std::env::temp_dir().join("mcrust_test_world_save");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("world.dat");

        let original = WorldSave {
            seed: 12345,
            player_pos: [1.0, 64.0, -3.5],
            player_yaw: 90.0,
            player_pitch: -15.0,
            time_of_day: 6000.0,
        };

        save_world(&path, &original).unwrap();
        let loaded = load_world(&path).unwrap();

        assert_eq!(loaded.seed, original.seed);
        assert_eq!(loaded.player_pos, original.player_pos);
        assert_eq!(loaded.player_yaw, original.player_yaw);
        assert_eq!(loaded.player_pitch, original.player_pitch);
        assert_eq!(loaded.time_of_day, original.time_of_day);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn round_trip_chunk_preserves_blocks() {
        let dir = std::env::temp_dir().join("mcrust_test_chunk_rt");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let pos = ChunkPos::new(3, -7);
        let mut chunk = Chunk::new();

        // Place specific blocks at known positions.
        chunk.set_block(0, WORLD_BOTTOM, 0, BlockId::Bedrock);
        chunk.set_block(5, WORLD_BOTTOM + 16, 5, BlockId::Stone);
        chunk.set_block(15, WORLD_BOTTOM + 64, 15, BlockId::DiamondOre);

        save_chunk(&dir, &chunk, pos).unwrap();
        let loaded = load_chunk(&dir, pos)
            .unwrap()
            .expect("chunk file should exist");

        assert_eq!(loaded.get_block(0, WORLD_BOTTOM, 0), BlockId::Bedrock);
        assert_eq!(loaded.get_block(5, WORLD_BOTTOM + 16, 5), BlockId::Stone);
        assert_eq!(
            loaded.get_block(15, WORLD_BOTTOM + 64, 15),
            BlockId::DiamondOre
        );
        // Unset block should be air.
        assert_eq!(loaded.get_block(7, WORLD_BOTTOM + 32, 7), BlockId::Air);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_sections_are_skipped() {
        let pos = ChunkPos::new(0, 0);
        let mut chunk = Chunk::new();

        // Only place a block in section 0.
        chunk.set_block(0, WORLD_BOTTOM, 0, BlockId::Stone);

        let save_data = chunk_to_save(&chunk, pos);

        // Only the non-empty section should be serialized.
        assert_eq!(save_data.sections.len(), 1);
        assert_eq!(save_data.sections[0].index, 0);
    }

    #[test]
    fn load_missing_chunk_returns_none() {
        let dir = std::env::temp_dir().join("mcrust_test_missing_chunk");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = load_chunk(&dir, ChunkPos::new(99, 99)).unwrap();
        assert!(result.is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn chunk_to_save_and_back_preserves_all_blocks() {
        let pos = ChunkPos::new(1, 2);
        let mut chunk = Chunk::new();

        // Fill section 0 with a pattern.
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, WORLD_BOTTOM, z, BlockId::Cobblestone);
            }
        }
        // Place a different block in section 2.
        chunk.set_block(8, WORLD_BOTTOM + 32, 8, BlockId::GoldOre);

        let save_data = chunk_to_save(&chunk, pos);
        let restored = save_to_chunk(&save_data);

        // Verify section 0 pattern.
        for x in 0..16 {
            for z in 0..16 {
                assert_eq!(
                    restored.get_block(x, WORLD_BOTTOM, z),
                    BlockId::Cobblestone,
                    "mismatch at ({x}, {}, {z})",
                    WORLD_BOTTOM
                );
            }
        }

        // Verify section 2 block.
        assert_eq!(
            restored.get_block(8, WORLD_BOTTOM + 32, 8),
            BlockId::GoldOre
        );
    }
}
