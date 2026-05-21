# Autoresearch Configuration

## Primary Metric
**`frame_render_us`** — Microseconds to render a showcase frame (lower is better)

This measures the hot path: widget rendering → plane compositing → terminal output.

## Secondary Metrics
- `compile_time_s` — Full project compile time in seconds
- `clippy_warnings` — Clippy warnings count
- `test_count` — Total passing tests

## Why This Metric
Frame render time is the most impactful UX factor for a TUI framework. Users notice lag when scrolling lists, typing in inputs, or navigating scenes. A 1ms improvement per frame at 60 FPS saves 60ms of latency per second.

## Constraints
- **No cheating** — Don't optimize benchmark-specific paths that don't reflect real usage
- **No overfitting** — Improvements should generalize across all 29 scenes, not just one
- **Backward compatible** — All public APIs must remain unchanged
- **Zero clippy warnings** — Any change must pass `cargo clippy --lib --examples --tests`

## Optimization Targets

### High-Impact (from RESEARCH.md)
1. **Plane blitting** — `blit_from()`, `blit_to()` hot path
2. **Widget rendering** — `render()` method implementations
3. **Dirty tracking** — `needs_render()` + dirty regions
4. **Input parsing** — SGR mouse, keyboard chord parsing

### Medium-Impact
5. **Cell allocation** — `CellPool` reuse efficiency
6. **Color resolution** — Theme cascading
7. **String operations** — `draw_text()`, Unicode width

### Lower-Impact (but measurable)
8. **Scene transitions** — blend_planes()
9. **Event dispatch** — Focus, keybinding resolution

## Benchmark Script
```bash
#!/bin/bash
# bench.sh - Measure frame render time
cargo build --release --example showcase 2>/dev/null
# Run showcase in headless mode with frame timing
```

## Ideas (from RESEARCH.md)

### TODO: Benchmark the baseline first
Before optimizing, establish a reproducible benchmark.

### Ideas in progress
- [ ] CellPool pre-allocation based on max plane size
- [ ] Fast-path for fully-opaque `blit_from_fast()`
- [ ] Dirty region coarse-graining for large areas
- [ ] Inline hot path functions (render, blit)
- [ ] SIMD for Cell memcpy (if applicable)

### Deferred ideas
- [ ] Custom allocator for Cell pool (overkill)
- [ ] GPU rendering (terminal limitation)
- [ ] Multi-threaded widget rendering (contention)

## Session Log

| Run | Metric (µs) | Status | Description |
|-----|------------|--------|-------------|
| 1 | TBD | pending | Baseline measurement |