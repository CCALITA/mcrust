//! Block, item, and biome type definitions with their property registries.
//!
//! Provides [`BlockId`], [`ItemId`], [`BiomeId`], position types ([`BlockPos`], [`ChunkPos`]),
//! cardinal [`Direction`]s, and nether/end [`PortalTransition`] logic.

pub mod biome;
pub mod block;
pub mod block_access;
pub mod direction;
pub mod item;
pub mod portal;
pub mod pos;
pub mod stairs;

pub use biome::{BiomeId, BiomeProperties};
pub use block::{BlockId, BlockProperties, BlockRegistry};
pub use block_access::BlockAccess;
pub use direction::Direction;
pub use item::{ItemId, ItemProperties, ItemStack, ToolTier, ToolType};
pub use portal::{Axis, DimensionId, EndPortal, NetherPortal, PortalFrame, PortalTransition};
pub use pos::{BlockPos, ChunkPos};
