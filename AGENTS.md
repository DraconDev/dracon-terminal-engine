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
| `CellTextFn<T>` | `src/framework/widgets/table.rs` | `Box<dyn Fn(&T, usize) -> String>` |
| `HeaderClickCallback` | `src/framework/widgets/table.rs` | `Box<dyn FnMut(usize)>` |
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

### Bridge Pattern for Shared State (Pattern 2)

Pattern 2 apps (using `on_input`/`on_tick` closures) cannot access app state directly from `on_input` callbacks. Use `Rc<RefCell<T>>` to share state between closures:

```rust
// Shared state via Rc<RefCell>
let show_help = Rc::new(RefCell::new(false));
let show_help_input = Rc::clone(&show_help);
let show_help_render = Rc::clone(&show_help);

// In on_input closure:
.on_input(move |key| {
    if key.code == KeyCode::Char('?') && key.kind == KeyEventKind::Press {
        let mut h = show_help_input.borrow_mut();
        *h = !*h;
        return true;
    }
    // ...
})

// In on_tick or run closure:
app.run(move |ctx| {
    if *show_help_render.borrow() {
        // render help overlay via ctx.add_plane()
    }
})
```

For atomic shared state (e.g., toggling between `on_input` and `on_tick`):
```rust
let show_help = Arc::new(AtomicBool::new(false));
let show_help_input = Arc::clone(&show_help);
let show_help_render = Arc::clone(&show_help);
```

### Blank Screen Debugging

If an example shows nothing:
1. Check `needs_render()` — does it return `true` (or `false` with `ctx.add_plane()` in on_tick)?
2. Check if widget is added to App via `app.add_widget()`
3. Check if `on_tick` callback exists and runs
4. Verify `dirty` flag is set after state changes (Pattern 1)

### Help Overlay Pattern

All examples MUST implement a help overlay (toggle with `F1` or `?`, dismiss with `Esc` or the same key):

```rust
// In struct:
show_help: bool,

// In handle_key:
KeyCode::Char('?') => {
    self.show_help = !self.show_help;
    self.dirty = true;
    true
}
KeyCode::Esc => {
    if self.show_help {
        self.show_help = false;
        self.dirty = true;
        true
    } else { false }
}

// In render (drawn last, after main content):
if self.show_help {
    let t = &self.theme;
    let hw = 40u16.min(area.width.saturating_sub(4));
    let hh = 12u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;
    
    // Background fill
    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }
    
    // Rounded border
    let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
    for (ch, cx, cy) in corners.iter() {
        let idx = (cy * area.width + cx) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
    }
    for x in hx + 1..hx + hw - 1 {
        let top = (hy * area.width + x) as usize;
        let bot = ((hy + hh - 1) * area.width + x) as usize;
        if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
        if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
    }
    for y in hy + 1..hy + hh - 1 {
        let left = (y * area.width + hx) as usize;
        let right = (y * area.width + hx + hw - 1) as usize;
        if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
        if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
    }
    
    // Title (centered, primary color, bold)
    let title = "Example Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    for (i, c) in title.chars().enumerate() {
        let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = t.primary;
            plane.cells[idx].style = Styles::BOLD;
        }
    }
    
    // Shortcuts (two columns: keys + descriptions)
    let shortcuts = [
        ("↑/↓", "Navigate"),
        ("Enter", "Select"),
        ("Ctrl+T", "Cycle theme"),
        ("F1", "Toggle help"),
        ("Ctrl+Q", "Quit"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        for (j, c) in key.chars().enumerate() {
            let idx = (row * area.width + hx + 2 + j as u16) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
        }
        for (j, c) in desc.chars().enumerate() {
            let idx = (row * area.width + hx + 14 + j as u16) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
        }
    }
}
```

**Required elements:**
- `show_help: bool` field in struct
- `?` key toggles, `Esc` dismisses
- Rounded corners (╭╮╰╯) with `theme.outline`
- Background: `theme.surface_elevated`
- Title centered with `theme.primary` + `Styles::BOLD`
- Two-column layout: keys (`theme.primary`) + descriptions (`theme.fg`)

