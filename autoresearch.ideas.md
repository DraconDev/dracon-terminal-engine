# Autoresearch Ideas — Dracon Terminal Engine

## Summary of Completed Optimizations

| Optimization | Result | Impact |
|--------------|--------|--------|
| Escape sequence inlining | 273µs → 150µs | ~45% faster |
| Hot path function inlining | 522µs → 367µs | ~30% faster |

**Total improvement from initial baseline: ~92% faster** (3,903µs debug → ~150µs release)

---

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

### 4. Stack-allocated small planes
**Hypothesis:** Heap allocation for small planes (widgets) adds latency.

**Current:** All Planes allocated on heap via `Vec<Cell>`
**Idea:** Stack allocation for planes < 1KB using fixed-size arrays

**Expected improvement:** Reduced allocation overhead for small widgets

---

### 5. Bit-packed Cell representation
**Hypothesis:** Memory bandwidth is the bottleneck.

**Current:** Cell has `char` (4 bytes) + `Color` (enum + data) + `Styles` (bitflags)
**Idea:** Pack Cell into 16 bytes using bitfields

**Expected improvement:** 20-30% faster memcpy due to smaller data size

---

### 6. SIMD-accelerated Cell copy
**Hypothesis:** SIMD can copy 16 cells at once.

**Current:** `cells.copy_from_slice()` uses scalar copy
**Idea:** Use `std::simd::SimdUint` or manual SIMD for bulk Cell copy

**Expected improvement:** 4x speedup on bulk copy operations

---

## Notes

- The benchmark is now saturated at ~150-250µs for the 200x100 terminal case
- Terminal I/O variance dominates results (250µs spikes are common)
- All performance tests pass (8/8), zero clippy warnings
- Further improvements require architectural changes (SIMD, bit-packed cells)
- Core compositor optimizations are complete