# Manual Testing Report: Dracon Terminal Engine

## Executive Summary

This report identifies features across the dracon-terminal-engine project that are **NOT covered by automated tests** and **require manual verification**. While the project has extensive unit tests (70+ test files, 3,200+ test cases), many interactive, visual, and integration-level features can only be validated through manual testing in a real terminal environment.

---

## 1. EXAMPLE APPS — Interactive Features Not Unit Tested

### 1.1 Full Application Examples (`examples/_apps/`)

| Example | Key Untested Features |
|---------|----------------------|
| **system_monitor** | Real `/proc` filesystem reading (CPU, memory, disk, network, processes), sparkline history rendering, process tree view with connector rendering, tree/flat view toggle, live data updates every second, 15-theme cycling, hover highlighting on process list |
| **file_manager** | Real filesystem directory traversal, file type icons (Nerd Font symbols), breadcrumb click navigation, SplitPane divider drag resize, async directory loading (with spinner animation), create/delete/rename file operations, context menu actions, Toast notifications, mouse click on tree nodes |
| **dashboard_builder** | Real-time system metric collection, sparkline rendering with color-coded thresholds, process table selection and scrolling, pause/resume live updates, draggable panel resize (SplitPane), 20-theme cycling |
| **chat_client** | Simulated chat message rendering, message input and sending, scrollback buffer, user list sidebar |
| **ide.rs** | Multi-tab editor interface, TabBar mouse click switching, MenuBar dropdown menus, CommandPalette (Ctrl+P) overlay, SearchInput overlay, Breadcrumbs click navigation, Tooltip hover timing, ContextMenu right-click, Profiler overlay (F12), Toast notifications, file open/save dialogs |
| **git_tui.rs** | Real `git status/log/branch` command execution, staging/unstaging files, diff view rendering, branch checkout, tab switching (Status/Log/Diff/Branches), 20-theme cycling, Toast notifications for git operations |
| **sqlite_browser.rs** | Real SQLite database queries, table schema display, query result rendering, SQL syntax highlighting |

**Test Coverage:**
- `example_smoke_test.rs` — Only verifies these examples **compile** (build test)
- `example_quit_test.rs` — Only tests `q` quit, `?` help toggle, and Ctrl+C (all `#[ignore]` due to TTY requirement)
- **No tests for**: mouse interactions, theme cycling, drag-and-drop, real data reading, overlays, context menus, async operations

### 1.2 Cookbook Examples (`examples/_cookbook/`)

| Example | Key Untested Features |
|---------|----------------------|
| **widget_gallery** | Interactive widget showcase: Checkbox toggle, Radio selection, Slider drag, Spinner animation, Toggle switch, Select dropdown, Search input typing, Progress bar animation, Button click feedback |
| **tree_navigator** | Tree expand/collapse, node selection, mouse click navigation, right-click context menu |
| **log_monitor** | Streaming log tailing, log level filtering, auto-scroll behavior |
| **tabbed_panels** | Tab switching via click and keyboard, panel content rendering |
| **data_table** | Sortable column headers (click to sort), row selection, hover highlighting |
| **split_resizer** | Drag-to-reside split panes, orientation switching |
| **menu_system** | MenuBar hover dropdowns, nested menu navigation, menu item click |
| **debug_overlay** | Debug info overlay toggle, FPS counter, frame timing display |
| **rich_text** | Rich text rendering with inline styles, markdown-like formatting |
| **calendar** | Calendar grid rendering, date selection, month navigation |
| **autocomplete** | Type-ahead suggestions, arrow navigation through suggestions, Enter to accept |
| **form_validation** | Form field validation, error message display, focus navigation |
| **command_bindings** | Custom keybinding configuration, action mapping |
| **cell_pool** | Cell pool performance optimization demonstration |
| **accessibility** | Screen reader compatibility, focus indicators, high-contrast mode |
| **notification_center** | Toast stacking, notification dismissal, notification history |
| **stat_widget_plugin** | Plugin system integration, dynamic widget loading |

**Test Coverage:**
- Only `example_smoke_test.rs` compile checks exist
- No interactive or behavioral tests for any cookbook example

