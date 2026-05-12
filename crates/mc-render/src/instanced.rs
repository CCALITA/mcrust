//! Instanced rendering data for batching identical geometry with per-instance transforms.

use bytemuck::{Pod, Zeroable};

/// Maximum number of instances in a single draw call.
pub const MAX_INSTANCES_PER_DRAW: u32 = 4096;

/// Threshold above which instanced rendering should be preferred over individual draw calls.
const INSTANCING_THRESHOLD: usize = 16;

/// Per-instance data uploaded to the GPU for instanced draw calls.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub texture_offset: [f32; 2],
    pub color_tint: [f32; 4],
    // 2 floats padding for 16-byte alignment
    _padding: [f32; 2],
}

/// Creates a batch of [`InstanceData`] from world positions and a block id.
///
/// Each instance gets an identity-scale model matrix translated to the given position.
/// The `block_id` determines the texture offset (row in a texture atlas).
pub fn batch_instances(positions: &[[f32; 3]], block_id: u16) -> Vec<InstanceData> {
    let tex_row = block_id as f32;
    positions
        .iter()
        .map(|pos| {
            let model_matrix = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [pos[0], pos[1], pos[2], 1.0],
            ];
            InstanceData {
                model_matrix,
                texture_offset: [0.0, tex_row],
                color_tint: [1.0, 1.0, 1.0, 1.0],
                _padding: [0.0; 2],
            }
        })
        .collect()
}

/// Returns the byte size required for an instance buffer holding `count` instances.
pub fn instance_buffer_size(count: usize) -> usize {
    count * std::mem::size_of::<InstanceData>()
}

/// Returns `true` when the instance count is large enough to benefit from instanced rendering.
pub fn should_use_instancing(count: usize) -> bool {
    count > INSTANCING_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_data_is_pod() {
        // Compile-time proof that InstanceData is Pod + Zeroable.
        let _zero: InstanceData = bytemuck::Zeroable::zeroed();
    }

    #[test]
    fn batch_instances_empty() {
        let result = batch_instances(&[], 1);
        assert!(result.is_empty());
    }

    #[test]
    fn batch_instances_sets_translation() {
        let positions = [[1.0, 2.0, 3.0]];
        let instances = batch_instances(&positions, 0);
        assert_eq!(instances.len(), 1);
        let m = instances[0].model_matrix;
        assert_eq!(m[3][0], 1.0);
        assert_eq!(m[3][1], 2.0);
        assert_eq!(m[3][2], 3.0);
        assert_eq!(m[3][3], 1.0);
        // Identity upper-left 3x3
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[1][1], 1.0);
        assert_eq!(m[2][2], 1.0);
    }

    #[test]
    fn batch_instances_uses_block_id_for_texture_offset() {
        let positions = [[0.0, 0.0, 0.0]];
        let instances = batch_instances(&positions, 42);
        assert_eq!(instances[0].texture_offset, [0.0, 42.0]);
    }

    #[test]
    fn batch_instances_default_color_tint() {
        let positions = [[0.0, 0.0, 0.0]];
        let instances = batch_instances(&positions, 0);
        assert_eq!(instances[0].color_tint, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn batch_instances_multiple() {
        let positions = [[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let instances = batch_instances(&positions, 5);
        assert_eq!(instances.len(), 3);
        assert_eq!(instances[0].model_matrix[3][0], 1.0);
        assert_eq!(instances[1].model_matrix[3][0], 2.0);
        assert_eq!(instances[2].model_matrix[3][0], 3.0);
    }

    #[test]
    fn instance_buffer_size_zero() {
        assert_eq!(instance_buffer_size(0), 0);
    }

    #[test]
    fn instance_buffer_size_matches_struct() {
        let size = std::mem::size_of::<InstanceData>();
        assert_eq!(instance_buffer_size(1), size);
        assert_eq!(instance_buffer_size(10), size * 10);
    }

    #[test]
    fn instance_data_size_is_aligned() {
        // Should be 96 bytes: 64 (matrix) + 8 (tex) + 16 (color) + 8 (pad)
        assert_eq!(std::mem::size_of::<InstanceData>() % 16, 0);
    }

    #[test]
    fn should_use_instancing_below_threshold() {
        assert!(!should_use_instancing(0));
        assert!(!should_use_instancing(1));
        assert!(!should_use_instancing(16));
    }

    #[test]
    fn should_use_instancing_above_threshold() {
        assert!(should_use_instancing(17));
        assert!(should_use_instancing(100));
        assert!(should_use_instancing(4096));
    }

    #[test]
    fn max_instances_value() {
        assert_eq!(MAX_INSTANCES_PER_DRAW, 4096);
    }
}
