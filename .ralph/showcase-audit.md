Implement showcase audit findings — create new scenes for undemonstrated widgets.

## This iteration: 2 new scenes

### Scene 1: "IDE Lite" — CommandPalette + MenuBar (~500 lines) ✅
- Created `examples/showcase/scenes/command_palette_scene.rs` (620 lines)
- MenuBar at top with File/Edit/View menus (real MenuBar widget)
- CommandPalette overlay (Ctrl+P) with 16 commands in 4 categories
- Action log showing executed commands with category badges + shortcuts
- Sidebar (toggleable with Ctrl+B) and minimap (toggleable)
- Full Scene trait, help overlay, Esc/BACK, mouse, theme propagation
- Registered in mod.rs, state.rs, data.rs

### Scene 2: "Server Dashboard" — Table + List (~450 lines) ✅
- Created `examples/showcase/scenes/table_list_scene.rs` (510 lines)
- Table widget with 5 sortable columns (PID, Name, CPU%, Memory, Status)
- List widget with category filter (all/system/network/shell/editor/build/browser/database)
- 18 process entries with realistic data
- Header click sorting with ▲/▼ indicators
- Detail panel for selected process
- Full Scene trait, help overlay, Esc/BACK, mouse, theme propagation
- Registered in mod.rs, state.rs, data.rs

## Acceptance:
- Both scenes compile with 0 clippy warnings
- cargo clippy --lib --examples passes
- cargo test passes
- Scenes registered in mod.rs + data.rs
- Both have help overlays, Esc/BACK, mouse handling