### 1.3 Standalone Examples

| Example | Key Untested Features |
|---------|----------------------|
| **showcase** (main launcher) | Grid card navigation (arrows), search filtering (`/`), category cycling (Tab), theme cycling (`t`), debug overlay (`d`), preview modal (Space), primitives bar interaction (1-5), right-click context menu, binary example spawning, theme return file mechanism, FPS counter display, returned-from toast |
| **showcase scenes** (11 scenes) | Embedded scene transitions (Fade/Slide), theme propagation to scenes, `B`/`Esc` back navigation, scene-specific interactions (widget gallery, form demo, tree navigator, calendar, etc.) |
| **desktop.rs** | Raw terminal desktop simulation, icon rendering, window management mock |
| **game_loop.rs** | Raw terminal game loop, frame timing, input polling |
| **input_debug.rs** | Real-time input event display, mouse coordinate tracking, key code visualization |
| **modal_demo.rs** | Modal overlay rendering, backdrop dimming, modal dismiss behavior |
| **theme_switcher.rs** | Interactive theme preview, theme cycling with visual feedback |
| **text_editor_demo.rs** | TextEditor widget: cursor movement, selection, syntax highlighting, undo/redo, mouse click/drag selection, scrollbar interaction, line numbers, word wrap, find/replace |
| **todo_app.rs** | Task list management, add/complete/delete tasks, task filtering |
| **tutorial_app.rs** | Interactive tutorial steps, guided navigation |
| **event_bus_demo.rs** | Event bus pub/sub, cross-widget communication |
| **network_client.rs** | Network connection status, request/response display |
| **scene_router_demo.rs** | Scene push/pop navigation, transition animations (Fade, SlideLeft, SlideRight, SlideUp, SlideDown), lifecycle hooks (on_enter/on_exit/on_pause/on_resume) |
| **plugin_demo.rs** | Dynamic plugin loading, plugin widget rendering |
| **framework_*** | Various framework feature demonstrations |

**Test Coverage:**
- `showcase_smoke_test.rs` — Spawns showcase, verifies it doesn't crash within 3.5 seconds (`#[ignore]`)
- `editor_smoke_test.rs` — Spawns text_editor_demo, allows exit code 0 or 1 (`#[ignore]`)
- No tests for scene transitions, theme propagation, embedded scene interactions, or any example-specific features

---

## 2. MOUSE HANDLING — Widget Interactions Not Unit Tested

### 2.1 Widgets with Mouse Support

The following widgets implement `handle_mouse()` but mouse interactions are only tested at the unit level for a few widgets:

