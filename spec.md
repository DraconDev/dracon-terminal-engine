# Dracon Terminal Engine — Specification

> **Version:** 0.1.10  
> **License:** AGPL-3.0-only + Commercial  
> **Repository:** https://github.com/DraconDev/dracon-terminal-engine  
> **Documentation:** https://docs.rs/dracon-terminal-engine  
> **Last Audit:** 2026-05-22

---

## Table of Contents

1. [Vision & Philosophy](#1-vision--philosophy)
2. [Architecture Overview](#2-architecture-overview)
3. [Core Layers](#3-core-layers)
4. [Framework Layer](#4-framework-layer)
5. [Widget System](#5-widget-system)
6. [Theme System](#6-theme-system)
7. [Input System](#7-input-system)
8. [Compositor & Rendering](#8-compositor--rendering)
9. [Command-Driven Architecture](#9-command-driven-architecture)
10. [Event System](#10-event-system)
11. [Scene Router](#11-scene-router)
12. [Plugin System](#12-plugin-system)
13. [TextEditor (Standalone Widget)](#13-texteditor-standalone-widget)
14. [Application Patterns](#14-application-patterns)
15. [Build Configuration & Features](#15-build-configuration--features)
16. [Examples & Showcase](#16-examples--showcase)
17. [Test Coverage](#17-test-coverage)
18. [API Surface](#18-api-surface)
19. [Completeness Assessment](#19-completeness-assessment)
20. [Future Roadmap](#20-future-roadmap)

---

## 1. Vision & Philosophy

### 1.1 Mission Statement

Dracon Terminal Engine is a **terminal application framework** for Rust — not a "TUI library" in the traditional sense. It provides a complete runtime that **owns the terminal, input, rendering, and event loop**, allowing developers to build GUI-grade terminal applications with minimal code.

### 1.2 Core Principles

| Principle | Description |
|-----------|-------------|
| **One import, complete app** | `use dracon_terminal_engine::framework::prelude::*;` is the only import needed |
| **Framework, not library** | App owns the event loop, compositor, input parsing, and rendering |
| **Widgets own state** | Each widget manages its own lines, cursor, selection, etc. |
| **App owns composition** | App manages widgets via registry, z-index, focus |
| **Mouse-first** | Widgets respond to clicks, not just keys |
| **Keyboard-enhanced** | Navigation shortcuts exist but aren't required |
| **Terminal as universal target** | No platform-specific code, no external dependencies beyond std |
| **RAII terminal state** | `Terminal` struct enters raw mode on creation, restores on Drop |
| **Z-indexed compositor** | Painter's algorithm with per-plane opacity and filters |
| **Command-driven architecture** | Every widget can bind a CLI command; AI-inspectable actions |

### 1.3 What It Is NOT

- **Not a vim/Helix competitor** — TextEditor is a view/edit widget, not a modal editor
- **Not an LSP-powered editor** — No LSP integration (deferred as out of scope)
- **Not a "CLI+"** — Not hotkey-centric; mouse-friendly is the primary interaction mode
- **Not a browser-based GUI** — Runs natively in any terminal (VPS, SSH, containers, CI)

### 1.4 Deployment Advantages Over GUI

| Advantage | Detail |
|-----------|--------|
| **Universal** | Runs on VPS, SSH, containers, CI, embedded — anywhere with a terminal |
| **Zero user dependencies** | No browser, no runtime, no permissions, no install |
| **Single binary** | Ships as one executable, instant startup |
| **Cross-platform** | No Tauri/Dioxus/egui platform issues, no browser bugs, no lag |

### 1.5 UX Advantages Over CLI

| Advantage | Detail |
|-----------|--------|
| **Persistent state** | Don't re-run commands to see output |
| **Visible structure** | Panels, trees, forms — not just scrolling text |
| **Mouse-friendly** | Click, drag, scroll — natural interactions |
| **Composability** | Mix widgets (list + editor + form) freely |

---

## 2. Architecture Overview

### 2.1 Module Architecture

```
dracon-terminal-engine
├── src/
│   ├── lib.rs              # Crate root: module declarations + re-exports
│   ├── backend/            # POSIX tty ioctls, raw mode (non-Windows only)
│   ├── compositor/         # Plane, Cell, Color, Styles, Compositor, filters, cell pool
│   ├── contracts.rs        # UiRenderer, UiEventSource, UiRuntime trait contracts
│   ├── core/               # Terminal wrapper (RAII raw mode + alt screen)
│   ├── error.rs            # DraconError unified error type
│   ├── framework/          # App, Ctx, Widget trait, widgets, themes, subsystems
│   │   ├── app.rs          # App struct: event loop, widget lifecycle, input dispatch
│   │   ├── ctx.rs          # Ctx: context passed to render/tick callbacks
│   │   ├── widget.rs       # Widget trait + sub-traits (Renderable, Focusable, etc.)
│   │   ├── theme.rs        # Theme struct with 21 built-in themes
│   │   ├── command.rs      # BoundCommand, OutputParser, CommandRunner, TOML config
│   │   ├── layout.rs       # Constraint-based layout engine
│   │   ├── hitzone.rs      # HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry
│   │   ├── dragdrop.rs     # DragManager, DragItem, DragGhost, DropTarget
│   │   ├── marquee.rs      # MarqueeState, MarqueeRect, render_marquee
│   │   ├── focus.rs        # FocusManager (tab-order ring, focus trapping)
│   │   ├── scroll.rs       # ScrollState, ScrollContainer
│   │   ├── keybindings.rs  # KeybindingSet, resolve_keybindings, actions
│   │   ├── scene_router.rs # SceneRouter, Scene trait, SceneTransition
│   │   ├── event_bus.rs    # EventBus, Reactive, SubscriptionId
│   │   ├── animation.rs    # Animation, AnimationManager, Easing
│   │   ├── plugin.rs       # PluginRegistry, WidgetFactory
│   │   ├── dirty_regions.rs# DirtyRegion, DirtyRegionTracker
│   │   ├── widget_container.rs # WidgetContainer, WidgetRegistry
│   │   ├── i18n.rs         # I18n, tr! macro (JSON locale files)
│   │   ├── logging.rs      # tracing integration (feature-gated)
│   │   ├── event_dispatcher.rs # Event dispatch helpers
│   │   └── widgets/        # 47 framework widget types
│   ├── input/              # InputReader, Parser, event type definitions
│   ├── integration/        # Ratatui integration bridge
│   ├── layout/             # Grid, border, padding helpers
│   ├── system/             # SystemMonitor (feature-gated)
│   ├── utils.rs            # Visual width, truncate, formatting helpers
│   ├── text.rs             # Unicode grapheme cluster utilities
│   └── visuals/            # Icons, OSC sequences, accessibility, sync mode
└── src/widgets/            # Standalone widgets (TextEditor, TextInput, etc.)
```

### 2.2 Layer Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│  Examples / Apps                                                    │
│  (57 binary examples in examples/)                                  │
├─────────────────────────────────────────────────────────────────────┤
│  FRAMEWORK LAYER                                                    │
│                                                                     │
│  App (event loop, widget lifecycle, input dispatch)                 │
│  ├── Ctx (render/tick context)                                      │
│  ├── Widget trait + 47 framework widgets                            │
│  ├── Theme (21 built-in)                                            │
│  ├── SceneRouter (multi-screen navigation)                          │
│  ├── EventBus (pub/sub inter-widget)                                │
│  ├── HitZone / DragDrop / Marquee (interaction models)              │
│  ├── FocusManager (tab-order ring)                                  │
│  ├── Animation / Easing                                             │
│  ├── Layout engine (constraint-based)                               │
│  ├── KeybindingSet (config-driven)                                  │
│  ├── PluginRegistry (dynamic widgets)                               │
│  ├── I18n (internationalization)                                    │
│  └── DirtyRegionTracker (partial updates)                           │
├─────────────────────────────────────────────────────────────────────┤
│  STANDALONE WIDGETS                                                 │
│  TextEditor (syntect highlighting, undo/redo, search/filter)        │
│  TextInput, Button, Panel, Component, HotkeyHint                    │
├─────────────────────────────────────────────────────────────────────┤
│  ENGINE LAYER                                                       │
│  Compositor (z-indexed planes, painter's algorithm)                 │
│  Plane / Cell / Color / Styles                                      │
│  CellPool (object pool for per-frame allocation)                    │
│  Filters (Dim, Invert, Scanline, Pulse, Glitch)                     │
│  Terminal (RAII raw mode + alt screen)                              │
│  InputReader / Parser (SGR mouse, keyboard chords, bracketed paste) │
│  Backend (POSIX tty ioctls)                                         │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.3 Data Flow

```
Terminal Input → Parser → Event → App.handle_event()
                                        │
                          ┌─────────────┼─────────────┐
                          ▼             ▼              ▼
                    dispatch_key  dispatch_mouse  dispatch_resize
                          │             │               │
                          ▼             ▼               ▼
                    Widget.handle_  Widget.handle_  Compositor.resize()
                    key()           mouse()         Widget.set_area()
                          │             │
                          ▼             ▼
                    dirty = true    dirty = true
                    (state change)  (state change)

Render Loop (per frame):
  Widget.render(area) → Plane → Compositor.add_plane()
                                              │
                                              ▼
                                    Compositor.render(writer)
                                    (composite planes → escape codes)
                                              │
                                              ▼
                                    Terminal → stdout (sync mode 2026)
```

---

## 3. Core Layers

### 3.1 Backend (`src/backend/`)

**Purpose:** Low-level POSIX terminal interface.

| Component | Description |
|-----------|-------------|
| `tty` module | Platform-specific ioctls: `get_window_size()`, `poll_input()` |
| **Platform:** | Unix-only (`libc` dependency, gated with `cfg(not(windows))`) |
| **Dependencies:** | `libc` (Unix), `signal-hook` for SIGINT/SIGTERM handling |

**Contract:**
- `poll_input(fd, timeout_ms) -> Result<bool, Error>` — polls stdin for available bytes
- `get_window_size(fd) -> io::Result<(u16, u16)>` — returns terminal dimensions via `TIOCGWINSZ`

### 3.2 Terminal (`src/core/terminal.rs`)

**Purpose:** RAII wrapper that enters raw mode + alternate screen buffer on creation, restores on Drop.

| Component | Description |
|-----------|-------------|
| `Terminal<W>` | Generic over writer type; typically `io::Stdout` |
| `Capabilities` | Detected terminal features (true color, SGR mouse, bracketed paste, etc.) |
| `CursorShape` | Enum for cursor shape (Block, Underline, Bar, Hidden) |

**Capabilities Detection:**
- `TERM` environment variable parsing
- `COLORTERM` for true color support
- Kitty keyboard protocol detection
- Bracketed paste mode enable/disable
- SGR mouse mode enable/disable
- Sync mode 2026 support

**Lifecycle:**
```
Terminal::new(stdout)
  ├── Save terminal state (DECSC)
  ├── Enter alternate screen (DECSET 1049)
  ├── Enable SGR mouse (DECSET 1006)
  ├── Enable bracketed paste (DECSET 2004)
  ├── Enable kitty keyboard (if supported)
  └── Set raw mode (cfmakeraw)

Terminal::drop
  ├── Restore saved state (DECRC)
  └── Leave alternate screen (DECSET 1049)
```

### 3.3 Compositor (`src/compositor/`)

**Purpose:** Z-indexed plane compositing engine. Implements painter's algorithm.

| Component | Lines | Description |
|-----------|-------|-------------|
| `Plane` | ~500 | 2D cell grid with position, z-index, opacity, filters, position |
| `Cell` | — | Single terminal cell: `char`, `fg`, `bg`, `style`, `transparent`, `skip` |
| `Color` | — | Enum: `Reset`, `Ansi(u8)`, `Rgb(u8,u8,u8)` |
| `Styles` | — | Bitflags: `BOLD`, `DIM`, `ITALIC`, `UNDERLINE`, `BLINK`, `REVERSE`, `HIDDEN`, `STRIKETHROUGH` |
| `Compositor` | ~600 | Plane compositing, dirty region rendering, escape code output |
| `Filter` (submodule) | — | Visual filters: Dim, Invert, Scanline, Pulse, Glitch |
| `CellPool` (submodule) | — | Object pool for Cell allocation to reduce per-frame allocations |
| `PoolConfig` | — | Configuration for cell pool (initial capacity, max capacity) |

**Plane Operations:**
- `put_char(x, y, c)` — write single char with Unicode width awareness (wide chars set `skip` on adjacent cell)
- `put_str(x, y, text)` — write string with grapheme cluster awareness, 2-cell emoji support, zero-width char skipping
- `put_cell(x, y, cell)` — place a pre-configured cell
- `set_style(x, y, fg, bg, style)` — style a cell
- `fill_bg(color)` — fill all cells with background color (sets `transparent = false`)
- `clear()` — reset all cells to defaults
- `blit_from(source, dx, dy)` — copy non-transparent, non-skip cells with bounds checking
- `blit_from_fast(source)` — bulk memcpy when source is fully opaque; falls back to per-cell blit otherwise
- `reset_cells()` — reset to transparent defaults without reallocation
- `crop(rect)` — extract sub-plane at rectangle
- `set_filter(filter)` — apply visual filter
- `set_transparent(bool)` / `set_skip(x, y, bool)`

**Compositor Operations:**
- `add_plane(plane)` — push plane to compositing stack
- `render(writer)` — composite all planes, emit escape sequences with:
  - Synchronous mode 2026 wrapping
  - Dirty-region optimization (only emit changed cells)
  - TrueColor escape sequences (38;2;R;G;B / 48;2;R;G;B)
  - ANSI 256-color fallback (38;5;N / 48;5;N)
  - Style tracking (only emit SGR changes, not redundant codes)
  - Braille cell merging for overlapping braille characters
  - Alpha blending for opacity < 1.0
  - Per-plane filter application
  - `final_buffer` reuse to avoid per-frame allocation
  - Cursor positioning inline (`\x1b[Y;XH`)
- `resize(w, h)` — resize frame buffers
- `hit_test(x, y)` — find topmost non-transparent plane at position
- `force_clear()` / `invalidate_last_frame()` — force full redraw
- `draw_text(text, x, y, fg, bg, style)` — convenience text rendering
- `draw_rect(x, y, w, h, char, fg, bg, style)` — convenience rectangle fill
- `draw_ratatui_line(line, x, y)` — ratatui integration rendering

**Dirty Region Rendering:**
- `DirtyRegionTracker` tracks changed screen regions per frame
- `set_dirty_regions(tracker)` — compositor copies dirty region info
- Full refresh mode: renders entire screen
- Partial refresh mode: only renders cells within dirty regions
- Skip optimization: cells identical to last frame are skipped
- Both modes fall back to full refresh if dirty regions are empty

### 3.4 Error Types

**`DraconError`** — Unified error type for the engine. Implements `std::error::Error`.

Variants (from `src/error.rs`):
- `Io(io::Error)` — Wrapped I/O errors
- `InvalidKeybinding(String)` — Keybinding parsing errors
- `ThemeNotFound(String)` — Unknown theme name
- `WidgetNotFound(WidgetId)` — Widget lookup failures
- `ConfigError(String)` — TOML configuration errors
- `PluginError(String)` — Plugin registration/loading errors

---

## 4. Framework Layer

### 4.1 App (`src/framework/app.rs`)

**Purpose:** The main application entry point. Owns the terminal, compositor, input parser, widget registry, and event loop.

**Specification & Public API:**

```rust
// Construction
App::new() -> io::Result<App>                        // Default terminal init
App::from_toml(path) -> io::Result<App>              // From TOML config
App::default() -> App                                 // Panics on terminal init failure

// Builder pattern
.title(&str) -> Self                                  // Terminal window title (OSC 0)
.fps(u32) -> Self                                     // Target FPS (clamped 1-120)
.theme(Theme) -> Self                                 // Initial theme
.on_tick(FnMut(&mut Ctx, u64)) -> Self               // Tick callback (every tick_interval)
.on_input(FnMut(KeyEvent) -> bool) -> Self            // Keyboard input handler (creates InputRouter)
.tick_interval(u64) -> Self                          // Tick interval in ms (default 250)

// Widget management (requires &mut self)
.add_widget(Box<dyn Widget>, area: Rect) -> WidgetId
.remove_widget(id: WidgetId)
.widget(id: WidgetId) -> Option<WidgetRef>
.widget_mut(id: WidgetId) -> Option<WidgetRefMut>
.widget_count() -> usize

// Theme management
.set_theme(Theme) -> &mut Self                       // Propagates to all widgets

// Command registry
.add_command(BoundCommand)                            // Register AI-inspectable command
.available_commands() -> Vec<BoundCommand>             // Enumerate all widget commands

// Input shield
.shield_input(Duration)                               // Swallow input after mode transitions
.is_input_shielded() -> bool

// Run
.run(FnMut(&mut Ctx)) -> io::Result<()>               // Start the event loop
.stop()                                                // Signal stop (thread-safe, via AtomicBool)

// Metrics
.frame_time_ms() -> f64
.plane_count() -> usize
```

**Widget Lifecycle:**
```
add_widget()
  ├── set_id(id)           ← App assigns WidgetId
  ├── set_area(rect)       ← Initial screen area
  ├── on_mount()           ← Widget initialization
  ├── on_theme_change()    ← Apply current theme
  ├── register focus()     ← Add to focus ring if focusable
  └── auto-focus if first widget

render loop (per frame):
  ├── needs_render()?      ← Skip if not dirty
  ├── render(area) → Plane ← Produce visual output
  ├── clear_dirty()        ← Reset dirty flag
  └── compositor.add_plane()

remove_widget(id):
  ├── on_unmount()          ← Widget cleanup
  ├── remove from registry
  ├── unregister focus()
  └── invalidate z-order cache
```

**Event Loop (run method):**
```
Frame loop (while running):
  1. poll_and_dispatch_input()
     ├── Read stdin bytes via tty::poll_input
     ├── Parse into Events via Parser
     └── handle_event (key, mouse, resize)
  2. render_dirty_widgets()
     └── For each widget: if needs_render(), render() and add_plane()
  3. run_tick_callback()
     └── If tick_interval elapsed, call on_tick closure
  4. run_periodic_commands()
     └── For widgets with refresh_seconds: re-run command, apply output
  5. User render closure: f(&mut Ctx)
  6. compositor.render(&mut terminal)
  7. Focused cursor positioning
  8. Animation tick
  9. Frame rate limiter (sleep if frame completed early)
  10. Frame counter + timing

Panic safety:
  ├── Install panic hook that restores terminal state
  └── On exit: write theme name to DTRON_THEME_FILE if env var set
```

**Borrow Safety:**
- `widgets` field uses `RefCell<Vec<Box<dyn Widget>>>` for interior mutability
- `WidgetRef` / `WidgetRefMut` wrapper types hide borrow guards from public API
- Framework guarantees borrow safety by never nesting mutable borrows
- Render phase: `borrow()` (immutable iteration)
- Event phase: `borrow_mut()` (mutable iteration)
- Event loop processes one phase at a time: input → tick → render

**Input Shield:**
- After mode transitions (modal open/close, view switch), stale keypresses can leak
- `shield_input(duration)` swallows all key/mouse events for cooldown period
- Resize events are NOT shielded (must always be processed)
- Used after: modal/overlay dismiss, view transitions, command palette close

### 4.2 Ctx (`src/framework/ctx.rs`)

**Purpose:** Application context passed to render and tick callbacks. Provides access to compositor, theme, animations, focus, dirty regions, scene router, event bus.

**Public API:**

```rust
// Rendering
add_plane(plane)                                    // Add plane to compositor
show_cursor() / hide_cursor() / set_cursor(col, row) // Cursor control

// Terminal lifecycle
suspend_terminal() -> io::Result<()>                // Restore normal mode (for child processes)
resume_terminal() -> io::Result<()>                 // Re-enter raw mode + full redraw

// Focus
set_focus(id: WidgetId)
focused() -> Option<WidgetId>

// Dirty regions
mark_dirty(x, y, width, height)
mark_all_dirty()
needs_full_refresh() -> bool

// Compositor
compositor() -> &Compositor
compositor_mut() -> &mut Compositor
widget_count() -> usize
plane_count() -> usize
frame_time_ms() -> f64
fps() -> u64

// Theme
theme() -> &Theme
set_theme(Theme)                                    // Changes theme (detected by App::run)

// UI
clear()                                             // Force full terminal clear

// Split panes
split_h(|left, right|)                              // Horizontal split (50/50)
split_v(|top, bottom|)                              // Vertical split (50/50)

// Scene router
scene_router() -> &mut SceneRouter
push_scene(id: &str)
pop_scene() -> bool
replace_scene(id: &str)
go_to_scene(id: &str)

// Event bus
publish(event: E)
subscribe::<E, F>(callback) -> SubscriptionId
event_bus() -> &EventBus

// Layout
layout(constraints: Vec<Constraint>) -> Vec<Rect>

// Commands
run_command(cmd: &str) -> (String, String, i32)
available_commands() -> Vec<BoundCommand>

// App control
stop()
```

### 4.3 Widget Trait (`src/framework/widget.rs`)

**Purpose:** Core trait implemented by all framework widgets.

**Decomposition into sub-traits:**

| Sub-trait | Methods | Purpose |
|-----------|---------|---------|
| `Renderable` | `render()`, `needs_render()`, `mark_dirty()`, `clear_dirty()` | Rendering lifecycle |
| `Focusable` | `focusable()`, `on_focus()`, `on_blur()`, `cursor_position()` | Focus management |
| `Themable` | `on_theme_change()`, `current_theme()` | Theme propagation |
| `Commandable` | `commands()`, `apply_command_output()` | CLI command binding |
| `InputHandler` | `handle_key()`, `handle_mouse()` | Input event handling |

**`Widget` trait (full):**

```rust
trait Widget {
    // Identity & Geometry
    fn id(&self) -> WidgetId;
    fn set_id(&mut self, id: WidgetId);
    fn area(&self) -> Rect;                          // ratatui::layout::Rect
    fn set_area(&mut self, area: Rect);
    fn z_index(&self) -> u16;                        // Default: 0

    // Rendering
    fn render(&self, area: Rect) -> Plane;           // &self — no side effects
    fn draw_to(&mut self, target: &mut Plane, x: u16, y: u16);  // Direct-plane optimization
    fn needs_render(&self) -> bool;                  // Default: true
    fn mark_dirty(&mut self);
    fn clear_dirty(&mut self);

    // Focus
    fn focusable(&self) -> bool;                     // Default: true
    fn on_focus(&mut self);
    fn on_blur(&mut self);
    fn cursor_position(&self) -> Option<(u16, u16)>; // For text input cursors

    // Lifecycle
    fn on_mount(&mut self);                          // Called at registration
    fn on_unmount(&mut self);                        // Called at removal

    // Theme
    fn on_theme_change(&mut self, theme: &Theme);
    fn current_theme(&self) -> Option<Theme>;        // For pattern 2 theme sync

    // Input
    fn handle_key(&mut self, key: KeyEvent) -> bool;  // True = consumed
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool;

    // Commands
    fn commands(&self) -> Vec<BoundCommand>;
    fn apply_command_output(&mut self, output: &ParsedOutput);
}
```

**Widget sub-traits (for generic bounds):**
- `WidgetId(pub usize)` — unique identifier with `AtomicUsize` auto-counter (`WidgetId::next()`)
- `WidgetState` trait for JSON serialization: `state_id()`, `to_json()`, `apply_json()`
- `AsyncWidget` trait (feature-gated `async`): `on_mount_async()`, `on_unmount_async()`

**Z-Index Ranges:**

| Range | Layer |
|-------|-------|
| 0 | Background/base widgets |
| 5 | Content areas (panels, split panes) |
| 10 | Interactive widgets (lists, forms, editors) |
| 50 | Overlays (tooltips, dropdowns) |
| 100 | Modal dialogs |
| 500 | Toasts/notifications |
| 9000 | Drag ghost (reserved) |

### 4.4 Keybinding System (`src/framework/keybindings.rs`)

**Purpose:** Config-driven keybinding resolution with TOML support.

**Resolution Order:**
1. Engine defaults (compiled-in)
2. User global: `~/.config/dracon/dracon.toml`
3. Project-local: `./dracon.toml` (highest priority)

**String Format:**
- Simple: `"q"`, `"?"`, `"esc"`, `"enter"`, `"tab"`, `"backspace"`, `"up"`, `"down"`
- Modifiers: `"ctrl+q"`, `"ctrl+t"`, `"alt+f4"`, `"shift+tab"`
- Multi-modifier: `"ctrl+shift+t"`

**Standard Actions:**

| Action Constant | Default | Purpose |
|-----------------|---------|---------|
| `actions::QUIT` | `ctrl+q` | Exit application |
| `actions::HELP` | `f1` | Toggle help overlay |
| `actions::BACK` | `esc` | Dismiss/go back |
| `actions::THEME` | `ctrl+t` | Cycle theme |
| `actions::SUBMIT` | `enter` | Confirm/submit |
| `actions::SEARCH` | `ctrl+f` | Open search |
| `actions::NEW` | `ctrl+n` | New item/tab |
| `actions::CLOSE` | `ctrl+w` | Close item/tab |
| `actions::SAVE` | `ctrl+s` | Save |
| `actions::COPY` | `ctrl+c` | Copy |
| `actions::PASTE` | `ctrl+v` | Paste |
| `actions::CUT` | `ctrl+x` | Cut |
| `actions::DELETE` | `delete` | Delete |
| `actions::REFRESH` | `f5` | Refresh/reload |
| `actions::PAUSE` | `ctrl+p` | Pause/resume |

**Caching:**
- `resolve_keybindings()` caches result after first call
- `invalidate_keybinding_cache()` forces re-resolution
- `KeybindingSet::from_config(config)` — create from resolved config
- `KeybindingSet::matches(action, key_event)` — query if key matches an action

**Philosophy:**
- Modifier keys for actions (never single letters)
- Single-letter keys reserved for text input
- Exceptions: `↑/↓/←/→` (navigation), `Enter` (selection), `Tab` (focus), `Backspace` (delete) — universal, hardcoded

### 4.5 Focus System (`src/framework/focus.rs`)

**Purpose:** Manages widget focus ordering, tab navigation, and focus trapping.

**`FocusManager` API:**
- `register(id, focusable)` — add widget to tab-order ring
- `unregister(id)` — remove widget from ring
- `set_focus(id)` — set focused widget
- `focused() -> Option<WidgetId>` — get current focus
- `tab_next() / tab_prev() -> bool` — cycle focus forward/backward
- `set_trapped(bool)` — lock focus within current set (for modals)

**Callbacks:**
- `on_focus_change: Vec<Arc<FocusCallback>>` — focus transition callbacks
- `on_trap_change: Vec<Arc<TrapCallback>>` — trap enter/exit callbacks
- `on_focus_change_internal: Vec<FocusChangeCallback>` — old/new focus callbacks

### 4.6 Hit Zone System (`src/framework/hitzone.rs`)

**Purpose:** Declarative mouse event routing with click, double-click, triple-click, right-click, drag, and hover detection.

| Component | Description |
|-----------|-------------|
| `HitZone<T>` | Rectangular zone with callbacks for all mouse interactions |
| `HitZoneGroup<T>` | Multi-zone dispatcher to first matching zone |
| `ScopedZone<T>` | Lightweight geometry-only zone (no callbacks) |
| `ScopedZoneRegistry<T>` | Per-frame scoped registry: clear, register during render, dispatch in mouse handler |

**HitZone Callbacks:**
- `on_click(ClickKind)` — Single/Double/Triple click detection with timeout
- `on_right_click()` — Right-click handler
- `on_drag_start(DragState)` / `on_drag_move(DragState)` / `on_drag_end(DragState)`

**Click Detection:**
- Tracks `last_click_time` and `last_click_pos`
- Double-click within `double_click_timeout` (default 300ms)
- Triple-click: three rapid clicks within timeout
- Drag active flag to distinguish click from drag

### 4.7 Drag-and-Drop (`src/framework/dragdrop.rs`)

**Purpose:** Full drag-and-drop lifecycle with visual ghost rendering.

| Component | Description |
|-----------|-------------|
| `DragItem<T>` | Payload with data and source ID |
| `DragGhost` | Visual ghost rendered during drag at z=9000 |
| `DropTarget<T>` | Rectangular target zone with accept/reject |
| `DragManager<T>` | State machine: Idle → Dragging → Dropped/Cancelled |

**DragManager API:**
- `start_drag(item, col, row)` — begin drag
- `move_ghost(col, row)` — update ghost position
- `end_drag(col, row, targets)` — check drop targets, complete or cancel
- `cancel()` — abort drag
- `is_dragging() -> bool` / `current_item() -> Option<&DragItem<T>>`

### 4.8 Marquee Selection (`src/framework/marquee.rs`)

**Purpose:** Rectangle-based drag selection for List, Table, Tree, Kanban widgets.

**State Machine:**
```
Idle → Tracking (MouseDown)
Tracking → Active (Drag exceeds threshold)
Tracking → Idle (MouseUp without exceeding threshold → resolve pending_click)
Active → Idle (MouseUp → commit selection)
Active → Idle (Escape / MouseMove → cancel)
```

**`MarqueeState` API:**
- `start_tracking(col, row)` — begin potential marquee
- `defer_click(item_index)` — defer plain click selection
- `update(col, row) -> bool` — update during drag; returns true if just activated
- `rect() -> Option<MarqueeRect>` — normalized bounding rectangle
- `take_pending_click() -> Option<usize>` — resolve deferred click
- `is_active -> bool` — whether marquee is currently active
- `clear()` / `reset()` — cancel or reset state

**Key Design:**
- Deferred click pattern: plain clicks don't immediately change selection
- Staggered thresholds: marquee at 2px, file drag at 3px
- Marquee and drag-drop are mutually exclusive
- `Ctrl+drag` toggles items into selection instead of replacing
- Border-only rendering (╭╮╰╯─│) with `theme.primary` + BOLD, no background fill

### 4.9 Layout Engine (`src/framework/layout.rs`)

**Purpose:** Constraint-based layout computation, inspired by CSS flexbox and ratatui's Layout.

| Constraint | Description |
|------------|-------------|
| `Percentage(u16)` | Percentage of available space (0-100) |
| `Fixed(u16)` | Fixed size in cells |
| `Min(u16)` | Minimum size (grows to fill remaining) |
| `Max(u16)` | Maximum size (shrinks to fit) |
| `Ratio(u16, u16)` | Ratio of remaining space (numerator/denominator) |

**`Layout` builder:**
- `Layout::new(constraints)` / `Layout::horizontal(constraints)`
- `Layout::vertical(constraints)` / `.direction(Direction)`
- `.spacing(u16)` / `.margin(u16)`
- `.layout(area: Rect) -> Vec<Rect>` — compute child rectangles
- `.with_caching()` — cache results for repeated calls

**Direction:**
- `Direction::Horizontal` — distribute left-to-right (default)
- `Direction::Vertical` — distribute top-to-bottom

### 4.10 Scroll System (`src/framework/scroll.rs`)

**Purpose:** Scroll position tracking and scrollable container.

**`ScrollState`:**
- `offset: usize`, `content_height: usize`, `viewport_height: usize`
- `max_offset()` / `page_size()` / `scroll_up(n)` / `scroll_down(n)` / `scroll_to(offset)`
- `start_row()` / `end_row()` / `is_scrollable()` / `visible_range()`

**`ScrollContainer`:** Wraps content with scrollbar rendering.
- `render(content_plane, area, state, theme) -> Plane`
- Renders proportional scrollbar thumb (`▐`) with `theme.primary`
- Default scrollbar width: `DEFAULT_SCROLLBAR_WIDTH = 1`

### 4.11 Animation System (`src/framework/animation.rs`)

**Purpose:** Tweening animations with easing curves.

| Component | Description |
|-----------|-------------|
| `Animation` | Keyframe-based animation on widget properties (position, size) |
| `AnimationManager` | Manages active animations, ticks all on frame |
| `Easing` | Interpolation functions: Linear, Sine, Quadratic, Cubic, Exponential, Elastic, Bounce, Back |
| `EasingType` | In, Out, InOut variants for each easing function |

**`Animation` fields:**
- `duration: Duration`, `elapsed: Duration`, `easing: Easing`
- `start_value: f64`, `end_value: f64`, `current_value: f64`
- `looping: bool`, `yoyo: bool`, `completed: bool`
- `on_complete: Option<Box<dyn FnOnce()>>`

**`AnimationManager`:** `add(animation)`, `tick()`, `clear()`, `active_count()`

### 4.12 Dirty Region Tracking (`src/framework/dirty_regions.rs`)

**Purpose:** Efficient partial screen update tracking.

**`DirtyRegionTracker`:**
- `mark_dirty(x, y, width, height)` — add a rectangular dirty region (max 256 tracked)
- `mark_all_dirty()` — flag for full refresh
- `clear()` — reset all regions
- `needs_full_refresh() -> bool`
- `dirty_regions() -> &[DirtyRegion]` — list of current dirty regions

### 4.13 I18n (`src/framework/i18n.rs`)

**Purpose:** Basic internationalization via JSON locale files.

**`I18n`:**
- `new(default_locale)` — create with default locale
- `load_locale(code)` — load JSON locale file
- `set_locale(code)` — switch active locale
- `t(key) -> &str` — translate key
- `tr!(key)` — macro for compile-time key validation

**Locale file format:** `{ "greeting": "Hello", "items": { "one": "1 item", "many": "{count} items" } }`

---

## 5. Widget System

### 5.1 Complete Framework Widget Inventory (47 Widgets)

| # | Widget | File | Description | Hover | Focus | Scroll | Keys | Mouse |
|---|--------|------|-------------|-------|-------|--------|------|-------|
| 1 | `Autocomplete` | autocomplete.rs | Type-ahead text input with suggestions dropdown | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2 | `Breadcrumbs` | breadcrumbs.rs | Hierarchical path with clickable segments | ✅ | ❌ | ❌ | ❌ | ✅ |
| 3 | `Button` | button.rs | Clickable button with press state | ✅ | ❌ | ❌ | ❌ | ✅ |
| 4 | `Calendar` | calendar.rs | Date picker with month/year navigation | ✅ | ✅ | ❌ | ✅ | ✅ |
| 5 | `Checkbox` | checkbox.rs | Two-state toggle with check mark | ✅ | ❌ | ❌ | ❌ | ✅ |
| 6 | `ColorPicker` | color_picker.rs | Color swatch picker with hex/RGB input | ✅ | ✅ | ❌ | ✅ | ✅ |
| 7 | `CommandPalette` | command_palette.rs | Filterable command overlay with search | ✅ | ✅ | ✅ | ✅ | ✅ |
| 8 | `ConfirmDialog` | confirm_dialog.rs | Modal yes/no with optional danger styling | ❌ | ✅ | ❌ | ✅ | ✅ |
| 9 | `ContextMenu` | context_menu.rs | Right-click popup with nested submenus | ✅ | ✅ | ✅ | ✅ | ✅ |
| 10 | `DebugOverlay` | debug_overlay.rs | FPS, widget count, and debug info | ❌ | ❌ | ❌ | ❌ | ❌ |
| 11 | `Divider` | divider.rs | Horizontal/vertical separator line | ❌ | ❌ | ❌ | ❌ | ❌ |
| 12 | `EventLogger` | event_logger.rs | Scrollable event log panel | ❌ | ✅ | ✅ | ✅ | ✅ |
| 13 | `Form` | form.rs | Multi-field form container with validation | ❌ | ✅ | ✅ | ✅ | ✅ |
| 14 | `Gauge` | gauge.rs | Filled progress bar with warn/crit thresholds | ❌ | ❌ | ❌ | ❌ | ❌ |
| 15 | `Hud` | hud.rs | Top-right HUD with system metrics | ❌ | ❌ | ❌ | ❌ | ❌ |
| 16 | `Kanban` | kanban.rs | Kanban board with draggable columns/cards | ✅ | ❌ | ✅ | ✅ | ✅ |
| 17 | `KeyValueGrid` | key_value_grid.rs | Key-value display from JSON/Scalar CLI output | ❌ | ❌ | ✅ | ✅ | ✅ |
| 18 | `Label` | label.rs | Static text label | ❌ | ❌ | ❌ | ❌ | ❌ |
| 19 | `List` | list.rs | Scrollable list with keyboard/touch navigation | ✅ | ✅ | ✅ | ✅ | ✅ |
| 20 | `LogViewer` | log_viewer.rs | Auto-scrolling log with severity detection | ❌ | ✅ | ✅ | ✅ | ✅ |
| 21 | `MenuBar` | menu_bar.rs | Top menu bar with dropdown menus | ✅ | ✅ | ❌ | ✅ | ✅ |
| 22 | `Modal` | modal.rs | Modal dialog overlay with backdrop | ❌ | ✅ | ❌ | ✅ | ✅ |
| 23 | `NotificationCenter` | notification_center.rs | Queued notification display with auto-dismiss | ❌ | ✅ | ✅ | ✅ | ✅ |
| 24 | `PasswordInput` | password_input.rs | Password input with masking | ❌ | ✅ | ❌ | ✅ | ✅ |
| 25 | `Profiler` | profiler.rs | Frame timing profiler with bar chart | ❌ | ❌ | ❌ | ❌ | ❌ |
| 26 | `ProgressBar` | progress_bar.rs | Animated progress indicator | ❌ | ❌ | ❌ | ❌ | ❌ |
| 27 | `ProgressRing` | progress_ring.rs | Circular progress indicator | ❌ | ❌ | ❌ | ❌ | ❌ |
| 28 | `Radio` | radio.rs | Radio button group (single selection) | ✅ | ❌ | ❌ | ✅ | ✅ |
| 29 | `RichText` | rich_text.rs | Rich text display with formatting (headers, bold, italic, code) | ❌ | ❌ | ✅ | ✅ | ✅ |
| 30 | `SearchInput` | search_input.rs | Search input with clear button | ❌ | ✅ | ❌ | ✅ | ✅ |
| 31 | `Select` | select.rs | Dropdown select/combobox | ✅ | ✅ | ✅ | ✅ | ✅ |
| 32 | `Slider` | slider.rs | Horizontal slider with value display | ❌ | ✅ | ❌ | ✅ | ✅ |
| 33 | `Sparkline` | sparkline.rs | Mini inline chart for trending data | ❌ | ❌ | ❌ | ❌ | ❌ |
| 34 | `Spinner` | spinner.rs | Animated loading spinner | ❌ | ❌ | ❌ | ❌ | ❌ |
| 35 | `SplitPane` | split.rs | Resizable split panel with draggable divider | ✅ | ❌ | ❌ | ✅ | ✅ |
| 36 | `StatusBadge` | status_badge.rs | Colored status badge (OK/WARN/ERROR) | ❌ | ❌ | ❌ | ❌ | ❌ |
| 37 | `StatusBar` | status_bar.rs | Bottom status bar with segments | ❌ | ❌ | ❌ | ❌ | ❌ |
| 38 | `StreamingText` | streaming_text.rs | Live-updating text with word-wrap | ❌ | ❌ | ✅ | ❌ | ❌ |
| 39 | `TabBar` | tabbar.rs | Tab bar for panel switching | ✅ | ❌ | ❌ | ✅ | ✅ |
| 40 | `Table` | table.rs | Multi-column sortable data table | ✅ | ✅ | ✅ | ✅ | ✅ |
| 41 | `TagsInput` | tags_input.rs | Tag input with autocomplete and remove | ✅ | ✅ | ❌ | ✅ | ✅ |
| 42 | `TextEditorAdapter` | text_editor_adapter.rs | Framework adapter for standalone TextEditor | ❌ | ✅ | ❌ | ✅ | ✅ |
| 43 | `Toast` | toast.rs | Temporary notification toast messages | ❌ | ❌ | ❌ | ❌ | ❌ |
| 44 | `Toggle` | toggle.rs | Two-state on/off toggle switch | ✅ | ❌ | ❌ | ❌ | ✅ |
| 45 | `Tooltip` | tooltip.rs | Hover tooltip popup | ❌ | ❌ | ❌ | ❌ | ✅ |
| 46 | `Tree` | tree.rs | Collapsible tree view with expand/collapse | ✅ | ✅ | ✅ | ✅ | ✅ |
| 47 | `WidgetInspector` | widget_inspector.rs | Widget tree debugging inspector | ❌ | ❌ | ✅ | ✅ | ✅ |

### 5.2 Additional Framework Types (Helper Structs)

| Type | Module | Description |
|------|--------|-------------|
| `Column` | table.rs | Table column definition (header, width) |
| `TableRow<T>` | table.rs | Table row data wrapper |
| `CellTextFn<T>` | table.rs | Table cell text formatter closure alias |
| `ConfirmResult` | confirm_dialog.rs | Yes/No/Cancel enum |
| `ContextAction` | context_menu.rs | Context menu action definition |
| `LoggedEvent` | event_logger.rs | Event log entry |
| `FormField` | form.rs | Form field definition (label, input type, validation) |
| `ValidationRule` | form.rs | Form validation rule |
| `LogLevel` | log_viewer.rs | Log severity level enum |
| `LogLine` | log_viewer.rs | Log line entry |
| `MenuEntry` | menu_bar.rs | Menu bar top-level entry |
| `MenuItem` | menu_bar.rs | Dropdown menu item |
| `ModalResult<T>` | modal.rs | Modal result wrapper |
| `NotificationKind` | notification_center.rs | Notification severity enum |
| `Orientation` | split.rs | Horizontal/Vertical enum |
| `Metric` | profiler.rs | Profiler metric entry |
| `ScrollState` | scroll.rs | Scroll position state |
| `TreeNode` | tree.rs | Tree node wrapper |
| `DragState` | hitzone.rs | HitZone drag state |
| `DragGhost` | dragdrop.rs | Drag operation ghost |
| `DragPhase` | dragdrop.rs | Drag lifecycle phase |
| `Animation` | animation.rs | Animation definition |
| `AnimationManager` | animation.rs | Animation controller |
| `Easing` | animation.rs | Easing function enum |
| `FocusManager` | focus.rs | Tab-order focus ring |
| `DirtyRegion` | dirty_regions.rs | Dirty region for optimization |
| `DirtyRegionTracker` | dirty_regions.rs | Dirty region tracking |
| `EventBus` | event_bus.rs | Pub/sub event system |
| `Reactive<T>` | event_bus.rs | Observable value wrapper |
| `SubscriptionId` | event_bus.rs | Event subscription handle |
| `NavigationEvent` | scene_router.rs | Scene navigation event |
| `Scene` | scene_router.rs | Scene trait for router |
| `SceneRouter` | scene_router.rs | Scene navigation controller |
| `PluginRegistry` | plugin.rs | Dynamic widget loading |
| `WidgetFactory` | plugin.rs | Widget factory trait |
| `KeybindingSet` | keybindings.rs | Keybinding configuration |
| `KeybindingConfig` | keybindings.rs | Keybinding loader |
| `Constraint` | layout.rs | Layout constraint |
| `Direction` | layout.rs | Layout direction |
| `Layout` | layout.rs | Constraint-based layout engine |
| `ScrollContainer` | scroll.rs | Scrollable container wrapper |

### 5.3 Standalone Widgets (`src/widgets/`)

| Widget | File | LOC | Description | Dependencies |
|--------|------|-----|-------------|-------------|
| `TextEditor` | editor.rs | 3,025 | Full-featured code editor with syntax highlighting, undo/redo, search/filter, multi-cursor, clipboard | syntect (feature-gated) |
| `TextInput` | input.rs | — | Single-line text input with cursor, selection, IME | none |
| `Button` | button.rs | — | Standalone button widget (not framework Button) | none |
| `Panel` | panel.rs | — | Bordered panel container | none |
| `Component` | component.rs | — | Base component trait | none |
| `HotkeyHint` | hotkey.rs | — | Keyboard shortcut hint display | none |
| `ContextMenuAction` | context_menu.rs | — | Context menu action type | none |
| `EditorSearch` | editor_search.rs | — | TextEditor inline search/filter UI | syntect (feature-gated) |

**TextEditor Key Features:**
- Syntax highlighting via syntect (20+ language grammars)
- Undo/redo stack (persisted to `.file.undo`)
- Line numbers, word wrap, indent guides
- Status bar
- Search/filter/replace mode
- Selection (single cursor + basic multi-cursor)
- Goto line
- File I/O: `open(&path)`, `save()`, `save_as(&path)`
- Config persistence: `.file.dte.json`

### 5.4 Widget Rendering Pattern

All framework widgets MUST follow these rendering conventions:

**Background Fill:**
```rust
fn render(&self, area: Rect) -> Plane {
    let mut plane = Plane::new(0, area.width, area.height);
    plane.fill_bg(self.theme.bg);  // Fills all cells with theme.bg
    // ... render content on top
    plane
}
```

**Exception — StatusBar:** Uses `Color::Reset` for default fg/bg to inherit terminal defaults.
**Exception — Standalone widgets** (editor, input, hotkey): Use hardcoded `Color::Black` for cursor/highlight contrast (not theme-aware).

**Hover Pattern:**
```rust
// In struct: hovered: Option<usize>,
// In render: check self.hovered for hover_bg
// In handle_mouse: set/clear hovered on Moved, always clear on out-of-bounds
```

**Focus Pattern:**
```rust
// In struct: focused: bool,
// In render: use self.theme.focus_bg or self.theme.focus_border when focused
// Set via on_focus()/on_blur()
```

**Scrollbar Indicator:**
```rust
// Proportional thumb at area.width - 2
// Char: '▐', Color: theme.primary
// Height: (visible/total) * content_height
// Position: (offset/(total-visible)) * (content_height - thumb_height)
```

### 5.5 Command Palette Widget Details

**`CommandPalette`** — Filterable command overlay:
- `CommandItem { id, name, category }` — command definition
- `.with_size(w, h)` — set overlay dimensions
- `.show()` / `.hide()` / `.is_visible()` — visibility control
- `.on_execute(cb)` — callback when command selected
- Keyboard: ↑/↓ navigate, Enter execute, Esc dismiss, type to filter
- Mouse: click items, click outside dismiss, scroll wheel
- Uses `ScopedZoneRegistry<usize>` for mouse dispatch
- Semi-transparent backdrop

### 5.6 Table Widget Details

**`Table<T>`** — Multi-column sortable data table:
- Builder: `.with_columns(cols)`, `.on_header_click(f)`, `.with_cell_text_fn(f)`
- Sorting: `.set_sort(column, ascending)`, sort indicators (▲/▼)
- State: `TableState` snapshot for undo/redo
- Selection: single row selection + multi-select via `selected_indices: HashSet<usize>`
- Navigation: keyboard (↑/↓/Home/End/PageUp/PageDown) + mouse (click row, scroll)
- `SelectCallback<T>` / `CellTextFn<T>` / `HeaderClickCallback` type aliases
- Drag-and-drop support for reordering

### 5.7 Form Widget Details

**`Form`** — Multi-field form container:
- `FormField { label, input_type, validation, placeholder, required }`
- `ValidationRule { rule_type, value, message }` — min/max/required/pattern/match
- Renders fields with labels, validation hints (✓/✗), focus styling
- Tab/Shift+Tab between fields
- Submit callback on Enter
- Mouse: click fields to focus

**Form Input Types:**
- Text, Password, Email, Number, Search
- Select (dropdown), Checkbox, Toggle, Radio, Slider
- Date, Color, TextArea (multi-line)

### 5.8 SplitPane Widget Details

**`SplitPane`** — Resizable split panel:
- `.new(orientation)` — Horizontal (left-right) or Vertical (top-bottom)
- `.ratio(f32)` — initial ratio (0.0-1.0)
- `.min_size(u16)` — minimum pane size
- `.split(area) -> (Rect, Rect)` — compute child rectangles
- Divider drag resize: track `is_drag` state, update ratio on mouse drag
- Render divider as `│` (vertical) or `─` (horizontal) with `theme.divider`

---

## 6. Theme System

### 6.1 Complete Theme Inventory (21 Themes)

| # | Name | `.name` field | Kind | Constructor | Key Colors |
|---|------|---------------|------|-------------|------------|
| 1 | Dark | `"dark"` | Dark | `Theme::dark()` | Green accents on dark blue-gray |
| 2 | Light | `"light"` | Light | `Theme::light()` | Blue accents on white |
| 3 | High Contrast | `"high_contrast"` | Dark | `Theme::high_contrast()` | Pure black/white, vivid colors |
| 4 | Cyberpunk | `"cyberpunk"` | Dark | `Theme::cyberpunk()` | Neon green + hot pink on black |
| 5 | Dracula | `"dracula"` | Dark | `Theme::dracula()` | Dark purple, vivid accents |
| 6 | Nord | `"nord"` | Dark | `Theme::nord()` | Arctic blue-gray palette |
| 7 | Catppuccin Mocha | `"catppuccin_mocha"` | Dark | `Theme::catppuccin_mocha()` | Warm pastel dark |
| 8 | Gruvbox Dark | `"gruvbox_dark"` | Dark | `Theme::gruvbox_dark()` | Retro warm, earthy tones |
| 9 | Tokyo Night | `"tokyo_night"` | Dark | `Theme::tokyo_night()` | Vivid blue on dark |
| 10 | Solarized Dark | `"solarized_dark"` | Dark | `Theme::solarized_dark()` | Precision dark |
| 11 | Solarized Light | `"solarized_light"` | Light | `Theme::solarized_light()` | Precision light |
| 12 | One Dark | `"one_dark"` | Dark | `Theme::one_dark()` | Atom editor dark |
| 13 | Rosé Pine | `"rose_pine"` | Dark | `Theme::rose_pine()` | Elegant muted rose |
| 14 | Kanagawa | `"kanagawa"` | Dark | `Theme::kanagawa()` | Hokusai art (deep blues + golds) |
| 15 | Everforest | `"everforest"` | Dark | `Theme::everforest()` | Forest green |
| 16 | Monokai | `"monokai"` | Dark | `Theme::monokai()` | Classic syntax highlighting |
| 17 | Warm | `"warm"` | Dark | `Theme::warm()` | Amber and bronze |
| 18 | Cool | `"cool"` | Dark | `Theme::cool()` | Purple and ice blue |
| 19 | Forest | `"forest"` | Dark | `Theme::forest()` | Moss green and pine |
| 20 | Sunset | `"sunset"` | Dark | `Theme::sunset()` | Orange coral and pink |
| 21 | Mono | `"mono"` | Dark | `Theme::mono()` | Soft silver monochrome |

### 6.2 Theme Struct Fields (Semantic Color System)

**Surface/Elevation:**
- `bg` — Root viewport background
- `surface` — Panel/card surface (slightly elevated)
- `surface_elevated` — Dropdowns, dialogs (highest surface)

**Text Hierarchy:**
- `fg` — Primary text
- `fg_muted` — Secondary text (labels, descriptions)
- `fg_subtle` — Tertiary text (placeholders, hints)
- `fg_on_accent` — Text color on accent backgrounds

**Interactive / Primary:**
- `primary` — Primary action color
- `primary_hover` — Hover state
- `primary_active` — Active/pressed state

**Secondary:**
- `secondary` / `secondary_hover` / `secondary_active`

**Borders:**
- `outline` — Standard borders
- `outline_variant` — Subtle borders
- `divider` — Section dividers

**Semantic:**
- `error` / `error_bg` — Error states
- `success` / `success_bg` — Success states
- `warning` / `warning_bg` — Warning states
- `info` / `info_bg` — Info states

**Selection:**
- `selection_bg` — Selected item background
- `selection_fg` — Selected item foreground

**Input Fields:**
- `input_bg` / `input_fg` / `input_border`

**Scrollbar:**
- `scrollbar_track` / `scrollbar_thumb` / `scrollbar_thumb_hover`
- `scrollbar_width` — **Deprecated** (use `DEFAULT_SCROLLBAR_WIDTH`)

**Disabled:**
- `disabled_fg` / `disabled_bg`

**Focus/Hover:**
- `hover_bg` — Hovered item background
- `focus_bg` — Focused element background
- `focus_border` — Focused element border

### 6.3 Theme Helpers

| Method | Description |
|--------|-------------|
| `Theme::from_name(name: &str) -> Option<Theme>` | Case-insensitive lookup with hyphen/underscore normalization |
| `Theme::from_env_or(default: Theme) -> Theme` | Read `DTRON_THEME` env var, fallback to default |
| `Theme::random() -> Theme` | Select random built-in theme |
| `Theme::all() -> Vec<Theme>` | Return all 21 themes (not const — uses runtime vec) |

**`from_env_or` behavior:** Reads `DTRON_THEME` env var, does case-insensitive lookup. Falls back to `default` if env var is unset, empty, or names unknown theme.

**Hyphen/underscore normalization:** All `from_name` lookups normalize `-` to `_` before matching.

---

## 7. Input System

### 7.1 Event Types (`src/input/event.rs`)

```rust
enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Paste(String),
    FocusGained,
    FocusLost,
    Unsupported(Vec<u8>),
}

struct KeyEvent {
    code: KeyCode,           // Char, Enter, Tab, Backspace, Esc, F(n), arrows, etc.
    modifiers: KeyModifiers, // SHIFT, CONTROL, ALT, SUPER, HYPER, META
    kind: KeyEventKind,      // Press, Repeat, Release
}

enum KeyCode {
    Char(char), Backspace, Enter, Left, Right, Up, Down,
    Home, End, PageUp, PageDown, Tab, BackTab, Delete, Insert,
    F(u8), Esc, Null, CapsLock, ScrollLock, NumLock,
    PrintScreen, Pause, Menu, KeypadBegin,
    Media(MediaKeyCode), Modifier(ModifierKeyCode),
}

struct MouseEvent {
    kind: MouseEventKind,   // Down, Up, Drag, Moved, ScrollDown/Up/Left/Right
    column: u16, row: u16,
    modifiers: KeyModifiers,
}

enum MouseButton { Left, Right, Middle, Back, Forward, Other(u8) }
```

### 7.2 Parser (`src/input/parser.rs`)

**Purpose:** Byte-level event parser. Reads raw terminal bytes and produces `Event` values.

- Handles: ANSI escape sequences, CSI sequences, SGR mouse, kitty keyboard, bracketed paste
- `advance(byte) -> Option<Event>` — feed one byte, get event if complete
- `check_timeout() -> Option<Event>` — handle pending sequences after timeout
- Event buffering: accumulates bytes until complete sequence detected
- Supports: all key types, mouse events, resize events, paste events

### 7.3 Reader (`src/input/reader.rs`)

**Purpose:** High-level input reading with optional blocking/polling.

- `InputReader::new(stdin)` — create reader
- `read() -> io::Result<Option<Event>>` — read next event (non-blocking)
- `read_blocking() -> io::Result<Event>` — blocking read

### 7.4 Input Router Pattern

`App::on_input(handler)` creates a hidden `InputHandler` widget that:
- Has full-screen area (terminal dimensions)
- Is focusable (receives keyboard focus)
- Returns `needs_render() = false` (no visual output)
- Delegates `handle_key()` to the closure
- Returns `current_theme()` for pattern 2 theme sync

### 7.5 Modifier Guards

All examples must check `key.modifiers.is_empty()` on non-configurable `Char` handlers to prevent Ctrl+X from triggering actions:

```rust
// RIGHT — Ctrl+P is ignored
KeyCode::Char('p') if key.modifiers.is_empty() => { self.paused = true; true }
// WRONG — Ctrl+P triggers pause
KeyCode::Char('p') => { self.paused = true; true }
```

---

## 8. Compositor & Rendering

### 8.1 Plane Lifecycle

```
Creation: Plane::new(id, width, height)  → allocates Vec<Cell> (W × H)
  → set_z_index(z)   → render ordering
  → set_absolute_position(x, y)  → compositor-relative position
  → fill_bg(color)   → fill all cells
  → put_str/char     → draw content
  → set_filter(f)    → apply visual filter

Per-frame reset: reset_cells()  → reuse Plane without reallocation
  (sets all cells to transparent defaults)
```

### 8.2 Compositing Algorithm

```
For each frame:
  1. Clear final_buffer with clear_color (or per-region for dirty mode)
  2. Sort planes by z_index (ascending)
  3. For each visible plane:
     a. Compute source/dest bounds (clip to compositor dimensions)
     b. For each cell in overlapping region:
        - If plane has filter: apply filter to cell
        - Blend cell into final_buffer (consider opacity)
        - Non-transparent, non-skip cells overwrite or blend into buffer
  4. Diff final_buffer vs last_frame:
     - Emit cursor-positioning escape sequences for changed cells
     - Emit SGR sequences (fg/bg/style) only when changed
     - Emit character bytes
     - Skip identical cells (no redraw)
  5. Write buffered output to terminal via single write_all()
  6. Copy final_buffer → last_frame for next frame's diff
  7. Clear planes list for next frame
```

### 8.3 Escape Sequence Output

The compositor emits raw ANSI escape sequences (no dependency like crossterm):

- Cursor positioning: `\x1b[{row};{col}H`
- RGB foreground: `\x1b[38;2;{R};{G};{B}m`
- RGB background: `\x1b[48;2;{R};{G};{B}m`
- ANSI foreground: `\x1b[38;5;{N}m`
- ANSI background: `\x1b[48;5;{N}m`
- Style: `\x1b[1m` (bold), `\x1b[3m` (italic), `\x1b[4m` (underline), `\x1b[22m` (bold off), etc.
- Reset fg: `\x1b[39m`, Reset bg: `\x1b[49m`
- Sync mode begin: `\x1b[?2026h`, Sync mode end: `\x1b[?2026l`
- Wraparound disable: `\x1b[?7l`, Wraparound enable: `\x1b[?7h`

### 8.4 Optimization Techniques

| Technique | Description |
|-----------|-------------|
| Dirty regions | Only render cells in changed areas |
| Cell diffing | Skip cells identical to last frame |
| Style tracking | Only emit SGR codes when style changes |
| Final buffer reuse | Pre-allocated buffer, reset per frame |
| Bulk write | Single `write_all()` per frame (Vec<u8> buffer) |
| Fast blit | `copy_from_slice` for fully opaque planes |
| Plane reuse | `reset_cells()` instead of reallocation |
| CellPool | Object pool for Cell allocation |
| Inline cursor positioning | `\x1b[Y;XH` instead of separate sequences |
| Skip cells | Wide-char padding cells are skipped entirely |

### 8.5 Visual Filters (`src/compositor/filter.rs`)

| Filter | Description |
|--------|-------------|
| `Dim` | Reduce brightness (multiply RGB by factor) |
| `Invert` | Invert RGB values |
| `Scanline` | CRT scanline effect (dim every other row) |
| `Pulse` | Pulsing brightness oscillation |
| `Glitch` | Random color offset / character corruption |

### 8.6 CellPool (`src/compositor/pool.rs`)

**Purpose:** Object pool for `Cell` allocation to reduce per-frame allocation pressure.

- `CellPool::new(config)` — create pool with config
- `PoolConfig { initial_capacity, max_capacity }`
- `acquire_plane_cells(pool, count)` — get cells from pool
- `release_plane_cells(pool, cells)` — return cells to pool

---

## 9. Command-Driven Architecture

### 9.1 Architecture

**Design Principle:** Widgets have zero business logic — they only render command output. AI can enumerate every action via `Ctx::available_commands()` and trigger any action by running the same CLI command.

### 9.2 BoundCommand

```rust
struct BoundCommand {
    command: String,                    // CLI command string
    parser: OutputParser,              // How to parse stdout
    confirm_message: Option<String>,    // Confirmation dialog text
    refresh_seconds: Option<u64>,       // Auto-refresh interval
    label: String,                      // Human-readable name
    description: String,                // Human-readable description
}
```

Builder pattern: `.new(cmd)`, `.parser(p)`, `.confirm(msg)`, `.refresh(secs)`, `.label(l)`, `.description(d)`

### 9.3 Output Parsers

| Parser | Variant | Output Type | Use Case |
|--------|---------|-------------|----------|
| `JsonKey` | `{ key: "status" }` | `Scalar` | Extract single field from JSON |
| `JsonPath` | `{ path: "data.cpu" }` | `Scalar` | Navigate nested JSON path |
| `JsonArray` | `{ item_key: "name" }` | `List` | Extract array items |
| `Regex` | `{ pattern: "...", group: 1 }` | `Scalar` | Extract via regex capture |
| `LineCount` | — | `Scalar` | Count output lines |
| `ExitCode` | — | `Scalar` | Map exit code |
| `SeverityLine` | `{ patterns: {"ERROR":"red"} }` | `Lines(LoggedLine)` | Log line severity detection |
| `Plain` | — | `Text` | Raw text output |

### 9.4 ParsedOutput

```rust
enum ParsedOutput {
    Scalar(String),              // Single value (gauge, badge)
    List(Vec<String>),           // List of items (table rows, list items)
    Lines(Vec<LoggedLine>),      // Log lines with severity
    Text(String),                // Raw text (log viewer, streaming text)
    None,                        // No output
}

struct LoggedLine { text: String, severity: String }
```

### 9.5 CommandRunner

```rust
CommandRunner::new(cmd)
  .run_sync() -> (String stdout, String stderr, i32 exit_code)
  .run_and_parse(parser) -> ParsedOutput
  .spawn() -> io::Result<()>      // For streaming commands
  .recv_line() -> Option<String>  // Get next line from spawned process
```

### 9.6 TOML Configuration

Complete apps definable in TOML — no Rust code needed for dashboards:

```toml
title = "My Dashboard"
theme = "nord"
fps = 30

[[widget]]
type = "StatusBadge"
id = 1
bind = "dracon-sync status --json"
parser = { type = "json_key", key = "status" }
refresh = 5

[[widget]]
type = "Gauge"
id = 2
bind = "df -h / | tail -1"
parser = { type = "regex", pattern = r"(\d+)%", group = 1 }
refresh = 10
```

**Config types:**
- `AppConfig` — title, theme, fps, layout, widgets, commands
- `WidgetConfig` — id, type, area, bind, parser, refresh, confirm, label, description, options
- `LayoutConfig` — header_height, sidebar_width, footer_height
- `AreaConfig` — x, y, width, height
- `ParserConfig` — type, key, path, pattern, group, patterns

### 9.7 Ctx Command API

```rust
ctx.run_command("df -h")           // (String, String, i32)
ctx.available_commands()           // Vec<BoundCommand>
```

---

## 10. Event System

### 10.1 EventBus (`src/framework/event_bus.rs`)

**Purpose:** Type-safe, synchronous pub/sub event system for inter-widget communication.

**`EventBus` API:**
- `new()` — create bus
- `publish(event: E)` where `E: Any + Clone` — publish event to all subscribers
- `subscribe::<E, F>(callback)` where `F: Fn(&E) + 'static` — subscribe to event type
- `subscribe_once::<E, F>(callback)` — fire callback once then auto-unsubscribe
- `unsubscribe(id: SubscriptionId)` — remove subscription
- `set_trace(bool)` / `set_max_history(usize)` — debugging

**`Reactive<T>`:** Observable value wrapper.
- `new(value)` — create with initial value
- `get()` — get current value
- `set(value)` — update value and notify subscribers
- `subscribe(callback)` — listen for changes

**`SubscriptionId`:** Opaque handle returned by `subscribe()`, used for `unsubscribe()`.

**`EventRecord`:**
- `timestamp: Instant`, `type_name: String`, `payload: Rc<dyn Any>`
- Circular buffer of recent events for debugging (default max: 100)

### 10.2 Event History & Debugging

- `history()` — returns `Vec<EventRecord>` of recent events
- `clear_history()` — empty the history buffer
- Trace mode: writes events to stderr when `set_trace(true)` (visible when running with `debug_events` feature)

---

## 11. Scene Router

### 11.1 Scene Trait

```rust
trait Scene: Any {
    fn scene_id(&self) -> &str;
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}
    fn on_resume(&mut self) {}
    fn on_pause(&mut self) {}
    fn render(&self, area: Rect) -> Plane;
    fn handle_key(&mut self, key: KeyEvent) -> bool;
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool;
    fn on_theme_change(&mut self, theme: &Theme) {}
    fn needs_render(&self) -> bool;
    fn mark_dirty(&mut self);
    fn clear_dirty(&mut self);
}
```

### 11.2 SceneRouter

**API:**
- `new()` — create empty router
- `register(id, scene)` — register scene by ID
- `push(id)` — push scene onto navigation stack (calls `on_pause` on previous, `on_enter` on new)
- `pop() -> bool` — pop current scene (calls `on_exit`, `on_resume` on previous)
- `replace(id)` — replace current scene without navigation history
- `go(id)` — clear stack and set scene
- `active()` — get current scene ID
- `can_go_back() -> bool` — check if back navigation is possible
- `stack_depth() -> usize` — current stack size
- `with_default_transition(transition)` — set default transition animation

### 11.3 SceneTransition

| Variant | Description |
|---------|-------------|
| `Fade` | Fade out old, fade in new |
| `SlideLeft` | New scene slides in from right |
| `SlideRight` | New scene slides in from left |
| `SlideUp` | New scene slides in from bottom |
| `SlideDown` | New scene slides in from top |
| `None` | Instant switch |

### 11.4 NavigationEvent

Events published on the EventBus when navigation occurs:
- `NavigationEvent::Pushed(String)` — scene ID pushed
- `NavigationEvent::Popped(String)` — scene ID popped
- `NavigationEvent::Replaced(String, String)` — old ID, new ID

---

## 12. Plugin System

### 12.1 PluginRegistry (`src/framework/plugin.rs`)

**Purpose:** Dynamic widget loading by name, without compile-time dependencies.

**API:**
- `new()` — create empty registry
- `register(name, factory)` — register `WidgetFactory` by name
- `create(name, id, theme) -> Option<Box<dyn Widget>>` — instantiate widget by name
- `unregister(name)` — remove registration
- `names() -> Vec<String>` — list registered names
- `is_registered(name) -> bool`

**`WidgetFactory`:** `Box<dyn Fn(WidgetId, Theme) -> Box<dyn Widget> + Send + Sync>`

### 12.2 Example Plugin

`examples/plugin_demo.rs` and `examples/_cookbook/stat_widget_plugin.rs` demonstrate the plugin system.

---

## 13. TextEditor (Standalone Widget)

### 13.1 Scope & Philosophy

**TextEditor is a view/edit widget** for composing into larger applications:
- File managers: view/edit config files
- Chat UIs: edit messages
- Forms: text input fields
- Log viewers: search, filter, navigate

**NOT a vim/Helix competitor.** Not a modal editor. Not LSP-powered.

### 13.2 API

```rust
// Creation
TextEditor::new()                        // Empty editor
TextEditor::with_content("...")         // From string
TextEditor::open(&path)                 // From file (loads .undo too)

// File I/O
editor.save()                           // Save to current path
editor.save_as(&path)                   // Save to new path
editor.file_path()                      // Current path if any

// View options
.with_show_line_numbers(bool)
.with_word_wrap(bool)
.with_indent_guides(bool)
.with_status_bar(bool)
.with_language("rust")                  // For syntax highlighting
.with_theme(theme)                       // Visual theme

// Navigation & Search
.goto_line(line, area)                  // Jump to line
.set_filter("query")                    // Filter/highlight mode
.replace_all(find, replace)             // Global replace
.replace_next(find, replace)            // Next occurrence

// Selection & Clipboard
.get_selected_text()                    // Get selection
.select_all()
.select_word_at(row, col)

// Multi-cursor (basic)
.add_cursor(row, col)                   // Add extra cursor
.clear_extra_cursors()

// Persistence
.load_undo_stack()                      // Load from .file.undo
.save_undo_stack()                      // Save to .file.undo
.load_config()                          // Load from .file.dte.json
.save_config()                          // Save to .file.dte.json

// Search state
editor.search: editor_search::SearchState
//   .filter_query: String              // Current filter string
//   .filtered_indices: Vec<usize>      // display-row → real-line mapping
//   .mode: editor_search::SearchMode   // Normal / Search / Replace / GotoLine
//   .mode_input: String               // Input buffer for search/replace/goto modes
//   .is_replacing: bool               // Whether replace mode is active
```

### 13.3 Editor Size (3,025 LOC)

Largest single file in the project. Contains:
- Public API (~400 LOC)
- Cursor movement logic (~500 LOC)
- Selection logic (~400 LOC)
- Syntax highlighting integration (~300 LOC) — feature-gated (`syntax-highlighting`)
- Undo/redo stack (~400 LOC)
- Search/filter/replace (~500 LOC)
- Rendering (~500 LOC)
- Tests (inline `#[cfg(test)]`)

---

## 14. Application Patterns

### 14.1 Two Rendering Patterns

**Pattern 1: Widget Trait Auto-Render**
```rust
impl Widget for MyApp {
    fn needs_render(&self) -> bool { self.dirty }
    fn render(&self, area: Rect) -> Plane { /* full render */ }
}
```
- App automatically calls `render()` when `needs_render()` returns true
- Set `self.dirty = true` after state changes
- Used by: file_manager, git_tui, sqlite_browser, widget_gallery, dashboard_builder

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
- `needs_render()` returns false
- Used by: system_monitor, ide, chat_client, log_monitor, modal_demo

### 14.3 Bridge Pattern for Pattern 2

Pattern 2 apps (using `on_input`/`on_tick` closures) cannot access app state directly from `on_input`. Use `Rc<RefCell<T>>` to share state between closures:

```rust
let show_help = Rc::new(RefCell::new(false));
let show_help_input = Rc::clone(&show_help);
let show_help_render = Rc::clone(&show_help);

.on_input(move |key| {
    if key.code == KeyCode::Char('?') { *show_help_input.borrow_mut() = !*show_help_input.borrow_mut(); true } else { false }
})
```

For atomic shared state: `Arc<AtomicBool>`.

### 14.4 Pattern 2 Theme Sync

Pattern-2 apps must implement `Widget::current_theme()` to sync local theme back to the framework:

```rust
fn current_theme(&self) -> Option<Theme> {
    Some(self.app.borrow().theme)
}
```

Without this, `DTRON_THEME_FILE` won't contain the correct final theme.

### 14.5 Theme Propagation Checklist

When adding theme cycling to an example:

```rust
fn cycle_theme(&mut self) {
    self.theme = themes[next];
    // Propagate to EVERY child widget:
    self.list.on_theme_change(&self.theme);
    self.search_input.on_theme_change(&self.theme);
    self.status_bar.on_theme_change(&self.theme);
    self.table.on_theme_change(&self.theme);
    // ... any other widgets
}
```

**Common widgets needing propagation:** List<T>, Table<T>, SearchInput, PasswordInput, StatusBar, MenuBar, Breadcrumbs, Tree, CommandPalette, Form, TabBar, SplitPane, Toast

### 14.6 Help Overlay Pattern (Required)

All examples MUST implement a help overlay:
- `show_help: bool` field in struct
- `?` or F1 toggles, `Esc` dismisses
- Rounded corners (╭╮╰╯) with `theme.outline`
- Background: `theme.surface_elevated`
- Title centered with `theme.primary` + `Styles::BOLD`
- Two-column layout: keys (`theme.primary`) + descriptions (`theme.fg`)
- Must contain all relevant keyboard shortcuts

### 14.7 `Theme::from_env_or()` (Required)

All examples MUST use `Theme::from_env_or(default)` instead of hardcoded theme constructors:

```rust
// WRONG
.theme(Theme::nord())

// RIGHT
.theme(Theme::from_env_or(Theme::nord()))
```

### 14.8 `DTRON_THEME_FILE` Mechanism

When a launched example cycles its theme and exits, the showcase can adopt the final theme:
1. Showcase sets `DTRON_THEME_FILE` env var to temp file path
2. App framework writes `self.theme.name` to file after event loop exits
3. Showcase reads the file after child exits, calls `ctx.set_theme(Theme::from_name(...))`

### 14.9 Status Bar / Footer Text

All status bars MUST include:
- Help key reference (F1: help)
- Back/dismiss key reference (Esc: dismiss)
- Quit key reference (Ctrl+Q: quit)
- Theme key reference if theme cycling supported (Ctrl+T: theme)

### 14.10 Background Fill Pattern

All widgets MUST fill their plane background with `self.theme.bg` to avoid black (`Color::Reset`) holes.

### 14.11 Text Boundary Clipping

Text in bounded panels MUST be clipped at the panel boundary to prevent bleeding:

- `draw_text`: Full-width rows (clips at plane width)
- `draw_text_clipped(plane, x, y, text, max_x, fg, bg, bold)`: Column-bounded text (clips at max_x)

---

## 15. Build Configuration & Features

### 15.1 Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `system` | System monitoring (CPU, memory, disk, processes) | `sysinfo` |
| `syntax-highlighting` | Syntax highlighting via syntect | `syntect`, `regex` |
| `sqlite` | SQLite database support | `rusqlite` |
| `async` | Async runtime support | `tokio + reqwest` |
| `tracing` | Structured logging with tracing | `tracing`, `tracing-subscriber` |
| `debug-events` | Debug event logging (mouse/key events to stderr) | — |
| `default` | `system` + `syntax-highlighting` | — |

### 15.2 Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `bitflags` | 2.4 | `Styles` bitflags (+ serde) |
| `ratatui` | 0.29 | Layout `Rect`, integration backend |
| `unicode-width` | 0.1 | Unicode character width detection |
| `unicode-segmentation` | 1.10 | Grapheme cluster segmentation |
| `chrono` | 0.4 | Calendar date handling (+ serde) |
| `signal-hook` | 0.3 | SIGINT/SIGTERM handling |
| `serde` / `serde_json` / `toml` | 1.0 | Serialization for config/command output |
| `libc` | 0.2 | POSIX tty ioctls (non-Windows only) |

**Dev-dependencies:** `rand`, `tempfile`, `criterion`, `proptest`, `insta`

### 15.3 Workspace Crates

| Crate | Path | Description |
|-------|------|-------------|
| `cargo-dracon` | `crates/cargo-dracon/` | Project scaffolding tool |
| `dracon-lsp-server` | `extensions/lsp-server/` | LSP server extension |
| `dracon-vscode` | `extensions/vscode/` | VS Code live TUI preview extension |

### 15.4 License

Dual-licensed:
- **AGPL-3.0-only** — Default for open source use
- **Commercial License** — For organizations not wanting AGPLv3 source disclosure

---

## 16. Examples & Showcase

### 16.1 Complete Example Inventory (57 Binaries)

**Apps (`examples/_apps/`) — 4:**

| Example | Type | Pattern | Description |
|---------|------|---------|-------------|
| `system_monitor` | Binary | Pattern 2 | Real-time system monitoring (CPU, memory, disk, processes via `/proc`) |
| `file_manager` | Binary | Pattern 1 | File browser with SplitPane, breadcrumbs, and click navigation |
| `chat_client` | Binary | Pattern 2 | Chat interface with message list and input |
| `dashboard_builder` | Binary | Pattern 1 | Composable dashboard with gauges, gauges, sparklines |

**Cookbook (`examples/_cookbook/`) — 15:**

| Example | Pattern | Description |
|---------|---------|-------------|
| `accessibility` | Pattern 1 | Accessibility features demo |
| `autocomplete` | Pattern 1 | Autocomplete widget demo |
| `calendar` | Pattern 1 | Calendar/DatePicker widget demo |
| `cell_pool` | Pattern 2 | CellPool memory visualization |
| `command_bindings` | Pattern 1 | Command binding and output parsing demo |
| `data_table` | Pattern 1 | Sortable table widget demo |
| `debug_overlay` | Pattern 1 | Debug overlay widget demo |
| `form_validation` | Pattern 1 | Form with validation demo |
| `log_monitor` | Pattern 2 | Log viewer with severity detection |
| `menu_system` | Pattern 1 | Menu bar and context menu demo |
| `notification_center` | Pattern 1 | Notification center demo |
| `plugin_demo` | Pattern 2 | Plugin system demo |
| `rich_text` | Pattern 1 | Rich text rendering demo |
| `scrollable_content` | Pattern 1 | Scrollable content demo |
| `split_resizer` | Pattern 2 | Split pane with drag resize demo |
| `stat_widget_plugin` | Pattern 1 | Custom stat widget via plugin system |
| `tabbed_panels` | Pattern 2 | Tab bar with panel switching |
| `tree_navigator` | Pattern 1 | Tree navigation widget demo |
| `widget_gallery` | Pattern 1 | Framework widget gallery |

**Root Examples (`examples/`) — 34 main examples:**

| Example | Pattern | Description |
|---------|---------|-------------|
| `arena` | Pattern 2 | Real-time arena game |
| `basic_raw` | Raw | Minimal raw terminal example |
| `command_dashboard` | Pattern 1 | Command-driven dashboard |
| `cyberpunk_dashboard` | Pattern 1 | Cyberpunk-themed dashboard |
| `desktop` | Raw | Raw terminal desktop metaphor |
| `event_bus_demo` | Pattern 2 | Event bus pub/sub demo |
| `form_demo` | Pattern 1 | Form widget demo |
| `form_widget` | Pattern 1 | Form widget standalone demo |
| `framework_chat` | Pattern 1 | Chat app using framework |
| `framework_demo` | Pattern 1 | General framework demo |
| `framework_file_manager` | Pattern 1 | File manager using framework |
| `framework_widgets` | Pattern 1 | Framework widgets demo |
| `from_toml` | TOML | App loaded from TOML config |
| `game_loop` | Raw | Raw terminal game loop |
| `git_tui` | Pattern 1 | Git TUI interface |
| `god_mode` | — | Advanced demo |
| `ide` | Pattern 2 | IDE with CommandPalette |
| `input_debug` | Raw | Raw input debugging (mouse/keys) |
| `modal_demo` | Pattern 2 | Modal dialog demo |
| `network_client` | Pattern 2 | Async network client (requires `async` feature) |
| `plugin_demo` | Pattern 2 | Plugin system demo |
| `scene_router_demo` | Pattern 2 | Scene router navigation demo |
| `sqlite_browser` | Pattern 1 | SQLite database browser (requires `sqlite` feature) |
| `table_widget` | Pattern 1 | Table widget standalone demo |
| `text_editor_demo` | Pattern 1 | TextEditor standalone demo |
| `theme_switcher` | Pattern 1 | Theme cycling demo |
| `todo_app` | Pattern 2 | SQLite-backed todo app (requires `sqlite` feature) |
| `tutorial_app` | Pattern 2 | Tutorial/onboarding app |
| `widget_tutorial` | Pattern 1 | Widget building tutorial |

**Showcase (`examples/showcase/`) — The primary demo launcher:**

The showcase is a modular example launcher with:
- 29 embedded scenes (in-process using `SceneRouter`)
- Card-based grid display with filtering and search
- Theme cycling via `t` key
- Category sidebar (all, apps, input, data, cookbook, tools, accessibility)
- FPS toggle, search input for filtering examples
- Smoke test: `tests/showcase_smoke_test.rs`

**Showcase Scene Modules:**

| Scene | File | Lines | Description |
|-------|------|-------|-------------|
| `app_scenes` | `scenes/app_scenes.rs` | — | Application example launchers |
| `widget_gallery` | `scenes/widget_gallery.rs` | 482 | Widget workshop with sidebar + live demo |
| `theme_switcher` | `scenes/theme_switcher.rs` | 451 | Theme studio with split preview |
| `password_input` | `scenes/password_input.rs` | 529 | Login screen with form widgets |
| `notification_center` | `scenes/notification_center.rs` | 549 | Notification hub with detail panel |
| `color_picker` | `scenes/color_picker.rs` | 501 | Color studio with palette generation |
| (and 23 more scenes) | | | |

### 16.2 Example Patterns Summary

| Pattern | Count | Mechanism |
|---------|-------|-----------|
| Pattern 1 (Widget trait) | ~25 | `impl Widget` with `needs_render()` |
| Pattern 2 (InputRouter) | ~12 | `Rc<RefCell<State>>` + `on_input` + `on_tick` |
| Raw terminal | ~5 | Direct compositor usage, no framework |
| TOML config | 1 | `App::from_toml()` |

---

## 17. Test Coverage

### 17.1 Test Statistics

| Category | Count | Status |
|----------|-------|--------|
| Library unit tests | 291 | ✅ Pass |
| Doc tests | 5 | ✅ Pass (25 `ignore`) |
| Integration tests | 26+ | ✅ Pass |
| Total test functions | ~1,436 | ✅ All pass |
| Clippy warnings | 0 | ✅ Zero |
| Benchmark suites | 3 | criterion bench |

### 17.2 Test Files (`tests/`)

**Widget Tests (43 test files):**

| Test File | Widget(s) | Tests |
|-----------|-----------|-------|
| `widget_test.rs` | Basic widget | 26 |
| `widget_tests.rs` | Various | 14 |
| `button_test.rs` | Button | 6 |
| `widget_gauge_test.rs` | Gauge | 12 |
| `gauge_test.rs` | Gauge | 15 |
| `label_test.rs` | Label | 10 |
| `list_test.rs` | List | 20+ |
| `list_common_test.rs` | ListCommon | 25 |
| `tree_widget_test.rs` | Tree | 18 |
| `modal_widget_test.rs` | Modal | 12 |
| `widget_confirm_dialog_test.rs` | ConfirmDialog | 11 |
| `menu_test.rs` | MenuBar | 14 |
| `form_widget_test.rs` | Form | 16 |
| `form_validation_test.rs` | Form validation | 10 |
| `widget_password_input_test.rs` | PasswordInput | 15 |
| `widget_slider_test.rs` | Slider | 12 |
| `widget_status_badge_test.rs` | StatusBadge | 10 |
| `widget_sparkline_test.rs` | Sparkline | 37 |
| `widget_progress_ring_test.rs` | ProgressRing | 38 |
| `widget_streaming_text_test.rs` | StreamingText | 10 |
| `widget_key_value_grid_test.rs` | KeyValueGrid | 10 |
| `widget_log_viewer_test.rs` | LogViewer | 10 |
| `widget_snapshot_tests.rs` | Snapshot tests | 8 |
| `widget_gallery_edge_test.rs` | WidgetGallery edge | — |
| `toast_test.rs` | Toast | 8 |
| `tooltip_test.rs` | Tooltip | — |
| `context_menu_test.rs` | ContextMenu | 17 |
| `text_editor_test.rs` | TextEditor | 30+ |
| `text_editor_adapter_test.rs` | TextEditorAdapter | 15 |
| `text_editor_adapter_edge_test.rs` | TextEditorAdapter edge | 8 |

**Framework Tests:**

| Test File | Module | Tests |
|-----------|--------|-------|
| `theme_test.rs` | Theme | 12 |
| `theme_validation_test.rs` | Theme validation | 5 |
| `theme_propagation_test.rs` | Theme propagation | 3 |
| `focus_test.rs` | FocusManager | 14 |
| `hitzone_test.rs` | HitZone | 11 |
| `scroll_test.rs` | ScrollState | 8 |
| `multi_widget_test.rs` | Multi-widget app | 3 |
| `resize_test.rs` | Resize handling | 4 |
| `event_bus_test.rs` | EventBus | 14 |
| `scene_router_test.rs` | SceneRouter | 10 |
| `splitpane_test.rs` | SplitPane | 6 |
| `status_bar_test.rs` | StatusBar | 6 |
| `streaming_text_test.rs` | StreamingText | 8 |
| `syntax_highlighting_test.rs` | Syntax highlighting | 6 |
| `panel_test.rs` | Panel | 5 |
| `input_reader_test.rs` | InputReader/Parser | 15 |
| `network_widget_test.rs` | Network widgets | 4 |
| `profiler_test.rs` | Profiler | 5 |
| `filter_test.rs` | Compositor filters | 8 |
| `utils_test.rs` | Utils | 10 |
| `phase1_widget_test.rs` | Phase 1 widgets | 8 |
| `phase2_3_4_widget_test.rs` | Phases 2-4 widgets | 8 |
| `untested_widgets_test.rs` | Coverage gap closure | 10 |
| `property_tests.rs` | Property-based (proptest) | 6 |

**Example Tests:**
| Test File | Tests |
|-----------|-------|
| `example_smoke_test.rs` | 1 (ignored, requires TTY) |
| `showcase_smoke_test.rs` | 1 (ignored, requires TTY) |

**Benchmarks:**
| File | Suite |
|------|-------|
| `framework_benchmarks.rs` | Criterion: compositor, widget rendering |
| `performance_benchmarks.rs` | Raw performance metrics |

### 17.3 Test Coverage Gaps (Closed)

All previously identified coverage gaps have been closed:
- `progress_ring` — ✅ 38 tests
- `sparkline` — ✅ 37 tests
- `list_common` — ✅ 25 tests
- `text_editor_adapter` — ✅ 23 tests across 2 files

### 17.4 Snapshot Tests

Using `insta` crate for visual regression testing:
- `tests/widget_snapshot_tests.rs` — 8 snapshot tests for List, Table, Tree widgets
- Snapshots stored in `tests/snapshots/`

---

## 18. API Surface

### 18.1 Public API Metrics

| Metric | Value |
|--------|-------|
| Total LOC | 41,488 |
| Framework widgets | 47 |
| Standalone widgets | 7 (TextEditor, TextInput, etc.) |
| Built-in themes | 21 |
| Example binaries | 57 |
| Public API items | 1,244+ |
| Rc/RefCell uses | ~403 |
| `unwrap()`/`expect()` calls | ~129 (across all code) |
| Transitive dependencies | ~310 |
| Integration tests | 26+ |
| Unit tests | 291+ |
| Total test functions | ~1,436 |

### 18.2 Main Re-exports

**From `dracon_terminal_engine`:**
- `Cell`, `Color`, `Compositor`, `Plane`, `Styles`
- `DraconError`
- `Terminal`, `Capabilities`, `CursorShape`
- `InputReader`, `Parser`
- `SystemMonitor`, `SystemData`, `ProcessInfo`, `DiskInfo` (behind `system` feature)
- `TextEditor`, `TextInput`, `StandaloneButton`, `Panel`, `Component`, `HotkeyHint`, `ContextMenuAction`
- `prelude::*` — the one-import entry point

**Prelude re-exports (`framework::prelude`):**
- All 47 framework widget types
- `App`, `Ctx`, `WidgetRef`, `WidgetRefMut`
- `Theme`
- `Widget`, `WidgetId`, `WidgetState`, `Commandable`, `Focusable`, `InputHandler`, `Renderable`, `Themable`
- `HitZone`, `HitZoneGroup`, `ScopedZone`, `ScopedZoneRegistry`, `DragState`
- `DragGhost`, `DragManager`, `DragPhase`
- `MarqueeRect`, `MarqueeState`, `render_marquee`
- `Animation`, `AnimationManager`, `Easing`
- `DirtyRegion`, `DirtyRegionTracker`
- `FocusManager`
- `ScrollContainer`, `ScrollState`
- `KeybindingConfig`, `KeybindingSet`, `actions`, `resolve_keybindings`
- `EventBus`, `Reactive`, `SubscriptionId`
- `NavigationEvent`, `Scene`, `SceneRouter`
- `PluginRegistry`, `WidgetFactory`
- `Constraint`, `Direction`, `Layout`
- `I18n`, `I18nError`, `tr`
- `BoundCommand`, `CommandRunner`, `AppConfig`, `OutputParser`, `ParsedOutput`, `WidgetConfig`, `LoggedLine`
- `Cell`, `CellPool`, `Color`, `Compositor`, `Plane`, `PoolConfig`, `Styles`
- `DraconError`
- `Event`, `KeyCode`, `KeyEvent`, `KeyEventKind`, `KeyModifiers`, `MouseButton`, `MouseEvent`, `MouseEventKind`
- `Terminal`
- `Rect` (from ratatui)

---

## 19. Completeness Assessment

### 19.1 Completeness Score: **87/100**

### 19.2 Scoring Breakdown

| Category | Weight | Score | Rationale |
|----------|--------|-------|-----------|
| **Core Engine** | 15% | 14/15 | Compositor, Terminal, Input, Color all solid. Missing: cross-platform (Windows backend), paste event dispatch to widgets works |
| **Framework** | 20% | 18/20 | App, Ctx, Widget trait, lifecycle all comprehensive. Missing: `Widget::render(&self)` prevents caching patterns; `AsyncWidget` exists but has no integration tests |
| **Widget Inventory** | 15% | 13/15 | 47 widgets covering most UI patterns. Missing: some widgets lack hover/focus (DebugOverlay, Divider, Gauge, etc.); several widgets lack mouse handlers (13 widgets with no mouse support) |
| **Theme System** | 10% | 9/10 | 21 themes with semantic color system. `scrollbar_width` deprecated field; theme constructors not const |
| **Input System** | 10% | 9/10 | SGR mouse, keyboard chords, bracketed paste, kitty keyboard. Missing: Windows console support |
| **TextEditor** | 10% | 7/10 | Full-featured but 3,025 LOC single file needs splitting; no LSP; basic multi-cursor only |
| **Examples** | 10% | 9/10 | 57 examples covering all major patterns. Scene enrichment gaps (modal_demo 30% filled, tooltip 45%) |
| **Documentation** | 5% | 3/5 | AGENTS.md is comprehensive; AI_GUIDE.md is useful; README outdated (41 → 47 widgets). 25/30 doc-tests ignored. Missing: API reference doc comments on ~30 pub functions in app.rs |
| **Testing** | 5% | 5/5 | 291+ unit, 26+ integration, all pass, 0 clippy warnings. Snapshot tests, proptest, benchmarks all present |

### 19.3 Gap Analysis

**Critical Gaps (blocking 90+):**

1. **Single-file TextEditor** — 3,025 LOC in `editor.rs` should be split into submodules (selection, syntax, movement, history)
2. **`utils.rs` sprawl** — 1,217 LOC catch-all should be split into proper modules
3. **Missing `// SAFETY:` comments** — 11 of 12 `unsafe` blocks in `src/` lack safety preambles

**Medium Gaps:**

4. **Several widgets lack hover events** — 13+ framework widgets implement no mouse handling (DebugOverlay, Divider, Gauge, Hud, Label, Profiler, ProgressBar, ProgressRing, Sparkline, Spinner, StatusBadge, StatusBar, StreamingText, Toast)
5. **Several widgets lack focus** — 30+ framework widgets don't implement `focusable()` (most return default `true` but don't actually handle focus); only ConfirmDialog, Form, SearchInput, PasswordInput, etc. properly implement focus
6. **No Windows backend** — `libc` gated to non-Windows; no Windows console API support
7. **Deprecated `scrollbar_width`** in Theme struct — layout dimensions shouldn't be in themes
8. **`App::new().unwrap()` in doc examples** — doesn't demonstrate proper error handling
9. **CHANGELOG format drift** — doesn't strictly follow keepachangelog.com spec

**Minor Gaps:**

10. **25 of 30 doc-tests are `ignore`** — not compile-tested
11. **`cargo outdated` not in CI** — no automated dependency freshness checking
12. **Event bus has no benchmarks** — no performance tests for pub/sub throughput
13. **`dracon.toml` has no schema validation** — invalid TOML produces opaque errors
14. **No iOS/WebAssembly targets** — terminal-only by design but should document explicitly
15. **Example enrichment gaps** — modal_demo (30% filled), tooltip (45%), tags_input (40%), password_input (50%)

### 19.4 Coverage Score: 87/100

| Component | Coverage | Notes |
|-----------|----------|-------|
| Widget trait test coverage | 100% | All 47 widgets have tests via `tests/` files or inline |
| Framework sub-systems | 95% | EventBus (14 tests), FocusManager (14), HitZone (11), ScrollState (8), SceneRouter (10), SplitPane (6) |
| Theme coverage | 100% | 21 themes all tested (creation, names, from_env_or) |
| Input parser | 90% | 15 tests covering key/mouse/escape sequences |
| Compositor | 85% | Filter tests (8), resize (4), rendering path tested via snapshot |
| TextEditor | 80% | Core editor tested (30+ tests), but editor_search, multi-cursor, undo/redo edge cases less tested |
| Integration tests | 85% | Multi-widget, theme propagation, resize all tested |
| Property-based tests | 6 cases | Layout, grapheme width, theme colors with proptest |
| Example smoke tests | 2 (ignored) | Require TTY, not run in CI |

---

## 20. Future Roadmap

### 20.1 Immediate (0.2.0)

| Item | Priority | Description |
|------|----------|-------------|
| TextEditor split | High | Split `editor.rs` into submodules |
| `utils.rs` split | Medium | Extract into proper modules |
| SAFETY comments | Medium | Add to all `unsafe` blocks |
| Widget decomposition Phase 2 | Medium | Sub-traits as primary API |
| Convert 25 ignored doc-tests | Low | Make compile-tested |

### 20.2 Medium Term

| Item | Priority | Description |
|------|----------|-------------|
| Windows backend | Medium | Windows console API support |
| Mouse support for all widgets | Medium | Add missing mouse/hover handlers |
| `cargo outdated` in CI | Low | Automated dependency freshness |
| Event bus benchmarks | Low | Criterion benchmarks for pub/sub |
| Schema validation for TOML | Low | Validate `dracon.toml` structural correctness |
| CHANGELOG format | Low | Enforce keepachangelog.com spec |

### 20.3 Deferred / Out of Scope

| Feature | Reason |
|---------|--------|
| LSP integration | Requires async runtime, external processes, complex state management |
| Syntax-aware folding | Requires tree-sitter integration, per-language grammar |
| Multi-cursor enhancements | Basic multi-cursor sufficient for light editing |
| Modal editing | Kakoune-style is complex, not needed for view/edit use cases |
| Advanced text objects | vim-style text objects require deep editor integration |
| iOS/WebAssembly | Terminal target explicitly; document as design constraint |

---

## Appendix A: Compliance Checklist

| Requirement | Status | Evidence |
|------------|--------|----------|
| RAII terminal management | ✅ | `Terminal` struct enters/exits raw mode |
| Z-indexed compositor | ✅ | Painter's algorithm in `Compositor::render()` |
| Input shield for mode transitions | ✅ | `App::shield_input()` |
| Help overlay in every example | ✅ | Verified in AUDIT_REPORT for all 29 scenes |
| `Theme::from_env_or()` in all examples | ✅ | Verified in 0.1.10 changelog |
| `DTRON_THEME_FILE` support | ✅ | Auto-writes in `App::run()` |
| Pattern 2 `current_theme()` sync | ✅ | All 12 Pattern 2 examples implement |
| Keybinding system in all examples | ✅ | `KeybindingSet` + `resolve_keybindings()` |
| Modifier guards on Char handlers | ✅ | Documented in AGENTS.md |
| Background fill in every widget | ✅ | `fill_bg(self.theme.bg)` in render |
| Text clipping at column boundaries | ✅ | `draw_text_clipped()` helper |
| u16 arithmetic safety in mouse handlers | ✅ | `saturating_sub()` + bounds checks |
| All unsafe blocks have SAFETY comments | ❌ | 11 of 12 missing |
| All pub fn have doc comments | ❌ | ~30 public methods in app.rs undocumented |
| Widget background fill pattern | ✅ | `fill_bg` in all framework widgets |
| 0 clippy warnings | ✅ | Verified in latest audit |

## Appendix B: Keybinding Conventions

| Key | Convention | Rationale |
|-----|------------|-----------|
| `Ctrl+Q` | Quit | Never single-letter `q` (conflicts with text input) |
| `F1` | Help toggle | Never `?` (conflicts with text input) |
| `Esc` | Back/Dismiss | Universal; `Backspace` is delete-only |
| `Ctrl+T` | Theme cycle | Configurable via `dracon.toml` |
| `Ctrl+F` | Search | Not `/` (conflicts with text input) |
| `Ctrl+S` | Save | Universal standard |
| `Ctrl+N` | New | Universal standard |
| `Ctrl+W` | Close tab | Browser/IDE standard |
| `↑/↓/←/→` | Navigation | Universal, hardcoded |
| `Enter` | Select/Submit | Universal, hardcoded |
| `Tab`/`Shift+Tab` | Focus cycle | Universal, hardcoded |
| `Backspace` | Delete | Delete-only, never navigation |

## Appendix C: Environment Variables

| Variable | Purpose | Used By |
|----------|---------|---------|
| `DTRON_THEME` | Inherit theme from parent | `Theme::from_env_or()` |
| `DTRON_THEME_FILE` | Return theme to parent on exit | `App::run()` |
| `HOME` | User config directory resolution | `resolve_keybindings()` |
| `TERM` | Terminal capability detection | `Capabilities` |
| `COLORTERM` | True color detection | `Capabilities` |
