//! Water animation, transparent mesh separation, and water-specific shaders.

use mc_core::block::BlockId;
use mc_core::pos::{CHUNK_SIZE, ChunkPos};
use mc_world::Chunk;

use crate::mesh::Vertex;

/// Tracks animation state for water surface effects.
pub struct WaterAnimation {
    time: f32,
}

impl WaterAnimation {
    pub fn new() -> Self {
        Self { time: 0.0 }
    }

    /// Advance the animation clock by `dt` seconds.
    pub fn advance(&mut self, dt: f32) {
        self.time += dt;
    }

    /// Returns a slow cyclic UV offset `(u_offset, v_offset)` for water surface
    /// animation. The offset oscillates in a figure-eight-like pattern so the
    /// water texture appears to drift.
    pub fn uv_offset(&self) -> (f32, f32) {
        let u = (self.time * 0.03).sin() * 0.02;
        let v = (self.time * 0.02).cos() * 0.015;
        (u, v)
    }

    /// Sine-based wave displacement at world-space `(x, z)`.
    /// Returns a small vertical offset suitable for displacing water-surface
    /// vertices.
    pub fn wave_height(&self, x: f32, z: f32) -> f32 {
        let wave1 = (x * 0.8 + self.time * 1.2).sin();
        let wave2 = (z * 0.6 + self.time * 0.9).cos();
        (wave1 + wave2) * 0.05
    }
}

