# Showcase Audit — Bug, Polish, and Categorization Report

**Date**: 2026-06-02
**Scope**: All 34 showcase scenes in `examples/showcase/scenes/`
**Method**: Static code review (no TTY available for runtime testing)

---

## Executive Summary

| Metric | Count |
|--------|-------|
| Total scenes | 34 |
| Scenes with help overlay | 34 (100%) |
| Scenes with status bar | 13 (38%) |
| Scenes with keybindings | 34 (100%) |
| Scenes with mouse handler | 34 (100%) |
| Scenes with key handler | 34 (100%) |
| Scenes with theme handler | 34 (100%) |
| Scenes with on_enter/on_exit | 11 (32%) |
| **Bugs found** | **8** |
| **Polish issues** | **3 categories** |
| **Recommended cuts** | **1–2 scenes** |
| **Recommended expansions** | **5–6 scenes** |

**Key finding**: The showcase is generally well-built. All scenes have help overlays, keybindings, and lifecycle methods. The main issues are 2-3 real bugs in `calendar_scene.rs`, DRY violations in `radio_scene.rs`, a redundant import in `raycaster_scene.rs`, and several scenes that are too simple or duplicate other scenes.

---

## 1. Scene-by-Scene Findings

### 1.1 Bugs Found

| # | Scene | Bug | Severity | Location |
|---|-------|-----|----------|----------|
| 1 | `calendar_scene.rs` | `mark_dirty` and `clear_dirty` are empty no-ops (no `self.dirty = true`) | Major | Lines 341-343 |
| 2 | `calendar_scene.rs` | Mouse click on sidebar events uses raw `EVENTS[event_idx]` index without accounting for `upcoming_events()` filter | Major | Lines 328-337 |
| 3 | `calendar_scene.rs` | Default date for upcoming events is hardcoded `"2026-05-17"` — filter is meaningless without selected date | Minor | Line 168 |
| 4 | `calendar_scene.rs` | Uses emoji icons (📅, 🔴, 🎉, 🔔) that may not render in all terminals | Minor | Lines 58-73 |
| 5 | `radio_scene.rs` | Duplicated label text: exists in both Radio constructor AND render's `label_text` match block | Minor | Lines 148-166 |
| 6 | `radio_scene.rs` | Unused `_radio: &Radio` parameter in `render_radio_group` — manually draws state instead of using the widget's render | Minor | Lines 114-166 |
| 7 | `raycaster_scene.rs` | Redundant `use dracon_terminal_engine::compositor::plane::Color;` on line 54 (Color is already in scope via `prelude::*`) | Minor | Line 54 |
| 8 | `settings_scene.rs` | 3 `expect()` calls with hardcoded regex messages — could panic if regex syntax changes | Minor | Lines 43-51 |

### 1.2 Calendar Scene Detail

**Bug 1**: Dirty tracking broken. In `calendar_scene.rs`:

```rust
fn mark_dirty(&mut self) {}
fn clear_dirty(&mut self) {}
```

Other scenes set `self.dirty = true/false`. This means:
- The scene never reports `needs_render() == true` unless called fresh
- Theme changes don't trigger re-render
- The SceneRouter may not properly track state

**Bug 2**: Event click index mismatch. In `handle_mouse`:

```rust
let event_idx = (row - 6) as usize;
if event_idx < EVENTS.len() {
    let event = &EVENTS[event_idx];  // BUG: uses raw index
    self.selected_date = Some(event.date.to_string());
}
```

But `render_sidebar` calls `self.upcoming_events(after)` which only returns the first 8 events. The visible event at position 2 is not `EVENTS[2]` — it's `EVENTS[2]` of the filtered list, which happens to be the same here but could differ if filtering changes.

### 1.3 Radio Scene Detail

The `render_radio_group` function manually draws all radio visuals instead of calling `radio.render(area)`. The Radio widgets are stored in Vec but never used for display — only for state (`select()`/`deselect()`). This is:
- Inconsistent with other scenes that use widget rendering
- Requires duplicating label text
- Makes the scene harder to maintain