| Widget | Mouse Features | Test Coverage |
|--------|---------------|---------------|
| **Button** | Hover highlighting (`hovered` state), click activation | `button_test.rs` — basic click test exists |
| **Checkbox** | Hover highlighting, click to toggle | No mouse interaction tests |
| **Toggle** | Hover highlighting, click to toggle | No mouse interaction tests |
| **Radio** | Hover highlighting, click to select | No mouse interaction tests |
| **Slider** | Drag thumb to change value, click track to jump | `widget_slider_test.rs` — basic tests exist |
| **Select** | Hover over options, click to select, dropdown open/close | No mouse interaction tests |
| **SearchInput** | Click to focus, cursor positioning | No mouse interaction tests |
| **PasswordInput** | Click to focus, cursor positioning | No mouse interaction tests |
| **List** | Hover row highlighting, click to select, right-click context menu, drag-and-drop reordering, scroll wheel | `widget_tests.rs` — basic list tests; no mouse tests |
| **Table** | Hover row highlighting, click header to sort, right-click context menu, drag-and-drop column reordering, scroll wheel | `table_sort_persistence_test.rs` — sort logic only; no mouse tests |
| **Tree** | Hover node highlighting, click to expand/collapse, right-click context menu, drag-and-drop node reordering, scroll wheel | `tree_widget_test.rs` — basic tree tests; no mouse tests |
| **TabBar** | Hover tab highlighting, click to switch tabs | No mouse interaction tests |
| **SplitPane** | Drag divider to resize, hover divider cursor change | `splitpane_test.rs` — basic layout tests; no drag tests |
| **MenuBar** | Hover to open dropdown, click menu item | `menu_test.rs` — basic menu tests; no mouse tests |
| **Breadcrumbs** | Click breadcrumb segment to navigate | `breadcrumbs_test.rs` — basic render tests; no click tests |
| **ContextMenu** | Click outside to dismiss, click item to select | `context_menu_test.rs` — basic tests; no mouse dismiss tests |
| **Modal** | Click outside to dismiss, click buttons | `modal_widget_test.rs` — basic tests; no mouse dismiss tests |
| **ConfirmDialog** | Click OK/Cancel buttons | `widget_confirm_dialog_test.rs` — basic tests; no mouse tests |
| **Kanban** | Drag cards between columns, hover card highlighting | **No tests at all** |
| **ProgressRing** | Click to toggle, drag to adjust | No mouse interaction tests |
| **ColorPicker** | Click/drag color sliders, type hex value | No tests at all |
| **Calendar** | Click date to select, month navigation buttons | No tests at all |
| **TagsInput** | Click tag to remove, click input to focus | No tests at all |
| **Sparkline** | Hover for tooltip/data point | No tests at all |
| **NotificationCenter** | Click to dismiss, hover highlighting | No tests at all |
| **CommandPalette** | Click item to execute, click outside to dismiss, scroll wheel | `command_palette_test.rs` — basic tests; no mouse tests |
| **Autocomplete** | Click suggestion to accept, hover highlighting | No tests at all |
| **TextEditor** (standalone) | Click to place cursor, drag to select, double-click to select word, triple-click to select line, right-click context menu, scroll wheel, scrollbar drag | `text_editor_test.rs` — extensive keyboard tests; minimal mouse tests |

### 2.2 Mouse Features Requiring Manual Testing

- **Hover state rendering**: Background color change on mouse hover (`hover_bg`)
- **Focus state rendering**: Background/border change on focus (`focus_bg`, `focus_border`)
- **Drag-and-drop ghost rendering**: Visual feedback during drag operations
- **Scroll wheel behavior**: Smooth scrolling in lists, tables, trees, text editor
- **Right-click context menus**: Positioning, item selection, dismissal
- **Double/triple click detection**: Word/line selection in TextEditor
- **Cursor style changes**: Resize cursor on SplitPane divider

---

## 3. KEYBOARD SHORTCUTS — Per-Example Shortcuts Not Tested

### 3.1 Config-Driven Keybindings (`KeybindingSet`)

The framework uses `KeybindingSet::from_config(&resolve_keybindings())` to load user-configurable keybindings from `dracon.toml`. While the keybinding system has unit tests (`keybindings.rs`), **per-example shortcut behavior is not tested**.

### 3.2 Standard Actions (from `keybindings.rs`)

| Action | Default | Examples Using It | Tested? |
|--------|---------|-------------------|---------|
| `QUIT` | `ctrl+q` | All examples | Partial (quit tests exist but `#[ignore]`) |
| `HELP` | `f1` | Most examples | Partial (help toggle tests exist but `#[ignore]`) |
| `BACK` | `esc` | Most examples | No |
| `THEME` | `ctrl+t` | Many examples | No |
| `SEARCH` | `ctrl+f` | IDE, file_manager | No |
| `NEW` | `ctrl+n` | IDE | No |
| `CLOSE` | `ctrl+w` | IDE | No |
| `SAVE` | `ctrl+s` | IDE, text_editor_demo | No |
| `COPY` | `ctrl+c` | IDE | No |
| `PASTE` | `ctrl+v` | IDE | No |
| `CUT` | `ctrl+x` | IDE | No |
| `DELETE` | `delete` | file_manager, todo_app | No |
| `REFRESH` | `f5` | git_tui, system_monitor | No |
| `PAUSE` | `ctrl+p` | dashboard_builder | No |

### 3.3 Example-Specific Hardcoded Shortcuts

