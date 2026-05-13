# Showcase Choppiness Investigation — Frame Pipeline Bottleneck Analysis

## Objective

Identify every source of frame drops, tearing, and choppiness in the Dracon Terminal Engine showcase by tracing the complete frame pipeline from input → render → compositor → terminal output, with specific line references for each bottleneck.

---

## Pipeline Overview

Each frame of the showcase follows this sequence (timed against a 33.33ms budget at 30fps):

```
[1] Resize check          — app.rs:422-433
[2] Input poll (1ms)      — app.rs:435, tty.rs:44-56
[3] Input drain (0ms×64)  — app.rs:556-637
[4] Widget render          — app.rs:687-711
[5] Tick callback          — app.rs:714-747
[6] Command execution      — app.rs:749-773
[7] User callback          — app.rs:775-788
[8] Compositor render      — app.rs:794-796 → engine.rs:209-332
[9] Animation tick          — app.rs:798
[10] Sleep                 — app.rs:803-806
```

---

## Bottleneck Findings (Ranked by Impact)

### BOTTLENECK 1: Thousands of write() syscalls per frame (CRITICAL)

**Location**: `src/compositor/engine.rs:209-332`

**Problem**: The compositor's `render()` method issues individual `write!()` calls for every changed cell, color transition, style transition, and cursor position. Each `write!()` on `Terminal<io::Stdout>` translates directly to a `write()` syscall — there is **no `BufWriter`** between the compositor and `io::Stdout` (`src/core/terminal.rs:118-125`).

**Estimated syscall count** (first frame, 200×56 terminal):
- 1 BSU escape (`engine.rs:251`)
- 1 wrap-off escape (`engine.rs:257`)
- ~56 cursor-position escapes (one per line, `engine.rs:276`)
- ~11,200 character writes (`engine.rs:322`)
- Hundreds of style/color escapes (`engine.rs:280-320`)
- 1 wrap-on escape (`engine.rs:326`)
- 1 ESU escape (`engine.rs:327`)
- 1 flush (`engine.rs:331`)

**Total: ~12,000+ syscalls per first frame on a large terminal.** Even on subsequent frames with partial updates, animated content (pulsing borders, oscillating gauges, blinking cursors) causes hundreds to thousands of changed cells per frame.

**Why this causes choppiness**: Each `write()` syscall involves a context switch to the kernel. On a typical Linux system, a syscall takes 0.5–2μs. At 12,000 syscalls, that's 6–24ms of pure kernel overhead — potentially consuming **most of the 33ms frame budget** before accounting for any actual computation.

**The sync update protocol IS used** (`engine.rs:251`, `engine.rs:327` — `\x1b[?2026h`/`\x1b[?2026l`), which prevents visual tearing inside the terminal emulator. However, the syscall overhead delays the ESU sequence, causing the terminal to wait longer before painting the frame — perceived as stutter.

**Mitigation**: Build a `Vec<u8>` output buffer in the render loop and issue a single `write_all()` at the end, reducing syscalls from O(changed_cells) to O(1) per frame.

---

### BOTTLENECK 2: Per-card Plane allocation every frame (HIGH)

**Location**: `examples/showcase/widget.rs:566-621`, `examples/showcase/render.rs:205-206`

**Problem**: For every visible card, `render_card()` creates a **fresh `Plane`** (heap-allocated `Vec<Cell>`), renders into it, blits the cells onto the main plane, then drops the card Plane. On a 200×56 terminal with 10 visible cards:
- 10 × `Plane::new(0, 32, 16)` = 10 × 512 cells = 5,120 cells allocated and freed per frame
- Each `Plane::new` calls `vec![Cell::default(); width * height]` (`src/compositor/plane.rs:106`), which is a heap allocation + zero-fill
- The main Plane adds another 11,200 cells

**Total per frame (200×56)**: ~16,320 Cell objects (~261 KB) allocated and freed, with 11 separate heap allocations. On a typical 80×24 terminal: ~2,704 cells (~43 KB), 3 allocations.