### Theme Propagation Checklist

When adding theme cycling to an example, verify ALL child widgets receive the new theme:

```rust
fn cycle_theme(&mut self) {
    let themes = [Theme::nord(), Theme::cyberpunk(), Theme::dracula()];
    let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
    self.theme = themes[(idx + 1) % themes.len()];
    
    // Propagate to EVERY child widget:
    self.list.on_theme_change(&self.theme);
    self.search_input.on_theme_change(&self.theme);
    self.status_bar.on_theme_change(&self.theme);
    self.table.on_theme_change(&self.theme);
    self.menu_bar.on_theme_change(&self.theme);
    // ... any other widgets
}
```

**Common widgets that need propagation:**
- `List<T>`
- `Table<T>`
- `SearchInput` / `PasswordInput`
- `StatusBar`
- `MenuBar`
- `Breadcrumbs`
- `Tree`
- `CommandPalette`
- `Form`
- `TabBar`
- `SplitPane` (for divider colors)
- `Toast` (recreated with `.with_theme()`)
- Custom sub-widgets

**Status bar hint** must reference keybinding actions:

```rust
StatusSegment::new("Tab: switch | Ctrl+T: theme | F1: help | Ctrl+Q: quit")
```

### Common Pitfalls

1. **`render(&self)` not `render(&mut self)`** — The `Widget` trait requires `fn render(&self, area: Rect) -> Plane`. All mutations must happen in `handle_key`, `handle_mouse`, `set_area`, or `on_theme_change`.

2. **Theme constructors are NOT const** — Cannot use `const THEMES: [Theme; 3] = [Theme::nord(), ...]`. Use runtime arrays:
   ```rust
   let themes = vec![Theme::nord(), Theme::cyberpunk(), Theme::dracula()];
   ```

3. **`Plane` has no `transparent` field** — Set `cell.transparent` on individual cells:
   ```rust
   for cell in plane.cells.iter_mut() {
       cell.transparent = false;
   }
   ```

4. **`put_str` takes 3 arguments, not 6** — Signature is `fn put_str(&mut self, x: u16, y: u16, text: &str)`. For styled text, manipulate `plane.cells[idx]` directly.

5. **`impl` blocks must be properly closed** — Every `impl MyStruct {` needs a matching `}`. Duplicate `impl` blocks cause "unclosed delimiter" errors that are hard to trace.

6. **Widget-based vs closure-based apps** — Widget-based (Pattern 1) supports theme cycling via `app.set_theme()`. Closure-based (Pattern 2) cannot easily cycle themes because `on_input` only receives `KeyEvent`, not `&mut App`.

### Theme Inheritance for Examples

All examples MUST use `Theme::from_env_or(default)` instead of hardcoded theme constructors so the theme can be inherited from a parent (showcase launcher, script, etc.) via the `DTRON_THEME` env var:

```rust
// WRONG — hardcoded theme, ignores parent
.theme(Theme::nord())

// RIGHT — uses DTRON_THEME if set, falls back to default
.theme(Theme::from_env_or(Theme::nord()))
```

**Why:** The showcase launcher sets `DTRON_THEME` before spawning external binary examples. Without `from_env_or`, launched examples always start with their hardcoded theme regardless of what the user selected.

**`Theme::from_env_or(default)`:** Reads `DTRON_THEME` env var, does case-insensitive lookup against all 20+ built-in theme names. Falls back to `default` if env var is unset, empty, or names an unknown theme.

### Theme Return via `DTRON_THEME_FILE`

When a launched example binary cycles its theme and exits, the showcase can adopt the final theme. This uses a **theme return file** mechanism:

1. **Showcase** sets `DTRON_THEME_FILE` env var to a temp file path before spawning the child binary
2. **App framework** (`App::run()`) writes `self.theme.name` to that file after the event loop exits, just before returning `Ok(())`
3. **Showcase** reads the file after the child exits, calls `ctx.set_theme(Theme::from_name(...))`, and removes the file

