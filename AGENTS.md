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

## Showcase Launcher (`examples/showcase/`)

Modular example launcher split across submodules:
- `main.rs` — Entry point, app setup, binary launch
- `data.rs` — `ExampleMeta` definitions for all examples
- `state.rs` — `Showcase` struct, filtering, selection state
- `render.rs` — Card rendering + all preview functions
- `widget.rs` — `Widget` impl (render, handle_key, handle_mouse)

Uses `ScopedZoneRegistry` for all mouse dispatch:
- Zone IDs: 100-104 (primitives bar), 200+ (theme palette), 300-304 (sidebar categories), 400 (FPS toggle), 500+ (cards)
- Hover detection via zone dispatch in render (primitives bar, palette swatches, sidebar categories)
- Eliminates all duplicated position math between `render()` and `handle_mouse()`

**CardConfig struct** (`render.rs`): Refactored `render_card` to take a `&CardConfig` struct instead of 8 separate parameters:
```rust
pub struct CardConfig<'a> {
    pub ex: &'a ExampleMeta,
    pub idx: usize,
    pub selected_idx: usize,
    pub hovered_idx: Option<usize>,
    pub theme: Theme,
    pub phase: f64,
    pub width: u16,
    pub height: u16,
}
```

**Smoke test** (`tests/showcase_smoke_test.rs`): Integration test that spawns the showcase binary and verifies it initializes without crashing (ignored by default, requires TTY).

## Callback Type Aliases

The following type aliases are used for cleaner signatures and to avoid "very complex type" clippy warnings:

| Alias | Location | Type |
|-------|----------|------|
| `TickCallback` | `src/framework/app.rs` | `Box<dyn FnMut(&mut Ctx, u64) + 'static>` |
| `ExecuteCallback` | `src/framework/widgets/command_palette.rs` | `Box<dyn FnMut(&str)>` |
| `SelectCallback<T>` | `src/framework/widgets/list.rs` | `Box<dyn FnMut(&T)>` |
| `ChangeCallback` | `src/framework/widgets/select.rs` | `Box<dyn FnMut(&str)>` |
| `SelectCallback<T>` | `src/framework/widgets/table.rs` | `Box<dyn FnMut(&T)>` |
| `SubmitCallback` | `src/framework/widgets/text_input_base.rs` | `Box<dyn FnMut(&str)>` |
| `SelectCallback` | `src/framework/widgets/tree.rs` | `Box<dyn FnMut(&str)>` |

## Example App Patterns

### Rendering Patterns

There are **two ways** to render content in the framework:

**Pattern 1: Widget Trait Auto-Render**
```rust
impl Widget for MyApp {
    fn needs_render(&self) -> bool { self.dirty }
    fn render(&self, area: Rect) -> Plane { /* full render */ }
}
```
- App framework automatically calls `render()` when `needs_render()` returns true
- Set `self.dirty = true` after state changes to trigger re-render
- Used by: `file_manager`, `git_tui`, `sqlite_browser`, `widget_gallery`, `dashboard_builder`

**Pattern 2: InputRouter + Manual `ctx.add_plane()`**
```rust
// Router widget with needs_render() -> false
impl Widget for MyRouter {
    fn needs_render(&self) -> bool { false }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.app.borrow_mut().handle_key(key)
    }
}

// App-level rendering in on_tick
app.on_tick(move |ctx, _| {
    let mut app = app.borrow_mut();
    app.tick();
    let (w, h) = ctx.compositor().size();
    let plane = app.render(Rect::new(0, 0, w, h));
    ctx.add_plane(plane);
})
```
- Must explicitly call `ctx.add_plane()` in `on_tick` callback
- `needs_render()` returns false — App doesn't auto-render from widget
- Used by: `system_monitor`, `ide`, `chat_client`, `log_monitor`, `modal_demo`

**Which pattern to use?**
- Use **Pattern 1** (Widget trait) for simpler apps where App framework handles render scheduling
- Use **Pattern 2** (InputRouter) when you need app-level control over render timing or shared state across ticks

### Blank Screen Debugging

If an example shows nothing:
1. Check `needs_render()` — does it return `true` (or `false` with `ctx.add_plane()` in on_tick)?
2. Check if widget is added to App via `app.add_widget()`
3. Check if `on_tick` callback exists and runs
4. Verify `dirty` flag is set after state changes (Pattern 1)

### Help Overlay Pattern

Many examples use a consistent help overlay (toggle with `?` key):
```rust
fn render_help_overlay(&self, plane: &mut Plane) {
    let w = 50.min(plane.width);
    let h = 15.min(plane.height);
    // Draw centered box with shortcuts
}
```
- Toggle: `?` key sets `self.show_help = true`
- Dismiss: `Esc` or `?` key sets `self.show_help = false`
- Rendered after main content as overlay

### Theme Cycling

Most examples support `t` key to cycle themes:
```rust
fn cycle_theme(&mut self) {
    let themes = [Theme::nord(), Theme::cyberpunk(), Theme::dracula()];
    let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
    self.theme = themes[(idx + 1) % themes.len()];
    // Update all widget themes
    self.menu_bar.on_theme_change(&self.theme);
    self.search_input.on_theme_change(&self.theme);
    // ...
}
```

### SparklineConfig Pattern

For rendering sparkline charts, use `SparklineConfig` struct to avoid too-many-args warnings:
```rust
struct SparklineConfig {
    x: u16, y: u16, w: u16, h: u16,
    color: Color, bg: Color,
}

fn render_sparkline(plane: &mut Plane, cfg: SparklineConfig, metric: &MetricHistory) {
    // ...
}
```

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

## Focus & Hover Styling

### Widgets that support hover (`hover_bg`)
- **Tree**: `hovered_path` field; highlights hovered nodes with `hover_bg`
- **Table**: `hovered_row` field; highlights hovered row across all columns
- **List**: `hovered` field; highlights hovered item
- **CommandPalette**: `hovered_index` field; highlights hovered command item

### Widgets that support focus (`focus_bg`, `focus_border`)
- **SearchInput / PasswordInput**: `BaseInput.focused` flag; uses `focus_bg` instead of `input_bg` when focused
- **Form**: Focused field renders entire row with `focus_bg` background

### Pattern for adding hover to a widget
```rust
// In struct:
hovered: Option<usize>,

// In render():
let is_hovered = self.hovered == Some(idx);
let bg = if is_selected {
    self.theme.selection_bg
} else if is_hovered {
    self.theme.hover_bg
} else {
    self.theme.bg
};

// In handle_mouse():
MouseEventKind::Moved => {
    if col >= self.width || row >= self.visible_count as u16 {
        if self.hovered.is_some() {
            self.hovered = None;
            self.dirty = true;
        }
        return false;
    }
    let idx = self.offset + row as usize;
    if idx >= self.items.len() {
        if self.hovered.is_some() {
            self.hovered = None;
            self.dirty = true;
        }
        return false;
    }
    if self.hovered != Some(idx) {
        self.hovered = Some(idx);
        self.dirty = true;
    }
    true
}
```

**Key rule**: Always clear hover state when mouse moves out of widget bounds. Always set `dirty = true` when hover changes.

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
