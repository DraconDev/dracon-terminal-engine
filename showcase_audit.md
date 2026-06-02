# Showcase Audit — Completed Work

**Date**: 2026-06-02

---

## Completed Tasks

### 1. Bug Fixes (8 bugs fixed)

#### calendar_scene.rs (4 bugs fixed)
- **Fixed dirty tracking**: `mark_dirty()` and `clear_dirty()` now properly set `self.dirty`
- **Fixed event click index mismatch**: Now uses filtered `upcoming` list instead of raw `EVENTS`
- **Fixed hardcoded default date**: Uses `EVENTS[0].date` instead of hardcoded "2026-05-17"
- **Fixed emoji icons**: Replaced emoji icons (📅🔴🎉🔔) with ASCII characters (MDHR)

#### radio_scene.rs (2 bugs fixed)
- **Use Radio widget render**: Now uses `radio.render(radio_area)` instead of manual drawing
- **Removed duplicated label text**: Labels now come from the widget's render method

#### raycaster_scene.rs (1 bug fixed)
- **Removed redundant import**: Removed `use dracon_terminal_engine::compositor::plane::Color;` (Color already in scope via `prelude::*`)

#### settings_scene.rs (1 bug fixed)
- **Replaced expect() calls**: Changed 3 `expect()` calls to `ok().unwrap_or(ValidationRule::Required)`

### 2. Scene Cut (1 scene removed)

#### control_panel_scene
- **Deleted**: `examples/showcase/scenes/control_panel_scene.rs`
- **Removed from mod.rs**: Removed `pub mod control_panel_scene;`
- **Removed from state.rs**: Removed scene registration
- **Removed from data.rs**: Removed ExampleMeta entry

### 3. Polish Categories (3 categories applied)

#### Hardcoded Keys
- **radio_scene.rs**: Added `KeybindingSet` usage for key bindings
- **calendar_scene.rs**: Uses `keybindings.matches()` for help and back keys

#### Status Bars
- **calendar_scene.rs**: Added `StatusBar` with key hints
- **raycaster_scene.rs**: Added `StatusBar` with movement controls
- **paint_scene.rs**: Added `StatusBar` with tool shortcuts
- **workshop_scene.rs**: Added `StatusBar` with widget controls

#### Lifecycle Hooks
- **calendar_scene.rs**: Added `on_enter()` (resets state) and `on_exit()` (clears help)
- **hud_demo_scene.rs**: Added `on_enter()` (resets game state) and `on_exit()` (clears help)
- **note_editor_scene.rs**: Added `on_enter()` (clears help) and `on_exit()` (clears help)

### 4. Showcase Star Expansions (2 scenes expanded)

#### hud_demo_scene.rs
- **Real-time shield regen**: Shield regenerates 2.0 per tick when below 100
- **Particle effects**: 3x3 particle burst around enemy when hit
- **Damage numbers**: Floating damage numbers (e.g., "+500", "-25") appear at hit location

#### live_feed_scene.rs
- **Network metrics**: Added `net_in_data`, `net_out_data`, `latency_data` sparklines
- **Severity filter**: Press `F` to cycle through INFO/WARN/ERROR/DEBUG filters
- **Export-to-file**: Press `E` to export logs to `live_feed_export_{tick}.log`
- **Updated metrics overview**: Shows CPU, Memory, Net In, Net Out, Latency, Lines
- **Updated help overlay**: Added F (filter), E (export) key descriptions

---

## Verification

- `cargo check` — ✅ Pass
- `cargo clippy` — ✅ No warnings
- `cargo test` — ✅ 15 passed, 0 failed, 21 ignored
- `cargo fmt` — ✅ Formatted

---

## Files Modified

| File | Changes |
|------|---------|
| `examples/showcase/scenes/calendar_scene.rs` | Bug fixes, status bar, lifecycle hooks |
| `examples/showcase/scenes/radio_scene.rs` | Bug fixes |
| `examples/showcase/scenes/raycaster_scene.rs` | Bug fix, status bar |
| `examples/showcase/scenes/settings_scene.rs` | Bug fix |
| `examples/showcase/scenes/control_panel_scene.rs` | DELETED |
| `examples/showcase/scenes/paint_scene.rs` | Status bar |
| `examples/showcase/scenes/workshop_scene.rs` | Status bar |
| `examples/showcase/scenes/note_editor_scene.rs` | Lifecycle hooks |
| `examples/showcase/scenes/hud_demo_scene.rs` | Lifecycle hooks, shield regen, particles, damage numbers |
| `examples/showcase/scenes/live_feed_scene.rs` | Network metrics, severity filter, export-to-file |
| `examples/showcase/data.rs` | Removed control_panel entry |
| `examples/showcase/state.rs` | Removed control_panel registration |
| `examples/showcase/scenes/mod.rs` | Removed control_panel_scene module |