**Why this causes choppiness**: Heap allocation involves the system allocator (likely `jemalloc` or `glibc malloc`), which must find free blocks, potentially taking locks and causing cache misses. The per-frame allocation pattern creates GC-like pressure. When multiple cards are visible, this can take 1–5ms depending on allocator and terminal size.

**Mitigation**: Pre-allocate a card Plane at max card size and reuse it across frames (cell pool pattern). Only 1 allocation ever, reset with `plane.clear()` instead of `Plane::new()`.

---

### BOTTLENECK 3: Animation-driven unconditional re-renders (HIGH)

**Location**: `examples/showcase/widget.rs:30-50`

**Problem**: The `needs_render()` method returns `true` whenever ANY animation is active:

```rust
if !self.animations.is_empty() {
    return true;  // line 42-44
}
```

This means a hover animation on **one card** triggers re-rendering of the **entire showcase** — all cards, the sidebar, the title bar, the features bar — every frame for 200ms. Since card previews include animated oscillating gauges, pulsing borders, and blinking cursors (`examples/showcase/render.rs:219-233`, `317-419`, etc.), the `phase` parameter changes continuously, which changes cell values, which defeats the compositor's dirty-cell optimization.

Additionally, the features bar at `examples/showcase/render.rs:102-142` uses sinusoidal pulsing (`(phase * 2.5).sin()`) that changes every frame, meaning **every frame has at least 6 changed cells in the features bar** even without any user interaction.

**Why this causes choppiness**: Animations cause the entire widget to re-render AND the compositor to re-diff the full screen, AND the terminal to write all changed cells. A single hover animation triggers the full expensive pipeline 10–15 times (200ms at 30fps).

**Mitigation**: Track dirty sub-regions per card. Only re-render the animating card's cells. Or, move animated elements into separate overlay planes that can be independently composited.

---

### BOTTLENECK 4: Compositor final_buffer allocation every frame (MEDIUM-HIGH)

**Location**: `src/compositor/engine.rs:210-217`

**Problem**: The `render()` method allocates a fresh `final_buffer` every frame:

```rust
let mut final_buffer = vec![
    Cell { bg: self.clear_color, transparent: false, ..Cell::default() };
    (self.width * self.height) as usize
];
```

On a 200×56 terminal, this is 11,200 cells (~179 KB) allocated and freed every frame. This is separate from the `self.last_frame` buffer (which is reused).

**Why this causes choppiness**: This adds another significant heap allocation per frame. Combined with the card planes, total per-frame heap allocation on a large terminal approaches 500 KB.

**Mitigation**: Make `final_buffer` a persistent field of `Compositor` (like `last_frame`), reusing it across frames with a simple `fill` or iterator reset instead of `vec!`.

---

### BOTTLENECK 5: Compositor planes Vec cleared and re-sorted every frame (MEDIUM)

**Location**: `src/compositor/engine.rs:72-74`, `src/compositor/engine.rs:219-223`, `src/compositor/engine.rs:330`

**Problem**: The compositor's `planes` Vec is:
1. Populated during widget rendering (`add_plane()` at `engine.rs:71-74`, which calls `sort_planes()`)
2. Used for compositing (building `layers` Vec at `engine.rs:219-223`, which is ALSO allocated per frame and sorted)
3. Cleared at `engine.rs:330` (`self.planes.clear()`)

So every frame:
- A `layers` Vec of `(z_index, plane_idx)` tuples is allocated, populated, and sorted
- The `planes` Vec itself is sorted again
- Both are dropped at end of render

**Mitigation**: Reuse the `layers` Vec as a field. Avoid double-sorting (sort `planes` once, iterate in order instead of building a separate `layers` Vec).

---

### BOTTLENECK 6: `poll_input` 1ms minimum blocking per frame (MEDIUM)

**Location**: `src/framework/app.rs:435`, `src/backend/tty.rs:44-56`