**Fix**: Call `radio.render(area)` in the loop instead of manually drawing characters.

### 1.4 Raycaster Scene Detail

Line 54: `use dracon_terminal_engine::compositor::plane::Color;` is redundant because `framework::prelude::*` (line 8) already exports `Color`. This should generate a clippy `redundant_import` warning.

---

## 2. Polish Issues

### 2.1 Hardcoded Keys (not using KeybindingSet)

Several scenes use `KeyCode::Char('1')`, `KeyCode::Char('c')`, etc. instead of going through the `KeybindingSet`:

| Scene | Hardcoded Keys |
|-------|----------------|
| `radio_scene.rs` | '1', '2', '3' (quick-select), 'c' (clear) |
| `calendar_scene.rs` | 'c' (clear selection) |

**Impact**: Users who remap these keys in `dracon.toml` won't get the expected behavior. For showcase demos, consistency with the action system is important.

### 2.2 Missing Status Bars (21 of 34 scenes)

Scenes without status bars: calendar, raycaster, widget_gallery, radio, theme_switcher, paint, autocomplete, cell_pool, rich_text, tags_input, color_picker, tree_navigator, workshop, animation, debug_overlay, password_input, progress, notification_center, accessibility, modal_demo, tooltip

**Impact**: Status bars provide keyboard hints and live state feedback. Scenes without them rely on a footer line in the render — but the pattern is inconsistent.

### 2.3 Missing `on_enter`/`on_exit` Lifecycle Hooks (23 of 34 scenes)

Scenes without lifecycle hooks: calendar, raycaster, widget_gallery, radio, theme_switcher, paint, autocomplete, cell_pool, rich_text, tags_input, color_picker, tree_navigator, workshop, animation, debug_overlay, password_input, progress, notification_center, accessibility, modal_demo, tooltip

**Impact**: Scenes with `on_enter` can reset state or start timers when they become active. Without it, stale state may persist when re-entering a scene (e.g., after going back from another scene).

---

## 3. Scene Categorization

### 3.1 EXPAND — High-impact scenes worth investing in

These scenes are the showcase stars. They're already good but could be even more impressive:

| Scene | Why Expand | Specific Improvements |
|-------|-----------|----------------------|
| `live_feed_scene` | Recently enhanced, but the streaming text could be more realistic | Add network-style sparklines (latency, error rate), add filter by severity, add export-to-file |
| `hud_demo_scene` | Recently enhanced with enemies, but the game loop is static | Add real-time shield regen on tick, add particle effects, add damage numbers |
| `note_editor_scene` | Recently enhanced with tabs, but lacks syntax highlighting | Add Rust syntax highlighting via `TextEditorAdapter`, add file tree sidebar |
| `command_palette_scene` | IDE Lite is ambitious but only shows the palette | Add an actual mini-editor that responds to "New File", "Save File" commands |
| `metrics_hub_scene` | Real /proc data but basic layout | Add time-range selector, add per-process drill-down, add alerts |
| `paint_scene` | Pixel art canvas is fun but limited | Add layers, add selection tools, add export to ANSI art file |

### 3.2 KEEP — Solid scenes that work well

These scenes are good as-is. No changes needed unless polishing:

| Scene | Why Keep |
|-------|----------|
| `workshop_scene` | Excellent interactive widget playground |
| `widget_gallery` | Comprehensive widget showcase with 12+ widgets |
| `accessibility_scene` | Thorough accessibility demo with OSC 99 |
| `command_palette_scene` | Clean IDE-lite with menu integration |
| `debug_overlay_scene` | Good debugging visualization |
| `tree_navigator` | Rich file browser with icons and details |
| `kanban_scene` | Full drag-and-drop kanban board |
| `notification_center_scene` | Complete notification management |
| `table_list_scene` | Sortable table with detail panel |
| `navigator_scene` | Good navigation UI |
| `action_center_scene` | ContextMenu + ConfirmDialog + Toast |
| `dev_console_scene` | LogViewer + EventLogger |
| `theme_switcher` | Theme cycling demo |
| `animation_scene` | Animation framework demo |
| `raycaster` | 3D raycasting — impressive technical demo |
| `paint_scene` | Pixel art canvas |
| `cell_pool_scene` | Cell pool visualization |
| `autocomplete_scene` | Autocomplete widget |
| `tags_input_scene` | Tags input widget |
| `color_picker_scene` | Color picker |
| `rich_text_scene` | Rich text rendering |
| `progress_scene` | Progress indicators |
| `password_input_scene` | Login form |
| `tooltip_scene` | Tooltip demo |
| `modal_demo` | Modal + backdrop |
| `settings_scene` | Form + KeyValueGrid |

### 3.3 KEEP WITH POLISH — Needs minor fixes

| Scene | Issue | Fix |
|-------|-------|-----|
| `calendar_scene` | Bugs 1-4 above | Fix dirty tracking, fix event click index, replace emojis with ASCII icons |
| `radio_scene` | Bugs 5-6, missing status bar | Use Radio widget render, add status bar with key hints |
| `raycaster_scene` | Bug 7 (redundant import) | Remove redundant `use` statement |
| `form demo` | No status bar (recently enhanced with validation) | Add status bar with validation summary |
| `settings_scene` | 3 expect() calls | Replace with `ok()` and handle errors |

### 3.4 CONSIDER CUTTING — Too simple or duplicates

| Scene | Why Consider Cutting | Recommendation |
|-------|---------------------|----------------|
| `control_panel_scene` | Duplicates `settings_scene` (both are settings forms with Select/Toggle/Checkbox). No unique value. | **Cut** — keep only `settings_scene` |
| `note_editor_scene` | Despite recent enhancement, it's still primarily a single-editor demo. The `text_editor_demo` standalone and `ide` scene show more advanced editor features. | **Keep but de-emphasize** — it's a good simple editor demo |

---

## 4. Prioritized Recommendations

### 4.1 High Priority — Fix Bugs (1-2 hours)

1. **Fix `calendar_scene.rs` bugs**:
   - Add `self.dirty = true` in `mark_dirty()`, `self.dirty = false` in `clear_dirty()`
   - Fix sidebar event click to use the filtered/visible event list
   - Replace emoji icons with ASCII alternatives (or keep emoji with fallback)
   - Fix default date for upcoming events

2. **Fix `radio_scene.rs` DRY violation**:
   - Use `radio.render(area)` instead of manually drawing chars
   - Remove duplicated label text in render function

3. **Fix `raycaster_scene.rs` redundant import**:
   - Remove line 54: `use dracon_terminal_engine::compositor::plane::Color;`

### 4.2 Medium Priority — Polish (2-3 hours)

4. **Replace hardcoded keys with KeybindingSet** in calendar_scene and radio_scene

5. **Add status bars** to the most-visited scenes that lack them:
   - `calendar_scene` (most important — it has help but no status bar with hints)
   - `raycaster_scene` (movement keys are not visible)
   - `paint_scene` (tool palette is in content, but no key hints)
   - `workshop_scene`

6. **Add `on_enter`/`on_exit` lifecycle hooks** to scenes that need state reset:
   - `calendar_scene` (reset selected date)
   - `hud_demo_scene` (reset health/ammo when re-entering)
   - `note_editor_scene` (reset to first tab)

### 4.3 Low Priority — Strategic Expansions (future sessions)

7. **Cut `control_panel_scene`** — it's a duplicate of `settings_scene`
8. **Expand live_feed_scene** with network-style metrics
9. **Expand hud_demo_scene** with real-time game loop
10. **Expand note_editor_scene** with syntax highlighting

---

## 5. Per-Scene Scorecard

