use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Per-section serialization payload including blocks and lighting data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SectionData {
    pub y: i32,
    pub blocks: Vec<u16>,
    pub block_light: Vec<u8>,
    pub sky_light: Vec<u8>,
}

/// Full chunk serialization payload suitable for on-disk storage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkSaveData {
    pub pos: (i32, i32),
    pub sections: Vec<SectionData>,
    pub biomes: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Serialization helpers
// ---------------------------------------------------------------------------

/// Encode a chunk into a compact binary representation via bincode.
pub fn serialize_chunk(data: &ChunkSaveData) -> Vec<u8> {
    // bincode::serialize returns Result; an OOM or internal error is
    // extremely unlikely for well-formed data, but we handle it by
    // returning an empty vec rather than panicking in production.
    bincode::serialize(data).unwrap_or_default()
}

/// Decode a chunk from its binary representation.  Returns `None` on
/// malformed or incompatible input.
pub fn deserialize_chunk(bytes: &[u8]) -> Option<ChunkSaveData> {
    bincode::deserialize(bytes).ok()
}

/// Estimate the in-memory byte size of the serialized chunk without
/// actually performing serialization.  Useful for budget checks and
/// pre-allocation.
pub fn estimate_chunk_size(data: &ChunkSaveData) -> usize {
    // Fixed fields: pos (2 × i32 = 8), biomes length prefix (8),
    // sections length prefix (8).
    let header = 8 + 8 + 8;
    let biomes = data.biomes.len();
    let sections: usize = data
        .sections
        .iter()
        .map(|s| {
            // y (4) + 3 length prefixes (3 × 8 = 24) + payload bytes
            4 + 24 + s.blocks.len() * 2 + s.block_light.len() + s.sky_light.len()
        })
        .sum();
    header + biomes + sections
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_section(y: i32) -> SectionData {
        SectionData {
            y,
            blocks: vec![1; 4096],
            block_light: vec![0; 2048],
            sky_light: vec![15; 2048],
        }
    }

    fn sample_chunk() -> ChunkSaveData {
        ChunkSaveData {
            pos: (3, -7),
            sections: vec![sample_section(0), sample_section(1)],
            biomes: vec![4; 256],
        }
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let original = sample_chunk();
        let bytes = serialize_chunk(&original);
        let restored = deserialize_chunk(&bytes).expect("deserialization should succeed");
        assert_eq!(original, restored);
    }

    #[test]
    fn empty_chunk_roundtrip() {
        let empty = ChunkSaveData {
            pos: (0, 0),
            sections: Vec::new(),
            biomes: Vec::new(),
        };
        let bytes = serialize_chunk(&empty);
        let restored = deserialize_chunk(&bytes).expect("empty chunk should deserialize");
        assert_eq!(empty, restored);
    }

    #[test]
    fn deserialize_invalid_bytes_returns_none() {
        let garbage = vec![0xFF, 0xFE, 0x00, 0x01];
        assert!(deserialize_chunk(&garbage).is_none());
    }

    #[test]
    fn estimate_chunk_size_is_reasonable() {
        let data = sample_chunk();
        let estimated = estimate_chunk_size(&data);
        let actual = serialize_chunk(&data).len();
        // The estimate should be within a small margin of the actual size.
        // bincode overhead is minimal, so we allow a generous 20% tolerance.
        let lower = actual * 80 / 100;
        let upper = actual * 120 / 100;
        assert!(
            estimated >= lower && estimated <= upper,
            "estimate {estimated} should be within 20% of actual {actual}"
        );
    }

    #[test]
    fn estimate_empty_chunk_size() {
        let empty = ChunkSaveData {
            pos: (0, 0),
            sections: Vec::new(),
            biomes: Vec::new(),
        };
        let estimated = estimate_chunk_size(&empty);
        // Should be at least the fixed header cost.
        assert!(estimated > 0, "estimate should be positive even for empty chunk");
        // The actual encoded size should not exceed the estimate by much.
        let actual = serialize_chunk(&empty).len();
        assert!(
            estimated >= actual / 2,
            "estimate {estimated} should be at least half of actual {actual}"
        );
    }
}
