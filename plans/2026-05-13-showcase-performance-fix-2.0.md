# Showcase Performance Fix Plan

## Root Cause Analysis

The showcase lag (5-second scroll delay) has **two compounding bottlenecks**:

### Bottleneck 1: 20ms Blocking Poll (`src/framework/app.rs:438`)
`poll_input` uses `libc::poll()` with a 20ms timeout. Every frame, the event loop blocks for up to 20ms waiting for input. At 30fps (33ms frame budget), **60% of the frame time is wasted just waiting**.

### Bottleneck 2: No Input Draining
Only one `poll_input` call per frame. Scroll events arrive in bursts (10-20 per wheel action). Queue builds up across frames.

### Bottleneck 3: Per-Card Plane Allocation in Showcase
Every frame creates `Plane::new(0, card_w, card_h)` for each visible card (~20 cards), allocating ~10,000+ cells per frame.

## Fixes

### Fix 1: Reduce poll timeout from 20ms to 1ms
**File:** `src/framework/app.rs:438`
**Impact:** Frees up 19ms per frame. Input-to-render latency drops from ~20ms to ~1ms.

### Fix 2: Add input draining loop
**File:** `src/framework/app.rs:438-601`
After first `poll_input` returns data, keep reading non-blockingly until the buffer is empty.

### Fix 3: Skip re-render on unchanged mouse move
**File:** `examples/showcase/widget.rs`
Only set `self.dirty = true` when state actually changes.

## Implementation Steps
- [ ] Reduce poll timeout from 20ms to 1ms
- [ ] Add input draining loop after first successful poll
- [ ] Make showcase handle_mouse only set dirty on state change
- [ ] Verify build, clippy, and tests
