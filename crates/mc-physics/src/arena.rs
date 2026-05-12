//! Memory arena allocator for batch physics allocations.
//!
//! Provides a bump allocator that returns offsets (not raw pointers) into
//! contiguous byte chunks. Call [`Arena::reset`] between frames to reclaim
//! all memory without per-object deallocation.

/// A simple bump-allocator arena that hands out byte offsets.
pub struct Arena {
    chunks: Vec<Vec<u8>>,
    current_offset: usize,
    chunk_size: usize,
}

impl Arena {
    /// Create a new arena with the given chunk size in bytes.
    pub fn new(chunk_size: usize) -> Self {
        let initial_chunk = vec![0u8; chunk_size];
        Self {
            chunks: vec![initial_chunk],
            current_offset: 0,
            chunk_size,
        }
    }

    /// Allocate `size` bytes with the given alignment. Returns the global
    /// offset (chunk_index * chunk_size + offset_within_chunk).
    ///
    /// # Panics
    ///
    /// Panics if `align` is zero or not a power of two.
    pub fn alloc(&mut self, size: usize, align: usize) -> usize {
        assert!(align > 0 && align.is_power_of_two(), "alignment must be a non-zero power of two");

        let aligned_offset = align_up(self.current_offset, align);

        if aligned_offset + size > self.chunk_size {
            // Current chunk can't fit; allocate a new one.
            self.chunks.push(vec![0u8; self.chunk_size]);
            self.current_offset = 0;
            let chunk_index = self.chunks.len() - 1;
            // Offset 0 is always aligned for any power-of-two alignment.
            self.current_offset = size;
            return chunk_index * self.chunk_size;
        }

        let global_offset = (self.chunks.len() - 1) * self.chunk_size + aligned_offset;
        self.current_offset = aligned_offset + size;
        global_offset
    }

    /// Reset the arena, reusing existing chunks without deallocating.
    pub fn reset(&mut self) {
        self.chunks.truncate(1);
        self.current_offset = 0;
    }

    /// Total bytes currently in use (including alignment padding).
    pub fn bytes_used(&self) -> usize {
        if self.chunks.is_empty() {
            return 0;
        }
        (self.chunks.len() - 1) * self.chunk_size + self.current_offset
    }

    /// Total bytes allocated but not yet used (remaining in current chunk).
    pub fn bytes_wasted(&self) -> usize {
        if self.chunks.is_empty() {
            return 0;
        }
        self.chunk_size - self.current_offset
    }
}

/// Round `offset` up to the next multiple of `align`.
fn align_up(offset: usize, align: usize) -> usize {
    (offset + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_alloc_returns_zero_offset() {
        let mut arena = Arena::new(1024);
        let offset = arena.alloc(16, 1);
        assert_eq!(offset, 0);
    }

    #[test]
    fn sequential_allocs_are_contiguous() {
        let mut arena = Arena::new(1024);
        let a = arena.alloc(8, 1);
        let b = arena.alloc(8, 1);
        assert_eq!(a, 0);
        assert_eq!(b, 8);
    }

    #[test]
    fn alignment_is_respected() {
        let mut arena = Arena::new(1024);
        arena.alloc(3, 1); // offset now at 3
        let offset = arena.alloc(4, 8); // should align to 8
        assert_eq!(offset, 8);
    }

    #[test]
    fn overflow_allocates_new_chunk() {
        let mut arena = Arena::new(32);
        arena.alloc(30, 1);
        // Only 2 bytes left; next alloc needs a new chunk.
        let offset = arena.alloc(16, 1);
        assert_eq!(offset, 32); // second chunk starts at chunk_size
        assert_eq!(arena.chunks.len(), 2);
    }

    #[test]
    fn reset_reuses_first_chunk() {
        let mut arena = Arena::new(64);
        arena.alloc(32, 1);
        arena.alloc(32, 1);
        // Force a new chunk.
        arena.alloc(16, 1);
        assert_eq!(arena.chunks.len(), 2);

        arena.reset();
        assert_eq!(arena.chunks.len(), 1);
        assert_eq!(arena.current_offset, 0);

        let offset = arena.alloc(8, 1);
        assert_eq!(offset, 0);
    }

    #[test]
    fn bytes_used_tracks_usage() {
        let mut arena = Arena::new(64);
        assert_eq!(arena.bytes_used(), 0);
        arena.alloc(10, 1);
        assert_eq!(arena.bytes_used(), 10);
        arena.alloc(20, 1);
        assert_eq!(arena.bytes_used(), 30);
    }

    #[test]
    fn bytes_wasted_reports_remaining() {
        let mut arena = Arena::new(64);
        arena.alloc(10, 1);
        assert_eq!(arena.bytes_wasted(), 54);
    }

    #[test]
    #[should_panic(expected = "alignment must be a non-zero power of two")]
    fn zero_alignment_panics() {
        let mut arena = Arena::new(64);
        arena.alloc(8, 0);
    }

    #[test]
    #[should_panic(expected = "alignment must be a non-zero power of two")]
    fn non_power_of_two_alignment_panics() {
        let mut arena = Arena::new(64);
        arena.alloc(8, 3);
    }

    #[test]
    fn large_alignment_on_new_chunk() {
        let mut arena = Arena::new(128);
        arena.alloc(120, 1);
        // 8 bytes left, need 16 with align 16 -> new chunk
        let offset = arena.alloc(16, 16);
        assert_eq!(offset, 128);
        assert_eq!(offset % 16, 0);
    }
}
