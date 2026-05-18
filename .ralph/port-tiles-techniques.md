Port high-value techniques from Tiles (TUI file manager) to dracon-terminal-engine framework.

## Phase 1: Marquee + Deferred Click + Thresholds (items 1-3)
These are tightly coupled — implement together.

### 1. Marquee Drag Selection System
- New `src/framework/marquee.rs` module
- `MarqueeState` struct: `is_active: bool`, `start: Option<(u16,u16)>`, `end: Option<(u16,u16)>`, `threshold: f32`
- `marquee_rect() -> Option<MarqueeRect>` normalizes start/end
- `clear()` resets all state
- `update(col, row)` updates end, activates if distance > threshold
- `MarqueeRect { min_col, min_row, max_col, max_row }` — normalized, Copy
- Render: border-only rounded rect, transparent background

### 2. Deferred Click Pattern
- `pending_click_idx: Option<usize>` field
- On MouseDown: plain clicks set pending_click instead of immediately selecting
- On MouseUp: if no drag/marquee occurred, resolve pending_click to selection
- Ctrl/Shift clicks fire immediately (not deferred)

### 3. Staggered Drag Thresholds
- Marquee threshold: 2px (dist_sq >= 4.0)
- File drag threshold: 3px (dist_sq >= 9.0)
- Configurable via builder methods

### Phase 2: Input Shield (item 4)
- Add `input_shield_until: Option<Instant>` to App
- Check at top of event dispatch loop
- After mode transitions, set 100ms cooldown
- Swallow all key/mouse input during shield period

## Acceptance:
- New marquee module compiles and passes tests
- MarqueeState has unit tests (same pattern as Tiles' tests)
- Input shield added to App
- `cargo clippy --lib --examples` 0 warnings
- `cargo test` all pass
