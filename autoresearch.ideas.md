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

## Completed Optimizations

### Escape Sequence Inlining (ideas.md #7)
**Status:** COMPLETED
**Result:** ~44% improvement (273µs → ~150µs). Replaced all write!() macros with direct byte buffer writes.

**Changes:**
- Cursor positioning: inline digit conversion
- SGR color codes: inline digit conversion for fg/bg
- Character output: ASCII fast-path (direct push), multi-byte fallback

### Hot Path Function Inlining (ideas.md #4)
**Status:** COMPLETED
**Result:** ~30% improvement (522µs → 367µs). Added #[inline] to render(), sort_planes(), blend_cells(), is_braille().

---

## Deferred Ideas

### 6. Bit-packed Cell representation
**Hypothesis:** Memory bandwidth is the bottleneck.

**Current:** Cell has `char` (4 bytes) + `Color` (enum + data) + `Styles` (bitflags)
**Idea:** Pack Cell into 16 bytes using bitfields

**Expected improvement:** 20-30% faster memcpy due to smaller data size

**Note:** Struct size currently ~40 bytes. Can reduce but need careful handling of Color enum.

---

### 7. Terminal output optimization
**Hypothesis:** Escape sequence generation dominates output time.

**Status:** COMPLETED
**Result:** ~39% improvement (273µs → 166µs). Replaced write!() macros with direct byte buffer writes for cursor positioning, SGR color codes, and character output.

**Note:** Also removed all write!() calls from the hot path render loop. Consider batching SGR sequences for further improvement.

---

## Notes

- Ideas are prioritized by expected impact × implementation difficulty
- Start with ideas 1-4 (high impact, moderate difficulty)
- Benchmark before and after each change
- Watch for regressions in secondary metrics (compile time, memory)