| Scene | Lines | Quality | Bugs | Polish | Action |
|-------|-------|---------|------|--------|--------|
| `settings_scene` | 310 | ★★★ | 0 | Minor | Polish (remove expect) |
| `dev_console_scene` | 406 | ★★★ | 0 | Good | Keep |
| `kanban_scene` | 469 | ★★★ | 0 | Good | Keep |
| `note_editor_scene` | 482 | ★★ | 0 | Good | Keep (consider expand) |
| `hud_demo_scene` | 525 | ★★ | 0 | Good | Keep (consider expand) |
| `calendar_scene` | 542 | ★★ | 4 | Needs work | Fix bugs |
| `control_panel_scene` | 557 | ★★ | 0 | OK | **Consider cutting** |
| `raycaster_scene` | 561 | ★★★ | 1 | Good | Fix import |
| `widget_gallery` | 583 | ★★★ | 0 | Good | Keep |
| `radio_scene` | 595 | ★ | 2 | Needs work | Fix DRY, add status bar |
| `theme_switcher` | 616 | ★★★ | 0 | Good | Keep |
| `paint_scene` | 616 | ★★★ | 0 | Good | Keep (consider expand) |
| `autocomplete_scene` | 627 | ★★ | 0 | Good | Keep |
| `cell_pool_scene` | 630 | ★★ | 0 | Good | Keep |
| `live_feed_scene` | 647 | ★★★ | 0 | Good | Keep (consider expand) |
| `rich_text_scene` | 659 | ★★ | 0 | Good | Keep |
| `tags_input_scene` | 703 | ★★ | 0 | Good | Keep |
| `color_picker_scene` | 710 | ★★ | 0 | Good | Keep |
| `action_center_scene` | 714 | ★★★ | 0 | Good | Keep |
| `navigator_scene` | 730 | ★★★ | 0 | Good | Keep |
| `metrics_hub_scene` | 761 | ★★★ | 0 | Good | Keep (consider expand) |
| `tree_navigator` | 767 | ★★★ | 0 | Good | Keep |
| `workshop_scene` | 769 | ★★★ | 0 | Good | Keep |
| `animation_scene` | 777 | ★★ | 0 | Good | Keep |
| `debug_overlay_scene` | 804 | ★★★ | 0 | Good | Keep |
| `password_input_scene` | 807 | ★★ | 0 | Good | Keep |
| `progress_scene` | 826 | ★★ | 0 | Good | Keep |
| `table_list_scene` | 831 | ★★★ | 0 | Good | Keep |
| `command_palette_scene` | 835 | ★★★ | 0 | Good | Keep (consider expand) |
| `form demo` | 837 | ★★★ | 0 | Good | Keep |
| `notification_center_scene` | 851 | ★★★ | 0 | Good | Keep |
| `accessibility_scene` | 856 | ★★★ | 0 | Good | Keep |
| `modal_demo` | 860 | ★★★ | 0 | Good | Keep |
| `tooltip_scene` | 862 | ★★ | 0 | Good | Keep |

---

## 6. Summary

**Key takeaways:**

1. **8 bugs found** — most are minor (DRY violations, redundant imports) but `calendar_scene` has 2 real bugs (dirty tracking broken, event click index wrong)

2. **Polish is generally good** — all 34 scenes have help overlays, keybindings, and proper lifecycle (key/mouse/theme handlers). The main gaps are status bars (only 38% have them) and `on_enter`/`on_exit` hooks (only 32% have them)

3. **1 scene should be cut**: `control_panel_scene` is a duplicate of `settings_scene`

4. **6 scenes are worth expanding** to make the showcase more impressive: live_feed, hud_demo, note_editor, command_palette, metrics_hub, paint

5. **No critical bugs** that would prevent the showcase from working — all issues are polish/quality improvements

**Time estimate to address all high-priority issues**: ~2-3 hours
**Time estimate for all recommended changes**: ~1 day

**Net recommendation**: Fix the 3 high-priority bugs (calendar, radio, raycaster), then optionally cut `control_panel_scene` and expand the top 2-3 showcase stars. The showcase is in good shape overall.