This is **automatic** for any example using the `App` framework — no per-example changes needed. Raw terminal examples (like `desktop.rs`, `game_loop.rs`) that don't use `App::run()` won't write back unless they manually check for `DTRON_THEME_FILE` on exit.

### Pattern 2 Theme Sync via `Widget::current_theme()`

Pattern-2 apps (InputRouter + manual rendering) manage their own `self.theme` field and cycle themes via `handle_key`. To sync the widget's local theme back to the framework:

1. **Override `current_theme()`** on the InputRouter:
```rust
fn current_theme(&self) -> Option<Theme> {
    Some(self.app.borrow().theme)  // or self.monitor, self.state, etc.
}
```

2. **Framework detects it automatically**: After `handle_key()` returns, `App::run()` calls `widget.current_theme()`. If it returns a theme different from `App.theme`, the framework calls `self.set_theme(theme)`, which triggers `on_theme_change()` propagation to all widgets.

**This is critical for `DTRON_THEME_FILE`**: When the App framework writes the final theme to the return file on exit, it writes `self.theme.name`. If Pattern-2 apps don't sync their local theme back via `current_theme()`, the file will contain the stale theme.

All 12 Pattern-2 examples implement this: `system_monitor`, `log_monitor`, `split_resizer`, `plugin_demo`, `ide`, `tabbed_panels`, `chat_client`, `event_bus_demo`, `modal_demo`, `scene_router_demo`, `todo_app`, `tutorial_app`.

### Widget Background Pattern

All widgets MUST fill their plane background with `self.theme.bg` to avoid black (`Color::Reset`) holes:

```rust
fn render(&self, area: Rect) -> Plane {
    let mut plane = Plane::new(0, area.width, area.height);
    plane.fill_bg(self.theme.bg);  // Fills all cells with theme.bg
    // ... render content on top
    plane
}
```

**Why:** `Plane::new()` creates cells with `Cell::default()`, which has `bg: Color::Reset`. `Color::Reset` renders as the terminal's default background (usually black), causing visual glitches when widgets overlap or when the theme is not the terminal default.

**Where to apply:** Every widget's `render()` method. Check existing widgets for the pattern.

**Exception — StatusBar:** `StatusBar` intentionally uses `Color::Reset` for its default `fg` and `bg` fields to inherit terminal defaults. This allows StatusBar to blend with the terminal background when not explicitly themed. Document this exception if adding StatusBar tests or modifications.

**Exception — Standalone Editor/Hotkey/Input widgets** (`src/widgets/`): These widgets use hardcoded `Color::Black` for cursor and highlight contrast (e.g., cursor: blue bg + black fg, bracket matching: yellow bg + black fg). These are intentional contrast choices for specific UI features, not theme-aware. They live in `src/widgets/` rather than `src/framework/widgets/` and are not part of the framework theme system.

### u16 Arithmetic Safety in Mouse Handlers

**Always bounds-check before subtracting from `u16` mouse coordinates.** `u16` underflow panics in debug mode and wraps in release — both are bugs.

**Wrong (panics when `col == rect.x`):**
```rust
let rel_col = col - rect.x - 1;  // PANIC if col == rect.x
let rel_row = row - rect.y - 2;  // PANIC if row < rect.y + 2
```

**Right (explicit bounds check):**
```rust
if row >= rect.y + 2 && row < rect.y + 2 + widget_area.height
    && col >= rect.x + 1 && col < rect.x + 1 + widget_area.width
{
    let rel_col = col - rect.x - 1;
    let rel_row = row - rect.y - 2;
    return self.widget.handle_mouse(kind, rel_col, rel_row);
}
```

**Alternative (saturating_sub for simple cases):**
```rust
let rel_col = col.saturating_sub(rect.x + 1);
let rel_row = row.saturating_sub(rect.y + 2);
// Then check if rel_col < widget_width && rel_row < widget_height
```

**Rule:** If you write `a - b` where `a` is a mouse coordinate (`col` or `row`), ensure `a >= b` first.