**Problem**: Every frame, the event loop calls `poll(POLLIN, timeout=1ms)`. If no input is available, this **blocks for 1ms** before returning. At 30fps with a 33.33ms budget, 1ms is only ~3% — tolerable. But at 60fps (16.67ms budget), it's ~6%. And at 120fps (8.33ms budget), it's ~12%.

More importantly, the 1ms is **wasted time** when there's no input. The thread could be doing useful work (rendering, sleeping) instead of blocking in the kernel.

**Mitigation**: Use the remaining frame budget as the poll timeout when no input is pending: `let poll_timeout = remaining_budget.as_millis().min(1) as i32;` — or better, use the full remaining budget so the OS wakes the thread just in time for the next frame.

---

### BOTTLENECK 7: Synchronous command execution in frame loop (MEDIUM)

**Location**: `src/framework/app.rs:749-773`

**Problem**: The `CommandRunner::run_sync()` at `app.rs:761` executes subprocess commands synchronously within the frame loop. If a widget has `refresh_seconds` set, the framework spawns a child process, waits for it to complete, reads its output, and applies it — all within the frame.

While the showcase doesn't use this feature directly, the check iterates over `command_tracking` every frame, cloning the HashMap (`app.rs:752`), which is wasted work when no commands are registered.

**Mitigation**: Skip the command check entirely when `command_tracking` is empty. For apps that do use it, move command execution to a background thread.

---

### BOTTLENECK 8: Cell clone() in the compositing loop (LOW-MEDIUM)

**Location**: `src/compositor/engine.rs:240`

**Problem**: In the compositing inner loop:

```rust
let mut src_cell = plane.cells[src_idx].clone();
```

Every composited cell is cloned (to apply the filter without mutating the source). On a 200×56 terminal with 10 card planes, this means ~16,000 `Cell::clone()` calls per frame. Each clone copies ~16 bytes (char + 2 Colors + Styles + 2 bools).

**Mitigation**: If no filter is present (the common case), skip the clone and borrow the cell directly. Or apply the filter in-place on the destination cell.

---

### BOTTLENECK 9: RefCell<ScopedZoneRegistry> borrow overhead (LOW)

**Location**: `examples/showcase/widget.rs:110`, `166-174`, `236-244`, `438-458`, `534-536`, `624-632`

**Problem**: The showcase calls `self.zones.borrow_mut()` at least 6 times per render frame — for clearing, registering palette zones, primitive zones, category zones, and card zones. Each `borrow_mut()` performs a runtime check that the RefCell isn't already borrowed.

While individually cheap (~10ns each), these add up over 6+ calls per frame and create a pattern of borrow/unborrow that prevents the compiler from optimizing the zone registration path.

**Mitigation**: Borrow once at the start of render, use the `&mut` reference throughout, and drop at the end.

---

### BOTTLENECK 10: Animation system grows unboundedly (LOW)

**Location**: `src/framework/animation.rs:90-100`

**Problem**: The `AnimationManager::start()` method always pushes to the end of `self.animations`, growing the Vec. The `cleanup()` method at line 120-122 is called by `tick()` at `app.rs:798`, but only after the compositor render — meaning stale animations are checked once per frame. For hover animations, each mouse enter/exit creates a new Animation, and old ones pile up until the next tick.

**Mitigation**: The `is_done()` check in `cleanup()` is cheap (time comparison), so this is a minor issue. But slot reuse (finding a free slot instead of pushing) would prevent unbounded growth.

---

## Frame Budget Breakdown (Estimated, 200×56 terminal)

| Phase | Estimated Time | Notes |
|-------|---------------|-------|
| Input poll + drain | 1-2ms | 1ms poll + up to 64×0ms drain |
| Widget render (Showcase) | 3-8ms | 10 card planes + main plane + cell ops |
| Compositor final_buffer alloc | 0.5-1ms | 11,200 cells allocation |
| Compositor layer build + sort | 0.2-0.5ms | Small Vec allocation + sort |
| Compositor blend loop | 1-3ms | ~16,000 cells, blend_cells + clone |
| Compositor diff + write | 6-24ms | **12,000+ write() syscalls** |
| Compositor flush | 0.1ms | 1 flush() syscall |
| **Total before sleep** | **12-39ms** | **Exceeds 33ms budget on large terminals** |