| Example | Hardcoded Shortcut | Action |
|---------|-------------------|--------|
| **git_tui** | `1/2/3/4` | Switch views |
| **git_tui** | `d` | View diff |
| **git_tui** | `r` | Refresh |
| **file_manager** | `n` | New file |
| **file_manager** | `f` | New folder |
| **file_manager** | `d` | Delete |
| **file_manager** | `m` | Rename |
| **file_manager** | `r` | Refresh |
| **file_manager** | `l` | Async load |
| **dashboard_builder** | `p` | Pause/resume |
| **dashboard_builder** | `r` | Force refresh |
| **system_monitor** | `t` | Cycle theme |
| **showcase** | `/` | Focus search |
| **showcase** | `Tab` | Cycle categories |
| **showcase** | `d` | Toggle debug overlay |
| **showcase** | `Space` | Preview modal |
| **showcase** | `1-5` | Primitives bar |
| **widget_gallery** (scene) | `↑↓←→` | Navigate widgets |
| **widget_gallery** (scene) | `Enter` | Activate widget |
| **text_editor_demo** | `Ctrl+Z` / `Ctrl+Y` | Undo/redo |
| **text_editor_demo** | `Ctrl+F` | Find |
| **text_editor_demo** | `Ctrl+H` | Replace |
| **ide** | `Ctrl+O` | Open file |
| **ide** | `Ctrl+G` | Go to line |
| **ide** | `Ctrl+P` | Command palette |
| **ide** | `F12` | Profiler overlay |

**None of these example-specific shortcuts are tested.**

---

## 4. THEME SWITCHING BEHAVIOR

### 4.1 What's Tested

- `theme_test.rs` — Theme color values for all 21 themes
- `theme_validation_test.rs` — No black background holes in widgets
- `theme_propagation_test.rs` — `App::set_theme()` calls `on_theme_change()` on all widgets
- `widget_snapshot_tests.rs` — Widget render snapshots with default theme

### 4.2 What's NOT Tested (Requires Manual Verification)

| Feature | Description | Why Manual? |
|---------|-------------|-------------|
| **Visual theme cycling** | Cycling through 21 themes and verifying visual appearance | Requires human eye for color correctness |
| **Theme inheritance** | `DTRON_THEME` env var propagation from showcase to child examples | Cross-process env var handling |
| **Theme return file** | `DTRON_THEME_FILE` written by child example on exit, read by showcase | File-based IPC across process boundaries |
| **Theme transition smoothness** | No flickering or black holes during theme switch | Visual rendering artifact |
| **Per-widget theme updates** | All child widgets update colors correctly (no stale colors) | Visual verification required |
| **Pattern 2 theme sync** | `current_theme()` override syncs local theme back to framework | Requires integrated app testing |
| **Light theme readability** | Text remains readable on light backgrounds | Visual contrast check |
| **High-contrast theme** | Accessibility-focused theme visibility | Accessibility verification |
| **Terminal default inheritance** | `Color::Reset` inheritance for StatusBar | Terminal-specific behavior |
| **Syntax highlighting themes** | Syntect theme matching with Dracon themes | Visual color coordination |

---

## 5. CONDITIONAL COMPILATION FEATURES (`cfg` flags)

### 5.1 Features Defined in `Cargo.toml`

| Feature | Code Paths | Tested? |
|---------|-----------|---------|
| **`async`** | `src/input/async_reader.rs`, `on_mount_async()`, `on_unmount_async()`, `subscribe_once_async()`, `tokio` integration | **No tests** — entire async input system untested |
| **`tracing`** | Debug logging in app.rs, widgets, framework modules | **No tests** — tracing instrumentation untested |
| **`debug_events`** | Event logging to stderr in app.rs, logging.rs | **No tests** — debug event output untested |

### 5.2 Platform-Specific Code

| Platform | Code | Tested? |
|----------|------|---------|
| **`cfg(target_os = "linux")`** | `/proc` filesystem reading in `system.rs`, `system_monitor.rs`, `dashboard_builder.rs` | **Partial** — unit tests mock some data, but real /proc reading is untested |
| **`cfg(not(target_os = "windows"))`** | `libc` dependency for Unix-specific terminal operations | **No tests** on non-Linux Unix systems |

