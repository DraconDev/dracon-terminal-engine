# Bug Fix TODO

## Critical (Scenes broken/unusable)

- [x] **Accessibility scene — can't type in text fields**
  - Fixed: `KeyCode::Char(c)` now accepts SHIFT modifier for uppercase; Backspace guarded with `key.modifiers.is_empty()`
  - File: `examples/showcase/scenes/accessibility_scene.rs`

- [x] **Action Center — context menu mouse clicks didn't execute actions**
  - Fixed: Added `on_select` callback with Rc<RefCell<Option<String>>> bridge pattern
  - Added `sync_action_bridge()` method called after mouse/key events
  - Fixed borrow issues with block-scoped extraction of selected_id
  - File: `examples/showcase/scenes/action_center_scene.rs`

- [x] **Color Picker — arrow keys did nothing on first use**
  - Fixed: `selected_slider` defaults to `Some(SliderKind::Hue)` instead of `None`
  - Previously required pressing Tab first to select a slider before arrow keys worked
  - File: `src/framework/widgets/color_picker.rs`

## Needs Runtime Testing

- [ ] **Action Center — "failed to start" (shows "launching" then nothing)**
  - Code looks correct, no obvious panics, builds clean
  - May be a runtime issue (RefCell borrow at render, ContextMenu render, etc.)
  - Needs actual terminal testing to reproduce
  - File: `examples/showcase/scenes/action_center_scene.rs`

- [ ] **Chat Client — "seemingly crashed"**
  - Code looks correct, no obvious panics, builds clean
  - No unwrap/expect calls, all index access guarded
  - Emoji characters may cause rendering misalignment (multi-column chars in single-column cells)
  - Needs actual terminal testing to reproduce
  - File: `examples/_apps/chat_client.rs`

- [ ] **Autocomplete — "UI is a bit broken"**
  - Scene code looks structurally correct
  - Autocomplete widget properly handles typing → filter → dropdown
  - May be visual alignment issue with blit positions
  - Needs actual terminal testing to see the issue
  - File: `examples/showcase/scenes/autocomplete_scene.rs`

- [ ] **Control Panel — "admin selection is a bit off"**
  - Scene uses custom SelectState instead of real Select dropdown
  - Dropdown never opens — just cycles values with Space/Up/Down
  - May need to render actual Select widget instead of manual value display
  - File: `examples/showcase/scenes/control_panel_scene.rs`

## Medium (Visual/UX improvements)

- [ ] **Data showcase — "could use a UI update"**
  - Main showcase launcher (card grid, scrolling, layout)
  - Already has: smooth scrolling, proportional scrollbar, impressive-first ordering, categories
  - Could improve: card visual design, category section headers, better preview content
  - Files: `examples/showcase/render.rs`, `examples/showcase/widget.rs`

## Build Status
- ✅ `cargo clippy --lib --examples` — 0 errors, 0 warnings
- ✅ `cargo test` — all pass
