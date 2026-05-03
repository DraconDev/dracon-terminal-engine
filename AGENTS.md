# Dracon Terminal Engine — Agent Notes

## Vision

**GUI-grade terminal applications.** Persistent, visible, mouse-friendly, composable widgets.

Not "CLI+" (hotkey-centric like vim/Helix). Not an editor competitor.
More like a GUI that happens to run in a terminal.

## Why TUI Over GUI?

### Deployment Advantages
- **Universal**: Runs on VPS, SSH, containers, CI, embedded — anywhere with a terminal
- **Zero user dependencies**: No browser, no runtime, no permissions, no install
- **Single binary**: Ships as one executable, instant startup
- **Cross-platform without bridge hell**: No Tauri/Dioxus/egui platform issues, no browser bugs, no lag

### UX Advantages Over CLI
- **Persistent state**: Don't re-run commands to see output
- **Visible structure**: Panels, trees, forms — not just scrolling text
- **Mouse-friendly**: Click, drag, scroll — natural interactions
- **Composability**: Mix widgets (list + editor + form) freely

## TextEditor Scope

TextEditor is a **view/edit widget** for composing into larger applications:
- File managers: view/edit config files
- Chat UIs: edit messages
- Forms: text input fields
- Log viewers: search, filter, navigate

**NOT a vim/Helix competitor.** Not a modal editor. Not LSP-powered.

Single cursor + selection is sufficient. Framework integration is the priority.

## Architecture Principles

| Principle | Meaning |
|-----------|---------|
| Widgets own state | TextEditor manages its own lines, cursor, selection |
| App owns composition | App manages widgets via registry, z-index, focus |
| Mouse-first | Widgets respond to clicks, not just keys |
| Keyboard-enhanced | Navigation shortcuts exist but aren't required |
| Terminal as universal target | No platform-specific code, no external dependencies |

## TextEditor Public API

```rust
// Creation
TextEditor::new()                        // Empty editor
TextEditor::with_content("...")         // From string
TextEditor::open(&path)                 // From file (loads .undo too)

// File I/O
editor.save()                           // Save to current path
editor.save_as(&path)                   // Save to new path
editor.file_path()                       // Current path if any

// View options
editor.with_show_line_numbers(bool)
editor.with_word_wrap(bool)
editor.with_indent_guides(bool)
editor.with_status_bar(bool)
editor.with_language("rust")           // For syntax highlighting

// Navigation & Search
editor.goto_line(line, area)            // Jump to line
editor.set_filter("query")               // Filter/highlight mode
editor.replace_all(find, replace)        // Global replace
editor.replace_next(find, replace)      // Next occurrence

// Selection & Clipboard
editor.get_selected_text()               // Get selection
editor.select_all()
editor.select_word_at(row, col)

// Multi-cursor (basic)
editor.add_cursor(row, col)              // Add extra cursor
editor.clear_extra_cursors()

// Persistence
editor.load_undo_stack()                 // Load from .file.undo
editor.save_undo_stack()                 // Save to .file.undo
editor.load_config()                     // Load from .file.dte.json
editor.save_config()                     // Save to .file.dte.json
```

## Framework Built-in Systems (Available for Examples)

### Hit Zone System (`src/framework/hitzone.rs`)
- `HitZone<T>` — Rectangular zone with callbacks for click, right-click, drag (with auto double/triple click detection)
- `HitZoneGroup<T>` — Multi-zone dispatcher to first matching zone
- `ScopedZone<T>` — Lightweight geometry-only zone (no callbacks)
- `ScopedZoneRegistry<T>` — Per-frame scoped registry: clear at start of render, register zones during render, dispatch in mouse handler

**Usage pattern:**
```rust
// In widget struct:
zones: RefCell<ScopedZoneRegistry<usize>>,

// In render (cleared each frame):
self.zones.borrow_mut().clear();
// ... register zones:
self.zones.borrow_mut().register(ZONE_ID, x, y, width, height);
// ... query for hover:
let hovered = self.zones.borrow().dispatch(mouse_x, mouse_y);

// In handle_mouse:
if let Some(id) = self.zones.borrow().dispatch(col, row) {
    match id {
        ZONE_ID => { /* handle click */ }
        _ => {}
    }
}
```

