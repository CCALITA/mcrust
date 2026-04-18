//! wgpu-based rendering: chunk meshing, sky, entities, particles, and water.
//!
//! Provides the [`Renderer`] pipeline, [`Camera`] with frustum culling,
//! [`ChunkMesh`] construction, [`ParticleSystem`], [`DayNightCycle`] sky, and translucent water.

pub mod block_break;
pub mod camera;
pub mod entity_render;
pub mod frustum;
pub mod mesh;
pub mod particle;
pub mod renderer;
pub mod shader;
pub mod sky;
pub mod texture;
pub mod water;

pub use camera::{Camera, CameraUniform};
pub use entity_render::{
    EntityRenderData, MobModel, MobModelPart, animate_idle, animate_walk, model_for_mob,
};
pub use frustum::Frustum;
pub use mesh::{ChunkMesh, NeighborChunks, Vertex};
pub use particle::{Particle, ParticleSystem, ParticleType};
pub use renderer::Renderer;
pub use sky::{DayNightCycle, SkyUniform};
pub use water::{
    TransparentMeshData, WATER_SHADER_SOURCE, WaterAnimation, separate_transparent_faces,
};
