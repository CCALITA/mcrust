//! Draw call batching: sorts and merges draw commands by texture to reduce GPU draw calls.

/// A single draw command for the GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawCommand {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub instance_count: u32,
    pub texture_id: u16,
}

/// Collects, sorts, and merges draw commands to minimize draw call count.
#[derive(Debug)]
pub struct DrawBatcher {
    commands: Vec<DrawCommand>,
}

impl DrawBatcher {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add_mesh(&mut self, vertex_offset: u32, index_offset: u32, index_count: u32, texture_id: u16) {
        self.commands.push(DrawCommand {
            vertex_offset,
            index_offset,
            index_count,
            instance_count: 1,
            texture_id,
        });
    }

    pub fn sort_by_texture(&mut self) {
        self.commands.sort_by_key(|cmd| cmd.texture_id);
    }

    /// Merge adjacent commands that share the same texture and have contiguous index ranges.
    /// Returns the number of merges performed.
    pub fn merge_adjacent(&mut self) -> usize {
        if self.commands.len() <= 1 {
            return 0;
        }

        let mut merged = Vec::with_capacity(self.commands.len());
        let mut merges = 0usize;

        let mut current = self.commands[0].clone();

        for next in &self.commands[1..] {
            if next.texture_id == current.texture_id
                && next.vertex_offset == current.vertex_offset
                && next.index_offset == current.index_offset + current.index_count
            {
                current.index_count += next.index_count;
                current.instance_count += next.instance_count;
                merges += 1;
            } else {
                merged.push(current);
                current = next.clone();
            }
        }
        merged.push(current);

        self.commands = merged;
        merges
    }

    pub fn total_draw_calls(&self) -> usize {
        self.commands.len()
    }

    pub fn total_triangles(&self) -> u64 {
        self.commands.iter().map(|cmd| u64::from(cmd.index_count) / 3).sum()
    }

    /// Returns a slice of the current draw commands.
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}

impl Default for DrawBatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_batcher_is_empty() {
        let batcher = DrawBatcher::new();
        assert_eq!(batcher.total_draw_calls(), 0);
        assert_eq!(batcher.total_triangles(), 0);
    }

    #[test]
    fn add_mesh_creates_command() {
        let mut batcher = DrawBatcher::new();
        batcher.add_mesh(0, 0, 6, 1);
        assert_eq!(batcher.total_draw_calls(), 1);
        assert_eq!(batcher.total_triangles(), 2);
        assert_eq!(batcher.commands()[0].instance_count, 1);
    }

    #[test]
    fn sort_by_texture_orders_commands() {
        let mut batcher = DrawBatcher::new();
        batcher.add_mesh(0, 0, 6, 3);
        batcher.add_mesh(0, 6, 6, 1);
        batcher.add_mesh(0, 12, 6, 2);
        batcher.sort_by_texture();
        let ids: Vec<u16> = batcher.commands().iter().map(|c| c.texture_id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn merge_adjacent_same_texture_contiguous() {
        let mut batcher = DrawBatcher::new();
        batcher.add_mesh(0, 0, 6, 1);
        batcher.add_mesh(0, 6, 6, 1);
        batcher.add_mesh(0, 12, 6, 1);
        let merges = batcher.merge_adjacent();
        assert_eq!(merges, 2);
        assert_eq!(batcher.total_draw_calls(), 1);
        assert_eq!(batcher.commands()[0].index_count, 18);
        assert_eq!(batcher.total_triangles(), 6);
    }

    #[test]
    fn merge_does_not_merge_different_textures() {
        let mut batcher = DrawBatcher::new();
        batcher.add_mesh(0, 0, 6, 1);
        batcher.add_mesh(0, 6, 6, 2);
        let merges = batcher.merge_adjacent();
        assert_eq!(merges, 0);
        assert_eq!(batcher.total_draw_calls(), 2);
    }

    #[test]
    fn merge_does_not_merge_non_contiguous() {
        let mut batcher = DrawBatcher::new();
        batcher.add_mesh(0, 0, 6, 1);
        batcher.add_mesh(0, 100, 6, 1);
        let merges = batcher.merge_adjacent();
        assert_eq!(merges, 0);
        assert_eq!(batcher.total_draw_calls(), 2);
    }

    #[test]
    fn merge_empty_and_single() {
        let mut empty = DrawBatcher::new();
        assert_eq!(empty.merge_adjacent(), 0);

        let mut single = DrawBatcher::new();
        single.add_mesh(0, 0, 6, 1);
        assert_eq!(single.merge_adjacent(), 0);
        assert_eq!(single.total_draw_calls(), 1);
    }

    #[test]
    fn sort_then_merge_pipeline() {
        let mut batcher = DrawBatcher::new();
        batcher.add_mesh(0, 0, 6, 2);
        batcher.add_mesh(0, 0, 6, 1);
        batcher.add_mesh(0, 6, 6, 1);
        batcher.add_mesh(0, 6, 6, 2);

        batcher.sort_by_texture();
        let merges = batcher.merge_adjacent();
        assert_eq!(merges, 2);
        assert_eq!(batcher.total_draw_calls(), 2);
    }

    #[test]
    fn total_triangles_multiple_commands() {
        let mut batcher = DrawBatcher::new();
        batcher.add_mesh(0, 0, 9, 1);   // 3 triangles
        batcher.add_mesh(0, 9, 12, 2);  // 4 triangles
        assert_eq!(batcher.total_triangles(), 7);
    }
}
