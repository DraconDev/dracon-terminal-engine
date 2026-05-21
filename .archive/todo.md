# Bug Fix TODO

## CRITICAL FIX ✅

**11 embedded scenes couldn't launch** — `is_embedded()` in `state.rs` was missing them, so the showcase tried to launch them as external binaries (which don't exist).

**Root cause**: When 11 new embedded scenes were created, they were registered in `scene_router` but never added to the `is_embedded()` match list. The showcase's `launch_selected()` method checks `is_embedded()` to decide whether to push a scene to the router or spawn an external binary. Missing names → silent launch failure.

**Affected scenes** (ALL 11 were broken — showed "Launching..." then nothing):
- `action_center` ← **the one the user reported**
- `command_palette`
- `control_panel`
- `dev_console`
- `hud_demo`
- `live_feed`
- `metrics_hub`
- `navigator`
- `note_editor`
- `settings_panel`
- `table_list`

**Fix**: Added all 11 missing names to `is_embedded()` in `examples/showcase/state.rs`.

## Previously Fixed (from audit)

1. Accessibility typing (SHIFT modifier, Backspace guard)
2. Action Center context menu on_select bridge
3. Color Picker default slider selection
4. Control Panel Select::set_selected() + index sync
5. Autocomplete open_dropdown() on init
6. Notification Center area.set() in render
7. Raycaster footer + MAP bounds safety
8. Settings scene .unwrap() → .expect()
9. File manager Option::take() unwrap → match
10. Plugin demo / cell pool unwraps → .expect()
11. All #[allow(dead_code)] removed

## Build Status
- ✅ `cargo clippy --lib --examples` — 0 errors, 0 warnings
- ✅ `cargo test` — all pass
- ✅ 0 production `.unwrap()` calls