### 5.3 Async Feature Untested Areas

- `AsyncReader::spawn()` — Async stdin reading with tokio
- `AsyncReader::spawn_with_shutdown()` — Graceful async reader shutdown
- `Widget::on_mount_async()` — Async widget initialization
- `Widget::on_unmount_async()` — Async widget cleanup
- `EventBus::subscribe_once_async()` — Async event callbacks
- File manager async directory loading (`FsNode::read_dir` in async context)

---

## 6. DRAG-AND-DROP, ANIMATIONS, SCENE TRANSITIONS

### 6.1 Drag-and-Drop

| Component | Unit Tests | Integration Tests | Manual Testing Needed |
|-----------|-----------|-------------------|----------------------|
| **DragManager** | `dragdrop_test.rs` — 11 tests for basic drag lifecycle | None | Drag ghost visual rendering, drag over multiple targets, cancel during drag, drag-and-drop in actual widgets (Tree, Table, List, Kanban) |
| **Tree drag** | None | None | Drag node to reorder, drop indicator positioning, drag ghost label |
| **Table drag** | None | None | Drag column to reorder, drop target highlighting |
| **List drag** | None | None | Drag item to reorder, drop position indicator |
| **Kanban drag** | None | None | Drag card between columns, column drop target highlighting |
| **SplitPane drag** | None | None | Drag divider to resize, real-time resize feedback |
| **ColorPicker drag** | None | None | Drag color sliders |
| **Slider drag** | `widget_slider_test.rs` — basic tests | None | Drag thumb smoothly, jump-to-click |

### 6.2 Animations

| Component | Unit Tests | Integration Tests | Manual Testing Needed |
|-----------|-----------|-------------------|----------------------|
| **AnimationManager** | `animation_boundary_test.rs` — 13 tests, `animation.rs` — 10 tests | None | Visual smoothness, frame rate consistency, animation glitching |
| **Spinner** | `widget_gauge_test.rs` — spinner frame tests | None | Smooth frame cycling animation |
| **ProgressBar/Ring** | `gauge_test.rs` — basic tests | None | Smooth progress animation, color transitions |
| **Toast fade** | `toast_test.rs` — basic tests | None | Fade-in/fade-out visual smoothness |
| **Showcase card animations** | None | None | Card hover animation, selection phase animation |

### 6.3 Scene Transitions

| Transition | Unit Tests | Integration Tests | Manual Testing Needed |
|-----------|-----------|-------------------|----------------------|
| **Fade** | `scene_router_test.rs` — lifecycle hooks only | None | Visual fade smoothness, no flickering |
| **SlideLeft** | None | None | Smooth slide animation, no tearing |
| **SlideRight** | None | None | Smooth slide animation, no tearing |
| **SlideUp** | None | None | Smooth slide animation, no tearing |
| **SlideDown** | None | None | Smooth slide animation, no tearing |
| **Transition timing** | None | None | Duration consistency, no frame drops |
| **Scene lifecycle** | `scene_router_test.rs` — hooks tested | None | `on_enter`/`on_exit`/`on_pause`/`on_resume` timing with transitions |

---

## 7. VISUAL/RENDERING FEATURES NOT EASILY UNIT TESTED

### 7.1 Rendering Artifacts

| Feature | Why Untested | Manual Check |
|---------|-------------|--------------|
| **Transparency handling** | `Cell::transparent` field compositing | Overlapping widgets show correct blending |
| **Z-index layering** | Plane stacking order | Widgets render in correct depth order |
| **Background fill** | `plane.fill_bg(theme.bg)` | No black holes where `Color::Reset` shows through |
| **Border rendering** | Corner characters, line continuity | Rounded corners (`╭╮╰╯`) and lines (`─│`) connect correctly |
| **Scrollbar thumb** | Proportional height/position calculation | Thumb size and position reflect content correctly |
| **Text truncation** | Ellipsis insertion for overflow | Text truncates gracefully with `…` |
| **Unicode width** | `UnicodeWidthStr` for CJK/emoji | Wide characters don't break layout |
| **Nerd Font icons** | File type icons in file_manager | Icons render correctly in supported terminals |

