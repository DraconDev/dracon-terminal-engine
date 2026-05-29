//! The compositor module provides rendering infrastructure for the terminal engine.

#[doc = "Compositor engine (compositing algorithm, plane ordering, rendering pipeline)."]
pub mod engine;
#[doc = "Visual filters for planes (Dim, Invert, Scanline, Pulse, Glitch)."]
pub mod filter;
#[doc = "Plane, Cell, Color, and Styles types."]
pub mod plane;
#[doc = "Object pool for Plane and Cell allocation to reduce per-frame allocations."]
pub mod pool;

/// Re-exports the core [`Compositor`] type from the [`engine`] module.
pub use engine::Compositor;
/// Re-exports types for plane-based rendering: [`Cell`], [`Color`], [`Plane`], and [`Styles`].
pub use plane::{Cell, Color, Plane, Styles};
/// Re-exports pool types: [`CellPool`], [`PoolConfig`], [`acquire_plane_cells`], [`release_plane_cells`].
pub use pool::{acquire_plane_cells, release_plane_cells, CellPool, PoolConfig};
