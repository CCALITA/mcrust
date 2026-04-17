use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use mc_core::direction::Direction;
use mc_core::pos::{ChunkPos, CHUNK_SIZE, WORLD_BOTTOM, WORLD_TOP};
use mc_world::Chunk;

use crate::texture::atlas_uv;

/// Vertex format: position + texture coordinates + normal.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
            2 => Float32x3,
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS,
        }
    }
}

/// Optional neighboring chunk data for boundary face culling.
/// Indexed by direction: [+X (East), -X (West), +Z (South), -Z (North)].
pub struct NeighborChunks<'a> {
    pub east: Option<&'a Chunk>,
    pub west: Option<&'a Chunk>,
    pub south: Option<&'a Chunk>,
    pub north: Option<&'a Chunk>,
}

impl<'a> NeighborChunks<'a> {
    pub fn none() -> Self {
        Self {
            east: None,
            west: None,
            south: None,
            north: None,
        }
    }
}

/// A meshed chunk ready for rendering.
pub struct ChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub chunk_pos: ChunkPos,
}

impl ChunkMesh {
    /// Build a mesh from a chunk and its optional neighbors.
    pub fn build(
        device: &wgpu::Device,
        chunk: &Chunk,
        chunk_pos: ChunkPos,
        neighbors: &NeighborChunks<'_>,
    ) -> Self {
        let (vertices, indices) = generate_mesh(chunk, chunk_pos, neighbors);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            chunk_pos,
        }
    }
}

/// Check whether the neighbor block in a given direction is transparent
/// (meaning we should emit a face).
fn is_face_visible(
    chunk: &Chunk,
    neighbors: &NeighborChunks<'_>,
    x: i32,
    y: i32,
    z: i32,
    dir: Direction,
) -> bool {
    let nx = x + dir.normal().x;
    let ny = y + dir.normal().y;
    let nz = z + dir.normal().z;

    // Check vertical bounds
    if !(WORLD_BOTTOM..WORLD_TOP).contains(&ny) {
        return true;
    }

    let cs = CHUNK_SIZE;

    // Within this chunk
    if nx >= 0 && nx < cs && nz >= 0 && nz < cs {
        let neighbor_block = chunk.get_block(nx as usize, ny, nz as usize);
        return neighbor_block.is_transparent();
    }

    // Need to check a neighbor chunk
    let neighbor_chunk = match dir {
        Direction::East => neighbors.east,
        Direction::West => neighbors.west,
        Direction::South => neighbors.south,
        Direction::North => neighbors.north,
        // Up/Down never leave the chunk horizontally
        _ => return true,
    };

    match neighbor_chunk {
        Some(nc) => {
            let lx = nx.rem_euclid(cs) as usize;
            let lz = nz.rem_euclid(cs) as usize;
            nc.get_block(lx, ny, lz).is_transparent()
        }
        None => true, // No neighbor data — show the face
    }
}

/// Generate vertices and indices for a chunk.
fn generate_mesh(
    chunk: &Chunk,
    chunk_pos: ChunkPos,
    neighbors: &NeighborChunks<'_>,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let origin = chunk_pos.block_origin();
    let world_x0 = origin.x as f32;
    let world_z0 = origin.z as f32;

    let cs = CHUNK_SIZE;

    for y in WORLD_BOTTOM..WORLD_TOP {
        for z in 0..cs {
            for x in 0..cs {
                let block = chunk.get_block(x as usize, y, z as usize);
                if block.is_air() {
                    continue;
                }

                let props = block.properties();
                let wx = world_x0 + x as f32;
                let wy = y as f32;
                let wz = world_z0 + z as f32;

                for dir in Direction::ALL {
                    if !is_face_visible(chunk, neighbors, x, y, z, dir) {
                        continue;
                    }

                    // tex_indices order: [top, bottom, north, south, east, west]
                    let tex_idx = props.tex_indices[dir as usize];
                    let (u0, v0, u1, v1) = atlas_uv(tex_idx);

                    let normal = dir.normal();
                    let n = [normal.x as f32, normal.y as f32, normal.z as f32];

                    let face_verts = face_vertices(wx, wy, wz, dir, u0, v0, u1, v1, n);
                    let base = vertices.len() as u32;

                    vertices.extend_from_slice(&face_verts);
                    // Two triangles per quad
                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base + 2,
                        base + 3,
                        base,
                    ]);
                }
            }
        }
    }

    (vertices, indices)
}

/// Generate the 4 vertices for a single block face.
/// The block occupies [wx, wx+1] x [wy, wy+1] x [wz, wz+1].
#[allow(clippy::too_many_arguments)]
fn face_vertices(
    wx: f32,
    wy: f32,
    wz: f32,
    dir: Direction,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    normal: [f32; 3],
) -> [Vertex; 4] {
    let (x0, y0, z0) = (wx, wy, wz);
    let (x1, y1, z1) = (wx + 1.0, wy + 1.0, wz + 1.0);

    match dir {
        Direction::Up => [
            Vertex { position: [x0, y1, z0], tex_coords: [u0, v0], normal },
            Vertex { position: [x0, y1, z1], tex_coords: [u0, v1], normal },
            Vertex { position: [x1, y1, z1], tex_coords: [u1, v1], normal },
            Vertex { position: [x1, y1, z0], tex_coords: [u1, v0], normal },
        ],
        Direction::Down => [
            Vertex { position: [x0, y0, z1], tex_coords: [u0, v0], normal },
            Vertex { position: [x0, y0, z0], tex_coords: [u0, v1], normal },
            Vertex { position: [x1, y0, z0], tex_coords: [u1, v1], normal },
            Vertex { position: [x1, y0, z1], tex_coords: [u1, v0], normal },
        ],
        Direction::North => [
            // -Z face
            Vertex { position: [x1, y1, z0], tex_coords: [u0, v0], normal },
            Vertex { position: [x1, y0, z0], tex_coords: [u0, v1], normal },
            Vertex { position: [x0, y0, z0], tex_coords: [u1, v1], normal },
            Vertex { position: [x0, y1, z0], tex_coords: [u1, v0], normal },
        ],
        Direction::South => [
            // +Z face
            Vertex { position: [x0, y1, z1], tex_coords: [u0, v0], normal },
            Vertex { position: [x0, y0, z1], tex_coords: [u0, v1], normal },
            Vertex { position: [x1, y0, z1], tex_coords: [u1, v1], normal },
            Vertex { position: [x1, y1, z1], tex_coords: [u1, v0], normal },
        ],
        Direction::East => [
            // +X face
            Vertex { position: [x1, y1, z1], tex_coords: [u0, v0], normal },
            Vertex { position: [x1, y0, z1], tex_coords: [u0, v1], normal },
            Vertex { position: [x1, y0, z0], tex_coords: [u1, v1], normal },
            Vertex { position: [x1, y1, z0], tex_coords: [u1, v0], normal },
        ],
        Direction::West => [
            // -X face
            Vertex { position: [x0, y1, z0], tex_coords: [u0, v0], normal },
            Vertex { position: [x0, y0, z0], tex_coords: [u0, v1], normal },
            Vertex { position: [x0, y0, z1], tex_coords: [u1, v1], normal },
            Vertex { position: [x0, y1, z1], tex_coords: [u1, v0], normal },
        ],
    }
}