### 7.2 Real-Time Data Visualization

| Feature | Examples | Manual Check |
|---------|----------|--------------|
| **Sparkline graphs** | system_monitor, dashboard_builder | Smooth curve rendering, color-coded thresholds |
| **Gauge/progress** | system_monitor, widget_gallery | Fill level accuracy, color transitions |
| **Live process list** | system_monitor | Auto-refresh without flickering, selection persistence |
| **Network metrics** | system_monitor, dashboard_builder | RX/TX rate accuracy, sparkline history |
| **CPU/memory bars** | system_monitor, dashboard_builder | Bar width accuracy, warning/critical colors |

### 7.3 Terminal Compatibility

| Feature | Manual Check Required |
|---------|----------------------|
| **Mouse support** | Terminal must report mouse events (xterm, iTerm2, etc.) |
| **True color** | 24-bit color rendering in supported terminals |
| **256-color fallback** | Ansi color approximation in limited terminals |
| **Alternate screen buffer** | Clean exit restoring original terminal state |
| **Raw mode** | Input handling without line buffering |
| **Resize handling** | Responsive reflow on terminal resize |
| **Focus events** | Focus gain/loss reporting |
| **Bracketed paste** | Paste mode for large text input |

### 7.4 Hover and Focus Visual States

| Widget | Hover Effect | Focus Effect | Tested? |
|--------|-------------|--------------|---------|
| Button | `hover_bg` background change | N/A | No visual tests |
| Checkbox | `hover_bg` background change | N/A | No visual tests |
| Toggle | `hover_bg` background change | N/A | No visual tests |
| Radio | `hover_bg` background change | N/A | No visual tests |
| Select | `hover_bg` for dropdown items | N/A | No visual tests |
| List | `hover_bg` for hovered row | N/A | No visual tests |
| Table | `hover_bg` for hovered row | N/A | No visual tests |
| Tree | `hover_bg` for hovered node | N/A | No visual tests |
| TabBar | `hover_bg` + bold for hovered tab | N/A | No visual tests |
| CommandPalette | `hover_bg` for hovered item | N/A | No visual tests |
| SearchInput | N/A | `focus_bg` + underline | No visual tests |
| PasswordInput | N/A | `focus_bg` + underline | No visual tests |
| TextEditor | N/A | Cursor blink, selection highlight | No visual tests |
| Form | N/A | `focus_bg` for entire focused row | No visual tests |

---

## 8. SPECIFIC MANUAL TEST CHECKLIST

### 8.1 Must Test (Critical User Paths)

1. **Showcase Launcher**
   - [ ] Arrow navigation through grid cards
   - [ ] `/` search filtering
   - [ ] `Tab` category cycling
   - [ ] `t` theme cycling (all 21 themes)
   - [ ] `Enter` to launch example
   - [ ] `Space` for preview modal
   - [ ] `d` debug overlay toggle
   - [ ] Right-click context menu
   - [ ] `q` quit
   - [ ] Theme propagation to launched examples
   - [ ] Theme return from child examples

2. **IDE Example**
   - [ ] `Ctrl+T` new tab
   - [ ] `Ctrl+W` close tab
   - [ ] `Ctrl+O` open file dialog
   - [ ] `Ctrl+S` save file
   - [ ] `Ctrl+F` search toggle
   - [ ] `Ctrl+P` command palette
   - [ ] `F12` profiler overlay
   - [ ] Mouse click tab switching
   - [ ] Mouse click menu items
   - [ ] Tooltip hover timing
   - [ ] Context menu right-click
   - [ ] Breadcrumb click navigation

3. **File Manager**
   - [ ] Tree navigation (arrows, Enter, Esc)
   - [ ] Mouse click to expand/collapse
   - [ ] Mouse click to select file
   - [ ] SplitPane drag resize
   - [ ] `n` new file, `f` new folder
   - [ ] `d` delete with confirmation
   - [ ] `m` rename
   - [ ] `l` async load with spinner
   - [ ] Breadcrumb click navigation
   - [ ] Right-click context menu

