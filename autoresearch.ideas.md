# Autoresearch Ideas — Dracon Terminal Engine

## Promising Ideas (Not Yet Tried)

### 1. CellPool pre-allocation strategy
**Hypothesis:** Pre-allocate CellPool based on terminal size at startup instead of growing incrementally.

**Current:** CellPool starts small, grows on demand with reallocation
**Idea:** `CellPool::prealloc(width * height * layers)` at App::new()

**Expected improvement:** Reduced allocation churn during render cycles

---

### 2. Fast-path for fully-opaque blit
**Hypothesis:** The `blit_from_fast()` path exists but may not be utilized in hot paths.

**Current:** Widget renders create Plane with some transparent cells
**Idea:** Ensure all widget renders fill their backgrounds, enabling `blit_from_fast()`

**Expected improvement:** 2-3x faster blit for fully-opaque planes (single memcpy vs per-cell)

---

### 3. Dirty region coarse-graining
**Hypothesis:** Fine-grained dirty regions cause excessive re-renders.

**Current:** DirtyTracker tracks individual cells or small rectangles
**Idea:** Coarse-grain dirty regions to widget-level (whole widget re-renders)

**Expected improvement:** Fewer partial re-renders, simpler dirty tracking

---

### 4. Hot path function inlining
**Hypothesis:** Function call overhead in hot render path.

**Current:** `render()` → `fill_bg()` → `blit_to()` with multiple function calls
**Idea:** Use `#[inline(always)]` on hot path functions

**Expected improvement:** 5-10% reduction in call overhead

---

### 5. Stack-allocated small planes
**Hypothesis:** Heap allocation for small planes (widgets) adds latency.

**Current:** All Planes allocated on heap via `Vec<Cell>`
**Idea:** Stack allocation for planes < 1KB using fixed-size arrays

**Expected improvement:** Reduced allocation overhead for small widgets

---

### 6. Bit-packed Cell representation
**Hypothesis:** Memory bandwidth is the bottleneck.

**Current:** Cell has `char` (4 bytes) + `Color` (enum + data) + `Styles` (bitflags)
**Idea:** Pack Cell into 16 bytes using bitfields

**Expected improvement:** 20-30% faster memcpy due to smaller data size

---

### 7. SIMD-accelerated Cell copy
**Hypothesis:** SIMD can copy 16 cells at once.

**Current:** `cells.copy_from_slice()` uses scalar copy
**Idea:** Use `std::simd::SimdUint` or manual SIMD for bulk Cell copy

**Expected improvement:** 4x speedup on bulk copy operations

---

## In Progress

### 8. Color::Reset skip in blit_to
**Status:** Implemented
**Hypothesis:** blit_to() copying Color::Reset bg cells caused visual bugs
**Result:** Fixed white lines bug, no measurable performance impact

---

## Rejected Ideas

### 9. Custom allocator for Cell pool
**Reason:** Overkill for typical terminal sizes (< 100KB total)
**Rejected:** Complexity without measurable benefit

### 10. GPU rendering
**Reason:** Terminal limitation — no GPU access
**Rejected:** Framework cannot control terminal display

### 11. Multi-threaded widget rendering
**Reason:** Terminal is single-threaded by nature, contention would outweigh benefits
**Rejected:** Single-threaded event loop is the correct model

---

## Notes

- Ideas are prioritized by expected impact × implementation difficulty
- Start with ideas 1-4 (high impact, moderate difficulty)
- Benchmark before and after each change
- Watch for regressions in secondary metrics (compile time, memory)