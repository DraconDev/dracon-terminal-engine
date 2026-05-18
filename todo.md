# Bug Fix TODO

## Critical (Scenes broken/unusable)

- [ ] **Accessibility scene — can't type in text fields**
  - `handle_key` has `KeyCode::Char(c) if key.modifiers.is_empty()` — uppercase letters fail when SHIFT modifier is set
  - Need to also accept `key.modifiers == KeyModifiers::SHIFT` for uppercase typing
  - Also check: Backspace guard should be `key.modifiers.is_empty()` too (currently unguarded — Ctrl+Backspace triggers it)
  - File: `examples/showcase/scenes/accessibility_scene.rs:529-550`

- [ ] **Action Center scene — fails to start (shows "launching" then nothing)**
  - Builds fine, API usage looks correct (ContextMenu, Toast, ConfirmDialog)
  - Likely runtime panic in `render()` — need to test
  - Check: `blit_to` calls with stale area, RefCell borrow issues at runtime
  - File: `examples/showcase/scenes/action_center_scene.rs`

- [ ] **Autocomplete scene — UI broken**
  - Need to check: Autocomplete widget API, scene integration, render blitting
  - File: `examples/showcase/scenes/autocomplete_scene.rs`

- [ ] **Color Picker scene — doesn't work**
  - Scene forwards keys/mouse to `picker.handle_key/handle_mouse` — may need `set_area()` call
  - Check: ColorPicker widget API, render blitting
  - File: `examples/showcase/scenes/color_picker_scene.rs`

## High (Usability issues)

- [ ] **Chat Client — seemingly crashes at runtime**
  - Builds fine (no clippy errors), likely runtime panic
  - Need to test and find crash source
  - File: `examples/_apps/chat_client.rs`

- [ ] **Control Panel — "admin selection is a bit off"**
  - Select dropdowns may have positioning/rendering issues
  - File: `examples/showcase/scenes/control_panel_scene.rs`

## Medium (Visual/UX improvements)

- [ ] **Data showcase — could use a UI update**
  - Main showcase launcher (card grid, scrolling, layout)
  - Files: `examples/showcase/state.rs`, `examples/showcase/render.rs`, `examples/showcase/widget.rs`

## Investigation Needed

- [ ] Run `cargo clippy --lib --examples` to check current build status
- [ ] Run `cargo test` to verify test suite
- [ ] Build and test each broken scene individually if possible