### Pattern 2 Theme Sync

Closure-based (Pattern 2) apps cannot access `App` state from `on_input`. To sync the framework theme each frame:

```rust
app.on_tick(move |ctx, _| {
    let theme = ctx.theme();  // Get current framework theme
    let mut app = app.borrow_mut();
    app.theme = theme.clone();     // Sync to app state
    app.tab_bar.on_theme_change(&theme);
    app.list.on_theme_change(&theme);
    // ... sync all child widgets
})
```

**Key points:**
- `ctx.theme()` returns the current framework theme
- Must sync every tick because theme can change via `app.set_theme()` or global cycling
- All child widgets need `on_theme_change()` calls
- Store theme in app state so `render()` can use it

### Scrollbar Indicator Pattern

For scrollable content, render a proportional scrollbar thumb:

```rust
if content_len > visible_count {
    let sb_x = area.width - 2; // inside right border
    let content_h = area.height.saturating_sub(header_and_footer);
    let thumb_h = (visible_count as f32 / content_len as f32 * content_h as f32).max(1.0) as u16;
    let thumb_y = (scroll_offset as f32 / (content_len - visible_count).max(1) as f32
        * (content_h - thumb_h) as f32) as u16 + content_start_y;
    for i in 0..thumb_h {
        let y = thumb_y + i;
        if y >= content_start_y && y < content_end_y {
            let idx = (y * area.width + sb_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '▐';
                plane.cells[idx].fg = theme.primary;
            }
        }
    }
}
```

**Key values:**
- Thumb character: `'▐'`
- Color: `theme.primary`
- Position: `area.width - 2` (inside border, not on it)
- Proportional height: `(visible / total) * content_height`
- Proportional position: `(offset / (total - visible)) * (content_height - thumb_height)`

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

### Table Sorting (`src/framework/widgets/table.rs`)

The `Table<T>` widget supports sortable column headers with click detection:

**Builder methods:**
- `.on_header_click(f)` — registers a callback `FnMut(usize)` invoked when a column header is clicked
- `.with_cell_text_fn(f)` — sets a per-column cell text formatter `Fn(&T, usize) -> String`
- `.set_sort(column, ascending)` — sets the active sort column and direction for rendering indicators

**Header click detection:**
- In `handle_mouse`, when `row == 0` (header row) and `MouseEventKind::Down(Left)`, determines which column was clicked via running x-offset accumulation
- Fires `on_header_click(i)` with the column index

**Sort indicators:**
- When `sort_column == Some(i)` and `sort_ascending == true`, header displays `▲`
- When `sort_column == Some(i)` and `sort_ascending == false`, header displays `▼`
- Active sort column header text uses `theme.primary` color and `Styles::BOLD`

**Example app pattern** (`examples/table_widget.rs`):
```rust
// App state stores sort column and direction
sort_column: Option<usize>,
sort_ascending: bool,

// Header click in handle_mouse dispatches to toggle_sort
if local_row == 0 {
    let mut col_x: u16 = 0;
    for (i, w) in column_widths.iter().enumerate() {
        if local_col >= col_x && local_col < col_x + w {
            self.toggle_sort(i);
            return true;
        }
        col_x += w;
    }
}

// toggle_sort implementation
fn toggle_sort(&mut self, col: usize) {
    if self.sort_column == Some(col) {
        self.sort_ascending = !self.sort_ascending;
    } else {
        self.sort_column = Some(col);
        self.sort_ascending = true;
    }
    self.rebuild_table(); // filter + sort then reconstruct Table widget
}
```

## Focus & Hover Styling

### Widgets that support hover (`hover_bg`)
- **Tree**: `hovered_path` field; highlights hovered nodes with `hover_bg`
- **Table**: `hovered_row` field; highlights hovered row across all columns
- **List**: `hovered` field; highlights hovered item
- **CommandPalette**: `hovered_index` field; highlights hovered command item
- **Button**: `hovered` field; button background changes to `hover_bg` on mouse hover
- **Checkbox**: `hovered` field; background changes to `hover_bg` on hover
- **Toggle**: `hovered` field; background changes to `hover_bg` on hover
- **Radio**: `hovered` field; background changes to `hover_bg` on hover
- **Select**: `hovered_index` field; dropdown option items highlight with `hover_bg`
- **TabBar**: `hovered_tab` field; hovered tab uses `hover_bg` + bold text

