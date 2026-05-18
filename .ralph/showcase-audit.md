Implement showcase audit findings — create new scenes for undemonstrated widgets.

## This iteration: 8 scenes + 1 fix total

### Scene 1: "IDE Lite" — CommandPalette + MenuBar ✅
### Scene 2: "Server Dashboard" — Table + List ✅
### Scene 3: "Settings Panel" — Form + KeyValueGrid ✅
### Scene 4: "Live Feed" — SplitPane + TabBar + StreamingText + Sparkline ✅
### Scene 5: "Action Center" — ContextMenu + ConfirmDialog + Toast ✅
### Scene 6: "Dev Console" — LogViewer + EventLogger + Label + Divider + WidgetInspector ✅
- `examples/showcase/scenes/dev_console_scene.rs` (300 lines)
- LogViewer with filter levels (ALL/DBG/INFO/WARN/ERR) via clickable filter bar
- EventLogger tracking all UI actions below the log viewer
- Label widget for title, Divider widgets for section separators
- WidgetInspector toggle (I key) showing widget hierarchy
- Space to add realistic log entries, C to clear
- Filter bar clickable with mouse, log viewer scrollable

### Fix: rich_text_scene scroll keys ✅

## Build: 0 clippy errors, 291+ tests pass, 28 scenes total

## Acceptance:
- Both scenes compile with 0 clippy warnings
- cargo clippy --lib --examples passes
- cargo test passes
- Scenes registered in mod.rs + data.rs
- Both have help overlays, Esc/BACK, mouse handling
