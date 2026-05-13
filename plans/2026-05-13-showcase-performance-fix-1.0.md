# Showcase Performance Fix Plan

## Root Cause

The 5-second lag is caused by **scroll event backlog**. The event loop processes only **one input poll per frame** at 30 FPS, but scroll gestures generate 60-120 events/sec. Each event triggers a full render cycle (~15-50ms). Backlog grows at ~70 events/sec, creating multi-second delays.

## Bottleneck Chain

1. User scrolls → terminal generates 10-30+ scroll events
2. `poll_input(20ms)` blocks → reads one chunk → dispatches one scroll event
3. `needs_render() -> true` (always, due to animations) → full 1260-line render
4. Compositor allocates fresh `Vec<Cell>`, blends, diffs, writes ANSI
5. `thread::sleep()` fills remaining frame budget to 33ms
6. Next frame: process next scroll event → repeat

**Effective rate: ~1 scroll event per 33ms frame. 30 events = ~1 second minimum. With render overhead: 3-5 seconds.**

## Implementation Plan

### Fix 1: Input Draining (Critical — eliminates backlog)
- [ ] After `poll_input` returns `Ok(true)`, loop with `poll_input(fd, 0)` (non-blocking) to drain all queued events before rendering
- [ ] File: `src/framework/app.rs` — insert drain loop after line 438

### Fix 2: Scroll Event Coalescing (High — reduces per-event cost)
- [ ] In the showcase's `handle_mouse`, accumulate scroll delta instead of processing each event individually
- [ ] File: `examples/showcase/widget.rs` — batch ScrollDown/ScrollUp events

### Fix 3: Reduce poll_input Timeout (Medium — reduces idle latency)
- [ ] Change `poll_input(fd, 20)` to `poll_input(fd, 5)` — 5ms is enough for most terminals
- [ ] File: `src/framework/app.rs:438`

### Fix 4: Compositor Buffer Reuse (Medium — reduces per-frame allocation)
- [ ] Reuse `final_buffer` Vec between frames instead of allocating fresh each render
- [ ] File: `src/compositor/engine.rs`

### Fix 5: Skip Render When No Visual Change (Low — reduces CPU when idle)
- [ ] Track whether any widget actually changed, skip compositor entirely if not
- [ ] Already partially implemented via `needs_render()` but showcase always renders due to animations

## Verification Criteria
- [ ] Scroll response time < 100ms (currently 3-5 seconds)
- [ ] `cargo check --all-targets` passes
- [ ] `cargo clippy --all-targets` 0 warnings
- [ ] `cargo test` all passing