### Widgets that support focus (`focus_bg`, `focus_border`)
- **SearchInput / PasswordInput**: `BaseInput.focused` flag; uses `focus_bg` instead of `input_bg` when focused; text is underlined when focused
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

## Raw Terminal Examples (Low-Level Demos)

Some examples (`desktop.rs`, `game_loop.rs`, `input_debug.rs`) deliberately use raw terminal/compositor APIs instead of the framework. These are intentionally low-level to demonstrate engine internals. They still implement help overlays via simple `write!` calls:

```rust
// In the main loop, after polling input:
if let Some(Event::Key(KeyEvent { code: KeyCode::Char('?'), .. })) = parser.advance(byte) {
    show_help = !show_help;
}
// During render:
if show_help {
    write!(term, "\x1b[2J\x1b[H")?;
    write!(term, "╭────────────────────────────────────────────╮\r\n")?;
    write!(term, "│  q          — Quit                         │\r\n")?;
    write!(term, "│  ?          — Toggle this help              │\r\n")?;
    write!(term, "╰────────────────────────────────────────────╯\r\n")?;
    term.flush()?;
}
```

## Embedded Showcase Scenes

The showcase launcher (`examples/showcase/`) supports **embedded scenes** — examples that run in-process via `SceneRouter` instead of as external processes. This provides instant launch, seamless theme propagation, and smooth transitions.

### Benefits of Embedded Scenes
- **Instant launch** — no process spawn, no terminal state save/restore
- **Theme sharing** — showcase theme propagates to embedded scenes on `t` key
- **Smooth transitions** — fade/slide animations between showcase and scenes
- **Shared state** — scenes can share data via the app's `Arc` primitives

### Embedded Scene Architecture

```rust
// examples/showcase/scenes/mod.rs
pub mod widget_gallery;
pub mod theme_switcher;
pub mod form_demo;
pub mod tree_navigator;
pub mod modal_demo;

/// Actions a scene can request from the router
#[derive(Clone, Debug, PartialEq)]
pub enum SceneAction {
    None,
    Pop,      // Return to previous scene
    Push(String), // Push a new scene
    Quit,
}

// Register scenes in state.rs:
let mut scene_router = SceneRouter::new()
    .with_default_transition(SceneTransition::Fade);
scene_router.register("widget_gallery", Box::new(WidgetGalleryScene::new(theme)));
```

### Scene Trait Implementation

```rust
use dracon_terminal_engine::framework::scene_router::Scene;

pub struct MyScene {
    theme: Theme,
    // ... scene-specific state
}

impl Scene for MyScene {
    fn on_enter(&mut self) { /* called when scene becomes active */ }
    fn on_exit(&mut self) { /* called when scene becomes inactive */ }
    fn on_pause(&mut self) { /* called when another scene is pushed on top */ }
    fn on_resume(&mut self) { /* called when scene returns to top */ }
    fn render(&self, area: Rect) -> Plane { /* draw the scene */ }
    fn handle_key(&mut self, key: KeyEvent) -> bool { /* handle input */ }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool { true }
    fn on_theme_change(&mut self, theme: &Theme) { self.theme = *theme; }
}
```

### Scene Title Bar

When rendering an embedded scene, the showcase draws a title bar at the top showing the scene name. The scene's render output is drawn below the title bar (y=1 onwards).

### Navigation

- `B` or `Esc` — pops the current scene, returns to showcase
- Theme changes (`t` key) propagate to active scenes via `on_theme_change`
- Embedded scenes work alongside external binary launches — both patterns coexist

## Process Tree View Pattern (`system_monitor.rs`)

The `system_monitor` example implements a process tree view. Key patterns:

