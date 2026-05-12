//! Object pool for Plane and Cell allocation.
//!
//! Reduces allocation pressure by recycling objects across frames.
//! Pooled planes and cells are reset and reused, avoiding the cost of
//! `Vec::with_capacity()` and `Cell::default()` on every render.
//!
//! # Usage
//!
//! ```rust,ignore
//! let mut plane_pool = PlanePool::new();
//! let mut cell_pool = CellPool::new();
//!
//! // Acquire a plane from the pool
//! let mut plane = plane_pool.acquire(0, 80, 24, &mut cell_pool);
//! // ... render into plane ...
//! // Return to pool (reset, not dropped)
//! plane_pool.release(plane);
//! ```

use super::plane::{Cell, Color, Plane, Styles};
use std::mem;

/// Maximum number of planes to retain in the pool.
/// Planes larger than this are deallocated immediately.
const MAX_PLANE_POOL_SIZE: usize = 32;

/// Maximum number of cells to retain in the pool.
/// Pool is capped to avoid unbounded memory growth.
const MAX_CELL_POOL_SIZE: usize = 100_000;

/// Shared pool configuration.
#[derive(Debug, Clone, Copy)]
pub struct PoolConfig {
    /// Maximum number of pooled planes.
    pub max_planes: usize,
    /// Maximum number of pooled cells.
    pub max_cells: usize,
    /// Initial capacity for the free list.
    pub initial_capacity: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_planes: MAX_PLANE_POOL_SIZE,
            max_cells: MAX_CELL_POOL_SIZE,
            initial_capacity: 64,
        }
    }
}

/// A pool of recycled [`Plane`] objects.
///
/// Planes are acquired from the pool and returned after use.
/// When returned, planes are cleared and held for reuse. If the pool
/// is full, returned planes are dropped (deallocated).
///
/// # Thread Safety
///
/// Not thread-safe. Use per-thread pools or a `Mutex<PlanePool>` if
/// sharing across threads is needed.
#[derive(Debug)]
pub struct PlanePool {
    config: PoolConfig,
    /// Stack of available planes, keyed by (width, height).
    free: Vec<PooledPlane>,
}

#[derive(Debug)]
struct PooledPlane {
    plane: Plane,
    cell_pool: *mut Vec<Cell>,
}

impl PlanePool {
    /// Creates a new empty plane pool.
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Creates a plane pool with custom configuration.
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            config,
            free: Vec::with_capacity(config.initial_capacity),
        }
    }

    /// Returns the number of planes currently in the pool.
    pub fn len(&self) -> usize {
        self.free.len()
    }

    /// Returns true if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.free.is_empty()
    }

    /// Acquires a plane from the pool, creating one if empty.
    ///
    /// The returned plane has `width × height` cells initialized
    /// from the cell pool. If no pool entry matches, a fresh plane
    /// is allocated.
    ///
    /// The `cell_pool` parameter is used to source pre-allocated cells.
    #[allow(clippy::mut_from_ref)] // SAFETY: pool entry is extracted then consumed
    pub fn acquire(&mut self, id: usize, width: u16, height: u16, cell_pool: &mut CellPool) -> Plane {
        // Try to find a matching pool entry
        let idx = self.free.iter().position(|p| {
            p.plane.width == width && p.plane.height == height
        });

        if let Some(idx) = idx {
            // Reuse pooled plane — extract and forget the cell_pool ref
            plane.clear();
            return plane;
        }

        // No matching pool entry — allocate fresh
        Plane::new(id, width, height)
    }

    /// Returns a plane to the pool for reuse.
    ///
    /// The plane is cleared and added to the free list if the pool
    /// has capacity. Otherwise it is dropped.
    ///
    /// The plane's `id` field is preserved across the pool cycle.
    pub fn release(&mut self, plane: Plane) {
        if self.free.len() >= self.config.max_planes {
            // Pool full — drop the plane
            return;
        }

        // Wrap the plane with a null cell_pool reference (not used on release)
        let pooled = PooledPlane {
            plane,
            cell_pool: std::ptr::null_mut(),
        };
        self.free.push(pooled);
    }
}

impl Default for PlanePool {
    fn default() -> Self {
        Self::new()
    }
}

