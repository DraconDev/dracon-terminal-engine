# Bug Fix TODO

## Fixed ✅

1. **Accessibility scene — can't type in text fields**
   - `KeyCode::Char(c)` now accepts SHIFT modifier for uppercase typing
   - `Backspace` guarded with `key.modifiers.is_empty()` to prevent Ctrl+Backspace
   - File: `examples/showcase/scenes/accessibility_scene.rs`

2. **Action Center — context menu mouse clicks didn't execute actions**
   - Added `on_select` callback with `Rc<RefCell<Option<String>>>` bridge pattern
   - Added `sync_action_bridge()` called after mouse/key events
   - Fixed borrow issues with block-scoped extraction of selected_id
   - File: `examples/showcase/scenes/action_center_scene.rs`

3. **Color Picker — arrow keys did nothing on first use**
   - `selected_slider` defaults to `Some(SliderKind::Hue)` in all 3 constructors
   - Previously required pressing Tab first to select a slider
   - File: `src/framework/widgets/color_picker.rs`

4. **Control Panel — Select widgets showed wrong value**
   - Added `Select::set_selected()` method to framework widget
   - `SelectState::next()/prev()` now sync the Select widget's selected index
   - Render now uses actual Select widget instead of manual value display
   - Files: `src/framework/widgets/select.rs`, `examples/showcase/scenes/control_panel_scene.rs`

5. **Autocomplete — dropdown not visible on scene load**
   - Added `Autocomplete::open_dropdown()` public method
   - Scene calls `open_dropdown()` on initialization
   - Files: `src/framework/widgets/autocomplete.rs`, `examples/showcase/scenes/autocomplete_scene.rs`

6. **Settings scene — `.unwrap()` on hardcoded regex patterns**
   - Replaced with `.expect("hardcoded regex ... is always valid")`
   - File: `examples/showcase/scenes/settings_scene.rs`

7. **Raycaster scene — potential OOB access on MAP**
   - Added `.clamp(0.0, (MAP_H-1) as f64)` before using player position as index
   - File: `examples/showcase/scenes/raycaster_scene.rs`

## Needs Runtime Testing 🔍

8. **Action Center — "failed to start"**
   - Code compiles, no obvious panics. Added on_select bridge for mouse clicks.
   - Needs terminal testing to confirm fix
   - File: `examples/showcase/scenes/action_center_scene.rs`

9. **Chat Client — "seemingly crashed"**
   - Code compiles, no obvious panics or unwraps
   - All index access bounds-checked
   - Needs terminal testing to reproduce
   - File: `examples/_apps/chat_client.rs`

10. **Autocomplete — "UI is a bit broken"**
    - Added `open_dropdown()` for initial visibility
    - Needs terminal testing to confirm fix
    - File: `examples/showcase/scenes/autocomplete_scene.rs`

## Medium Priority 📋

11. **Data showcase — "could use a UI update"**
    - Could improve: card visual design, category section headers, better preview content
    - Files: `examples/showcase/render.rs`, `examples/showcase/widget.rs`

## Build Status
- ✅ `cargo clippy --lib --examples` — 0 errors, 0 warnings
- ✅ `cargo test` — all pass

## Files Modified
- `examples/showcase/scenes/accessibility_scene.rs` — typing fix
- `examples/showcase/scenes/action_center_scene.rs` — on_select bridge, borrow fixes
- `examples/showcase/scenes/control_panel_scene.rs` — Select widget integration
- `examples/showcase/scenes/autocomplete_scene.rs` — open_dropdown on init
- `examples/showcase/scenes/settings_scene.rs` — unwrap → expect
- `examples/showcase/scenes/raycaster_scene.rs` — MAP bounds safety
- `src/framework/widgets/color_picker.rs` — default slider selection
- `src/framework/widgets/select.rs` — new `set_selected()` method
- `src/framework/widgets/autocomplete.rs` — new `open_dropdown()` method
