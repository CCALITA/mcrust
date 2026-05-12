//! AABB collision detection, movement resolution, and raycasting.
//!
//! Provides axis-aligned bounding box primitives, broad/narrow-phase collision
//! checks, and [`raycast`] for block targeting with [`RaycastHit`] results.

pub mod aabb;
pub mod collision;
pub mod raycast;
pub mod spatial_hash;

pub use raycast::{RaycastHit, raycast};