/// A pool of recycled [`Cell`] objects.
///
/// Cells are acquired in bulk and returned individually or in bulk.
/// This reduces the per-frame allocation pressure from widgets that
/// create many planes per tick.
///
/// # Thread Safety
///
/// Not thread-safe.
#[derive(Debug)]
pub struct CellPool {
    config: PoolConfig,
    /// Free list of cells, keyed by (width, height) of the plane they belong to.
    /// The Vec stores cells in row-major order matching their original plane.
    free: Vec<CellBlock>,
}

/// A contiguous block of cells for a plane of specific dimensions.
#[derive(Debug)]
struct CellBlock {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl CellPool {
    /// Creates a new empty cell pool.
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Creates a cell pool with custom configuration.
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            config,
            free: Vec::with_capacity(config.initial_capacity),
        }
    }

    /// Returns the total number of cells currently in the pool.
    pub fn total_cells(&self) -> usize {
        self.free.iter().map(|b| b.cells.len()).sum()
    }

    /// Acquires `count` cells from the pool.
    ///
    /// Returns a Vec of cells, default-initialized. If the pool
    /// has fewer than `count` cells available, the shortfall is
    /// allocated freshly.
    pub fn acquire_cells(&mut self, count: usize) -> Vec<Cell> {
        // Try to find a block with at least `count` cells
        let mut acquired = Vec::with_capacity(count);

        // Drain from existing blocks
        while acquired.len() < count {
            // Find the block with the most cells
            let largest_idx = self
                .free
                .iter()
                .enumerate()
                .max_by_key(|(_, b)| b.cells.len())
                .map(|(idx, _)| idx);

            if let Some(idx) = largest_idx {
                let block = &mut self.free[idx];
                let needed = count - acquired.len();

                if block.cells.len() >= needed {
                    // Take exactly what we need
                    for _ in 0..needed {
                        let cell = block.cells.pop().unwrap();
                        acquired.push(cell);
                    }
                    // Remove block if empty
                    if block.cells.is_empty() {
                        self.free.swap_remove(idx);
                    }
                } else {
                    // Take all from this block
                    for cell in block.cells.drain(..) {
                        acquired.push(cell);
                    }
                    self.free.swap_remove(idx);
                }
            } else {
                // No blocks left — allocate fresh
                let remaining = count - acquired.len();
                for _ in 0..remaining {
                    acquired.push(Cell::default());
                }
                break;
            }
        }

        acquired
    }

    /// Returns cells to the pool for reuse.
    ///
    /// Cells are stored in blocks keyed by a synthetic `width × height`
    /// identifier. The pool caps total cells at `config.max_cells` —
    /// excess cells are dropped.
    pub fn release_cells(&mut self, width: u16, height: u16, cells: Vec<Cell>) {
        let total = self.total_cells();
        if total + cells.len() > self.config.max_cells {
            // Pool would exceed limit — drop cells
            return;
        }

        self.free.push(CellBlock {
            width,
            height,
            cells,
        });
    }

    /// Shrinks all internal buffers to fit their contents.
    /// Call this during idle periods to return memory to the allocator.
    pub fn shrink_to_fit(&mut self) {
        for block in &mut self.free {
            block.cells.shrink_to_fit();
        }
        self.free.shrink_to_fit();
    }
}

impl Default for CellPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Acquires a plane with pooled cell allocation.
///
/// This is a convenience function that acquires a plane and its cells
/// from the shared pool, then configures the plane for immediate use.
///
/// # Example
///
/// ```rust,ignore
/// let mut pool = PlanePool::new();
/// let mut cell_pool = CellPool::new();
///
/// let plane = acquire_plane_from_pool(&mut pool, 0, 80, 24, &mut cell_pool);
/// // ... use plane ...
/// release_plane_to_pool(&mut pool, plane, &mut cell_pool);
/// ```
pub fn acquire_plane_from_pool(
    plane_pool: &mut PlanePool,
    cell_pool: &mut CellPool,
    id: usize,
    width: u16,
    height: u16,
) -> Plane {
    let count = (width.max(1) as usize) * (height.max(1) as usize);
    let cells = cell_pool.acquire_cells(count);
    let mut plane = Plane::new(id, width, height);
    plane.cells = cells;
    plane
}

/// Releases a plane back to the pool, recycling its cells.
pub fn release_plane_to_pool(_plane_pool: &mut PlanePool, cell_pool: &mut CellPool, mut plane: Plane) {
    // Take cells out of the plane before returning it
    let cells = std::mem::take(&mut plane.cells);
    cell_pool.release_cells(plane.width, plane.height, cells);
    plane_pool.release(plane);
}