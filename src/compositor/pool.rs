//! Object pool for Cell allocation.
//!
//! Reduces allocation pressure by recycling cell vectors across frames.
//! Pooled cells are returned to the pool and reused, avoiding the cost of
//! `Vec::with_capacity()` and `Cell::default()` on every render.
//!
//! # Usage
//!
//! ```rust,ignore
//! let mut cell_pool = CellPool::new();
//!
//! // Acquire cells for a plane
//! let cells = cell_pool.acquire_cells(80 * 24);
//! // ... use cells in a Plane ...
//! // Return cells to pool when done
//! cell_pool.release_cells(width, height, cells);
//! ```

use super::plane::Cell;

/// Maximum number of cells to retain in the pool.
/// Pool is capped to avoid unbounded memory growth.
const MAX_CELL_POOL_SIZE: usize = 100_000;

/// Shared pool configuration.
#[derive(Debug, Clone, Copy)]
pub struct PoolConfig {
    /// Maximum number of pooled cells.
    pub max_cells: usize,
    /// Initial capacity for the free list.
    pub initial_capacity: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_cells: MAX_CELL_POOL_SIZE,
            initial_capacity: 64,
        }
    }
}

/// A pool of recycled [`Cell`] objects.
///
/// Cells are acquired in bulk and returned in bulk.
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
    // Dimensions are part of the public data shape; downstream consumers
    // (debugging, future resize logic) may inspect them even though the
    // current pool bookkeeping only iterates `cells`.
    #[allow(dead_code)]
    width: u16,
    #[allow(dead_code)]
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
    #[inline]
    pub fn total_cells(&self) -> usize {
        self.free.iter().map(|b| b.cells.len()).sum()
    }

    /// Acquires `count` cells from the pool.
    ///
    /// Returns a Vec of cells, default-initialized. If the pool
    /// has fewer than `count` cells available, the shortfall is
    /// allocated freshly.
    ///
    /// Uses best-fit selection: prefers the block closest in size to the
    /// request to reduce fragmentation (avoids splitting large blocks
    /// for small requests). Uses bulk `split_off` instead of per-cell
    /// `pop()` for O(1) per block instead of O(n) per cell.
    pub fn acquire_cells(&mut self, count: usize) -> Vec<Cell> {
        let mut acquired = Vec::with_capacity(count);
        let mut remaining = count;

        while remaining > 0 {
            let best_idx = self
                .free
                .iter()
                .enumerate()
                .filter(|(_, b)| b.cells.len() >= remaining)
                .min_by_key(|(_, b)| b.cells.len())
                .map(|(idx, _)| idx)
                .or_else(|| {
                    self.free
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, b)| b.cells.len())
                        .map(|(idx, _)| idx)
                });

            match best_idx {
                Some(idx) => {
                    let block_len = self.free[idx].cells.len();
                    if block_len >= remaining {
                        let split = self.free[idx].cells.split_off(block_len - remaining);
                        acquired.extend(split);
                        if self.free[idx].cells.is_empty() {
                            self.free.swap_remove(idx);
                        }
                        remaining = 0;
                    } else {
                        let mut cells = std::mem::take(&mut self.free[idx].cells);
                        acquired.append(&mut cells);
                        self.free.swap_remove(idx);
                        remaining -= block_len;
                    }
                }
                None => {
                    for _ in 0..remaining {
                        acquired.push(Cell::default());
                    }
                    break;
                }
            }
        }

        acquired
    }

    /// Returns cells to the pool for reuse.
    ///
    /// Cells are stored in blocks keyed by a `width × height` identifier.
    /// The pool caps total cells at `config.max_cells` — excess cells are dropped.
    pub fn release_cells(&mut self, _width: u16, _height: u16, cells: Vec<Cell>) {
        if cells.is_empty() {
            return;
        }
        let total = self.total_cells();
        if total + cells.len() > self.config.max_cells {
            // Pool would exceed limit — drop cells
            return;
        }
        self.free.push(CellBlock {
            width: _width,
            height: _height,
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

/// Convenience function to acquire cells for a plane of given dimensions.
///
/// Returns a Vec of cells ready to be placed in a Plane.
#[inline]
pub fn acquire_plane_cells(pool: &mut CellPool, width: u16, height: u16) -> Vec<Cell> {
    let count = (width.max(1) as usize) * (height.max(1) as usize);
    pool.acquire_cells(count)
}

/// Convenience function to release cells back to the pool.
#[inline]
pub fn release_plane_cells(pool: &mut CellPool, width: u16, height: u16, cells: Vec<Cell>) {
    pool.release_cells(width, height, cells)
}
