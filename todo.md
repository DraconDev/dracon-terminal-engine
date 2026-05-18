# Bug Fix TODO

## Fixed

- [x] **Accessibility scene — can't type in text fields**
  - `KeyCode::Char(c)` now accepts SHIFT modifier for uppercase typing
  - `Backspace` guarded with `key.modifiers.is_empty()` to prevent Ctrl+Backspace triggering delete
  - File: `examples/showcase/scenes/accessibility_scene.rs`

- [x] **Action Center — context menu mouse clicks didn't execute actions**
  - Added `on_select` callback with Rc<RefCell<Option<String>>> bridge pattern
  - Added `sync_action_bridge()` method called after mouse/key events
  - Fixed borrow issues with block-scoped extraction of selected_id
  - File: `examples/showcase/scenes/action_center_scene.rs`

- [x] **Color Picker — arrow keys did nothing on first use**
  - `selected_slider` defaults to `Some(SliderKind::Hue)` in all 3 constructors
  - Previously required pressing Tab first to select a slider before arrow keys worked
  - File: `src/framework/widgets/color_picker.rs`

- [x] **Control Panel — Select widgets showed wrong value**
  - Added `Select::set_selected()` method to framework
  - `SelectState::next()/prev()` now sync the Select widget's selected index
  - Render now uses actual Select widget instead of manual value display
  - File: `src/framework/widgets/select.rs`, `examples/showcase/scenes/control_panel_scene.rs`

- [x] **Autocomplete — dropdown not visible on scene load**
  - Added `Autocomplete::open_dropdown()` public method
  - Scene calls `open_dropdown()` on initialization so suggestions appear immediately
  - File: `src/framework/widgets/autocomplete.rs`, `examples/showcase/scenes/autocomplete_scene.rs`

## Needs Runtime Testing

- [ ] **Action Center — "failed to start"**
  - Code compiles and looks correct, no obvious panics
  - Added on_select bridge pattern for mouse click handling
  - Needs actual terminal testing to confirm fix
  - File: `examples/showcase/scenes/action_center_scene.rs`

- [ ] **Chat Client — "seemingly crashed"**
  - Code compiles and looks correct, no obvious panics or unwraps
  - All index access bounds-checked
  - Emoji chars may cause visual misalignment (multi-column chars)
  - Needs actual terminal testing to reproduce
  - File: `examples/_apps/chat_client.rs`

- [ ] **Autocomplete — "UI is a bit broken"**
  - Added `open_dropdown()` for initial visibility
  - Code structure looks correct
  - May be visual alignment issue with blit positions
  - Needs actual terminal testing to confirm fix
  - File: `examples/showcase/scenes/autocomplete_scene.rs`

## Medium (Visual/UX improvements)

- [ ] **Data showcase — "could use a UI update"**
  - Main showcase launcher (card grid, scrolling, layout)
  - Already has: smooth scrolling, proportional scrollbar, impressive-first ordering
  - Could improve: card visual design, category section headers, better preview content
  - Files: `examples/showcase/render.rs`, `examples/showcase/widget.rs`

## Build Status
- ✅ `cargo clippy --lib --examples` — 0 errors, 0 warnings
- ✅ `cargo test` — all pass