4. **System Monitor**
   - [ ] Live data updates (CPU, memory, disk, network)
   - [ ] Process list navigation
   - [ ] `t` theme cycling
   - [ ] `?` help overlay
   - [ ] Mouse hover on process rows
   - [ ] Scroll wheel on process list

5. **Text Editor Demo**
   - [ ] Mouse click to place cursor
   - [ ] Mouse drag to select text
   - [ ] Double-click to select word
   - [ ] Triple-click to select line
   - [ ] Scroll wheel
   - [ ] `Ctrl+Z` / `Ctrl+Y` undo/redo
   - [ ] `Ctrl+F` find
   - [ ] `Ctrl+H` replace
   - [ ] Syntax highlighting accuracy

### 8.2 Should Test (Important Features)

6. **Git TUI**
   - [ ] `1/2/3/4` view switching
   - [ ] `Enter` stage/unstage
   - [ ] `d` diff view
   - [ ] `t` theme cycling
   - [ ] Real git data display

7. **Dashboard Builder**
   - [ ] `p` pause/resume updates
   - [ ] `r` force refresh
   - [ ] SplitPane drag resize
   - [ ] `t` theme cycling
   - [ ] Process table navigation

8. **Widget Gallery (Showcase Scene)**
   - [ ] Checkbox toggle (mouse + Enter)
   - [ ] Radio selection (mouse + Enter)
   - [ ] Slider drag (mouse)
   - [ ] Toggle switch (mouse + Enter)
   - [ ] Select dropdown (mouse + Enter)
   - [ ] Search input typing
   - [ ] Button click feedback
   - [ ] Spinner animation

9. **Scene Router Demo**
   - [ ] Scene push/pop navigation
   - [ ] Fade transition smoothness
   - [ ] Slide transitions (all 4 directions)
   - [ ] `B`/`Esc` back navigation

10. **Drag-and-Drop Examples**
    - [ ] Kanban card drag between columns
    - [ ] Tree node reorder drag
    - [ ] Table column reorder drag
    - [ ] List item reorder drag

### 8.3 Nice to Test (Edge Cases)

11. **Theme Switching**
    - [ ] All 21 themes render correctly
    - [ ] Light theme readability
    - [ ] High-contrast accessibility
    - [ ] Theme inheritance via `DTRON_THEME`

12. **Terminal Compatibility**
    - [ ] Resize handling
    - [ ] Mouse in different terminals (xterm, iTerm2, Windows Terminal)
    - [ ] True color vs 256-color fallback
    - [ ] Alternate screen buffer cleanup

13. **Async Features (with `--features async`)**
    - [ ] Async input reader
    - [ ] Async widget mount/unmount
    - [ ] Async event bus subscriptions
    - [ ] File manager async directory loading

14. **Debug Features**
    - [ ] `--features debug_events` event logging
    - [ ] `--features tracing` structured logging
    - [ ] Debug overlay (`d` in showcase)

---

## 9. SUMMARY BY CATEGORY

| Category | Unit Test Coverage | Manual Testing Needed |
|----------|-------------------|----------------------|
| Example app compilation | 100% (smoke tests) | None |
| Example app quit behavior | Partial (`#[ignore]`) | All interactive features |
| Widget rendering | Good (snapshot tests) | Hover/focus states, animations |
| Widget keyboard input | Good (most widgets) | Example-specific shortcuts |
| Widget mouse input | Minimal | All mouse interactions |
| Theme system | Good (21 themes tested) | Visual cycling, propagation |
| Drag-and-drop | Basic (DragManager only) | All widget integrations |
| Animations | Good (AnimationManager) | Visual smoothness |
| Scene transitions | Minimal | All transition types |
| Real-time data | None | system_monitor, dashboard_builder |
| Async features | **None** | All async code paths |
| Platform-specific | Partial (Linux /proc) | Non-Linux platforms |
| Terminal compatibility | None | Resize, mouse, colors |
| Accessibility | None | Screen reader, high-contrast |

---

*Report generated from analysis of 70+ test files, 42 example files, and 180+ source files in the dracon-terminal-engine project.*