### Drag-and-Drop System (`src/framework/dragdrop.rs`)
- `DragManager<T>` — Full drag-and-drop lifecycle: `start_drag()`, `move_ghost()`, `end_drag()`, `cancel()`
- `DragItem<T>` — Payload with data and source ID
- `DragGhost` — Visual ghost rendered during drag at z=9000
- `DropTarget<T>` — Rectangular target zone

### Animation System (`src/framework/animation.rs`)
- `Animation` — Keyframe-based on widget properties (position, size)
- `AnimationManager` — Manages active animations
- `Easing` — Interpolation functions (linear, sine, quadratic, etc.)

## Showcase Launcher (`examples/showcase.rs`)

Uses `ScopedZoneRegistry` for all mouse dispatch:
- Zone IDs: 100-104 (primitives bar), 200+ (theme palette), 300-304 (sidebar categories), 400 (FPS toggle), 500+ (cards)
- Hover detection via zone dispatch in render (primitives bar, palette swatches, sidebar categories)
- Eliminates all duplicated position math between `render()` and `handle_mouse()`

## Example App Patterns

### file_manager (`examples/_apps/file_manager.rs`)
- Uses `SplitPane` with stored mutable state + divider drag resize
- Breadcrumb click navigation via inline position computation
- Stores `is_dragging_split` boolean for Drag/Up tracking

### system_monitor (`examples/_apps/system_monitor.rs`)
- Real `/proc` data reading (CPU, memory, disk, network, processes)
- Falls back to simulated data on non-Linux
- Uses `Rc<RefCell<SystemMonitor>>` + `InputRouter` pattern for tick-driven updates

### IDE (`examples/ide.rs`)
- Uses `CommandPalette` widget (Ctrl+P) as a command overlay with filtering
- `Rc<RefCell<Option<String>>>` bridge pattern: callback stores command ID, app checks bridge after keyboard/mouse dispatch
- Command palette rendered as overlay (section 11), blits non-transparent cells at absolute positions
- Handles all palette keyboard (↑/↓/Enter/Esc/type-to-filter) and mouse events when visible
- Commands: new-tab, open, save, close-tab, search, cut/copy/paste, cycle-theme, toggle-profiler, show-shortcuts, about

## CommandPalette Widget (`src/framework/widgets/command_palette.rs`)

A filterable command overlay widget:
- `CommandItem { id, name, category }` — command definition
- `CommandPalette::new(commands)` — create with command list
- `.with_size(w, h)` — set overlay dimensions
- `.with_theme(theme)` — set visual theme
- `.on_execute(cb)` — callback when a command is selected
- `.show()` / `.hide()` / `.is_visible()` — visibility control
- Keyboard: ↑/↓ navigate, Enter execute, Esc dismiss, type to filter by name or category
- Mouse: click items to execute, click outside to dismiss, scroll wheel for list scrolling
- Uses `ScopedZoneRegistry<usize>` for mouse dispatch (click outside detection, item clicks)
- Renders with semi-transparent backdrop

## Deferred / Out of Scope

These are interesting but NOT priorities for an engine:

| Feature | Why Deferred |
|---------|-------------|
| LSP integration | Requires async runtime, external processes, complex state management |
| Syntax-aware folding | Requires tree-sitter integration, per-language grammar |
| Multi-cursor enhancements | Basic multi-cursor sufficient for light editing |
| Modal editing | Kakoune-style is complex, not needed for view/edit use cases |
| Advanced text objects | vim-style text objects require deep editor integration |

## Contributing

When adding features to TextEditor or other widgets:
1. Is it a widget feature or an app feature?
2. Does it require external processes/services?
3. Does it change the widget's core purpose?

**If a feature requires LSP, complex state, or makes the widget into an editor — it's probably out of scope.**
