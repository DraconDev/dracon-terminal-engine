# Showcase Audit — Completed Work

**Date:** 2026-06-02
**Status:** All items resolved

## Bug Fixes (8 total)

### calendar_scene.rs
1. **Dirty tracking** — `mark_dirty()` and `clear_dirty()` now properly set/clear the `dirty` flag (`calendar_scene.rs:490-495`)
2. **Event click index** — Clicking sidebar events now uses filtered list correctly (`calendar_scene.rs:417-425`)
3. **Hardcoded date** — Default date uses `EVENTS[0].date` instead of hardcoded "2026-05-17" (`calendar_scene.rs:238`)
4. **Emoji icons** — Replaced emoji icons (📅🔴🎉🔔) with ASCII letters M/D/H/R (`calendar_scene.rs:78-85`)

### radio_scene.rs
5. **Widget render** — Uses `radio.render(radio_area)` instead of manual character drawing (`radio_scene.rs:96-108`)
6. **Duplicated labels** — Removed duplicated label text rendering (`radio_scene.rs:90-108`)

### raycaster_scene.rs
7. **Redundant import** — Removed unused `use ... plane::Color;` import (was line ~15)

### settings_scene.rs
8. **Unsafe expect()** — Replaced 3 `expect()` calls with `.unwrap_or(ValidationRule::Required)` (`settings_scene.rs:89-105`)

## Scene Cut (1 total)

### control_panel_scene
- Removed stale `control_panel` entry from `is_embedded()` in `state.rs:545`
- Removed test entry from `main.rs:340-341` (scene_construction_tests)
- Updated test assertion from 34 to 33 scenes (`main.rs:353`)
- No scene file existed — was a dangling reference

## Polish (3 categories)

### Keybinding Standardization
- **calendar_scene.rs** — Uses `keybindings.matches(actions::HELP/BACK)` instead of hardcoded `F1`/`Esc` (`calendar_scene.rs:381-399`)
- **radio_scene.rs** — Uses `keybindings.matches(actions::HELP/BACK)` for help/back (`radio_scene.rs:454-462`)
- Number keys `1/2/3` in radio_scene kept hardcoded per AGENTS.md (universal navigation primitives)

### Status Bars Added
All 4 scenes now have StatusBar widgets:
- **calendar_scene.rs** — `WidgetId(100)` (`calendar_scene.rs:152-156`)
- **raycaster_scene.rs** — `WidgetId(200)` (`raycaster_scene.rs:83-87`)
- **paint_scene.rs** — `WidgetId(300)` (`paint_scene.rs:69-73`)
- **workshop_scene.rs** — `WidgetId(500)` (`workshop_scene.rs:110-114`)

### Lifecycle Hooks Added
All 3 scenes now have `on_enter()` and `on_exit()`:
- **calendar_scene.rs** — Resets `selected_date` and `show_help` on enter (`calendar_scene.rs:340-348`)
- **hud_demo_scene.rs** — Resets game state (health, ammo, shield, score) on enter (`hud_demo_scene.rs:114-122`)
- **note_editor_scene.rs** — Resets editor state on enter (`note_editor_scene.rs:163-170`)

## Showcase Stars Expanded (2 total)

### live_feed_scene
- **Network metrics** — `net_in_data`, `net_out_data`, `latency_data` sparklines (`live_feed_scene.rs:52-54`)
- **Severity filter** — `severity_filter: Option<String>` with filter toggle (`live_feed_scene.rs:57, 166-179`)
- **Export to file** — `export_logs()` writes entries to timestamped file (`live_feed_scene.rs:798-813`)

### hud_demo_scene
- **Shield regen** — Auto-regenerates +2/tick when shield < 100 (`hud_demo_scene.rs:529-531`)
- **Particle effects** — Hit particles spawned at enemy positions (`hud_demo_scene.rs:508`)
- **Damage numbers** — Floating "+/-N" numbers on hit/damage (`hud_demo_scene.rs:38, 98`)

## Verification Results

| Command | Result |
|---------|--------|
| `cargo check --all-targets` | ✅ 0 errors |
| `cargo clippy --all-targets` | ✅ 0 warnings |
| `cargo test` | ✅ All tests pass |
| `cargo fmt -- --check` | ✅ No changes needed |

## Code Quality Fix

- **radio_scene.rs:113** — Removed unnecessary `.clone()` on `Cell<T>` which implements `Copy` (clippy warning)