### TreeNode Struct with Pre-computed Connectors

```rust
struct TreeNode {
    idx: usize,              // Index into flat processes array
    depth: usize,            // Nesting depth (0 = root)
    prefix: String,          // Tree connector prefix ("├─ ", "└─ ", "│  ")
    ancestor_last: Vec<bool>, // Per-level sibling tracking for connector rendering
}
```

Built via DFS with cycle detection (HashSet<u32> of visited PIDs). Connector prefixes computed during DFS traversal using `ancestor_last` tracking:
- Last child at a level gets `└─ `, others get `├─ `
- Continuation bars (`│  `) for ancestors above non-last children

### View-Index Architecture

**CRITICAL**: When rendering a derived view (like a tree), navigation and mouse hit-testing must use **view indices** (display row), not flat array indices:

```rust
// WRONG — broken when view order != array order:
self.selected_process = Some(clicked_idx_into_processes);

// RIGHT — use view index, convert back for data access:
self.selected_process = Some(clicked_view_row);
// Detail panel: let proc = &self.data.processes[self.tree_view[self.selected_process.unwrap()].idx];
```

Key fields:
- `selected_process: Option<usize>` — view index (display row)
- `hovered_process: Option<usize>` — view index
- `tree_view: Vec<TreeNode>` — DFS-ordered nodes
- `visible_process_rows()` — returns `tree_view.len()` in tree mode, `processes.len()` in flat mode

### /proc/pid/stat Parsing Safety

Command names in `/proc/pid/stat` can contain `)` characters. Use state-char validation:

```rust
fn find_stat_end(content: &str) -> Option<usize> {
    // Find ") " followed by a valid single-character state
    let valid_states = ['R', 'S', 'D', 'Z', 'T', 'W', 't', 'X', 'x', 'K', 'P', 'I'];
    for (i, _) in content.match_indices(") ") {
        if let Some(next) = content[i + 2..].chars().next() {
            if valid_states.contains(&next) {
                return Some(i);
            }
        }
    }
    None
}
```

### Tree Connector Rendering

```rust
fn render_tree_connectors(plane: &mut Plane, x: u16, y: u16, prefix: &str, theme: &Theme) {
    for (ci, c) in prefix.chars().enumerate() {
        let idx = (y as usize * area.width as usize + (x + ci as u16) as usize) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = theme.dim;
        }
    }
}
```

## Keybinding Conventions

### Config-Driven Keybindings

All examples use the framework's `KeybindingSet` system from `src/framework/keybindings.rs`:

```rust
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};

// In your app struct:
keybindings: KeybindingSet,

// In new():
keybindings: KeybindingSet::from_config(&resolve_keybindings()),

// In handle_key():
if self.keybindings.matches(actions::QUIT, &key) {
    self.should_quit.store(true, Ordering::SeqCst);
    return true;
}
```

**Never hardcode `KeyCode::Char('q')`, `KeyCode::Char('?')`, `KeyCode::Char('t')`, or `KeyCode::Esc` for standard actions.** Always use `keybindings.matches()` so users can customize via `dracon.toml`.

### Standard Actions

| Action | Constant | Default | Purpose |
|--------|----------|---------|---------|
| Quit | `actions::QUIT` | `ctrl+q` | Exit application |
| Help | `actions::HELP` | `f1` | Toggle help overlay |
| Back | `actions::BACK` | `esc` | Dismiss/go back |
| Theme | `actions::THEME` | `ctrl+t` | Cycle theme |
| Submit | `actions::SUBMIT` | `enter` | Confirm/submit |
| Search | `actions::SEARCH` | `ctrl+f` | Open search |
| New | `actions::NEW` | `ctrl+n` | New item/tab |
| Close | `actions::CLOSE` | `ctrl+w` | Close item/tab |
| Save | `actions::SAVE` | `ctrl+s` | Save |
| Copy | `actions::COPY` | `ctrl+c` | Copy (when not quitting) |
| Paste | `actions::PASTE` | `ctrl+v` | Paste |
| Cut | `actions::CUT` | `ctrl+x` | Cut |
| Delete | `actions::DELETE` | `delete` | Delete |
| Refresh | `actions::REFRESH` | `f5` | Refresh/reload |
| Pause | `actions::PAUSE` | `ctrl+p` | Pause/resume |

