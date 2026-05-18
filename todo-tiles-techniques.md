# Port Tiles Techniques to Dracon Terminal Engine

Source: /home/dracon/Dev/tiles/ (20K-line TUI file manager built on dracon-terminal-engine)
Target: /home/dracon/Dev/dracon-terminal-engine/ (the engine framework itself)

## Done ✅

### 1. [HIGH] Marquee Drag Selection System — DONE
- New `src/framework/marquee.rs` module (340 lines)
- `MarqueeState` struct with `is_active`, `start`, `end`, `activation_threshold`, `pending_click_idx`
- `MarqueeRect { min_col, min_row, max_col, max_row }` — normalized, Copy
- `start_tracking()`, `update()`, `rect()`, `clear()`, `reset()`
- `contains_row()`, `contains()` for hit-testing during commit
- `defer_click()` / `take_pending_click()` for deferred click pattern
- `is_tracking()` for disambiguation
- `render_marquee()` — border-only rounded rect, transparent background, themed primary color
- Exported from prelude
- 12 unit tests (all pass)

### 2. [HIGH] Deferred Click Pattern — DONE
- `pending_click_idx: Option<usize>` in `MarqueeState`
- `defer_click(idx)` / `take_pending_click()` API
- Documented in module doc comment

### 3. [HIGH] Staggered Drag Thresholds — DONE
- `activation_threshold: f32` in `MarqueeState` (default: 4.0 = 2px)
- `with_activation_threshold()` builder
- Marquee threshold < drag threshold ensures marquee wins

### 4. [MEDIUM] Input Shield Cooldown — DONE
- `input_shield_until: Cell<Option<Instant>>` added to `App`
- Checked at top of `handle_event()` — swallows all key/mouse during shield
- `pub fn shield_input(&self, duration: Duration)` — public API
- `pub fn is_input_shielded(&self) -> bool` — query method
- Auto-clears when shield expires

## Remaining

### 5. [MEDIUM] Render-Bounds Registration for Mouse Dispatch
**Status:** Already have `ScopedZoneRegistry` in framework. The pattern from Tiles
(breadcrumb_bounds, column_bounds, etc.) is essentially the same concept.
**Action:** Document the pattern in AGENTS.md — our ScopedZoneRegistry IS this pattern.
No code changes needed.

### 6. [MEDIUM] AppMode Modal State Machine
**Status:** This is an app-level pattern, not a framework feature. Showcase scenes
could adopt a `SceneMode` enum instead of scattered `show_help: bool` fields.
**Action:** Document as recommended pattern in AGENTS.md.

### 7. [MEDIUM] previous_mode for Back-Navigation
**Status:** Tiles stores `app.core.previous_mode`. Our showcase scene router already
handles back-navigation via `pop_force()`. For stacked overlays within a scene,
this pattern would help.
**Action:** Low priority — our scenes don't have deeply stacked modals yet.

### 8. [LOW] Middle-Click Paste
**Action:** Already available via `utils::get_primary_selection_text()` in the engine.
No framework change needed — app developers can call it directly.

### 9. [LOW] Double-Click Detection Utility
**Action:** Tiles uses `is_double_click(pos, time, col, row)`. Our showcase has
this inline. Could add a utility but low priority.

### 10. [LOW] Drag Ghost Verification
**Action:** Our `DragGhost` in `src/framework/dragdrop.rs` already supports
`render()` and `render_with_theme()`. Feature-comparable to Tiles.

## Build Status
- `cargo clippy --lib --examples` — 0 warnings, 0 errors
- `cargo test` — All pass (269+ unit tests, 12 marquee tests)
