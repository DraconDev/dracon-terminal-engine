# Dracon Terminal Engine — Full Audit ✅

**Date:** 2026-05-20  
**Commit:** Current working tree

---

## Bug Fixes

### White Horizontal Lines in Showcase (2026-05-20)

**Root Cause:** `blit_to()` in `shared_helpers.rs` copied cells with `Color::Reset` background from input widgets (SearchInput, PasswordInput) into the showcase plane. Cells with `Color::Reset` rendered as the terminal's default background (white), causing horizontal lines.

**Trigger:** Typing in any text input field in a showcase scene (Login Screen, Theme Studio, etc.)

**Fix:** `blit_to()` now skips cells with `Color::Reset` bg:
```rust
if cell.bg == Color::Reset {
    continue;
}
```

**Why it works:**
- SearchInput/PasswordInput render text content with `fill_bg(theme.input_bg)` — all bg is a real color
- Empty cells outside the text content retain `Cell::default()` with `bg: Color::Reset`
- These unfilled cells should be transparent (show destination bg), not overwrite with terminal default
- `Color::Reset` semantically means "use terminal default" — which is wrong for nested widgets

**File changed:** `examples/showcase/scenes/shared_helpers.rs`

---

## Build Status

| Check | Status |
|-------|--------|
| `cargo clippy --lib --examples` | ✅ 0 warnings |
| `cargo clippy --tests` | ✅ 0 warnings |
| `cargo build --examples` | ✅ All build |
| `cargo test --lib` | ✅ 291 passed |
| `cargo test --doc` | ✅ 5 passed, 25 ignored |
| `cargo test --tests` | ✅ 26 passed |
| `cargo check` | ✅ Clean |

---

## Scene Coverage

**34 embedded scenes** — all registered in `is_embedded()` ✅

| Category | Count |
|----------|-------|
| **Total scenes** | 34 |
| **Tier 1 (upgraded)** | 21 (62%) |
| **Tier 1 (verified)** | 13 (38%) |
| **Missing from is_embedded()** | 0 |

**All 34 scenes verified with:**
- ✅ Help overlay (show_help + F1/Esc toggle)
- ✅ StatusBar or footer
- ✅ Theme propagation (on_theme_change)
- ✅ Keybinding system (KeybindingSet + actions)
- ✅ Mouse handler (handle_mouse)

---

## Test Warnings Fixed

| File | Before | After | Change |
|------|--------|-------|--------|
| `tests/theme_test.rs` | 2 warnings | 0 | Fixed `#[allow]` attr placement + deprecated field |
| `src/widgets/editor.rs` | 6 warnings | 0 | `field_reassign_with_default` → struct literal syntax |
| `src/widgets/editor_search.rs` | 4 warnings | 0 | `field_reassign_with_default` + `unused_mut` |

**12 test warnings eliminated.**

---

## Scene Line Counts

| File | Lines |
|------|-------|
| `accessibility_scene.rs` | 702 |
| `animation_scene.rs` | 633 |
| `modal_demo.rs` | 628 |
| `tooltip_scene.rs` | 618 |
| `command_palette_scene.rs` | 606 |
| `debug_overlay_scene.rs` | 590 |
| `progress_scene.rs` | 585 |
| `form_demo.rs` | 576 |
| `navigator_scene.rs` | 568 |
| `workshop_scene.rs` | 562 |
| `notification_center_scene.rs` | 549 |
| `rich_text_scene.rs` | 548 |
| `radio_scene.rs` | 548 |
| `metrics_hub_scene.rs` | 544 |
| `paint_scene.rs` | 539 |
| `tags_input_scene.rs` | 535 |
| `password_input_scene.rs` | 529 |
| `table_list_scene.rs` | 521 |
| `tree_navigator.rs` | 514 |
| `raycaster_scene.rs` | 513 |
| `color_picker_scene.rs` | 501 |
| `cell_pool_scene.rs` | 498 |
| `widget_gallery.rs` | 482 |
| `action_center_scene.rs` | 473 |
| `theme_switcher.rs` | 451 |
| `autocomplete_scene.rs` | 422 |
| `dev_console_scene.rs` | 394 |
| `control_panel_scene.rs` | 389 |
| `calendar_scene.rs` | 371 |
| `kanban_scene.rs` | 329 |
| `live_feed_scene.rs` | 319 |
| `note_editor_scene.rs` | 264 |
| `hud_demo_scene.rs` | 257 |
| `shared_helpers.rs` | 248 |
| `settings_scene.rs` | 230 |
| `mod.rs` | 35 |
| **Total** | **17,071** |

---

## Summary

**All targets compile clean with zero clippy warnings.** All tests pass. All 34 scenes are Tier 1 quality with help overlays, status bars/footers, theme propagation, keybindings, and mouse handlers.