### Conservative Philosophy

**Modifier keys only for actions.** Single-letter keys conflict with text input:

```rust
// WRONG — 'q' quits even while typing in a search box
KeyCode::Char('q') => { self.quit(); true }

// RIGHT — Ctrl+Q quits, 'q' types normally
if self.keybindings.matches(actions::QUIT, &key) { self.quit(); true }
```

This applies to all global actions: quit, help, theme, search, new, close, save.

### Non-Configurable Keys (Keep Hardcoded)

These are universal navigation/input primitives and should NOT be configurable:

| Key | Purpose | Why Hardcoded |
|-----|---------|---------------|
| `↑/↓/←/→` | Navigate lists, move cursor | Universal |
| `Enter` | Select, submit, expand | Universal |
| `Tab` / `Shift+Tab` | Focus next/previous field | Universal |
| `Backspace` | Delete character | Text input primitive |
| `Char(c)` | Type character | Text input primitive |

### Modifier Guards

**Always check `key.modifiers.is_empty()` on non-configurable `Char` handlers** to prevent Ctrl+X from triggering actions:

```rust
// WRONG — Ctrl+P triggers pause
KeyCode::Char('p') => {
    self.paused = !self.paused;
    true
}

// RIGHT — Ctrl+P ignored
KeyCode::Char('p') if key.modifiers.is_empty() => {
    self.paused = !self.paused;
    true
}
```

This applies to app-specific shortcuts that remain hardcoded (e.g., `p` for pause in dashboard_builder).

### Backspace Semantics

**Backspace is for delete only, never navigation.** Use `Esc` (BACK action) or arrow keys for going back/up:

```rust
// WRONG — Backspace as navigation
KeyCode::Backspace => {
    self.go_up();
    true
}

// RIGHT — Left arrow for navigation
KeyCode::Left => {
    self.go_up();
    true
}

// RIGHT — Backspace for delete
KeyCode::Backspace if !self.input.is_empty() => {
    self.input.pop();
    true
}
```

### Help Overlay Standard

All help overlays MUST:
1. Toggle with help action (`F1` by default)
2. Dismiss with back action (`Esc` by default) or re-pressing help
3. Mention dismiss key in status bar / footer
4. Use rounded corners (`╭╮╰╯`) with `theme.outline`
5. Use `theme.surface_elevated` for background
6. Show title centered with `theme.primary` + `Styles::BOLD`
7. Use two-column layout: keys (`theme.primary`) + descriptions (`theme.fg`)

### Status Bar / Footer Text

All status bars MUST include:
- Help key reference (e.g., `F1: help`)
- Back/dismiss key reference (e.g., `Esc: dismiss`)
- Quit key reference (e.g., `Ctrl+Q: quit`)
- Theme key reference if theme cycling is supported (e.g., `Ctrl+T: theme`)

### dracon.toml Configuration

Users can override keybindings via `dracon.toml`:

```toml
[keybindings]
quit = "ctrl+q"
help = "f1"
back = "esc"
theme = "ctrl+t"
search = "ctrl+f"
new = "ctrl+n"
close = "ctrl+w"
save = "ctrl+s"
```

Resolution order:
1. Engine defaults (conservative modifier keys)
2. User global: `~/.config/dracon/dracon.toml`
3. Project local: `./dracon.toml` (highest priority)

### Showcase Launcher Keybinding Rules

The showcase launcher is the **single source of truth** for theme cycling:
- `t`/`T` cycles the global theme and propagates to all scenes via `scene_router.on_theme_change()`
- Individual scenes must NOT implement their own `cycle_theme()` — they receive theme changes via `on_theme_change()`
- All scenes must check `actions::BACK` **before** delegating to widgets to ensure `Esc` always works to go back

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