**At 30fps on a large terminal, the showcase likely cannot sustain the target frame rate during animated content.** The dominant cost is syscall overhead from per-character terminal writes.

---

## Verification Criteria

- [ ] Count actual syscalls per frame with `strace -c` on the showcase binary
- [ ] Measure compositor render time with a timestamp before/after `engine.rs:render()`
- [ ] Profile heap allocations with `valgrind --tool=massif` during animated hover
- [ ] Compare frame time at 80×24 vs 200×56 to confirm scaling
- [ ] Test with `BufWriter<io::Stdout>` to quantify syscall reduction impact

---

## Potential Risks and Mitigations

1. **BufWriter could delay ESU delivery**: If `BufWriter` doesn't auto-flush before the ESU sequence, the terminal might not receive the frame boundary. Mitigation: explicitly flush after ESU write (already done at `engine.rs:331`).

2. **Cell pooling changes Plane semantics**: Pre-allocated planes have stale cells from the previous frame. Mitigation: call `plane.clear()` or `plane.fill_bg()` before each card render (already the pattern used for the main plane at `widget.rs:103-107`).

3. **Dirty-rect optimization adds complexity**: Tracking which sub-region changed requires careful invalidation. Mitigation: start with whole-card granularity (skip re-rendering unchanged cards) before attempting pixel-level dirty rects.

4. **Removing animations eliminates visual appeal**: The features bar and card previews are a core part of the showcase's visual identity. Mitigation: Keep animations but reduce their scope (only animate the affected sub-region, not the full screen).

---

## Alternative Approaches

1. **String buffer accumulation** (Recommended, highest impact): Build the entire terminal output as a `Vec<u8>` or `String` in the compositor render loop, then issue a single `write_all()`. This eliminates ~12,000 syscalls per frame, replacing them with 1. Estimated savings: 6-24ms per frame on large terminals.

2. **BufWriter wrapping**: Add `BufWriter<io::Stdout>` inside `Terminal::new()`. This batches syscalls automatically without changing the compositor code. Less optimal than approach 1 (still many small writes to the buffer), but much lower implementation cost.

3. **Cell pool for card planes**: Pre-allocate one card-sized Plane and reuse it for each card render. Eliminates N-1 heap allocations per frame. Estimated savings: 1-5ms per frame.

4. **Compositor buffer reuse**: Make `final_buffer` a persistent field, `fill()` it at frame start instead of `vec![]`. Estimated savings: 0.5-1ms per frame.

5. **Sub-widget dirty tracking**: Instead of re-rendering the full Showcase widget when one card animates, split the showcase into sub-regions (sidebar, each card, title bar, status bar) with independent dirty flags. Only re-render changed sub-regions. Estimated savings: 50-80% of render time during animations.

6. **Reduce animation scope**: Make card preview animations only trigger when the card is hovered/selected, not for all visible cards. Currently, ALL visible card previews animate via the `phase` parameter (`render.rs:216-219`), causing continuous cell changes.

---

## Summary

The showcase choppiness has a **clear primary cause**: the compositor writes each changed cell to the terminal via individual `write!()` syscalls, with no application-level buffering. On a large terminal with animated content, this produces 12,000+ syscalls per frame, consuming 6-24ms of the 33ms budget in kernel overhead alone.

Secondary causes compound the problem: per-card Plane allocations, animation-driven full re-renders, and the compositor's own per-frame `final_buffer` allocation. Together, these push the frame time above the budget on larger terminals, causing visible stutter.

The **single most impactful fix** is adding a write buffer to the compositor (approach 1 or 2), which would cut syscall overhead by 99%+ and likely bring frame times well within budget even on large terminals.