impl Default for WaterAnimation {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw transparent mesh data (vertices + indices) before GPU upload.
pub struct TransparentMeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Returns `true` for blocks that belong in the transparent render pass.
fn is_transparent_block(block: BlockId) -> bool {
    matches!(
        block,
        BlockId::Water
            | BlockId::Glass
            | BlockId::Ice
            | BlockId::OakLeaves
            | BlockId::BirchLeaves
            | BlockId::SpruceLeaves
            | BlockId::JungleLeaves
            | BlockId::DarkOakLeaves
    )
}

/// Separate already-generated mesh data into opaque and transparent sets.
///
/// Each face occupies 4 consecutive vertices and 6 consecutive indices
/// (two triangles per quad). For every face we recover the block position
/// from the vertex data, look up the block in the chunk, and route the
/// face into the appropriate bucket.
///
/// Returns `(opaque_verts, opaque_indices, transparent_verts, transparent_indices)`.
pub fn separate_transparent_faces(
    vertices: &[Vertex],
    indices: &[u32],
    chunk: &Chunk,
    chunk_pos: ChunkPos,
) -> (Vec<Vertex>, Vec<u32>, Vec<Vertex>, Vec<u32>) {
    let mut opaque_verts: Vec<Vertex> = Vec::new();
    let mut opaque_indices: Vec<u32> = Vec::new();
    let mut transparent_verts: Vec<Vertex> = Vec::new();
    let mut transparent_indices: Vec<u32> = Vec::new();

    let origin = chunk_pos.block_origin();
    let world_x0 = origin.x as f32;
    let world_z0 = origin.z as f32;

    // Each face is 6 indices referencing 4 consecutive vertices.
    let face_count = indices.len() / 6;
    for face_idx in 0..face_count {
        let idx_offset = face_idx * 6;
        let base_vertex = indices[idx_offset] as usize;

        // The four vertices for this quad start at base_vertex.
        let face_verts = &vertices[base_vertex..base_vertex + 4];

        // Recover the block position from the face vertices.
        // We compute the face centroid then nudge it inward along the
        // inverted face normal so it lands inside the block volume.
        // Flooring the result gives the integer block origin.
        let cx = face_verts.iter().map(|v| v.position[0]).sum::<f32>() / 4.0;
        let cy = face_verts.iter().map(|v| v.position[1]).sum::<f32>() / 4.0;
        let cz = face_verts.iter().map(|v| v.position[2]).sum::<f32>() / 4.0;

        let nx = face_verts[0].normal[0];
        let ny = face_verts[0].normal[1];
        let nz = face_verts[0].normal[2];

        // Nudge toward the block interior (away from the face surface).
        let nudge = 0.25;
        let block_x = (cx - nx * nudge).floor();
        let block_y = (cy - ny * nudge).floor();
        let block_z = (cz - nz * nudge).floor();

        // Convert world-space back to chunk-local coordinates.
        let local_x = (block_x - world_x0) as usize;
        let local_z = (block_z - world_z0) as usize;
        let world_y = block_y as i32;

        let cs = CHUNK_SIZE as usize;

        let block = if local_x < cs && local_z < cs {
            chunk.get_block(local_x, world_y, local_z)
        } else {
            // Face from a boundary — treat as opaque (safe fallback).
            BlockId::Stone
        };

        if is_transparent_block(block) {
            let new_base = transparent_verts.len() as u32;
            transparent_verts.extend_from_slice(face_verts);
            for i in 0..6 {
                let original = indices[idx_offset + i];
                let rebased = original - (base_vertex as u32) + new_base;
                transparent_indices.push(rebased);
            }
        } else {
            let new_base = opaque_verts.len() as u32;
            opaque_verts.extend_from_slice(face_verts);
            for i in 0..6 {
                let original = indices[idx_offset + i];
                let rebased = original - (base_vertex as u32) + new_base;
                opaque_indices.push(rebased);
            }
        }
    }

    (
        opaque_verts,
        opaque_indices,
        transparent_verts,
        transparent_indices,
    )
}

/// WGSL fragment shader variant for water and transparent blocks.
///
/// Uses the same vertex shader as the terrain shader but applies:
/// - Alpha = 0.7 for water blocks
/// - Blue tint toward water color
///
/// Bind groups (identical to terrain shader):
/// - group(0): CameraUniform (view_proj)
/// - group(1): texture atlas (t_diffuse, s_diffuse)
/// - group(2): SkyUniform (sky_color, sun_dir, ambient)
pub const WATER_SHADER_SOURCE: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct SkyUniform {
    sky_color: vec3<f32>,
    _pad0: f32,
    sun_dir: vec3<f32>,
    ambient: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> sky: SkyUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    out.normal = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Dynamic directional lighting (same as terrain)
    let light_dir = normalize(sky.sun_dir);
    let diffuse = max(dot(in.normal, light_dir), 0.0) * 0.6;
    let brightness = sky.ambient + diffuse;

    // Apply blue tint and water alpha
    let blue_tint = vec3<f32>(0.3, 0.5, 0.9);
    let tinted = mix(tex_color.rgb, blue_tint, 0.35);

    return vec4<f32>(tinted * brightness, 0.7);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::pos::WORLD_BOTTOM;

    /// Helper: build a tiny chunk with a single block placed at local (0, WORLD_BOTTOM, 0).
    fn single_block_chunk(block: BlockId) -> Chunk {
        let mut chunk = Chunk::new();
        chunk.set_block(0, WORLD_BOTTOM, 0, block);
        chunk
    }

    /// Build a minimal quad (4 verts, 6 indices) representing one face of a
    /// block at world position `(wx, wy, wz)`.
    fn fake_face(wx: f32, wy: f32, wz: f32) -> ([Vertex; 4], [u32; 6], u32) {
        let verts = [
            Vertex {
                position: [wx, wy + 1.0, wz],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [wx, wy + 1.0, wz + 1.0],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [wx + 1.0, wy + 1.0, wz + 1.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [wx + 1.0, wy + 1.0, wz],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
        ];
        // base_vertex = 0 for standalone use; caller adjusts.
        let indices = [0, 1, 2, 2, 3, 0];
        (verts, indices, 4)
    }

    #[test]
    fn separation_classifies_water_as_transparent() {
        let chunk_pos = ChunkPos::new(0, 0);
        let chunk = single_block_chunk(BlockId::Water);

        let origin = chunk_pos.block_origin();
        let wx = origin.x as f32;
        let wz = origin.z as f32;
        let wy = WORLD_BOTTOM as f32;

        let (face_verts, face_indices, _) = fake_face(wx, wy, wz);
        let vertices: Vec<Vertex> = face_verts.to_vec();
        let indices: Vec<u32> = face_indices.to_vec();

        let (ov, oi, tv, ti) = separate_transparent_faces(&vertices, &indices, &chunk, chunk_pos);

        assert!(ov.is_empty(), "opaque should be empty for water");
        assert!(oi.is_empty());
        assert_eq!(tv.len(), 4, "transparent should have 4 vertices");
        assert_eq!(ti.len(), 6, "transparent should have 6 indices");
    }

    #[test]
    fn separation_classifies_stone_as_opaque() {
        let chunk_pos = ChunkPos::new(0, 0);
        let chunk = single_block_chunk(BlockId::Stone);

        let origin = chunk_pos.block_origin();
        let wx = origin.x as f32;
        let wz = origin.z as f32;
        let wy = WORLD_BOTTOM as f32;

        let (face_verts, face_indices, _) = fake_face(wx, wy, wz);
        let vertices: Vec<Vertex> = face_verts.to_vec();
        let indices: Vec<u32> = face_indices.to_vec();

        let (ov, oi, tv, ti) = separate_transparent_faces(&vertices, &indices, &chunk, chunk_pos);

        assert_eq!(ov.len(), 4, "opaque should have 4 vertices");
        assert_eq!(oi.len(), 6);
        assert!(tv.is_empty(), "transparent should be empty for stone");
        assert!(ti.is_empty());
    }

    #[test]
    fn separation_classifies_glass_and_leaves_as_transparent() {
        let transparent_blocks = [
            BlockId::Glass,
            BlockId::Ice,
            BlockId::OakLeaves,
            BlockId::BirchLeaves,
            BlockId::SpruceLeaves,
            BlockId::JungleLeaves,
            BlockId::DarkOakLeaves,
        ];

        for block in transparent_blocks {
            let chunk_pos = ChunkPos::new(0, 0);
            let chunk = single_block_chunk(block);

            let origin = chunk_pos.block_origin();
            let wx = origin.x as f32;
            let wz = origin.z as f32;
            let wy = WORLD_BOTTOM as f32;

            let (face_verts, face_indices, _) = fake_face(wx, wy, wz);
            let vertices: Vec<Vertex> = face_verts.to_vec();
            let indices: Vec<u32> = face_indices.to_vec();

            let (ov, _oi, tv, _ti) =
                separate_transparent_faces(&vertices, &indices, &chunk, chunk_pos);

            assert!(
                ov.is_empty(),
                "{:?} should be classified as transparent",
                block
            );
            assert_eq!(
                tv.len(),
                4,
                "{:?} should produce transparent vertices",
                block
            );
        }
    }

    #[test]
    fn empty_chunk_produces_empty_transparent_set() {
        let chunk_pos = ChunkPos::new(0, 0);
        let chunk = Chunk::new(); // all air

        let vertices: Vec<Vertex> = Vec::new();
        let indices: Vec<u32> = Vec::new();

        let (ov, oi, tv, ti) = separate_transparent_faces(&vertices, &indices, &chunk, chunk_pos);

        assert!(ov.is_empty());
        assert!(oi.is_empty());
        assert!(tv.is_empty());
        assert!(ti.is_empty());
    }

    #[test]
    fn mixed_chunk_separates_correctly() {
        let chunk_pos = ChunkPos::new(0, 0);
        let mut chunk = Chunk::new();
        // Place stone at (0, WORLD_BOTTOM, 0) and water at (1, WORLD_BOTTOM, 0)
        chunk.set_block(0, WORLD_BOTTOM, 0, BlockId::Stone);
        chunk.set_block(1, WORLD_BOTTOM, 0, BlockId::Water);

        let origin = chunk_pos.block_origin();
        let wx0 = origin.x as f32;
        let wz0 = origin.z as f32;
        let wy = WORLD_BOTTOM as f32;

        // Build two faces: one for stone, one for water
        let (stone_verts, _, _) = fake_face(wx0, wy, wz0);
        let (water_verts, _, _) = fake_face(wx0 + 1.0, wy, wz0);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Stone face
        let base0 = vertices.len() as u32;
        vertices.extend_from_slice(&stone_verts);
        indices.extend_from_slice(&[base0, base0 + 1, base0 + 2, base0 + 2, base0 + 3, base0]);

        // Water face
        let base1 = vertices.len() as u32;
        vertices.extend_from_slice(&water_verts);
        indices.extend_from_slice(&[base1, base1 + 1, base1 + 2, base1 + 2, base1 + 3, base1]);

        let (ov, oi, tv, ti) = separate_transparent_faces(&vertices, &indices, &chunk, chunk_pos);

        assert_eq!(ov.len(), 4, "should have 4 opaque vertices (stone)");
        assert_eq!(oi.len(), 6);
        assert_eq!(tv.len(), 4, "should have 4 transparent vertices (water)");
        assert_eq!(ti.len(), 6);
    }

    #[test]
    fn water_shader_contains_alpha() {
        assert!(
            WATER_SHADER_SOURCE.contains("0.7"),
            "water shader should set alpha to 0.7"
        );
    }

    #[test]
    fn water_animation_uv_offset_cycles() {
        let mut anim = WaterAnimation::new();
        let (u0, v0) = anim.uv_offset();
        // At time=0 the sine is 0
        assert!((u0).abs() < 1e-6);

        anim.advance(10.0);
        let (u1, v1) = anim.uv_offset();
        // After advancing, offsets should differ from zero
        assert!((u1 - u0).abs() > 1e-6 || (v1 - v0).abs() > 1e-6);
    }

    #[test]
    fn water_animation_wave_height_varies() {
        let mut anim = WaterAnimation::new();
        let h0 = anim.wave_height(0.0, 0.0);
        anim.advance(5.0);
        let h1 = anim.wave_height(0.0, 0.0);
        // Wave height should change over time
        assert!((h1 - h0).abs() > 1e-6, "wave height should vary over time");
    }

    #[test]
    fn water_animation_default() {
        let anim = WaterAnimation::default();
        let (u, _v) = anim.uv_offset();
        assert!((u).abs() < 1e-6, "default animation starts at time=0");
    }
}
