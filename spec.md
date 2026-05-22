# Dracon Terminal Engine — Comprehensive Specification

> **Version:** 0.1.10  
> **License:** AGPL-3.0-only + Commercial  
> **Repository:** https://github.com/DraconDev/dracon-terminal-engine  
> **Documentation:** https://docs.rs/dracon-terminal-engine  
> **Last Audit:** 2026-05-22  
> **Total LOC:** 41,488  
> **Source Files:** ~110 Rust source files (src/)  
> **Example Binaries:** 57  
> **Framework Widgets:** 47  
> **Built-in Themes:** 21  
> **Test Functions:** ~1,436  
> **Public API Items:** ~1,244  

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
18. [API Surface & Prelude](#18-api-surface--prelude)
19. [Completeness Assessment](#19-completeness-assessment)
20. [Future Roadmap](#20-future-roadmap)
21. [Appendices](#21-appendices)

---

## 1. Vision & Philosophy

### 1.1 Mission Statement

Dracon Terminal Engine is a **terminal application framework** for Rust. It is not a "TUI library" in the traditional sense. It provides a complete runtime that owns the terminal, input parsing, rendering pipeline, and event loop. The developer writes widgets and app logic; the framework handles terminal management, input dispatch, compositing, and frame timing.

The goal: **GUI-grade terminal applications** — persistent, visible, mouse-friendly, composable widgets — that run anywhere a terminal runs.

### 1.2 Core Principles

| Principle | Explanation | Implementation |
|-----------|-------------|----------------|
| **One import, complete app** | `use dracon_terminal_engine::framework::prelude::*;` is the only import needed | `prelude` re-exports App, Ctx, all 47 widgets, Theme, Rect, Event types, and all framework sub-systems |
| **Framework, not library** | App owns the event loop, compositor, input parsing, and rendering | `App::run()` owns the entire frame cycle — polling, dispatch, render, timing |
| **Widgets own state** | Each widget manages its own internal state (lines, cursor, selection) | Widget trait has no concept of app-level state; widgets are self-contained |
| **App owns composition** | App manages widgets via registry, z-order, and focus | Widgets register via `add_widget()`, app sorts by `z_index()`, dispatches focus via `FocusManager` |
| **Mouse-first** | Widgets respond to clicks, not just keys | SGR mouse tracking enabled by default; `HitZone`, `ScopedZoneRegistry` for declarative mouse routing |
| **Keyboard-enhanced** | Navigation shortcuts exist but aren't required | Config-driven `KeybindingSet` with modifier-key actions |
| **Terminal as universal target** | No platform-specific code beyond POSIX tty | `libc` for ioctls; `signal-hook` for signals; pure ANSI escape sequences for rendering |
| **RAII terminal state** | Terminal struct manages raw mode lifecycle | `Terminal::new()` enters raw mode + alt screen; `Terminal::drop()` restores saved state via DECRC |
| **Z-indexed compositor** | Painter's algorithm with per-plane opacity and visual filters | `Compositor::render()` sorts planes by `z_index`, composites bottom-up with alpha blending |
| **Command-driven architecture** | Every action is a CLI command, AI-inspectable | `BoundCommand` + `OutputParser`; `Ctx::available_commands()` enumerates all actions |

### 1.3 What It Is NOT

| Misconception | Reality |
|---------------|---------|
| vim/Helix competitor | TextEditor is a **view/edit widget** — no modal editing, no LSP, no syntax-aware folding |
| CLI+ (ratatui successor) | Uses ratatui only for `Rect` type and Layout; has its own compositor, input parser, and rendering engine |
| Browser-based GUI (Tauri/Dioxus) | Runs natively in any terminal — no browser, no runtime, no install |
| Editor SDK | Not designed for building editors; designed for building terminal **applications** that may include text editing |
| Terminal multiplexer | Not tmux/tilix — it's an application framework that renders within a single terminal window |

### 1.4 Deployment Advantages Over GUI

| Advantage | Detail |
|-----------|--------|
| **Universal** | Runs on VPS, SSH, containers, CI, embedded — anywhere with a terminal emulator |
| **Zero user dependencies** | No browser, no runtime, no permissions, no package installation |
| **Single binary** | Ships as one executable; instant startup (~1ms to full UI) |
| **Cross-platform** | POSIX terminals behave identically regardless of OS; no WebView/GTK/Qt platform issues |
| **No update friction** | No browser extension updates, no runtime upgrades — replace the binary |

### 1.5 UX Advantages Over Traditional CLI

| Advantage | Detail |
|-----------|--------|
| **Persistent state** | Output remains visible indefinitely — no re-running commands to see prior results |
| **Visible structure** | Panels, trees, forms, tables — spatial layout conveys hierarchy, not flat text |
| **Mouse-friendly** | Click, drag, scroll, right-click — natural interactions work without keyboard memorization |
| **Composability** | Mix widgets (list + editor + form + table) freely in a single view |
| **Real-time updates** | Tick-driven refresh: gauges update, logs stream, timers count without user action |
| **Theming** | 21 built-in themes, instant switching, semantic color system, dark/light modes |

---

## 2. Architecture Overview

### 2.1 Module Dependency Graph

```
lib.rs (crate root)
├── backend/         ─── tty.rs              (POSIX ioctls, poll)
├── compositor/      ─── engine.rs           (Compositor)
│                    ├── plane.rs            (Plane, Cell, Color, Styles)
│                    ├── filter.rs           (Dim, Invert, Scanline, Pulse, Glitch)
│                    └── pool.rs             (CellPool, PoolConfig)
├── contracts.rs     ─── UiRenderer, UiEventSource, UiRuntime traits
├── core/            ─── terminal.rs         (Terminal, Capabilities, CursorShape)
├── error.rs         ─── DraconError
├── framework/       ─── mod.rs              (re-exports)
│                    ├── app.rs              (App struct, event loop)
│                    ├── ctx.rs              (Ctx struct)
│                    ├── widget.rs           (Widget trait, sub-traits, WidgetId)
│                    ├── theme.rs            (Theme struct, 21 constructors)
│                    ├── command.rs          (BoundCommand, OutputParser, CommandRunner)
│                    ├── layout.rs           (Layout, Constraint, Direction)
│                    ├── hitzone.rs          (HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry)
│                    ├── dragdrop.rs         (DragManager, DragItem, DragGhost, DropTarget)
│                    ├── marquee.rs          (MarqueeState, MarqueeRect)
│                    ├── focus.rs            (FocusManager, callbacks)
│                    ├── scroll.rs           (ScrollState, ScrollContainer)
│                    ├── keybindings.rs      (KeybindingSet, resolve_keybindings)
│                    ├── scene_router.rs     (SceneRouter, Scene, SceneTransition)
│                    ├── event_bus.rs        (EventBus, Reactive, SubscriptionId)
│                    ├── animation.rs        (Animation, AnimationManager, Easing)
│                    ├── plugin.rs           (PluginRegistry, WidgetFactory)
│                    ├── dirty_regions.rs    (DirtyRegion, DirtyRegionTracker)
│                    ├── widget_container.rs (WidgetContainer, WidgetRegistry)
│                    ├── i18n.rs             (I18n, tr! macro)
│                    ├── logging.rs          (tracing integration, feature-gated)
│                    ├── event_dispatcher.rs (Event dispatch helpers)
│                    └── widgets/            (47 widget modules)
├── input/           ─── mod.rs
│                    ├── event.rs            (Event, KeyEvent, MouseEvent, enums)
│                    ├── parser.rs           (byte-level escape sequence parser)
│                    ├── reader.rs           (InputReader: blocking/non-blocking)
│                    ├── kitty_key.rs        (Kitty keyboard protocol support)
│                    ├── async_reader.rs     (Async input reader, feature-gated)
│                    └── mapping.rs          (deprecated UiEvent→Event mapping)
├── integration/     ─── ratatui.rs          (RatatuiBackend bridge)
├── layout/          ─── mod.rs              (grid, border, padding helpers)
├── system/          ─── mod.rs              (SystemMonitor, feature-gated)
├── utils.rs         ─── visual width, truncate, formatting (1,217 LOC)
├── text.rs          ─── Unicode grapheme cluster utilities
└── visuals/         ─── mod.rs
                     ├── icons.rs            (file-type icons)
                     ├── osc.rs              (OSC sequences: clipboard, hyperlinks, bell)
                     ├── accessibility.rs    (OSC 99 screen reader announcements)
                     └── sync.rs             (sync mode 2026)

src/widgets/ (standalone, non-framework)
├── mod.rs
├── editor.rs        (TextEditor — 3,025 LOC)
├── editor_search.rs (TextEditor search/filter)
├── input.rs         (TextInput)
├── button.rs        (standalone Button)
├── panel.rs         (Panel container)
├── component.rs     (Component trait)
├── hotkey.rs        (HotkeyHint)
└── context_menu.rs  (ContextMenuAction type)
```

### 2.2 Data Flow (Detailed)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ INPUT PHASE (poll_and_dispatch_input)                                    │
│                                                                         │
│  stdin → tty::poll_input(fd, 1ms)                                       │
│    → Ok(true): read chunk [1024 bytes] → Parser::advance(byte)          │
│       → Event::Key/Event::Mouse/Event::Resize/Event::Paste              │
│       → App::handle_event(event)                                        │
│         ├── keyboard: dispatch_key → keybinding check → widget dispatch │
│         │   └── Tab → FocusManager::tab_next/prev                       │
│         │   └── Esc → SceneRouter::can_go_back → pop or stop            │
│         │   └── Ctrl+Q → running = false                                │
│         ├── mouse: dispatch_mouse → hit-test z-order → widget dispatch  │
│         │   └── Hit zone registration → ScopedZoneRegistry::dispatch()  │
│         ├── resize: dispatch_resize → compositor.resize → widget.set_area │
│         └── paste: dispatch_paste → synthetic KeyEvents to focused widget │
│    → drain remaining input (up to 64 iterations, non-blocking)          │
│    → Ok(false): check_timeout() for pending escape sequences            │
├─────────────────────────────────────────────────────────────────────────┤
│ RENDER PHASE (render_dirty_widgets)                                      │
│                                                                         │
│  For each widget in z-order cache:                                      │
│    if widget.needs_render():                                            │
│      plane = widget.render(widget.area())                               │
│      widget.clear_dirty()                                               │
│      compositor.add_plane(plane)                                        │
├─────────────────────────────────────────────────────────────────────────┤
│ TICK PHASE (run_tick_callback)                                          │
│                                                                         │
│  if elapsed >= tick_interval:                                           │
│    prev_theme = self.theme.name                                         │
│    on_tick(&mut Ctx, tick_count)                                        │
│    if theme.name != prev_theme: propagate theme to all widgets          │
├─────────────────────────────────────────────────────────────────────────┤
│ PERIODIC COMMAND PHASE (run_periodic_commands)                           │
│                                                                         │
│  For each widget with refresh_seconds:                                  │
│    if elapsed >= refresh_interval:                                      │
│      run command → parse output → widget.apply_command_output()         │
│      widget.mark_dirty()                                                │
├─────────────────────────────────────────────────────────────────────────┤
│ USER CLOSURE PHASE                                                      │
│                                                                         │
│  f(&mut Ctx) — user's per-frame callback                                │
│  ctx.add_plane() / ctx.mark_dirty() / ctx.set_theme() etc.             │
├─────────────────────────────────────────────────────────────────────────┤
│ COMPOSITOR RENDER PHASE                                                 │
│                                                                         │
│  if compositor.planes not empty:                                        │
│    compositor.set_dirty_regions(dirty_tracker)                          │
│    compositor.render(&mut terminal)                                     │
│      → sort planes by z_index                                           │
│      → composite into final_buffer (painters algorithm)                 │
│      → diff final_buffer vs last_frame                                  │
│      → emit ANSI escape sequences for changed cells                     │
│      → single write_all() to stdout                                     │
│      → clone final_buffer → last_frame                                  │
│      → clear planes                                                     │
├─────────────────────────────────────────────────────────────────────────┤
│ CURSOR + ANIMATION PHASE                                                │
│                                                                         │
│  if focused widget has cursor_position → set_cursor(col, row)           │
│  else → hide_cursor()                                                   │
│  animations.tick()                                                      │
├─────────────────────────────────────────────────────────────────────────┤
│ FRAME TIMING PHASE                                                      │
│                                                                         │
│  frame_count += 1                                                       │
│  compute frame_duration_ms                                              │
│  sleep(frame_duration - elapsed) if frame completed early               │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Key Design Decisions

| Decision | Rationale | Consequence |
|----------|-----------|-------------|
| `render(&self)` not `render(&mut self)` | Widget trait render takes `&self` to allow multiple simultaneous readers and caching | Widgets must compute all visual state in advance or use `RefCell` for lazy computation |
| `RefCell<Vec<Box<dyn Widget>>>` | Enables both immutable iteration (render) and mutable access (events) from `&self` methods | Panics if borrows are nested; `WidgetRef`/`WidgetRefMut` guard types prevent this |
| Painter's algorithm (not damage-based) | Simpler, correct for overlapping semi-transparent planes | Full-screen planes are always O(W×H); dirty regions reduce but don't eliminate cost |
| Own input parser (not crossterm) | Zero dependencies for core parsing; full control over sequence handling | Must maintain parser for new terminal features (kitty keyboard, extended mouse) |
| SGR mouse only (no X10/X11) | SGR (1006) supports position reporting, drag, all buttons, modifiers | Legacy terminals (PuTTY, old xterm) may not work; documented as requirement |
| Feature flags for heavy dependencies | Users shouldn't pay for syntect/sysinfo/tokio if they don't use them | Three feature gates complicate build matrix; examples must be split by feature |
| Widget trait not decomposed (yet) | Backward compatibility; sub-traits exist as markers | `Box<dyn Widget>` works but `Box<dyn Renderable>` doesn't (no blanket for trait object) |

---

## 3. Core Layers

### 3.1 Backend Module (`src/backend/tty.rs`)

**Purpose:** OS-level terminal interface. The only platform-specific code in the engine.

#### Functions

```rust
/// Poll stdin for available bytes with a timeout in milliseconds.
/// Returns Ok(true) if data is available, Ok(false) on timeout.
/// Uses poll(2) on Unix (PPOLL on Linux for microsecond precision).
pub fn poll_input(fd: impl AsFd, timeout_ms: i32) -> io::Result<bool>

/// Get terminal window size via TIOCGWINSZ ioctl.
/// Returns (width_in_columns, height_in_rows).
pub fn get_window_size(fd: impl AsFd) -> io::Result<(u16, u16)>
```

#### Implementation Details

**`poll_input`:**
1. Uses `libc::pollfd` with `POLLIN` event
2. Calls `libc::poll()` with timeout_ms (on macOS/FreeBSD) or `libc::ppoll()` (on Linux) for sub-millisecond precision
3. Returns `Ok(true)` if `pollfd.revents & POLLIN != 0`
4. Returns `Ok(false)` if `poll()` returns 0 (timeout)
5. Returns `Err(io::Error)` if `poll()` returns -1

**`get_window_size`:**
1. Calls `libc::ioctl(fd, TIOCGWINSZ, &ws)` 
2. If successful, returns `(ws.ws_col, ws.ws_row)`
3. On failure, falls back to `(80, 24)`

**Platform gating:** The entire module is behind `#[cfg(not(target_os = "windows"))]`. On Windows, these functions are unavailable and the engine will not compile — Windows support is deferred.

### 3.2 Terminal Wrapper (`src/core/terminal.rs`)

**Purpose:** RAII terminal manager. Enters raw mode and alternate screen on construction, restores state on drop.

#### Struct Definition

```rust
pub struct Terminal<W: Write> {
    writer: W,
    original_termios: libc::termios,          // Saved terminal attributes for restore
    raw_mode: bool,                           // Whether raw mode is currently active
    alt_screen: bool,                         // Whether alternate screen is active
    capabilities: Capabilities,               // Detected terminal features
    saved_cursor: bool,                       // Whether cursor position was saved (DECSC)
}
```

#### Public API

```rust
// Construction
Terminal::new(writer: W) -> io::Result<Self>

// Raw mode lifecycle
.suspend() -> io::Result<()>                 // Restore cooked mode temporarily
.resume() -> io::Result<()>                  // Re-enter raw mode

// Cursor control
.set_cursor(col: u16, row: u16) -> io::Result<()>  // \x1b[row;colH
.show_cursor() -> io::Result<()>             // \x1b[?25h
.hide_cursor() -> io::Result<()>             // \x1b[?25l
.set_cursor_shape(shape: CursorShape)         // \x1b[ q sequences

// Capabilities
.capabilities() -> &Capabilities

// Writer access (for escape sequence output)
.writer() -> &mut W
```

#### Capabilities Detection

```rust
pub struct Capabilities {
    pub true_color: bool,                     // COLORTERM=truecolor or 24-bit
    pub sgr_mouse: bool,                      // SGR mouse mode 1006 supported
    pub bracketed_paste: bool,                // Bracketed paste mode 2004 supported
    pub kitty_keyboard: bool,                 // Kitty keyboard protocol supported
    pub sync_mode: bool,                      // Synchronized output mode 2026 supported
    pub cursor_shape: bool,                   // Cursor shape escape sequences supported
    pub clipboard: bool,                      // OSC 52 clipboard access supported
    pub hyperlinks: bool,                     // OSC 8 hyperlink support
    pub sixel: bool,                          // Sixel graphics support (via sixel feature)
}
```

Detection logic:
- **true_color:** Checks `COLORTERM` env var for `truecolor` or `24bit`; also checks `TERM` for known true-color terminals
- **sgr_mouse:** Assumed supported; SGR mode is enabled unconditionally via DECSET 1006; legacy terminals silently ignore
- **bracketed_paste:** Enabled unconditionally via DECSET 2004
- **kitty_keyboard:** Probe via CSI?u query; checks response
- **sync_mode:** Enabled unconditionally via DECSET 2026
- **cursor_shape:** Assumed supported on xterm-compatible terminals
- **clipboard:** Assumed available; OSC 52 is silently ignored by terminals that don't support it

#### Terminal State Management (RAII Sequence)

```
Terminal::new(writer):
  1. Save terminal attributes via tcgetattr → original_termios
  2. Save cursor position via DECSC (\x1b 7)
  3. Enter alternate screen via DECSET 1049 (\x1b[?1049h)
  4. Enable SGR mouse via DECSET 1006 (\x1b[?1006h)
  5. Enable any-event mouse via DECSET 1003 (\x1b[?1003h)
  6. Enable bracketed paste via DECSET 2004 (\x1b[?2004h)
  7. Set raw mode: cfmakeraw(&termios), tcsetattr(TCSAFLUSH)
  8. Enable kitty keyboard if detected (\x1b[=1u)
  9. Save terminal title (OSC 0)

Terminal::drop():
  1. Disable kitty keyboard (\x1b[=0u)
  2. Disable bracketed paste via DECRST 2004 (\x1b[?2004l)
  3. Disable SGR mouse via DECRST 1006 (\x1b[?1006l)
  4. Disable any-event mouse via DECRST 1003 (\x1b[?1003l)
  5. Restore cursor position via DECRC (\x1b 8)
  6. Leave alternate screen via DECRST 1049 (\x1b[?1049l)
  7. Show cursor (\x1b[?25h)
  8. Restore terminal attributes via tcsetattr(TCSAFLUSH, &original_termios)

RESTORE_SEQ: "\x1b[0m\x1b[?25h\x1b[?1006l\x1b[?1003l\x1b[?2004l\x1b[?1049l\x1b8"
  Used in panic hook to restore terminal state during panics.
```

#### Terminal Suspend/Resume

```
suspend():
  1. Disable raw mode: tcsetattr with original_termios
  2. Disable SGR mouse: \x1b[?1006l
  3. Disable bracketed paste: \x1b[?2004l
  4. Leave alternate screen: \x1b[?1049l
  5. Show cursor: \x1b[?25h

resume():
  1. Save cursor: DECSC
  2. Enter alternate screen: \x1b[?1049h
  3. Enable SGR mouse: \x1b[?1006h
  4. Enable bracketed paste: \x1b[?2004h
  5. Set raw mode: tcsetattr with cfmakeraw
  6. composer.invalidate_last_frame() — force full redraw
  7. dirty_tracker.mark_all_dirty()
```

#### CursorShape Enum

```rust
pub enum CursorShape {
    Default,       // Terminal default (usually Block)
    Block,         // █ blinking block
    BlockSteady,   // █ non-blinking block
    Underline,     // _ blinking underline
    UnderlineSteady, // _ non-blinking underline
    Bar,           // | blinking vertical bar
    BarSteady,     // | non-blinking vertical bar
    Hidden,        // Invisible cursor
}
```

Escape sequences (DECSCUSR): `\x1b[{N} q` where N = 0 (default), 1 (blinking block), 2 (steady block), 3 (blinking underline), 4 (steady underline), 5 (blinking bar), 6 (steady bar).

### 3.3 DraconError (`src/error.rs`)

```rust
pub enum DraconError {
    Io(io::Error),                              // Wrapped I/O errors
    InvalidKeybinding(String),                   // Keybinding parse failures
    ThemeNotFound(String),                       // Unknown theme name
    WidgetNotFound(WidgetId),                    // Widget lookup failures
    ConfigError(String),                         // TOML config parse/validate errors
    PluginError(String),                         // Plugin init/register/load errors
    Serialization(String),                       // Widget state serialization errors
    InvalidState(String),                        // Invalid widget state transitions
}

impl std::error::Error for DraconError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DraconError::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl fmt::Display for DraconError { /* human-readable messages */ }
impl From<io::Error> for DraconError { /* auto-convert io::Error */ }
```

### 3.4 Contracts Module (`src/contracts.rs`)

Contains `#![forbid(unsafe_code)]` — the only module with this attribute. Defines the abstract trait contracts for the UI runtime:

```rust
/// Renders application state to the terminal.
pub trait UiRenderer<State> {
    type Error;
    fn render(&mut self, state: &State) -> Result<(), Self::Error>;
}

/// Polls UI events from the environment.
pub trait UiEventSource {
    type Error;
    fn next_event(&mut self) -> Result<Option<UiEvent>, Self::Error>;
}

/// Main UI runtime loop coordinating rendering and events.
pub trait UiRuntime<State> {
    type Error;
    fn run<R, E>(&mut self, renderer: &mut R, events: &mut E, state: &mut State) -> Result<(), Self::Error>
    where R: UiRenderer<State>, E: UiEventSource;
}
```

Also defines legacy event types:
- `UiEvent` — Tick, Key { key: Cow<str> }, Resize(UiResize), QuitRequested
- `InputEvent` — Key(KeyEvent), Mouse(MouseEvent), Resize, Paste, FocusGained, FocusLost, Unsupported
- `UiResize` — width, height

### 3.5 Input Module (`src/input/`)

#### Event Type Hierarchy

```rust
/// Unified input event from the terminal.
pub enum Event {
    Key(KeyEvent),                              // Keyboard key press/repeat/release
    Mouse(MouseEvent),                          // Mouse button, drag, scroll, move
    Resize(u16, u16),                           // Terminal resize (new_width, new_height)
    Paste(String),                              // Bracketed paste content
    FocusGained,                                // Terminal gained focus (CSI I)
    FocusLost,                                  // Terminal lost focus (CSI O)
    Unsupported(Vec<u8>),                       // Unrecognized escape sequence
}

/// Keyboard event with full modifier support.
pub struct KeyEvent {
    pub code: KeyCode,                          // Which key
    pub modifiers: KeyModifiers,                // SHIFT, CTRL, ALT, SUPER, HYPER, META
    pub kind: KeyEventKind,                     // Press, Repeat, Release
}

pub enum KeyEventKind { Press, Repeat, Release }

/// Key code enumeration — exhaustive across standard + extended keys.
pub enum KeyCode {
    // Navigation
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    // Editing
    Backspace, Delete, Insert,
    // Control
    Enter, Tab, BackTab, Esc, Null,
    // Function keys
    F(u8),                                      // F1-F12 (and beyond via kitty protocol)
    // Printable
    Char(char),
    // Lock keys
    CapsLock, ScrollLock, NumLock,
    // System
    PrintScreen, Pause, Menu, KeypadBegin,
    // Media keys
    Media(MediaKeyCode),
    // Modifier keys (when pressed as keys, not modifiers)
    Modifier(ModifierKeyCode),
}

pub enum MediaKeyCode {
    Play, Pause, PlayPause, Reverse, Stop,
    FastForward, Rewind, TrackNext, TrackPrevious,
    Record, LowerVolume, RaiseVolume, MuteVolume,
}

pub enum ModifierKeyCode {
    LeftShift, LeftControl, LeftAlt, LeftSuper, LeftHyper, LeftMeta,
    RightShift, RightControl, RightAlt, RightSuper, RightHyper, RightMeta,
    IsoLevel3Shift, IsoLevel5Shift,
}

/// Modifier flags (bitflags).
pub struct KeyModifiers: u8 {
    const SHIFT   = 0b0000_0001;
    const CONTROL = 0b0000_0010;
    const ALT     = 0b0000_0100;
    const SUPER   = 0b0000_1000;
    const HYPER   = 0b0001_0000;
    const META    = 0b0010_0000;
}

/// Mouse event with position, button, and modifiers.
pub struct MouseEvent {
    pub kind: MouseEventKind,                   // Down, Up, Drag, Moved, Scroll*
    pub column: u16,                            // 1-based column (adjusted to 0-based)
    pub row: u16,                               // 1-based row (adjusted to 0-based)
    pub modifiers: KeyModifiers,                // Active modifiers during event
}

pub enum MouseEventKind {
    Down(MouseButton),                          // Button pressed
    Up(MouseButton),                            // Button released
    Drag(MouseButton),                          // Drag with button held
    Moved,                                      // Mouse moved (no button)
    ScrollDown, ScrollUp, ScrollLeft, ScrollRight,  // Wheel
}

pub enum MouseButton {
    Left, Right, Middle, Back, Forward, Other(u8),
}
```

#### Parser (`src/input/parser.rs`)

**Architecture:** Byte-level event parser. Maintains internal state machine and byte buffer. Fed one byte at a time via `advance()`. Emits `Event` when complete sequence is recognized.

```rust
pub struct Parser {
    buffer: Vec<u8>,                            // Accumulates bytes for current sequence
    state: ParserState,                         // Current parse state
    params: Vec<u16>,                           // CSI parameter values
    intermediates: Vec<u8>,                     // CSI intermediate bytes
    // State for specific sequences
    paste_buffer: Vec<u8>,                      // Bracketed paste accumulation
    kitty_key_buffer: Vec<u8>,                  // Kitty keyboard protocol accumulation
}

enum ParserState {
    Ground,                                     // Normal text
    Escape,                                     // Saw ESC (\x1b)
    CsiEntry,                                   // Saw ESC [ or CSI (\x9b)
    CsiParam,                                   // Accumulating CSI parameters
    CsiIntermediate,                            // CSI intermediate bytes
    CsiIgnore,                                  // Ignoring invalid CSI sequence
    OscString,                                  // OSC sequence (\x1b])
    DcsEntry,                                   // Device Control String entry
    DcsPassthrough,                             // DCS passthrough
    SosString,                                  // Start of String
    PmString,                                   // Privacy Message
    ApcString,                                  // Application Program Command
    PasteStart,                                 // Saw \x1b[200~ (bracketed paste start)
    PasteBody,                                  // Inside bracketed paste
}
```

**Supported Escape Sequences (complete table):**

| Sequence | Event | Description |
|----------|-------|-------------|
| `\x1b[M...` | `Mouse` | SGR mouse event (1006) |
| `\x1b[<{b};{x};{y}{M|m}` | `Mouse` | SGR-encoded mouse (button, x, y, press/release) |
| `\x1b[{n}~` | `Key(KeyCode::F(n))` | Function keys via ~ encoding (F1=11~, F2=12~, etc.) |
| `\x1b[200~` | (paste start) | Bracketed paste begin |
| `\x1b[201~` | `Paste(text)` | Bracketed paste end |
| `\x1b[{y};{x}R` | `Resize(x,y)` | Cursor position report (resize detection) |
| `\x1b[I` | `FocusGained` | Terminal focus in |
| `\x1b[O` | `FocusLost` | Terminal focus out |
| `\x1b[1;{n}A` | `Key(Code::Up, mods)` | Cursor up with modifiers |
| `\x1b[1;{n}B` | `Key(Code::Down, mods)` | Cursor down with modifiers |
| `\x1b[1;{n}C` | `Key(Code::Right, mods)` | Cursor right with modifiers |
| `\x1b[1;{n}D` | `Key(Code::Left, mods)` | Cursor left with modifiers |
| `\x1b[1;{n}H` | `Key(Code::Home, mods)` | Home with modifiers |
| `\x1b[1;{n}F` | `Key(Code::End, mods)` | End with modifiers |
| `\x1b[5;{n}~` | `Key(Code::PageUp, mods)` | PageUp with modifiers |
| `\x1b[6;{n}~` | `Key(Code::PageDown, mods)` | PageDown with modifiers |
| `\x1b[2;{n}~` | `Key(Code::Insert, mods)` | Insert with modifiers |
| `\x1b[3;{n}~` | `Key(Code::Delete, mods)` | Delete with modifiers |
| `\x1b[15;{n}~` | `Key(Code::F5, mods)` | F5 with modifiers |
| `\x1b[17;{n}~` | `Key(Code::F6, mods)` | F6 with modifiers |
| `\x1b[18;{n}~` | `Key(Code::F7, mods)` | F7 with modifiers |
| `\x1b[19;{n}~` | `Key(Code::F8, mods)` | F8 with modifiers |
| `\x1b[20;{n}~` | `Key(Code::F9, mods)` | F9 with modifiers |
| `\x1b[21;{n}~` | `Key(Code::F10, mods)` | F10 with modifiers |
| `\x1b[23;{n}~` | `Key(Code::F11, mods)` | F11 with modifiers |
| `\x1b[24;{n}~` | `Key(Code::F12, mods)` | F12 with modifiers |
| `\x1b[27;{n};{c}~` | `Key(Code::Char(c), mods)` | Kitty protocol (explicit Unicode) |
| `\x1bOA` | `Key(Up)` | SS3 cursor up (xterm legacy) |
| `\x1bOB` | `Key(Down)` | SS3 cursor down |
| `\x1bOC` | `Key(Right)` | SS3 cursor right |
| `\x1bOD` | `Key(Left)` | SS3 cursor left |
| `\x1bOH` | `Key(Home)` | SS3 home |
| `\x1bOF` | `Key(End)` | SS3 end |
| `\x1b...u` | `Key(...)` | Kitty keyboard protocol (full Unicode + modifiers) |
| `\x1b[?u` | (capability response) | Kitty protocol query response |
| `\b\x7f` | `Key(Backspace)` | ASCII backspace / DEL |
| `\r` | `Key(Enter)` | Carriage return |
| `\t` | `Key(Tab)` | Horizontal tab |
| `\x1b\t` | `Key(BackTab)` | ESC Tab (legacy backtab) |
| `\x1b[Z` | `Key(BackTab)` | CSI Z (modern backtab) |
| `\x1b\x7f` | `Key(Alt+Backspace)` | ESC DEL (Alt backspace) |
| Printable chars | `Key(Char(c))` | Normal text input |
| `\x1b` | (state start) | Start of escape sequence |

**Modifier encoding in CSI sequences:**
Parameter `n` encodes modifiers as bitmask + 1: 1=none, 2=shift, 3=alt, 4=alt+shift, 5=ctrl, 6=ctrl+shift, 7=ctrl+alt, 8=ctrl+alt+shift.

#### InputReader (`src/input/reader.rs`)

```rust
pub struct InputReader<R: Read> {
    reader: R,                                  // Underlying byte source
    parser: Parser,                             // Event parser
}

impl<R: Read> InputReader<R> {
    pub fn new(reader: R) -> Self;
    pub fn read(&mut self) -> io::Result<Option<Event>>;  // Non-blocking read
    pub fn read_blocking(&mut self) -> io::Result<Event>;  // Blocking read
}
```

#### Kitty Keyboard Protocol (`src/input/kitty_key.rs`)

Implements parsing for the Kitty keyboard protocol extension, which provides:
- Unicode code points for all keys (including modifiers)
- Press/repeat/release distinction
- All modifier combinations
- Media keys and special keys as Unicode values

Format: `\x1b[{code};{modifiers}u` where `code` is the Unicode code point of the key and `modifiers` is a bitmask.

#### Async Reader (`src/input/async_reader.rs`, feature-gated `async`)

```rust
#[cfg(feature = "async")]
pub struct AsyncReader { /* tokio::io::Stdin + Parser */ }

#[cfg(feature = "async")]
impl AsyncReader {
    pub fn new() -> Self;
    pub async fn read(&mut self) -> Option<Event>;  // Async non-blocking read
    pub async fn read_blocking(&mut self) -> Event;  // Async blocking read
}
```

---

## 4. Framework Layer

### 4.1 App Struct (`src/framework/app.rs`)

**Total LOC:** ~1,591  
**Purpose:** The main application entry point. Owns the terminal, compositor, input parser, widget registry, focus manager, dirty tracker, animation manager, event bus, scene router, and keybinding set.

#### Struct Fields

```rust
pub struct App {
    // Core
    terminal: Terminal<io::Stdout>,              // RAII terminal wrapper
    compositor: Compositor,                       // Plane compositing engine
    parser: Parser,                               // Input event parser

    // Identity
    title: String,                                // Terminal window title
    fps: u32,                                     // Target FPS (clamped 1-120)
    theme: Theme,                                 // Current UI theme

    // Event loop control
    running: Arc<AtomicBool>,                     // Thread-safe stop flag
    frame_count: Arc<AtomicU64>,                  // Monotonically increasing frame counter
    last_frame_time: Instant,
    last_tick_time: Instant,
    tick_interval: Duration,                      // Between tick callbacks (default 250ms)
    tick_count: u64,                              // Number of ticks fired

    // Callbacks
    on_tick: RefCell<Option<TickCallback>>,       // Tick callback closure

    // Widget management
    widgets: RefCell<Vec<Box<dyn Widget>>>,        // Registered widgets
    z_order_cache: RefCell<Vec<WidgetId>>,         // Cached z-order sorted IDs
    z_order_dirty: RefCell<bool>,                  // Whether cache needs rebuild
    next_widget_id: usize,                         // Monotonically increasing widget IDs

    // Framework subsystems
    focus_manager: FocusManager,
    dirty_tracker: DirtyRegionTracker,
    animations: AnimationManager,
    event_bus: EventBus,
    scene_router: SceneRouter,
    keybindings: KeybindingSet,

    // Command-driven architecture
    commands: RefCell<Vec<BoundCommand>>,          // Global command registry
    command_tracking: RefCell<HashMap<WidgetId, (Instant, BoundCommand)>>,  // Periodic command schedule

    // Input shield
    input_shield_until: Cell<Option<Instant>>,     // Swallow input until this time
}
```

#### Complete Public API

```rust
// ── Construction ─────────────────────────────────────────────────────
pub fn new() -> io::Result<Self>                          // Default: terminal init
pub fn from_toml(path: &Path) -> io::Result<Self>         // From TOML config file
pub fn default() -> Self                                   // Panics on terminal failure

// ── Builder Pattern ─────────────────────────────────────────────────
pub fn title(self, title: &str) -> Self                    // Sets terminal window title
pub fn fps(self, fps: u32) -> Self                         // Target FPS (clamped 1-120)
pub fn theme(self, theme: Theme) -> Self                   // Initial theme
pub fn on_tick<F>(self, f: F) -> Self                      // Tick callback
  where F: FnMut(&mut Ctx, u64) + 'static
pub fn on_input<F>(self, handler: F) -> Self               // Keyboard input handler
  where F: FnMut(KeyEvent) -> bool + 'static
pub fn tick_interval(self, ms: u64) -> Self                // Tick interval (ms)

// ── Widget Management ───────────────────────────────────────────────
pub fn add_widget(&mut self, widget: Box<dyn Widget>, area: Rect) -> WidgetId
pub fn remove_widget(&mut self, id: WidgetId)
pub fn widget(&self, id: WidgetId) -> Option<WidgetRef<'_>>
pub fn widget_mut(&mut self, id: WidgetId) -> Option<WidgetRefMut<'_>>
pub fn widget_count(&self) -> usize
pub fn plane_count(&self) -> usize

// ── Theme ───────────────────────────────────────────────────────────
pub fn set_theme(&mut self, theme: Theme) -> &mut Self

// ── Commands ────────────────────────────────────────────────────────
pub fn add_command(&mut self, cmd: BoundCommand)
pub fn available_commands(&self) -> Vec<BoundCommand>

// ── Input Shield ────────────────────────────────────────────────────
pub fn shield_input(&self, duration: Duration)
pub fn is_input_shielded(&self) -> bool

// ── Run ─────────────────────────────────────────────────────────────
pub fn run<F>(mut self, f: F) -> io::Result<()>
  where F: FnMut(&mut Ctx)
pub fn stop(&self)

// ── Metrics ─────────────────────────────────────────────────────────
pub fn frame_time_ms(&self) -> f64
```

#### WidgetRef / WidgetRefMut

```rust
/// Opaque wrapper around Ref<'_, Box<dyn Widget>> that hides the borrow guard.
pub struct WidgetRef<'a> {
    inner: Ref<'a, Box<dyn Widget>>,
}
impl<'a> Deref for WidgetRef<'a> { type Target = Box<dyn Widget>; }

/// Opaque wrapper around RefMut<'_, Box<dyn Widget>>.
pub struct WidgetRefMut<'a> {
    inner: RefMut<'a, Box<dyn Widget>>,
}
impl<'a> Deref for WidgetRefMut<'a> { type Target = Box<dyn Widget>; }
impl<'a> DerefMut for WidgetRefMut<'a> {}
```

#### Internal Event Dispatch

**`dispatch_key`:**
1. Check `keybindings.matches(QUIT)`: set `running = false`, return
2. Check `keybindings.matches(BACK)`: 
   - If focused widget handles it, done
   - If not, check `scene_router.can_go_back()` → pop or quit
3. Check `Tab`: cycle focus via `FocusManager::tab_next/prev`, call `on_blur`/`on_focus`
4. Other keys: forward to focused widget via `widget.handle_key(key)`
5. After dispatch, check `widget.current_theme()` for pattern 2 theme sync
6. Check if `scene_router.stack_depth()` changed (scene pop → mark all dirty)

**`dispatch_mouse`:**
1. Rebuild z-order cache if dirty
2. Reverse-iterate z-ordered widget IDs
3. For each widget, check if `col, row` falls within `widget.area()`
4. First match wins (topmost widget under cursor)
5. Update focus to matched widget
6. Compute local coordinates: `col - area.x`, `row - area.y`
7. Call `widget.handle_mouse(kind, local_col, local_row)`

**`dispatch_resize`:**
1. `compositor.resize(new_w, new_h)` — resize frame buffers
2. `dirty_tracker.mark_all_dirty()` — force full redraw
3. For each widget: `set_area(Rect::new(0, 0, new_w, new_h))` + `mark_dirty()`

**`dispatch_paste`:**
1. Get focused widget
2. For each character in paste text: create synthetic `KeyEvent(Code::Char(c))` and call `widget.handle_key()`
3. Newlines → `KeyCode::Enter`, tabs → `KeyCode::Tab`

#### Event Loop (run method) — Full Pseudocode

```
fn run(mut self, f):
  1. Write terminal title (OSC 0)
  2. Install panic hook:
     - On panic: write RESTORE_SEQ to stdout, then invoke original hook
  3. Register signal handlers (SIGINT, SIGTERM):
     - Both set running.store(false, SeqCst)  (async-signal-safe)
  4. Initialize stdin, frame_duration
  5. Initialize all widget areas to full terminal size
  6. Loop while running.load(SeqCst):
     a. frame_start = Instant::now()
     b. poll_and_dispatch_input(&mut stdin)
        - Read stdin byte by byte via poll_input
        - Parser::advance(byte) → Option<Event>
        - handle_event(event, &running)
        - Drain remaining input (up to 64 iterations)
     c. render_dirty_widgets()
        - Rebuild z-order cache
        - For each widget in z-order:
          if needs_render(): render(area) → compositor.add_plane(plane)
     d. run_tick_callback(&frame_count)
        - If tick_interval elapsed: call on_tick closure
        - If theme changed: propagate to all widgets
     e. run_periodic_commands()
        - For each registered periodic command:
          if refresh_interval elapsed: run command, parse output, apply to widget
     f. f(&mut Ctx)  // User's per-frame closure
     g. if compositor.planes not empty:
          compositor.set_dirty_regions(&dirty_tracker)
          compositor.render(&mut terminal)
     h. Focused cursor positioning
     i. animations.tick()
     j. frame_count += 1
     k. Update frame timing metrics
     l. Sleep if frame completed before frame_duration
  7. Restore original panic hook
  8. If DTRON_THEME_FILE: write self.theme.name to file
  9. Return Ok(())
```

### 4.2 Ctx Struct (`src/framework/ctx.rs`)

**Total LOC:** ~450  
**Purpose:** Context object passed to render and tick callbacks. Provides access to all framework subsystems without exposing the `App` struct directly.

#### Struct Fields

```rust
pub struct Ctx<'a> {
    pub(crate) compositor: &'a mut Compositor,
    pub(crate) theme: &'a mut Theme,
    pub(crate) frame_count: u64,
    pub(crate) last_frame: &'a Instant,
    pub(crate) terminal: &'a mut Terminal<io::Stdout>,
    pub(crate) focus_manager: &'a mut FocusManager,
    pub(crate) animations: &'a mut AnimationManager,
    pub(crate) dirty_tracker: &'a mut DirtyRegionTracker,
    pub(crate) commands: &'a RefCell<Vec<BoundCommand>>,
    pub(crate) running: &'a AtomicBool,
    pub(crate) event_bus: &'a EventBus,
    pub(crate) scene_router: &'a mut SceneRouter,
}
```

#### Complete Public API

```rust
// ── Rendering ───────────────────────────────────────────────────────
pub fn add_plane(&mut self, plane: Plane)
pub fn show_cursor(&mut self) -> io::Result<()>
pub fn hide_cursor(&mut self) -> io::Result<()>
pub fn set_cursor(&mut self, col: u16, row: u16) -> io::Result<()>

// ── Terminal Lifecycle ──────────────────────────────────────────────
pub fn suspend_terminal(&mut self) -> io::Result<()>
pub fn resume_terminal(&mut self) -> io::Result<()>

// ── Focus ───────────────────────────────────────────────────────────
pub fn set_focus(&mut self, id: WidgetId)
pub fn focused(&self) -> Option<WidgetId>

// ── Dirty Regions ───────────────────────────────────────────────────
pub fn mark_dirty(&mut self, x: u16, y: u16, width: u16, height: u16)
pub fn mark_all_dirty(&mut self)
pub fn needs_full_refresh(&self) -> bool

// ── Compositor ──────────────────────────────────────────────────────
pub fn compositor(&self) -> &Compositor
pub fn compositor_mut(&mut self) -> &mut Compositor
pub fn widget_count(&self) -> usize
pub fn plane_count(&self) -> usize
pub fn frame_time_ms(&self) -> f64
pub fn fps(&self) -> u64

// ── Theme ───────────────────────────────────────────────────────────
pub fn theme(&self) -> &Theme
pub fn set_theme(&mut self, theme: Theme)     // Changes theme (detected by App::run)

// ── Screen ──────────────────────────────────────────────────────────
pub fn clear(&mut self)                        // Force full terminal clear

// ── Split Panes ─────────────────────────────────────────────────────
pub fn split_h<F>(&mut self, f: F)             // Horizontal 50/50
  where F: FnOnce(&mut SplitPane, &mut SplitPane)
pub fn split_v<F>(&mut self, f: F)             // Vertical 50/50
  where F: FnOnce(&mut SplitPane, &mut SplitPane)

// ── Scene Router ────────────────────────────────────────────────────
pub fn scene_router(&mut self) -> &mut SceneRouter
pub fn push_scene(&mut self, id: &str)
pub fn pop_scene(&mut self) -> bool
pub fn replace_scene(&mut self, id: &str)
pub fn go_to_scene(&mut self, id: &str)

// ── Event Bus ───────────────────────────────────────────────────────
pub fn publish<E: Any + Clone>(&self, event: E)
pub fn subscribe<E: Any + Clone, F>(&self, callback: F) -> SubscriptionId
  where F: Fn(&E) + 'static
pub fn event_bus(&self) -> &EventBus

// ── Layout ──────────────────────────────────────────────────────────
pub fn layout(&self, constraints: Vec<Constraint>) -> Vec<Rect>

// ── Commands ────────────────────────────────────────────────────────
pub fn run_command(&self, cmd: &str) -> (String, String, i32)
pub fn available_commands(&self) -> Vec<BoundCommand>

// ── App Control ─────────────────────────────────────────────────────
pub fn stop(&mut self)

// ── Animations ──────────────────────────────────────────────────────
pub fn animations(&self) -> &AnimationManager
pub fn animations_mut(&mut self) -> &mut AnimationManager
```

#### Ctx Usage Examples

```
// Render pattern:
ctx.add_plane(list.render(Rect::new(0, 0, 40, 20)));

// Tick pattern:
if tick % 4 == 0 {
    let (out, _, _) = ctx.run_command("uptime");
    // update state from output
}

// Scene navigation:
ctx.push_scene("settings");
if ctx.pop_scene() { /* scene popped */ }

// Event bus:
ctx.publish(AppEvent::FileSelected(path));
```

### 4.3 InputHandler (Hidden Widget)

Created by `App::on_input()`. A zero-size widget that routes keyboard events to a closure.

```rust
struct InputHandler {
    handler: Box<dyn FnMut(KeyEvent) -> bool>,
    id: WidgetId,
    area: Rect,              // Full terminal size
    theme: Option<Theme>,
}

impl Widget for InputHandler {
    fn needs_render(&self) -> bool { false }       // Invisible
    fn focusable(&self) -> bool { true }            // Receives focus
    fn render(&self, _area: Rect) -> Plane { Plane::new(0, 0, 0) }  // Empty
    fn handle_key(&mut self, key: KeyEvent) -> bool { (self.handler)(key) }
    fn current_theme(&self) -> Option<Theme> { self.theme.clone() }
}
```

### 4.4 Widget Trait & Sub-traits (`src/framework/widget.rs`)

**Total LOC:** ~380  
**Purpose:** Core widget trait — the primary abstraction that all framework and custom widgets implement.

#### WidgetId

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WidgetId(pub usize);

impl WidgetId {
    pub fn new(id: usize) -> Self;               // Explicit ID assignment
    pub fn default_id() -> Self;                  // Self(0)
    pub fn next() -> Self;                        // Atomic counter (starts at 1)
}
```

`WidgetId::next()` uses `AtomicUsize` for auto-incrementing IDs. Thread-safe for concurrent construction.

#### Widget Trait (Complete)

```rust
pub trait Widget {
    // ── Identity & Geometry ─────────────────────────
    fn id(&self) -> WidgetId;
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect;
    fn set_area(&mut self, area: Rect);
    fn z_index(&self) -> u16 { 0 }

    // ── Rendering ────────────────────────────────────
    fn render(&self, area: Rect) -> Plane;          // &self — must be re-entrant safe
    fn draw_to(&mut self, target: &mut Plane, x: u16, y: u16) {
        // Default: render() then blit_from at (x, y)
        let plane = self.render(self.area());
        target.blit_from(&plane, x, y);
    }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}

    // ── Focus ────────────────────────────────────────
    fn focusable(&self) -> bool { true }
    fn on_focus(&mut self) {}
    fn on_blur(&mut self) {}
    fn cursor_position(&self) -> Option<(u16, u16)> { None }

    // ── Lifecycle ────────────────────────────────────
    fn on_mount(&mut self) {}                        // Called after registration
    fn on_unmount(&mut self) {}                      // Called before removal

    // ── Theme ────────────────────────────────────────
    fn on_theme_change(&mut self, _theme: &Theme) {}
    fn current_theme(&self) -> Option<Theme> { None }

    // ── Input ────────────────────────────────────────
    fn handle_key(&mut self, _key: KeyEvent) -> bool { false }   // true = consumed
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }

    // ── Commands ─────────────────────────────────────
    fn commands(&self) -> Vec<BoundCommand> { vec![] }
    fn apply_command_output(&mut self, _output: &ParsedOutput) {}
}
```

#### Sub-traits (Blanket Implementations via Widget)

Any type implementing `Widget` automatically implements all sub-traits:

```rust
pub trait Renderable {
    fn render(&self, area: Rect) -> Plane;
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}
// Blanket impl: impl<T: Widget> Renderable for T { ... }

pub trait Focusable {
    fn focusable(&self) -> bool { true }
    fn on_focus(&mut self) {}
    fn on_blur(&mut self) {}
    fn cursor_position(&self) -> Option<(u16, u16)> { None }
}

pub trait Themable {
    fn on_theme_change(&mut self, _theme: &Theme) {}
    fn current_theme(&self) -> Option<Theme> { None }
}

pub trait Commandable {
    fn commands(&self) -> Vec<BoundCommand> { vec![] }
    fn apply_command_output(&mut self, _output: &ParsedOutput) {}
}

pub trait InputHandler {
    fn handle_key(&mut self, _key: KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
}
```

#### WidgetState Trait (Serialization)

```rust
pub trait WidgetState {
    fn state_id(&self) -> Option<&str>;                     // Unique state identifier
    fn to_json(&self) -> JsonValue;                          // Serialize to JSON
    fn apply_json(&mut self, json: &JsonValue) -> Result<(), DraconError>;  // Restore from JSON
}
```

#### AsyncWidget Trait (Feature-gated)

```rust
#[cfg(feature = "async")]
pub trait AsyncWidget: Widget {
    async fn on_mount_async(&mut self) {}
    async fn on_unmount_async(&mut self) {}
}
```

### 4.5 Keybinding System (`src/framework/keybindings.rs`)

**Total LOC:** ~590  
**Purpose:** Configurable keybinding resolution with tiered override.

#### KeybindingConfig

```rust
pub struct KeybindingConfig {
    bindings: HashMap<String, String>,           // Action name → keybinding string
}
```

#### KeybindingSet (Runtime)

```rust
pub struct KeybindingSet {
    bindings: HashMap<String, ParsedBinding>,    // Action name → parsed key event
}

struct ParsedBinding {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeybindingSet {
    pub fn from_config(config: &KeybindingConfig) -> Self;
    pub fn matches(&self, action: &str, event: &KeyEvent) -> bool;
    pub fn parse_keybinding(s: &str) -> Option<(KeyCode, KeyModifiers)>;
}
```

#### Keybinding String Parsing

```
"ctrl+q"      → (KeyCode::Char('q'), CONTROL)
"ctrl+shift+t" → (KeyCode::Char('t'), CONTROL | SHIFT)
"f1"          → (KeyCode::F(1), empty)
"esc"         → (KeyCode::Esc, empty)
"enter"       → (KeyCode::Enter, empty)
"space"       → (KeyCode::Char(' '), empty)
"delete"      → (KeyCode::Delete, empty)
"backspace"   → (KeyCode::Backspace, empty)
"up"          → (KeyCode::Up, empty)
"down"        → (KeyCode::Down, empty)
"alt+f4"      → (KeyCode::F(4), ALT)
"?"           → (KeyCode::Char('?'), empty)
```

#### TOML Format

```toml
[keybindings]
quit = "ctrl+q"
help = "f1"
back = "esc"
theme = "ctrl+t"
submit = "enter"
search = "ctrl+f"
save = "ctrl+s"
new = "ctrl+n"
close = "ctrl+w"
delete = "ctrl+d"
edit = "ctrl+e"
refresh = "f5"
pause = "ctrl+p"
```

#### Standard Actions (Complete)

```rust
pub mod actions {
    pub const QUIT: &str = "quit";
    pub const HELP: &str = "help";
    pub const THEME: &str = "theme";
    pub const BACK: &str = "back";
    pub const SUBMIT: &str = "submit";
    pub const SEARCH: &str = "search";
    pub const SAVE: &str = "save";
    pub const NEW: &str = "new";
    pub const CLOSE: &str = "close";
    pub const COPY: &str = "copy";
    pub const PASTE: &str = "paste";
    pub const CUT: &str = "cut";
    pub const DELETE: &str = "delete";
    pub const REFRESH: &str = "refresh";
    pub const PAUSE: &str = "pause";
    pub const TAB_NEXT: &str = "tab_next";
    pub const TAB_PREV: &str = "tab_prev";
    pub const NEW_TAB: &str = "new_tab";
    pub const CLOSE_TAB: &str = "close_tab";
}
```

#### Resolution Order

```rust
pub fn resolve_keybindings() -> KeybindingConfig {
    let mut config = KeybindingConfig::default();          // 1. Engine defaults
    if let Ok(user) = load_config("~/.config/dracon/dracon.toml") {
        config.merge(user);                                // 2. User global
    }
    if let Ok(local) = load_config("./dracon.toml") {
        config.merge(local);                               // 3. Project local
    }
    config
}
```

Result is cached in `RwLock<Option<KeybindingConfig>>` after first call. `invalidate_keybinding_cache()` clears the cache.

### 4.6 FocusManager (`src/framework/focus.rs`)

**Total LOC:** ~330  
**Purpose:** Manages widget focus ordering, Tab navigation, and focus trapping.

#### Struct

```rust
pub struct FocusManager {
    tab_order: Vec<WidgetId>,                              // Ordered list of registered widgets
    tab_order_set: HashSet<WidgetId>,                      // Fast membership check
    focused: Option<WidgetId>,                             // Currently focused widget
    focusable: HashMap<WidgetId, bool>,                    // Per-widget focusability flag
    on_focus_change: Vec<Arc<FocusCallback>>,              // External focus callbacks
    on_trap_change: Vec<Arc<TrapCallback>>,                // Trap enter/exit callbacks
    on_focus_change_internal: Vec<FocusChangeCallback>,    // Old → new focus callbacks
    trapped: bool,                                         // Whether focus is trapped
    trap_exit_disabled: bool,                              // Whether exit via tab is disabled
}
```

#### Callback Types

```rust
pub type FocusCallback = Box<dyn Fn(WidgetId, Option<WidgetId>) + Send + Sync>;
pub type TrapCallback = Box<dyn Fn(bool) + Send + Sync>;
pub type FocusChangeCallback = Arc<dyn Fn(Option<WidgetId>, Option<WidgetId>) + Send + Sync>;
```

#### Complete API

```rust
impl FocusManager {
    pub fn new() -> Self;
    pub fn register(&mut self, id: WidgetId, focusable: bool);
    pub fn unregister(&mut self, id: WidgetId);
    pub fn set_focus(&mut self, id: WidgetId);
    pub fn focused(&self) -> Option<WidgetId>;
    pub fn tab_next(&mut self) -> bool;               // Returns true if focus changed
    pub fn tab_prev(&mut self) -> bool;
    pub fn set_trapped(&mut self, trapped: bool);
    pub fn is_trapped(&self) -> bool;
    pub fn add_focus_change_callback(&mut self, cb: FocusCallback);
    pub fn add_trap_change_callback(&mut self, cb: TrapCallback);
    pub fn add_focus_change_internal(&mut self, cb: FocusChangeCallback);
    pub fn widget_count(&self) -> usize;
    pub defocus(&mut self);                            // Clear focus entirely
}
```

### 4.7 HitZone System (`src/framework/hitzone.rs`)

**Total LOC:** ~400  
**Purpose:** Declarative interactive regions for mouse event dispatch.

#### HitZone<T>

```rust
pub struct HitZone<T: Clone + 'static> {
    pub id: T,                                         // Zone identifier
    pub x: u16, pub y: u16, pub width: u16, pub height: u16,  // Geometry
    on_click: Option<Box<dyn FnMut(ClickKind)>>,       // Click (single/double/triple)
    on_right_click: Option<Box<dyn FnMut()>>,           // Right-click
    on_drag_start: Option<Box<dyn FnMut(DragState)>>,   // Drag start
    on_drag_move: Option<Box<dyn FnMut(DragState)>>,    // Drag move
    on_drag_end: Option<Box<dyn FnMut(DragState)>>,     // Drag end
    double_click_timeout: Duration,                     // Default 300ms
    last_click_time: Option<Instant>,                   // For double/triple detection
    last_click_pos: Option<(u16, u16)>,                 // For drag detection
    click_count: u8,                                    // 1-3 for multi-click
    drag_active: bool,                                  // Whether drag is in progress
}
```

#### ClickKind

```rust
pub enum ClickKind { Single, Double, Triple }
```

#### DragState

```rust
pub enum DragState {
    Started { x: u16, y: u16 },
    Moved { x: u16, y: u16 },
    Ended { x: u16, y: u16 },
}
impl DragState {
    pub fn drag_delta(&self, other: &DragState) -> (i32, i32);
}
```

#### HitZone API

```rust
impl<T: Clone + 'static> HitZone<T> {
    pub fn new(id: T, x: u16, y: u16, width: u16, height: u16) -> Self;
    pub fn contains(&self, col: u16, row: u16) -> bool;
    pub fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, mods: KeyModifiers) -> bool;
    
    // Builder callbacks
    pub fn on_click<F>(mut self, f: F) -> Self where F: FnMut(ClickKind) + 'static;
    pub fn on_right_click<F>(mut self, f: F) -> Self where F: FnMut() + 'static;
    pub fn on_drag_start<F>(mut self, f: F) -> Self where F: FnMut(DragState) + 'static;
    pub fn on_drag_move<F>(mut self, f: F) -> Self where F: FnMut(DragState) + 'static;
    pub fn on_drag_end<F>(mut self, f: F) -> Self where F: FnMut(DragState) + 'static;
}
```

#### HitZoneGroup<T>

```rust
pub struct HitZoneGroup<T: Clone + 'static> {
    zones: Vec<HitZone<T>>,
}

impl<T: Clone + 'static> HitZoneGroup<T> {
    pub fn new() -> Self;
    pub fn add(&mut self, zone: HitZone<T>);
    pub fn dispatch_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, mods: KeyModifiers) -> Option<T>;
}
```

#### ScopedZone<T> & ScopedZoneRegistry<T>

```rust
/// Lightweight geometry-only zone (no callbacks).
pub struct ScopedZone<T> {
    pub id: T,
    pub x: u16, pub y: u16, pub width: u16, pub height: u16,
}

/// Per-frame registry: cleared at start of render, registered during render, dispatched in mouse handler.
pub struct ScopedZoneRegistry<T: Clone + PartialEq> {
    zones: Vec<ScopedZone<T>>,
}

impl<T: Clone + PartialEq> ScopedZoneRegistry<T> {
    pub fn new() -> Self;
    pub fn clear(&mut self);
    pub fn register(&mut self, id: T, x: u16, y: u16, width: u16, height: u16);
    pub fn dispatch(&self, col: u16, row: u16) -> Option<&T>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

Usage pattern:
```rust
// In widget struct:
zones: RefCell<ScopedZoneRegistry<usize>>,

// In render (cleared each frame):
self.zones.borrow_mut().clear();
self.zones.borrow_mut().register(ZONE_ID, x, y, width, height);

// In handle_mouse:
if let Some(id) = self.zones.borrow().dispatch(col, row) {
    match id {
        ZONE_ID => { /* handle click */ }
        _ => {}
    }
}
```

### 4.8 Drag-and-Drop (`src/framework/dragdrop.rs`)

**Total LOC:** ~225

#### DragPhase State Machine

```
Idle → start_drag() → Dragging
Dragging → move_ghost() → Dragging
Dragging → end_drag(over_target) → Dropped
Dragging → end_drag(no_target) → Cancelled
Dragging → cancel() → Cancelled
```

#### Types

```rust
pub enum DragPhase { Idle, Dragging, Dropped, Cancelled }

pub struct DragItem<T> {
    pub data: T,
    pub source_id: usize,
}

pub struct DragGhost {
    pub label: String,
    pub width: u16,
    pub height: u16,
}
impl DragGhost {
    pub fn new(label: impl Into<String>) -> Self;
    pub fn render(&self, col: u16, row: u16, theme: &Theme) -> Plane;  // Renders at z=9000
}

pub struct DropTarget<T> {
    pub id: T,
    pub x: u16, pub y: u16, pub width: u16, pub height: u16,
    pub accept_types: Vec<&'static str>,
}

pub struct DragManager<T> {
    phase: DragPhase,
    items: Vec<DragItem<T>>,
    source_id: usize,
    ghost: Option<DragGhost>,
    start_col: u16, start_row: u16,
    current_col: u16, current_row: u16,
}
```

#### DragManager API

```rust
impl<T: Clone> DragManager<T> {
    pub fn new() -> Self;
    pub fn start_drag(&mut self, item: DragItem<T>, ghost: Option<DragGhost>, col: u16, row: u16);
    pub fn move_ghost(&mut self, col: u16, row: u16);
    pub fn end_drag(&mut self, col: u16, row: u16, targets: &[DropTarget<T>]) -> Option<&T>;
    pub fn cancel(&mut self);
    pub fn is_dragging(&self) -> bool;
    pub fn current_item(&self) -> Option<&DragItem<T>>;
    pub fn ghost(&self) -> Option<&DragGhost>;
    pub fn phase(&self) -> DragPhase;
}
```

### 4.9 Marquee Selection (`src/framework/marquee.rs`)

**Total LOC:** ~425

#### MarqueeRect

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MarqueeRect {
    pub min_col: u16, pub min_row: u16,
    pub max_col: u16, pub max_row: u16,
}
```

#### MarqueeState

```rust
pub struct MarqueeState {
    pub is_active: bool,                              // Whether marquee is currently visible
    start: Option<(u16, u16)>,                        // Initial mouse down position
    current: Option<(u16, u16)>,                      // Current mouse position (during drag)
    pending_click_idx: Option<usize>,                 // Deferred click (resolved on no-drag up)
    threshold_sq: f32,                                // Distance-squared threshold (default 4.0)
}
```

#### MarqueeState API

```rust
impl MarqueeState {
    pub fn new() -> Self;
    pub fn start_tracking(&mut self, col: u16, row: u16);
    pub fn defer_click(&mut self, idx: usize);       // Deferred click → resolve on mouseUp
    pub fn update(&mut self, col: u16, row: u16) -> bool;  // Returns true if just activated
    pub fn rect(&self) -> Option<MarqueeRect>;        // Normalized bounding rect
    pub fn take_pending_click(&mut self) -> Option<usize>;
    pub fn clear(&mut self);
    pub fn reset(&mut self);
}

pub fn render_marquee(plane: &mut Plane, marquee: &MarqueeState, theme: &Theme);
```

#### Staggered Threshold Design

```
Marquee activation:  dist_sq ≥ 4.0   (2px)
File drag activation: dist_sq ≥ 9.0   (3px)
Marquee cancels file drag on activation.
```

### 4.10 Layout Engine (`src/framework/layout.rs`)

**Total LOC:** ~450

#### Types

```rust
pub enum Direction { Horizontal, Vertical }

pub enum Constraint {
    Percentage(u16),     // 0-100% of remaining space
    Fixed(u16),          // Fixed size in cells
    Min(u16),            // Minimum size (grows to fill)
    Max(u16),            // Maximum size (shrinks to fit)
    Ratio(u16, u16),     // Numerator/denominator of remaining space
}

pub struct Layout {
    constraints: Vec<Constraint>,
    direction: Direction,
    spacing: u16,
    margin: u16,
    name: Option<&'static str>,
    cached_layout: RefCell<Option<(Rect, Vec<Rect>)>>,  // Cache for repeated calls
}
```

#### Layout API

```rust
impl Layout {
    pub fn new(constraints: Vec<Constraint>) -> Self;    // Horizontal (default)
    pub fn horizontal(constraints: Vec<Constraint>) -> Self;
    pub fn vertical(constraints: Vec<Constraint>) -> Self;
    pub fn direction(mut self, direction: Direction) -> Self;
    pub fn spacing(mut self, spacing: u16) -> Self;      // Between children
    pub fn margin(mut self, margin: u16) -> Self;         // Outer margin
    pub fn name(mut self, name: &'static str) -> Self;    // Debug label
    pub fn layout(&self, area: Rect) -> Vec<Rect>;
    pub fn with_caching(mut self) -> Self;                // Enable result caching
    pub fn invalidate_cache(&self);                       // Force re-computation
}
```

#### Layout Algorithm

```
layout(area: Rect) → Vec<Rect>:
  1. If cached result exists and area unchanged, return cached
  2. Let available = area dimension along direction axis
  3. Subtract margins
  4. First pass: resolve Fixed and Percentage constraints
     - Fixed: exact size
     - Percentage: percentage of total available
     - Track consumed space
  5. Compute remaining space = available - fixed_consumed - spacing*(n-1)
  6. Second pass: resolve Ratio, Min, Max constraints against remaining
     - Ratio: remaining * num / den
     - Min: max(min, remaining/n)
     - Max: min(max, remaining/n)
  7. If horizontal: distribute rectangles left-to-right with spacing
  8. If vertical: distribute rectangles top-to-bottom with spacing
  9. Each child Rect has full perpendicular dimension (height if horizontal, width if vertical)
  10. Cache result if caching enabled
  11. Return Vec<Rect>
```

### 4.11 Scroll System (`src/framework/scroll.rs`)

**Total LOC:** ~250

#### ScrollState

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollState {
    pub offset: usize,                              // Rows scrolled off top
    pub content_height: usize,                      // Total content rows
    pub viewport_height: usize,                     // Visible rows
}

impl ScrollState {
    pub fn max_offset(&self) -> usize;
    pub fn page_size(&self) -> usize;               // viewport_height - 1, min 1
    pub fn scroll_up(&mut self, n: usize);
    pub fn scroll_down(&mut self, n: usize);
    pub fn scroll_to(&mut self, offset: usize);
    pub fn scroll_to_end(&mut self);
    pub fn scroll_to_beginning(&mut self);
    pub fn start_row(&self) -> usize;               // = offset
    pub fn end_row(&self) -> usize;
    pub fn is_scrollable(&self) -> bool;
    pub fn visible_range(&self) -> std::ops::Range<usize>;
    pub fn is_at_top(&self) -> bool;
    pub fn is_at_bottom(&self) -> bool;
    pub fn fraction(&self) -> f32;                  // Scroll position as 0.0-1.0
}
```

#### ScrollContainer

```rust
pub struct ScrollContainer;

impl ScrollContainer {
    pub fn render(content: &Plane, area: Rect, state: &ScrollState, theme: &Theme) -> Plane;
    pub fn render_with_width(content: &Plane, area: Rect, state: &ScrollState, theme: &Theme, scrollbar_width: u16) -> Plane;
}
```

### 4.12 Animation System (`src/framework/animation.rs`)

**Total LOC:** ~500

#### Types

```rust
pub enum Easing {
    Linear,
    Sine(EasingType),
    Quadratic(EasingType),
    Cubic(EasingType),
    Exponential(EasingType),
    Elastic(EasingType),
    Bounce(EasingType),
    Back(EasingType),
}

pub enum EasingType { In, Out, InOut }
```

**Easing function formulas:**
- Linear: `t`
- Sine In: `1 - cos(t * π / 2)`
- Sine Out: `sin(t * π / 2)`
- Sine InOut: `-(cos(π * t) - 1) / 2`
- Quadratic In: `t²`
- Quadratic Out: `t * (2 - t)`
- Cubic In: `t³`
- Cubic Out: `(t - 1)³ + 1`
- Exponential In: `2^(10 * (t - 1))`
- Exponential Out: `1 - 2^(-10 * t)`
- Elastic: spring-like overshoot
- Bounce: floor-impact bounce
- Back: overshoot with cubic return

#### Animation

```rust
pub struct Animation {
    start_value: f64, end_value: f64,
    current_value: f64,
    duration: Duration, elapsed: Duration,
    easing: Easing,
    looping: bool, yoyo: bool,
    completed: bool,
    on_complete: Option<Box<dyn FnOnce()>>,
}

impl Animation {
    pub fn new(start: f64, end: f64, duration: Duration, easing: Easing) -> Self;
    pub fn looping(mut self, looping: bool) -> Self;
    pub fn yoyo(mut self, yoyo: bool) -> Self;
    pub fn on_complete<F>(mut self, f: F) -> Self where F: FnOnce() + 'static;
    pub fn value(&self) -> f64;
    pub fn is_completed(&self) -> bool;
    pub fn reset(&mut self);
    fn tick(&mut self, delta: Duration) -> bool;    // Returns true if completed
}
```

#### AnimationManager

```rust
pub struct AnimationManager {
    animations: Vec<Animation>,
}

impl AnimationManager {
    pub fn new() -> Self;
    pub fn add(&mut self, animation: Animation);
    pub fn tick(&mut self);
    pub fn clear(&mut self);
    pub fn active_count(&self) -> usize;
}
```

### 4.13 Dirty Region Tracking (`src/framework/dirty_regions.rs`)

**Total LOC:** ~120

```rust
#[derive(Clone, Copy, Debug)]
pub struct DirtyRegion {
    pub x: u16, pub y: u16, pub width: u16, pub height: u16,
}

pub struct DirtyRegionTracker {
    full_refresh: bool,                              // Flag: redraw everything
    regions: Vec<DirtyRegion>,                        // Up to 256 tracked regions
}

impl DirtyRegionTracker {
    pub fn new() -> Self;
    pub fn mark_dirty(&mut self, x: u16, y: u16, width: u16, height: u16);
    pub fn mark_all_dirty(&mut self);
    pub fn clear(&mut self);
    pub fn needs_full_refresh(&self) -> bool;
    pub fn dirty_regions(&self) -> &[DirtyRegion];
}
```

### 4.14 I18n (`src/framework/i18n.rs`)

**Total LOC:** ~512

```rust
pub struct I18n {
    locales: HashMap<String, HashMap<String, serde_json::Value>>,
    current_locale: String,
    default_locale: String,
}

pub struct I18nError { /* NotConfigured, KeyNotFound, LocaleNotFound */ }

impl I18n {
    pub fn new(default_locale: &str) -> Self;
    pub fn load_locale(&mut self, code: &str) -> Result<(), I18nError>;
    pub fn set_locale(&mut self, code: &str);
    pub fn t(&self, key: &str) -> &str;
    pub fn t_with_args(&self, key: &str, args: &[(&str, &str)]) -> String;
    pub fn current_locale(&self) -> &str;
    pub fn available_locales(&self) -> Vec<&str>;
}

#[macro_export]
macro_rules! tr {
    ($key:expr) => { /* runtime lookup via thread-local I18n */ };
}
```

### 4.15 Plugin System (`src/framework/plugin.rs`)

**Total LOC:** ~200

```rust
pub type WidgetFactory = Box<dyn Fn(WidgetId, Theme) -> Box<dyn Widget> + Send + Sync>;

pub struct PluginRegistry {
    widgets: HashMap<String, WidgetFactory>,
}

impl PluginRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, name: &str, factory: WidgetFactory);
    pub fn create(&self, name: &str, id: WidgetId, theme: Theme) -> Option<Box<dyn Widget>>;
    pub fn unregister(&mut self, name: &str);
    pub fn names(&self) -> Vec<&str>;
    pub fn is_registered(&self, name: &str) -> bool;
}
```

### 4.16 WidgetContainer & WidgetRegistry (`src/framework/widget_container.rs`)

**Total LOC:** ~150

```rust
/// Thin wrapper around a single Box<dyn Widget>.
pub struct WidgetContainer {
    inner: Box<dyn Widget>,
}

impl WidgetContainer {
    pub fn new(widget: Box<dyn Widget>) -> Self;
    pub fn id(&self) -> WidgetId;
    pub fn render(&self, area: Rect) -> Plane;
    pub fn handle_key(&mut self, key: KeyEvent) -> bool;
    pub fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool;
    pub fn widget(&self) -> &dyn Widget;
    pub fn widget_mut(&mut self) -> &mut dyn Widget;
}

/// Managed collection of WidgetContainers.
pub struct WidgetRegistry {
    containers: Vec<WidgetContainer>,
    next_id: usize,
}

impl WidgetRegistry {
    pub fn new() -> Self;
    pub fn add(&mut self, widget: Box<dyn Widget>) -> WidgetId;
    pub fn remove(&mut self, id: WidgetId);
    pub fn get(&self, id: WidgetId) -> Option<&dyn Widget>;
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget>;
    pub fn iter(&self) -> impl Iterator<Item = &dyn Widget>;
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn Widget>;
    pub fn len(&self) -> usize;
}
```

---

## 5. Widget System

### 5.1 Complete Framework Widget Directory (47 Widgets)

Each entry includes: struct fields, public methods, rendering strategy, keyboard handling, mouse handling, hover/focus behavior, and any notable algorithms.

#### 5.1.1 Autocomplete (`autocomplete.rs`)

**Purpose:** Text input with type-ahead suggestions dropdown.

```rust
pub struct Autocomplete {
    // Framework
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    focused: bool,

    // State
    input: String,                                    // Current input text
    cursor_pos: usize,                                 // Cursor position in input
    suggestions: Vec<String>,                          // All available suggestions
    filtered: Vec<String>,                             // Filtered subset matching input
    selected_index: usize,                             // Selected suggestion index
    dropdown_open: bool,                               // Whether dropdown is visible
    hovered_index: Option<usize>,                      // Hovered suggestion for mouse
    scroll_offset: usize,                              // Dropdown scroll offset
    max_visible: usize,                                // Max dropdown items visible
    zone_registry: RefCell<ScopedZoneRegistry<usize>>, // Mouse dispatch zones
}
```

**Public API:**
```rust
impl Autocomplete {
    pub fn new(suggestions: Vec<String>) -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn value(&self) -> &str;
    pub fn set_value(&mut self, value: &str);
    pub fn open_dropdown(&mut self);
    pub fn close_dropdown(&mut self);
}
```

**Rendering:** Two-layer: (1) SearchInput-style text field with cursor and clear button. (2) Dropdown panel below input showing filtered suggestions with scrollbar.

**Keyboard:** Up/Down: navigate suggestions. Enter: select suggestion (closes dropdown). Tab/Esc: close dropdown without selection. Backspace/Char: filter suggestions live.

**Mouse:** Click input field: focus + cursor positioning. Click suggestion item: select. Scroll wheel: scroll dropdown. Hover: highlight suggestion under cursor.

#### 5.1.2 Breadcrumbs (`breadcrumbs.rs`)

**Purpose:** Hierarchical path navigation with clickable segments.

```rust
pub struct Breadcrumbs {
    segments: Vec<String>,
    theme: Theme,
    separator: char,                                   // Default: '›'
    zone_registry: RefCell<ScopedZoneRegistry<usize>>, // Per-segment click zones
}
```

**Public API:**
```rust
impl Breadcrumbs {
    pub fn new(segments: Vec<String>) -> Self;
    pub fn from_path(path: &std::path::Path) -> Self;
    pub fn with_separator(mut self, separator: char) -> Self;
    pub fn render(&self, area: Rect) -> (Plane, ScopedZoneRegistry<usize>);
    // Returns (plane, zones) where zones dispatch segment index on click
}
```

**Rendering:** Segments left-to-right with separator character between. Last segment highlighted (bold, primary color). Each segment is a clickable hit zone.

**Keyboard:** None (read-only widget).

**Mouse:** Click any segment: returns segment index via zone dispatch.

#### 5.1.3 Button (`button.rs`)

**Purpose:** Clickable button with press state and hover effects.

```rust
pub struct Button {
    label: String,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    pressed: bool,                                     // Currently pressed (visual state)
    hovered: bool,                                     // Mouse hovering
    on_click: Option<Box<dyn FnMut()>>,                // Click callback
    zone_registry: RefCell<ScopedZoneRegistry<bool>>,  // Mouse zone
}
```

**Public API:**
```rust
impl Button {
    pub fn new(label: &str) -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn on_click<F>(mut self, f: F) -> Self where F: FnMut() + 'static;
    pub fn set_label(&mut self, label: &str);
    pub fn label(&self) -> &str;
}
```

**Rendering:** Bordered rectangle (rounded corners with ╭╮╰╯) with centered label. Background:
- Normal: `theme.surface`
- Hovered: `theme.hover_bg`
- Pressed: `theme.primary_active` (inverts fg)
- Focused: `theme.focus_border` border

**Keyboard:** Enter/Space: fire click callback.

**Mouse:** Down (Left): pressed state. Up (Left): fire click + reset state. Hover: hovered state.

#### 5.1.4 Calendar / DatePicker (`calendar.rs`)

**Purpose:** Interactive date selection calendar.

```rust
pub struct Calendar {
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    focused: bool,
    current_date: NaiveDate,                           // Currently displayed month
    selected_date: Option<NaiveDate>,                  // Selected date
    hovered_date: Option<NaiveDate>,                   // Hovered date
    zone_registry: RefCell<ScopedZoneRegistry<String>>, // Day/month/year click zones
}
```

**Public API:**
```rust
impl Calendar {
    pub fn new() -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn selected_date(&self) -> Option<NaiveDate>;
    pub fn set_selected_date(&mut self, date: NaiveDate);
    pub fn next_month(&mut self);
    pub fn prev_month(&mut self);
    pub fn next_year(&mut self);
    pub fn prev_year(&mut self);
    pub fn go_to_today(&mut self);
}
```

**Rendering:** Header row with month/year and navigation arrows (◀ ▶). Day-of-week header row (Mo Tu We Th Fr Sa Su). 6-week grid of day cells. Today highlighted. Selected day has selection_bg. Days from adjacent months shown dimmed.

**Keyboard:** Left/Right: previous/next day. Up/Down: previous/next week. PageUp/PageDown: previous/next month. Home/End: first/last day of month. Enter: select hovered date. Esc: cancel.

**Mouse:** Click day: select. Click ◀▶ nav: previous/next month. Click month/year: jump navigation. Hover: highlight day under cursor.

#### 5.1.5 Checkbox (`checkbox.rs`)

**Purpose:** Two-state toggle with label and check mark.

```rust
pub struct Checkbox {
    label: String,
    checked: bool,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    hovered: bool,
    zone_registry: RefCell<ScopedZoneRegistry<bool>>,
}
```

**Public API:**
```rust
impl Checkbox {
    pub fn new(label: &str) -> Self;
    pub fn checked(&self) -> bool;
    pub fn set_checked(&mut self, checked: bool);
    pub fn toggle(&mut self);
    pub fn with_theme(mut self, theme: &Theme) -> Self;
}
```

**Rendering:** `[✓] Label` or `[ ] Label`. Checked: primary color ✓. Unchecked: outline. Hovered: hover_bg background.

**Mouse:** Click: toggle state.

#### 5.1.6 ColorPicker (`color_picker.rs`)

**Purpose:** Interactive color selection with HSL sliders, hex input, swatch palette.

```rust
pub struct ColorPicker {
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    focused: bool,
    current_color: (u16, u16, u16),                   // Current R, G, B
    previous_color: (u16, u16, u16),                   // Original color (for cancel)
    selected_slider: Option<SliderKind>,               // Which slider is active
    hue: u16, saturation: u16, lightness: u16,         // HSL representation
    hex_input: String,                                 // Hex input field buffer
    palette_colors: Vec<(u16, u16, u16)>,              // Generated palette swatches
    hovered_swatch: Option<usize>,                     // Hovered palette swatch
    recent_colors: Vec<(u16, u16, u16)>,               // Recently picked colors
    zone_registry: RefCell<ScopedZoneRegistry<String>>,
}

enum SliderKind { Hue, Saturation, Lightness, Red, Green, Blue }
```

**Public API:**
```rust
impl ColorPicker {
    pub fn new() -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn color(&self) -> (u16, u16, u16);
    pub fn set_color(&mut self, r: u16, g: u16, b: u16);
    pub fn hex(&self) -> String;
}
```

**Rendering:** Color preview swatch. HSL sliders (hue rainbow bar, saturation gradient, lightness gradient). RGB numeric display. Hex input field. Generated palette (8+ colors). Recent colors row.

**Keyboard:** Tab: cycle between sliders. Left/Right: adjust slider value. Enter: confirm. Esc: cancel.

**Mouse:** Click slider: position + drag. Click swatch: select color. Hover: highlight swatches.

#### 5.1.7 CommandPalette (`command_palette.rs`)

**Purpose:** Filterable command search overlay.

```rust
pub struct CommandPalette {
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    visible: bool,
    commands: Vec<CommandItem>,
    filtered: Vec<usize>,
    filter_query: String,
    filter_input: String,
    selected_index: usize,
    hovered_index: Option<usize>,
    scroll_offset: usize,
    max_visible: usize,
    overlay_w: u16, overlay_h: u16,                    // Overlay dimensions
    on_execute: Option<ExecuteCallback>,
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}

pub struct CommandItem {
    pub id: String,
    pub name: String,
    pub category: String,
}

pub type ExecuteCallback = Box<dyn FnMut(&str)>;
```

**Public API:**
```rust
impl CommandPalette {
    pub fn new(commands: Vec<CommandItem>) -> Self;
    pub fn with_size(mut self, w: u16, h: u16) -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn on_execute<F>(mut self, f: F) -> Self where F: FnMut(&str) + 'static;
    pub fn show(&mut self);
    pub fn hide(&mut self);
    pub fn is_visible(&self) -> bool;
}
```

**Rendering:** Centered overlay (default 60% width, 50% height). Dark semi-transparent backdrop. Search input at top. Filtered command list below with category grouping. Selected item highlighted. Scrollbar if needed.

**Keyboard:** Type: filter commands (matches name + category). Up/Down: navigate. Enter: execute selected. Esc: dismiss.

**Mouse:** Click item: execute. Click outside overlay: dismiss. Scroll wheel: scroll list. Hover: highlight item.

**Filter algorithm:** Case-insensitive substring match against `CommandItem.name` and `CommandItem.category`. Fuzzy prefix matching — characters must appear in order but not necessarily contiguous.

#### 5.1.8 ConfirmDialog (`confirm_dialog.rs`)

**Purpose:** Modal yes/no/cancel confirmation dialog.

```rust
pub struct ConfirmDialog {
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    visible: bool,
    title: String,
    message: String,
    confirm_label: String,
    cancel_label: String,
    danger: bool,                                      // Danger styling (red)
    result: Option<ConfirmResult>,
    zone_registry: RefCell<ScopedZoneRegistry<ConfirmResult>>,
}

pub enum ConfirmResult { Yes, No, Cancel }
```

**Public API:**
```rust
impl ConfirmDialog {
    pub fn new(title: &str, message: &str) -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn confirm_label(mut self, label: &str) -> Self;
    pub fn cancel_label(mut self, label: &str) -> Self;
    pub fn danger(mut self, danger: bool) -> Self;
    pub fn show(&mut self);
    pub fn hide(&mut self);
    pub fn is_visible(&self) -> bool;
    pub fn result(&self) -> Option<ConfirmResult>;
}
```

**Rendering:** Centered modal box with rounded border. Title bar. Message text. Two buttons (confirm/cancel). Danger mode: confirm button in error color.

**Keyboard:** Enter/Tab: confirm. Esc: cancel.

**Mouse:** Click confirm/cancel buttons.

#### 5.1.9 ContextMenu (`context_menu.rs`)

**Purpose:** Right-click popup menu with optional submenus.

```rust
pub struct ContextMenu {
    items: Vec<ContextMenuItem>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    visible: bool,
    position: (u16, u16),
    selected_index: usize,
    hovered_index: Option<usize>,
    scroll_offset: usize,
    on_select: Option<Box<dyn FnMut(usize)>>,
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}

pub struct ContextMenuItem {
    pub label: String,
    pub action: Option<usize>,
    pub disabled: bool,
    pub separator: bool,
    pub children: Option<Vec<ContextMenuItem>>,        // Nested submenu
}

pub type ContextAction = usize;
```

**Public API:**
```rust
impl ContextMenu {
    pub fn new(items: Vec<ContextMenuItem>) -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn on_select<F>(mut self, f: F) -> Self where F: FnMut(usize) + 'static;
    pub fn show(&mut self, col: u16, row: u16);
    pub fn hide(&mut self);
    pub fn is_visible(&self) -> bool;
}
```

**Rendering:** Popup menu at cursor position. Items with label. Disabled items dimmed. Separator lines. Selected item highlighted. Submenu indicator (▶). Scrollbar if items exceed screen.

**Keyboard:** Up/Down: navigate. Enter: select. Esc: dismiss. Left/Right: open/close submenu.

**Mouse:** Click item: select (fires callback with action index). Click outside: dismiss. Hover: highlight. Scroll: scroll items.

#### 5.1.10 DebugOverlay (`debug_overlay.rs`)

**Purpose:** Performance debug information overlay.

```rust
pub struct DebugOverlay {
    theme: Theme,
    visible: bool,
    fps: u64,
    widget_count: usize,
    plane_count: usize,
    frame_time: f64,
    dirty_regions: usize,
    memory_usage: usize,
}
```

**Public API:**
```rust
impl DebugOverlay {
    pub fn new() -> Self;
    pub fn set_metrics(&mut self, fps: u64, widgets: usize, planes: usize, frame_time: f64, dirty: usize, memory: usize);
}
```

**Rendering:** Top-left overlay box showing FPS, widget count, plane count, frame time, dirty regions, memory.

**Interactions:** Read-only display. No keyboard or mouse handling.

#### 5.1.11 Divider (`divider.rs`)

**Purpose:** Horizontal or vertical visual separator line.

```rust
pub struct Divider {
    orientation: Orientation,
    theme: Theme,
    char: char,                                        // Default: '─' or '│'
    label: Option<String>,                             // Optional centered label (horizontal only)
}
```

**Public API:**
```rust
impl Divider {
    pub fn new(orientation: Orientation) -> Self;
    pub fn with_char(mut self, char: char) -> Self;
    pub fn with_label(mut self, label: &str) -> Self;
}
```

**Rendering:** Draws a line of `char` across the full width/height. If label provided, renders centered text on the line.

#### 5.1.12 EventLogger (`event_logger.rs`)

**Purpose:** Scrollable event log display.

```rust
pub struct EventLogger {
    events: Vec<LoggedEvent>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    scroll: ScrollState,
    filter: Option<String>,
    auto_scroll: bool,
}

pub struct LoggedEvent {
    pub timestamp: String,
    pub message: String,
    pub kind: String,
}
```

**API:**
```rust
impl EventLogger {
    pub fn new() -> Self;
    pub fn log(&mut self, event: LoggedEvent);
    pub fn clear(&mut self);
    pub fn set_filter(&mut self, filter: &str);
    pub fn set_auto_scroll(&mut self, auto: bool);
}
```

#### 5.1.13 Form (`form.rs`)

**Purpose:** Multi-field form container with validation and keyboard navigation.

```rust
pub struct Form {
    fields: Vec<FormField>,
    values: Vec<String>,
    errors: Vec<Option<String>>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    focused_field: Option<usize>,
    validate_on_change: bool,
    show_validation_icons: bool,
    onSubmit: Option<Box<dyn FnMut(&[(&str, &str)])>>,
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}

pub struct FormField {
    pub label: String,
    pub input_type: InputType,
    pub placeholder: String,
    pub required: bool,
    pub validation: Vec<ValidationRule>,
}

pub enum InputType { Text, Password, Email, Number, Search, Select(Vec<String>), Checkbox, Toggle, Radio(Vec<String>), Slider(u16, u16), Date, Color, TextArea }

pub enum ValidationRule { Required, MinLength(usize), MaxLength(usize), Pattern(String), MatchField(usize), Custom(String) }
```

**API:**
```rust
impl Form {
    pub fn new(fields: Vec<FormField>) -> Self;
    pub fn on_submit<F>(mut self, f: F) -> Self where F: FnMut(&[(&str, &str)]) + 'static;
    pub fn validate(&mut self) -> bool;
    pub fn values(&self) -> Vec<(&str, &str)>;
    pub fn set_value(&mut self, index: usize, value: &str);
    pub fn set_validate_on_change(&mut self, val: bool);
}
```

**Rendering:** Each field renders as: label row, input row (with type-appropriate rendering), validation row (✓ or ✗ with message). Focused field gets `focus_bg`. Error field gets `error_bg`.

**Keyboard:** Tab/Shift+Tab: cycle fields. Enter: submit. Type: edit focused field.

**Mouse:** Click field: focus + cursor position. Click submit button (if rendered).

#### 5.1.14 Gauge (`gauge.rs`)

**Purpose:** Filled progress bar with optional warn/crit thresholds.

```rust
pub struct Gauge {
    label: String,
    value: f64,
    max: f64,
    min: f64,
    warn_threshold: Option<f64>,
    crit_threshold: Option<f64>,
    show_label: bool,
    show_percentage: bool,
    theme: Theme,
    width: u16,
}
```

**API:**
```rust
impl Gauge {
    pub fn new(label: &str) -> Self;
    pub fn set_value(&mut self, value: f64);
    pub fn set_max(&mut self, max: f64);
    pub fn with_thresholds(mut self, warn: f64, crit: f64) -> Self;
    pub fn bind_command(&mut self, cmd: BoundCommand);
}
```

**Rendering:** `[████████░░░░] 67% Label`. Filled portion color changes by threshold:
- Below warn: `theme.primary`
- Above warn: `theme.warning`
- Above crit: `theme.error`
Label and percentage optional.

#### 5.1.15 Hud (`hud.rs`)

**Purpose:** Floating HUD display for system metrics.

```rust
pub struct Hud {
    theme: Theme,
    width: u16, height: u16,
    z_index: i32,
}
```

**API:**
```rust
impl Hud {
    pub fn new(z_index: i32) -> Self;
    pub fn with_size(mut self, w: u16, h: u16) -> Self;
    pub fn render_gauge(&self, x: u16, y: u16, label: &str, value: f64, max: f64, width: u16) -> Plane;
    pub fn render_text(&self, x: u16, y: u16, text: &str) -> Plane;
}
```

**Rendering:** Position: absolute, typically top-right. Renders gauges and text labels with backdrop.

#### 5.1.16 Kanban (`kanban.rs`)

**Purpose:** Kanban board with draggable columns and cards.

```rust
pub struct Kanban {
    columns: Vec<KanbanColumn>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    scroll_offset: (u16, u16),                         // Horizontal + vertical scroll
    drag_manager: RefCell<DragManager<KanbanCard>>,
    marquee: MarqueeState,
    zone_registry: RefCell<ScopedZoneRegistry<(usize, Option<usize>)>>, // (column, optional card)
}

pub struct KanbanColumn { pub title: String, pub cards: Vec<KanbanCard>, pub color: Color }
pub struct KanbanCard { pub id: String, pub title: String, pub description: String, pub priority: KanbanPriority }
pub enum KanbanPriority { Low, Medium, High, Critical }
```

**API:**
```rust
impl Kanban {
    pub fn new(columns: Vec<KanbanColumn>) -> Self;
    pub fn move_card(&mut self, from_col: usize, from_idx: usize, to_col: usize, to_idx: usize);
    pub fn add_card(&mut self, column: usize, card: KanbanCard);
    pub fn remove_card(&mut self, column: usize, index: usize);
}
```

**Rendering:** Columns laid out horizontally (scrollable). Each column: title bar with color, card stack (scrollable). Cards show title, description preview, priority indicator (color dot).

**Keyboard:** Left/Right: scroll columns. Tab: move focus between columns. Arrow keys: navigate cards within column.

**Mouse:** Click column header: select column. Click card: select. Drag card: move between columns (drag-and-drop). Scroll: vertical column scroll, horizontal board scroll. Hover: highlight card.

#### 5.1.17 KeyValueGrid (`key_value_grid.rs`)

**Purpose:** Two-column key-value data display from JSON/Scalar CLI output.

```rust
pub struct KeyValueGrid {
    entries: Vec<(String, String)>,
    theme: Theme,
    key_width: u16,
    value_width: u16,
    scroll: ScrollState,
}
```

**API:**
```rust
impl KeyValueGrid {
    pub fn new(entries: Vec<(String, String)>) -> Self;
    pub fn from_json(json: &serde_json::Value) -> Self;
    pub fn set_entries(&mut self, entries: Vec<(String, String)>);
}
```

**Rendering:** Two-column grid: keys right-aligned (bold, muted), values left-aligned. Scrollable.

#### 5.1.18 Label (`label.rs`)

**Purpose:** Static text label.

```rust
pub struct Label {
    text: String,
    theme: Theme,
    bold: bool,
    italic: bool,
    fg: Option<Color>,
    bg: Option<Color>,
}
```

**API:**
```rust
impl Label {
    pub fn new(text: &str) -> Self;
    pub fn bold(mut self) -> Self;
    pub fn italic(mut self) -> Self;
    pub fn fg(mut self, color: Color) -> Self;
    pub fn bg(mut self, color: Color) -> Self;
}
```

**Rendering:** Single line of text with specified styling.

#### 5.1.19 List (`list.rs`)

**Purpose:** Scrollable, selectable vertical list with keyboard and mouse navigation.

```rust
pub struct List<T: Clone + ToString> {
    items: Vec<T>,
    selected: usize,
    offset: usize,                                     // Scroll offset
    visible_count: usize,                               // Items visible at once
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    hovered: Option<usize>,                            // Hovered item index
    on_select: Option<SelectCallback<T>>,              // Selection callback
    wrap: bool,                                         // Wrap around at boundaries
    show_numbers: bool,                                 // Show item numbers
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}

pub type SelectCallback<T> = Box<dyn FnMut(&T)>;
```

**API:**
```rust
impl<T: Clone + ToString> List<T> {
    pub fn new(items: Vec<T>) -> Self;
    pub fn set_items(&mut self, items: Vec<T>);
    pub fn selected_index(&self) -> usize;
    pub fn get_selected(&self) -> Option<&T>;
    pub fn set_selected(&mut self, index: usize);
    pub fn set_visible_count(&mut self, count: usize);
    pub fn on_select<F>(mut self, f: F) -> Self where F: FnMut(&T) + 'static;
    pub fn scroll_state(&self) -> ScrollState;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

**Rendering:** Vertical list with items rendered left-to-right. Selected item: `selection_bg` + `selection_fg`. Hovered item: `hover_bg`. Number prefix (optional). Scrollbar on right if content overflows.

**Keyboard:** Up/Down: navigate (with wrapping). Home/End: first/last. PageUp/PageDown: page scroll. Enter: fire `on_select` callback.

**Mouse:** Click item: select. Double-click: select + fire callback. Scroll wheel: scroll. Hover: highlight.

**Navigation logic (from `list_common.rs`):**
- `navigate_up/down(selected, max, wrap)` — bounded or wrapping movement
- `navigate_page_up/down(selected, visible_count, max)` — page at a time
- `navigate_home/end(selected, max)` — jump to boundaries

#### 5.1.20 LogViewer (`log_viewer.rs`)

**Purpose:** Auto-scrolling log viewer with severity-based coloring.

```rust
pub struct LogViewer {
    lines: Vec<LogLine>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    scroll: ScrollState,
    auto_scroll: bool,
    severity_colors: HashMap<String, Color>,
    filter: Option<String>,
    wrap: bool,
}

pub struct LogLine { pub text: String, pub severity: String }
pub enum LogLevel { Error, Warn, Info, Debug, Trace, Custom(String) }
```

**API:**
```rust
impl LogViewer {
    pub fn new() -> Self;
    pub fn add_line(&mut self, line: LogLine);
    pub fn add_lines(&mut self, lines: Vec<LogLine>);
    pub fn clear(&mut self);
    pub fn set_auto_scroll(&mut self, auto: bool);
    pub fn set_severity_color(&mut self, severity: &str, color: Color);
    pub fn set_filter(&mut self, filter: &str);
}
```

**Rendering:** Lines with severity-based color prefix. Auto-scrolls to bottom when new lines arrive. Scrollbar for manual navigation. Optional word wrap.

#### 5.1.21 MenuBar (`menu_bar.rs`)

**Purpose:** Top menu bar with dropdown menus.

```rust
pub struct MenuBar {
    entries: Vec<MenuEntry>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    open_index: Option<usize>,                         // Open dropdown menu index
    hovered_entry: Option<usize>,
    hovered_item: Option<usize>,
    zone_registry: RefCell<ScopedZoneRegistry<String>>,
}

pub struct MenuEntry { pub label: String, pub items: Vec<MenuItem> }
pub struct MenuItem { pub label: String, pub action: Option<String>, pub shortcut: Option<String>, pub disabled: bool, pub separator: bool }
```

**API:**
```rust
impl MenuBar {
    pub fn new(entries: Vec<MenuEntry>) -> Self;
    pub fn on_action<F>(mut self, f: F) -> Self where F: FnMut(&str) + 'static;
}
```

**Rendering:** Top bar with menu entry labels. Active entry highlighted with `primary` color. Dropdown panel below active entry. Items with keyboard shortcut hints. Separator lines.

**Keyboard:** Left/Right: cycle menu entries. Enter/Down: open dropdown. Up/Down: navigate items. Enter: select item. Esc: close dropdown.

**Mouse:** Click entry: open/close dropdown. Click item: select. Hover entry: switch to that menu (if another open). Hover item: highlight.

#### 5.1.22 Modal (`modal.rs`)

**Purpose:** Generic modal dialog container with backdrop.

```rust
pub struct Modal<T: Clone> {
    content: Option<Box<dyn FnOnce(Rect) -> Plane>>,  // Content renderer
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    visible: bool,
    backdrop: bool,                                    // Show dark backdrop
    result: Option<ModalResult<T>>,
    close_on_backdrop: bool,                           // Click backdrop to dismiss
    zone_registry: RefCell<ScopedZoneRegistry<bool>>,
}

pub enum ModalResult<T> { Ok(T), Cancel }
```

**API:**
```rust
impl<T: Clone> Modal<T> {
    pub fn new() -> Self;
    pub fn show(&mut self);
    pub fn hide(&mut self);
    pub fn is_visible(&self) -> bool;
    pub fn result(&self) -> Option<ModalResult<T>>;
    pub fn with_backdrop(mut self, show: bool) -> Self;
    pub fn close_on_backdrop(mut self, close: bool) -> Self;
}
```

**Rendering:** Dark backdrop overlay (semi-transparent fill). Centered modal box with rounded border.

**Keyboard:** Esc: dismiss (Cancel). Enter: confirm (Ok).

**Mouse:** Click backdrop: dismiss (if enabled). Click modal content: handled by content.

#### 5.1.23 NotificationCenter (`notification_center.rs`)

**Purpose:** Queued notification display system with auto-dismiss.

```rust
pub struct NotificationCenter {
    notifications: VecDeque<Notification>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    max_visible: usize,
    auto_dismiss_duration: Duration,
    paused: bool,
}

pub struct Notification { pub title: String, pub message: String, pub kind: NotificationKind, pub timestamp: Instant }
pub enum NotificationKind { Info, Success, Warning, Error }
```

**API:**
```rust
impl NotificationCenter {
    pub fn new(max_visible: usize) -> Self;
    pub fn notify(&mut self, title: &str, message: &str, kind: NotificationKind);
    pub fn dismiss(&mut self, index: usize);
    pub fn dismiss_all(&mut self);
    pub fn pause(&mut self);
    pub fn resume(&mut self);
}
```

**Rendering:** Stack of notification cards (bottom-right or top-right). Each card: icon (ℹ✓⚠✗), title (bold), message, time ago. Kind-colored left border. Animated slide-in.

**Interactions:** Click: dismiss single notification. Auto-dismiss after `auto_dismiss_duration`.

#### 5.1.24 PasswordInput (`password_input.rs`)

**Purpose:** Password input with masking and show/hide toggle.

```rust
pub struct PasswordInput {
    inner: BaseInput,                                   // Reuses text_input_base
    masked: bool,
    mask_char: char,                                    // Default: '•'
}
```

**API:**
```rust
impl PasswordInput {
    pub fn new() -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn value(&self) -> &str;
    pub fn set_value(&mut self, value: &str);
    pub fn placeholder(&mut self, placeholder: &str);
    pub fn set_masked(&mut self, masked: bool);
    pub fn is_masked(&self) -> bool;
    pub fn on_submit<F>(mut self, f: F) -> Self where F: FnMut(&str) + 'static;
}
```

**Rendering:** Single-line input field. When masked: show `mask_char` for each character. Show/hide toggle icon (🔒/👁) on right. Focused: `focus_bg` + underline.

**Keyboard:** All text input keys. Enter: fire submit callback. Ctrl+A: select all.

**Mouse:** Click: focus + cursor position. Click toggle: show/hide password.

#### 5.1.25 Profiler (`profiler.rs`)

**Purpose:** Frame timing profiler with bar chart display.

```rust
pub struct Profiler {
    metrics: Vec<Metric>,
    theme: Theme,
    max_samples: usize,
}

pub struct Metric { pub name: String, pub values: Vec<f64> }
```

**API:**
```rust
impl Profiler {
    pub fn new(max_samples: usize) -> Self;
    pub fn record(&mut self, name: &str, value: f64);
    pub fn clear(&mut self);
}
```

**Rendering:** Bar chart showing recent values for each metric. Labels on left, bars proportional to value/max.

#### 5.1.26 ProgressBar (`progress_bar.rs`)

**Purpose:** Animated progress indicator.

```rust
pub struct ProgressBar {
    value: f64, max: f64,
    width: u16,
    show_percentage: bool,
    theme: Theme,
    animation: Option<Animation>,
}
```

**API:**
```rust
impl ProgressBar {
    pub fn new() -> Self;
    pub fn set_progress(&mut self, value: f64, max: f64);
    pub fn animate_to(&mut self, value: f64, duration: Duration);
}
```

**Rendering:** `[████████░░░░]` bar with optional percentage label.

#### 5.1.27 ProgressRing (`progress_ring.rs`)

**Purpose:** Circular progress indicator.

```rust
pub struct ProgressRing {
    value: f64, max: f64,
    size: u16,
    show_percentage: bool,
    theme: Theme,
}
```

**API:**
```rust
impl ProgressRing {
    pub fn new() -> Self;
    pub fn set_progress(&mut self, value: f64, max: f64);
    pub fn render(&self, area: Rect) -> Plane;
}
```

**Rendering:** Circular arc using braille characters (⠁⠃⠇⠏⠟⠿) for partial fill. Center shows percentage text.

#### 5.1.28 Radio (`radio.rs`)

**Purpose:** Radio button group for single selection.

```rust
pub struct Radio {
    options: Vec<String>,
    selected: usize,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    hovered: Option<usize>,
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}
```

**API:**
```rust
impl Radio {
    pub fn new(options: Vec<String>) -> Self;
    pub fn selected(&self) -> usize;
    pub fn set_selected(&mut self, index: usize);
    pub fn with_theme(mut self, theme: &Theme) -> Self;
}
```

**Rendering:** Vertical list of `(○) Label` options. Selected: `(●) Label` with primary color. Hovered: hover_bg.

**Keyboard:** Up/Down: navigate. Enter/Space: select.

**Mouse:** Click option: select. Hover: highlight.

#### 5.1.29 RichText (`rich_text.rs`)

**Purpose:** Rich text display with Markdown-like formatting.

```rust
pub struct RichText {
    content: String,
    theme: Theme,
    parsed: Vec<RichTextBlock>,
    scroll: ScrollState,
    wrap: bool,
}

enum RichTextBlock {
    Heading { level: u8, text: String },
    Paragraph { text: Vec<RichSpan> },
    Code { text: String, language: Option<String> },
    List { items: Vec<String>, ordered: bool },
    Blockquote { text: String },
    HorizontalRule,
    Link { text: String, url: String },
}

struct RichSpan { text: String, bold: bool, italic: bool, code: bool, link: Option<String> }
```

**API:**
```rust
impl RichText {
    pub fn new(content: &str) -> Self;
    pub fn from_markdown(md: &str) -> Self;
    pub fn scroll_to(&mut self, offset: usize);
}
```

**Rendering:** Multi-block layout: headings (bold, primary color, varying sizes), paragraphs with inline styling, code blocks (mono, surface_elevated bg, bordered), lists (bulleted/numbered), blockquotes (indented, italic), horizontal rules, links (underlined, info color).

#### 5.1.30 SearchInput (`search_input.rs`)

**Purpose:** Search input with clear button and optional live filtering.

```rust
pub struct SearchInput {
    inner: BaseInput,
    clear_button: bool,
    live_filter: bool,
    zone_registry: RefCell<ScopedZoneRegistry<bool>>,
}
```

**API:**
```rust
impl SearchInput {
    pub fn new() -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn value(&self) -> &str;
    pub fn set_value(&mut self, value: &str);
    pub fn placeholder(&mut self, placeholder: &str);
    pub fn on_change<F>(mut self, f: F) -> Self where F: FnMut(&str) + 'static;
    pub fn on_submit<F>(mut self, f: F) -> Self where F: FnMut(&str) + 'static;
}
```

**Rendering:** Input field with magnifying glass icon (🔍) on left. Clear button (✕) on right when non-empty. Focused: focus_bg + underline.

**Keyboard:** All text input keys. Enter: submit. Esc: clear and blur.

**Mouse:** Click: focus. Click clear: clear text.

#### 5.1.31 Select (`select.rs`)

**Purpose:** Dropdown select / combobox.

```rust
pub struct Select {
    options: Vec<String>,
    selected: usize,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    open: bool,
    focused: bool,
    hovered_index: Option<usize>,
    scroll_offset: usize,
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}
```

**API:**
```rust
impl Select {
    pub fn new(options: Vec<String>) -> Self;
    pub fn selected(&self) -> usize;
    pub fn set_selected(&mut self, index: usize);
    pub fn with_theme(mut self, theme: &Theme) -> Self;
}
```

**Rendering:** Current value display with dropdown arrow (▼). Open: dropdown list below with options. Selected highlighted. Hovered highlighted.

**Keyboard:** Up/Down: navigate options. Enter: select and close. Esc: close without selection.

**Mouse:** Click: open/close dropdown. Click option: select. Hover: highlight. Scroll: scroll options.

#### 5.1.32 Slider (`slider.rs`)

**Purpose:** Horizontal slider for numeric value selection.

```rust
pub struct Slider {
    min: u16, max: u16, value: u16, step: u16,
    width: u16,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    focused: bool,
    show_value: bool,
    zone_registry: RefCell<ScopedZoneRegistry<bool>>,
}
```

**API:**
```rust
impl Slider {
    pub fn new(min: u16, max: u16) -> Self;
    pub fn value(&self) -> u16;
    pub fn set_value(&mut self, value: u16);
    pub fn step(mut self, step: u16) -> Self;
    pub fn show_value(mut self, show: bool) -> Self;
}
```

**Rendering:** `─────────●──────────` track with thumb. Filled portion in primary color. Value label (optional). Focus indicator when focused.

**Keyboard:** Left/Right: decrease/increase by step. Home/End: min/max.

**Mouse:** Click track: jump to position. Drag thumb: continuous adjustment.

#### 5.1.33 Sparkline (`sparkline.rs`)

**Purpose:** Mini inline chart for trending data.

```rust
pub struct Sparkline {
    data: Vec<f64>,
    width: u16, height: u16,
    min: f64, max: f64,
    color: Color,
    theme: Theme,
}
```

**API:**
```rust
impl Sparkline {
    pub fn new(data: Vec<f64>) -> Self;
    pub fn set_data(&mut self, data: Vec<f64>);
    pub fn set_bounds(&mut self, min: f64, max: f64);
    pub fn set_color(&mut self, color: Color);
}
```

**Rendering:** Uses braille characters (⣀⣤⣶⣿) for vertical resolution. Each column represents one data point. Scaled to fit within width/height.

#### 5.1.34 Spinner (`spinner.rs`)

**Purpose:** Animated loading spinner.

```rust
pub struct Spinner {
    frames: Vec<&'static str>,                         // Default: ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
    frame_index: usize,
    label: Option<String>,
    theme: Theme,
    animation: Animation,
}
```

**API:**
```rust
impl Spinner {
    pub fn new() -> Self;
    pub fn set_frames(&mut self, frames: Vec<&'static str>);
    pub fn set_label(&mut self, label: &str);
    pub fn tick(&mut self);
}
```

**Rendering:** Single character spinner (animated by cycling frame at interval). Optional label to right.

#### 5.1.35 SplitPane (`split.rs`)

**Purpose:** Resizable split panel container.

```rust
pub struct SplitPane {
    orientation: Orientation,
    ratio: f32,                                        // 0.0-1.0, portion for first pane
    min_size: u16,
    is_drag: bool,
    theme: Theme,
}

pub enum Orientation { Horizontal, Vertical }
```

**API:**
```rust
impl SplitPane {
    pub fn new(orientation: Orientation) -> Self;
    pub fn ratio(mut self, ratio: f32) -> Self;
    pub fn min_size(mut self, size: u16) -> Self;
    pub fn split(&self, area: Rect) -> (Rect, Rect);   // Compute child rectangles
    pub fn from_rect(area: Rect) -> Self;               // Create pane representing a sub-rect
    pub fn divider(&self, area: Rect) -> Rect;          // Get divider rectangle
    pub fn set_ratio(&mut self, ratio: f32);
    pub fn ratio_value(&self) -> f32;
    pub fn handle_mouse_drag(&mut self, col: u16, row: u16);
    pub fn orientation(&self) -> Orientation;
    pub fn on_theme_change(&mut self, theme: &Theme);
}
```

**Rendering:** Two side-by-side (or stacked) rectangles with a divider between them. Divider: `│` (vertical) or `─` (horizontal) in `theme.divider`.

**Keyboard:** None directly (used as container).

**Mouse:** Drag divider: resize panes interactively. Divider hit zone: 2 cells wide/tall.

#### 5.1.36 StatusBadge (`status_badge.rs`)

**Purpose:** Colored status indicator (OK, WARN, ERROR, etc.).

```rust
pub struct StatusBadge {
    label: String,
    status: String,                                     // 'ok', 'warn', 'error', or custom
    theme: Theme,
}

impl StatusBadge {
    pub fn new(label: &str) -> Self;
    pub fn set_status(&mut self, status: &str);
    pub fn bind_command(&mut self, cmd: BoundCommand);
}
```

**Rendering:** `[ OK ]` or `[WARN]` or `[ERROR]`. Color mapped: ok→success, warn→warning, error→error, other→primary. Compact bounding box.

#### 5.1.37 StatusBar (`status_bar.rs`)

**Purpose:** Bottom status bar with labeled segments.

```rust
pub struct StatusBar {
    segments: Vec<StatusSegment>,
    theme: Theme,
    mode: String,
}

pub struct StatusSegment {
    pub label: String, pub value: String, pub width: Option<u16>,
}
impl StatusSegment { pub fn new(text: &str) -> Self; }
```

**API:**
```rust
impl StatusBar {
    pub fn new(segments: Vec<StatusSegment>) -> Self;
    pub fn set_segment(&mut self, index: usize, value: &str);
    pub fn set_mode(&mut self, mode: &str);
}
```

**Rendering:** Full-width bottom bar. Segments distributed across width, right-aligned. Uses `Color::Reset` for default fg/bg to inherit terminal defaults.

#### 5.1.38 StreamingText (`streaming_text.rs`)

**Purpose:** Live-updating text display with word-wrap.

```rust
pub struct StreamingText {
    lines: Vec<String>,
    theme: Theme,
    scroll: ScrollState,
    wrap: bool,
    max_lines: usize,
}
```

**API:**
```rust
impl StreamingText {
    pub fn new() -> Self;
    pub fn append(&mut self, text: &str);
    pub fn clear(&mut self);
    pub fn set_wrap(&mut self, wrap: bool);
}
```

**Rendering:** Text lines with word-wrap. Scrollbar if content overflows. Auto-scroll to bottom on new content.

#### 5.1.39 TabBar (`tabbar.rs`)

**Purpose:** Tab navigation bar for panel/content switching.

```rust
pub struct TabBar {
    tabs: Vec<String>,
    selected: usize,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    hovered_tab: Option<usize>,
    closable: bool,
    on_close: Option<Box<dyn FnMut(usize)>>,
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}
```

**API:**
```rust
impl TabBar {
    pub fn new(tabs: Vec<String>) -> Self;
    pub fn select(&mut self, index: usize);
    pub fn selected(&self) -> usize;
    pub fn add_tab(&mut self, label: &str);
    pub fn remove_tab(&mut self, index: usize);
    pub fn on_close<F>(mut self, f: F) -> Self where F: FnMut(usize) + 'static;
}
```

**Rendering:** Tab labels with active tab highlighted (primary bg, bold). Inactive tabs: dimmed. Close button (✕) on each tab if closable. Active tab connected to content area (no bottom border).

**Keyboard:** Left/Right: cycle tabs. Ctrl+W: close tab (if closable).

**Mouse:** Click tab: select. Click close: remove tab. Hover: highlight with hover_bg.

#### 5.1.40 Table (`table.rs`)

**Total LOC:** 626

**Purpose:** Multi-column sortable data table with selection, sorting, and drag support.

```rust
pub struct Table<T> {
    id: WidgetId,
    columns: Vec<Column>,
    rows: Vec<TableRow<T>>,
    selected: usize,
    scroll: ScrollState,
    theme: Theme,
    dirty: bool,
    sort_column: Option<usize>,
    sort_ascending: bool,
    hovered_row: Option<usize>,
    multi_select: bool,
    selected_indices: HashSet<usize>,
    zone_registry: RefCell<ScopedZoneRegistry<String>>,
    // Callbacks
    on_select: Option<SelectCallback<T>>,
    cell_text_fn: Option<CellTextFn<T>>,
    on_header_click: Option<HeaderClickCallback>,
    on_selection_change: Option<SelectionChangeCallback>,
    // Drag
    drag_manager: Option<Box<DragManager<usize>>>,
    // Undo/redo
    undo_stack: Vec<TableState>,
    redo_stack: Vec<TableState>,
    max_undo: usize,
}

pub struct Column { pub header: String, pub width: u16 }
pub struct TableRow<T> { pub data: T }
pub struct TableState {
    pub selected: usize, pub offset: usize,
    pub sort_column: Option<usize>, pub sort_ascending: bool,
    pub selected_indices: HashSet<usize>,
}

// Callback type aliases
pub type SelectCallback<T> = Box<dyn FnMut(&T)>;
pub type CellTextFn<T> = Box<dyn Fn(&T, usize) -> String>;
pub type HeaderClickCallback = Box<dyn FnMut(usize)>;
pub type SelectionChangeCallback = Box<dyn FnMut(&HashSet<usize>)>;
pub type UndoRedoCallback = Box<dyn FnMut()>;
```

**API:**
```rust
impl<T: Clone + 'static> Table<T> {
    pub fn new(rows: Vec<T>) -> Self;
    pub fn with_columns(mut self, columns: Vec<Column>) -> Self;
    pub fn on_select<F>(mut self, f: F) -> Self where F: FnMut(&T) + 'static;
    pub fn with_cell_text_fn<F>(mut self, f: F) -> Self where F: Fn(&T, usize) -> String + 'static;
    pub fn on_header_click<F>(mut self, f: F) -> Self where F: FnMut(usize) + 'static;
    pub fn set_sort(&mut self, column: Option<usize>, ascending: bool);
    pub fn selected_index(&self) -> usize;
    pub fn set_selected(&mut self, index: usize);
    pub fn get_selected(&self) -> Option<&T>;
    pub fn scroll_state(&self) -> ScrollState;
    pub fn len(&self) -> usize;
    // Multi-select
    pub fn set_multi_select(&mut self, multi: bool);
    pub fn selected_indices(&self) -> &HashSet<usize>;
    pub fn clear_selection(&mut self);
    // Undo/redo
    pub fn save_state(&mut self);
    pub fn undo(&mut self);
    pub fn redo(&mut self);
}
```

**Rendering:** Header row (bold, sorted column has ▲/▼ indicator, clickable). Data rows with alternating row colors or selection highlighting. Selected row: selection_bg. Hovered row: hover_bg. Scrollbar on right. Sort indicators: primary color for active sort column.

**Keyboard:** Up/Down: navigate rows. Home/End: first/last row. PageUp/PageDown: page scroll. Enter: fire on_select. Ctrl+A: select all (multi-select).

**Mouse:** Click header: toggle sort (cycles none → asc → desc). Click row: select. Ctrl+click: toggle in multi-select. Shift+click: range select. Drag: marquee selection (if enabled). Hover: highlight row.

#### 5.1.41 TagsInput (`tags_input.rs`)

**Purpose:** Tag input with autocomplete and remove.

```rust
pub struct TagsInput {
    tags: Vec<String>,
    input: String,
    suggestions: Vec<String>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    focused: bool,
    hovered_tag: Option<usize>,
    zone_registry: RefCell<ScopedZoneRegistry<usize>>,
}
```

**API:**
```rust
impl TagsInput {
    pub fn new() -> Self;
    pub fn tags(&self) -> &[String];
    pub fn add_tag(&mut self, tag: &str);
    pub fn remove_tag(&mut self, index: usize);
    pub fn set_suggestions(&mut self, suggestions: Vec<String>);
}
```

**Rendering:** Input field with tags as colored pills. Each pill: label with ✕ remove button. Input area after last tag. Dropdown with autocomplete suggestions.

**Keyboard:** Enter: add tag from input. Backspace: remove last tag (when input empty). Remove hovered tag on delete key.

**Mouse:** Click ✕ on tag: remove. Click suggestion: add. Hover: highlight tag.

#### 5.1.42 TextEditorAdapter (`text_editor_adapter.rs`)

**Purpose:** Framework adapter wrapping the standalone TextEditor widget.

```rust
pub struct TextEditorAdapter {
    editor: RefCell<TextEditor>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
}
```

**API:**
```rust
impl TextEditorAdapter {
    pub fn new() -> Self;
    pub fn open(&self, path: &Path);
    pub fn set_content(&self, content: &str);
    pub fn get_content(&self) -> String;
}
```

Delegates to `TextEditor` (standalone widget) for all editing functionality.

#### 5.1.43 Toast (`toast.rs`)

**Purpose:** Temporary notification popup with auto-dismiss.

```rust
pub struct Toast {
    message: String,
    kind: ToastKind,
    duration: Duration,
    theme: Theme,
    created: Instant,
    dismissed: bool,
    animation: Animation,
}

pub enum ToastKind { Info, Success, Warning, Error }
```

**API:**
```rust
impl Toast {
    pub fn new(message: &str, kind: ToastKind) -> Self;
    pub fn with_duration(mut self, duration: Duration) -> Self;
    pub fn with_theme(mut self, theme: &Theme) -> Self;
    pub fn is_expired(&self) -> bool;
    pub fn dismiss(&mut self);
}
```

**Rendering:** Small popup box (typically bottom-right). Icon prefix (ℹ✓⚠✗). Kind-colored left border. Fade-in animation on creation. Auto-dismiss after duration.

#### 5.1.44 Toggle (`toggle.rs`)

**Purpose:** Two-state on/off toggle switch.

```rust
pub struct Toggle {
    label: String,
    on: bool,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    hovered: bool,
    zone_registry: RefCell<ScopedZoneRegistry<bool>>,
}
```

**API:**
```rust
impl Toggle {
    pub fn new(label: &str) -> Self;
    pub fn is_on(&self) -> bool;
    pub fn set_on(&mut self, on: bool);
    pub fn toggle(&mut self);
}
```

**Rendering:** `[●───] Label` or `[───○] Label`. On: filled track (primary) with dot at right. Off: empty track (outline) with dot at left. Hovered: hover_bg.

**Mouse:** Click: toggle.

#### 5.1.45 Tooltip (`tooltip.rs`)

**Purpose:** Hover-activated tooltip popup.

```rust
pub struct Tooltip {
    text: String,
    theme: Theme,
    visible: bool,
    position: (u16, u16),
    width: u16,
}
```

**API:**
```rust
impl Tooltip {
    pub fn new(text: &str) -> Self;
    pub fn show(&mut self, col: u16, row: u16);
    pub fn hide(&mut self);
}
```

**Rendering:** Small popup box at cursor position. Rounded border, surface_elevated bg.

#### 5.1.46 Tree (`tree.rs`)

**Purpose:** Collapsible tree view with expand/collapse and selection.

```rust
pub struct Tree {
    nodes: Vec<TreeNode>,
    expanded: HashSet<String>,
    selected: Option<String>,
    theme: Theme,
    id: WidgetId,
    area: Rect,
    dirty: bool,
    scroll: ScrollState,
    hovered_path: Option<String>,
    zone_registry: RefCell<ScopedZoneRegistry<String>>,
}

pub struct TreeNode { pub id: String, pub label: String, pub children: Vec<TreeNode>, pub icon: Option<String> }
```

**API:**
```rust
impl Tree {
    pub fn new(children: Vec<TreeNode>) -> Self;
    pub fn select(&mut self, id: &str);
    pub fn selected(&self) -> Option<&str>;
    pub fn expand(&mut self, id: &str);
    pub fn collapse(&mut self, id: &str);
    pub fn toggle(&mut self, id: &str);
    pub fn is_expanded(&self, id: &str) -> bool;
    pub fn flatten(&self) -> Vec<(&TreeNode, usize)>;   // Flat list with depth
}
```

**Rendering:** Hierarchical indentation with tree connectors (├─└─│). Expand/collapse toggle (▶▼) before each expandable node. Selected node: selection_bg. Hovered node: hover_bg. Scrollbar on right.

**Keyboard:** Up/Down: navigate nodes. Left: collapse. Right: expand. Enter: select.

**Mouse:** Click toggle: expand/collapse. Click label: select. Hover: highlight. Scroll: scroll tree.

#### 5.1.47 WidgetInspector (`widget_inspector.rs`)

**Purpose:** Widget tree debugging inspector.

```rust
pub struct WidgetInspector {
    widgets: Vec<WidgetNode>,
    selected: usize,
    theme: Theme,
    dirty: bool,
    scroll: ScrollState,
}

pub struct WidgetNode { pub name: String, pub id: WidgetId, pub area: Rect, pub z_index: u16, pub children: Vec<WidgetNode> }
```

**API:**
```rust
impl WidgetInspector {
    pub fn new(widgets: Vec<WidgetNode>) -> Self;
    pub fn set_widgets(&mut self, widgets: Vec<WidgetNode>);
}
```

**Rendering:** Hierarchical tree of widget nodes showing name, ID, area, z-index. Selected node highlighted.

### 5.2 BaseInput (Shared Input Widget)

**File:** `text_input_base.rs`

Shared base for `SearchInput`, `PasswordInput`, `Autocomplete`.

```rust
pub struct BaseInput {
    value: String,
    cursor_pos: usize,
    focused: bool,
    theme: Theme,
    placeholder: String,
    on_change: Option<Box<dyn FnMut(&str)>>,
    on_submit: Option<SubmitCallback>,
    selection_start: Option<usize>,
}

pub type SubmitCallback = Box<dyn FnMut(&str)>;
```

**Common behavior:**
- Cursor: blinking block at text position
- Selection: highlighted region (if `selection_start` set)
- Focus: `focus_bg` background, underline
- Placeholder: shown when empty and unfocused, `fg_subtle`
- Keys: Char (insert), Backspace (delete left), Delete (delete right), Left/Right (move cursor), Home/End, Ctrl+A (select all), Enter (submit)

### 5.3 Common Widget Patterns

#### Common Widget State Fields

```rust
// Every widget includes:
pub struct TypicalWidget {
    theme: Theme,           // Visual theme
    id: WidgetId,           // Unique identifier
    area: Rect,             // Screen area
    dirty: bool,            // Needs re-render flag
    // Optional:
    focused: bool,          // Has keyboard focus
    hovered: ...,           // Mouse hover state
    zone_registry: RefCell<ScopedZoneRegistry<...>>,  // Mouse dispatch
}
```

#### Common Widget Methods Pattern

```rust
impl Widget for TypicalWidget {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn z_index(&self) -> u16 { 10 }  // Interactive widgets at z=10

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }

    fn focusable(&self) -> bool { true }
    fn on_focus(&mut self) { self.focused = true; self.dirty = true; }
    fn on_blur(&mut self) { self.focused = false; self.dirty = true; }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }
}
```

---

## 6. Theme System

### 6.1 Theme Struct Fields (31 fields, 21 themes)

```rust
pub struct Theme {
    // Identification
    pub name: Arc<str>,                                    // Programmatic name: "nord", "dracula"
    pub display_name: Arc<str>,                            // Human label: "Nord", "Rosé Pine"
    pub kind: ThemeKind,                                   // Dark | Light

    // Surface / Elevation (3)
    pub bg: Color,                                         // Root background
    pub surface: Color,                                    // Panel/card surface
    pub surface_elevated: Color,                           // Dropdowns, dialogs

    // Text Hierarchy (4)
    pub fg: Color,                                         // Primary text
    pub fg_muted: Color,                                   // Secondary text
    pub fg_subtle: Color,                                  // Tertiary text
    pub fg_on_accent: Color,                               // Text on accent bg

    // Interactive (6)
    pub primary: Color, pub primary_hover: Color, pub primary_active: Color,
    pub secondary: Color, pub secondary_hover: Color, pub secondary_active: Color,

    // Borders (3)
    pub outline: Color, pub outline_variant: Color, pub divider: Color,

    // Semantic (8)
    pub error: Color, pub error_bg: Color,
    pub success: Color, pub success_bg: Color,
    pub warning: Color, pub warning_bg: Color,
    pub info: Color, pub info_bg: Color,

    // Selection (2)
    pub selection_bg: Color, pub selection_fg: Color,

    // Input (3)
    pub input_bg: Color, pub input_fg: Color, pub input_border: Color,

    // Scrollbar (3)
    pub scrollbar_track: Color, pub scrollbar_thumb: Color, pub scrollbar_thumb_hover: Color,

    // Disabled (2)
    pub disabled_fg: Color, pub disabled_bg: Color,

    // Focus/Hover (3)
    pub hover_bg: Color, pub focus_bg: Color, pub focus_border: Color,

    // Deprecated (1)
    #[deprecated(since = "0.3.0", note = "Use framework::scroll::DEFAULT_SCROLLBAR_WIDTH instead")]
    pub scrollbar_width: u16,
}
```

### 6.2 Theme Constructors (21 Complete)

Each constructor sets all 31 fields. Example for the `dark` theme:

```rust
pub fn dark() -> Self {
    Self {
        name: "dark".into(),
        display_name: "Dark".into(),
        kind: ThemeKind::Dark,
        bg: Color::Rgb(16, 16, 24),
        surface: Color::Rgb(24, 24, 36),
        surface_elevated: Color::Rgb(32, 32, 48),
        fg: Color::Rgb(200, 200, 220),
        fg_muted: Color::Rgb(140, 140, 160),
        fg_subtle: Color::Rgb(100, 100, 120),
        fg_on_accent: Color::Rgb(0, 0, 0),
        primary: Color::Rgb(0, 200, 120),
        primary_hover: Color::Rgb(0, 220, 140),
        primary_active: Color::Rgb(0, 180, 100),
        secondary: Color::Rgb(100, 150, 200),
        secondary_hover: Color::Rgb(120, 170, 220),
        secondary_active: Color::Rgb(80, 130, 180),
        outline: Color::Rgb(60, 60, 80),
        outline_variant: Color::Rgb(45, 45, 65),
        divider: Color::Rgb(50, 50, 70),
        error: Color::Rgb(255, 80, 80),
        error_bg: Color::Rgb(50, 20, 20),
        success: Color::Rgb(80, 255, 120),
        success_bg: Color::Rgb(20, 50, 30),
        warning: Color::Rgb(255, 180, 80),
        warning_bg: Color::Rgb(50, 40, 20),
        info: Color::Rgb(100, 180, 255),
        info_bg: Color::Rgb(20, 40, 60),
        selection_bg: Color::Rgb(50, 80, 60),
        selection_fg: Color::Rgb(200, 255, 220),
        input_bg: Color::Rgb(20, 20, 30),
        input_fg: Color::Rgb(220, 220, 240),
        input_border: Color::Rgb(60, 60, 80),
        scrollbar_track: Color::Rgb(30, 30, 40),
        scrollbar_thumb: Color::Rgb(80, 80, 100),
        scrollbar_thumb_hover: Color::Rgb(100, 100, 120),
        disabled_fg: Color::Rgb(80, 80, 100),
        disabled_bg: Color::Rgb(35, 35, 50),
        hover_bg: Color::Rgb(40, 40, 56),
        focus_bg: Color::Rgb(50, 50, 70),
        focus_border: Color::Rgb(0, 200, 120),
        scrollbar_width: Self::default_scrollbar_width(),
    }
}
```

### 6.3 Theme from_name Resolution

```rust
// Supports:
"dark" → Theme::dark()
"light" → Theme::light()
"high_contrast" → Theme::high_contrast()
"cyberpunk" → Theme::cyberpunk()
"dracula" → Theme::dracula()
"nord" → Theme::nord()
"catppuccin_mocha" | "catppuccin" → Theme::catppuccin_mocha()
"gruvbox_dark" | "gruvbox" → Theme::gruvbox_dark()
"tokyo_night" → Theme::tokyo_night()
"solarized_dark" → Theme::solarized_dark()
"solarized_light" → Theme::solarized_light()
"one_dark" → Theme::one_dark()
"rose_pine" → Theme::rose_pine()
"kanagawa" → Theme::kanagawa()
"everforest" → Theme::everforest()
"monokai" → Theme::monokai()
"warm" → Theme::warm()
"cool" → Theme::cool()
"forest" → Theme::forest()
"sunset" → Theme::sunset()
"mono" → Theme::mono()
```

Normalization: input is lowercased, hyphens replaced with underscores before matching.

### 6.4 Theme from_env_or Implementation

```rust
pub fn from_env_or(default: Theme) -> Theme {
    std::env::var("DTRON_THEME")
        .ok()
        .and_then(|name| {
            if name.is_empty() { None }
            else { Theme::from_name(&name) }
        })
        .unwrap_or(default)
}
```

---

## 7. Input System

### 7.1 Parser State Machine

```
Ground ────────────────────────────────────────────────────── Ground (mostly printable chars)
  │ ESC (\x1b) ─────────────────────────────────────────→ Escape
  │                                                      │ '[' ──────────────────────────→ CsiEntry
  │                                                      │ ']' ──────────────────────────→ OscString
  │                                                      │ 'P' ──────────────────────────→ DcsEntry
  │                                                      │ 'X' ──────────────────────────→ SosString
  │                                                      │ '^' ──────────────────────────→ PmString
  │                                                      │ '_' ──────────────────────────→ ApcString
  │ 0x7F (DEL) ─────────────────────────────────────────→ Key(Backspace)

Escape
  │ non-CSI sequence (O, o, etc.) → parse single-char escape → Ground
  │ '[' → CsiEntry
  │ ']' → OscString

CsiEntry
  │ parameter bytes (0x30-0x3F) → accumulate params → CsiParam
  │ intermediate bytes (0x20-0x2F) → CsiIntermediate
  │ final byte (0x40-0x7E) → match sequence → Ground

CsiParam
  │ parameter bytes → continue accumulating
  │ intermediate bytes → CsiIntermediate
  │ final byte → match sequence → Ground

OscString
  │ accumulate until OSC string terminator (ST: \x1b\ or BEL: \x07) → match → Ground

Each final byte dispatch:
  M/m → Mouse event (SGR format: <btn;col;row{M|m})
  A,B,C,D → Cursor keys with modifiers
  H,F → Home/End with modifiers
  ~ → Extended key (F-keys, Insert, Delete, PageUp/Down, etc.)
  u → Kitty keyboard protocol
  I → Focus gained
  O → Focus lost
  R → Cursor position report (used as resize detection)
  Z → BackTab
```

### 7.2 SGR Mouse Parsing Detail

Format: `\x1b[<{btn};{col};{row}{M|m}`

- `<` prefix identifies SGR-encoded mouse
- `btn` encodes button + modifiers + event type:
  - Bits 0-1: button (0=left, 1=middle, 2=right, 3=release, 64=scroll)
  - Bit 2: modifier (shift)
  - Bit 3: modifier (alt)
  - Bit 4: modifier (ctrl)
  - Bit 5: motion flag (drag)
  - Bit 6: scroll/extra button
- `M` = button press/drag, `m` = button release
- `col`, `row`: 1-based cell coordinates

### 7.3 Modifier Bitmask Encoding (CSI 1;N sequences)

```
Param N = 1 + modifier bits:
  1 = none
  2 = SHIFT
  3 = ALT
  4 = ALT + SHIFT
  5 = CTRL
  6 = CTRL + SHIFT
  7 = CTRL + ALT
  8 = CTRL + ALT + SHIFT
```

### 7.4 Kitty Keyboard Protocol

Format: `\x1b[{code};{modifiers}u`

- `code`: Unicode code point of the key (e.g., 97 for 'a', 13 for Enter, 27 for Esc)
- `modifiers`: bitmask (1=shift, 2=alt, 4=ctrl, 8=super, 16=hyper, 32=meta)
- Supports press/repeat/release distinction via protocol flags

---

## 8. Compositor & Rendering

### 8.1 Cell Struct

```rust
pub struct Cell {
    pub char: char,           // Display character
    pub fg: Color,            // Foreground color
    pub bg: Color,            // Background color
    pub style: Styles,        // Text style bitflags
    pub transparent: bool,    // If true, cell below shows through
    pub skip: bool,           // If true, renderer skips this cell (for wide char padding)
}

impl Default for Cell {
    fn default() -> Self {
        Self { char: ' ', fg: Color::Reset, bg: Color::Reset, style: Styles::empty(), transparent: true, skip: false }
    }
}
```

### 8.2 Compositor::render() — Detailed Algorithm

```
fn render<W: Write>(&mut self, writer: &mut W) -> io::Result<()>:

INPUTS:
  - self.planes: Vec<Plane>  (all planes added this frame)
  - self.last_frame: Vec<Cell>  (previous frame's final output)
  - self.final_buffer: Vec<Cell>  (working buffer, pre-allocated W×H)
  - self.clear_color: Color
  - self.dirty_regions: DirtyRegionTracker

ALGORITHM:
  A. PREPARE CLEAR CELL
     clear_cell = Cell { char: ' ', fg: Reset, bg: self.clear_color, transparent: false, skip: false }

  B. DETERMINE MODE
     full_refresh = self.dirty_regions.needs_full_refresh()
     regions = self.dirty_regions.dirty_regions()

  C. CLEAR FINAL BUFFER
     if full_refresh || regions.is_empty():
       for cell in final_buffer.iter_mut(): *cell = clear_cell
     else:
       for each dirty region (region.x, region.y, region.w, region.h):
         for y in region.y..min(region.y+region.h, height):
           for x in region.x..min(region.x+region.w, width):
             idx = y * width + x
             final_buffer[idx] = clear_cell

  D. SORT PLANES
     self.planes.sort_by_key(|a| a.z_index)  // Ascending z-index

  E. COMPOSITE PLANES (painters algorithm)
     for plane in planes:
       if !plane.visible: continue

       // Clip plane to compositor bounds
       px_end = min(plane.width, width - plane.x)
       py_end = min(plane.height, height - plane.y)
       plane_stride = plane.width
       dest_stride = width

       if full_refresh:
         // Full composite: visit all cells
         for py in 0..py_end:
           for px in 0..px_end:
             src_idx = py * plane_stride + px
             dest_idx = (plane.y + py) * dest_stride + (plane.x + px)
             src_cell = plane.cells[src_idx]
             
             if plane.filter: filter.apply(&mut src_cell, position, time)
             
             blend_cells(&mut final_buffer[dest_idx], &src_cell, plane.opacity)
       else:
         // Partial composite: only cells in dirty regions
         for py in 0..py_end:
           for px in 0..px_end:
             abs_x = plane.x + px
             abs_y = plane.y + py
             
             // Skip if not in any dirty region
             if !any_region_contains(abs_x, abs_y): continue
             
             src_idx = py * plane_stride + px
             dest_idx = abs_y * dest_stride + abs_x
             blend_cells(&mut final_buffer[dest_idx], &plane.cells[src_idx], plane.opacity)

  F. DIFF AND OUTPUT
     Initialize:
       buf = Vec<u8> (capacity: W × H × 20)
       current_fg = Reset, current_bg = Reset, current_style = empty

     // Sync mode begin
     buf.extend(b"\x1b[?2026h")
     // Disable wraparound
     buf.extend(b"\x1b[?7l")

     for y in 0..height:
       line_cursor_moved = false
       for x in 0..width:
         idx = y * width + x
         cell = final_buffer[idx]
         last = last_frame[idx]

         // Skip padding cells
         if cell.skip: continue

         // Skip unchanged cells
         if cell == last: continue

         // Skip non-dirty cells in partial mode
         if !full_refresh && !in_any_region(x, y): continue

         // Emit cursor position if needed
         if !line_cursor_moved:
           buf.push(b'\x1b'); buf.push(b'[')
           write_u16_decimal(&mut buf, y + 1)
           buf.push(b';')
           write_u16_decimal(&mut buf, x + 1)
           buf.push(b'H')
           line_cursor_moved = true

         // Emit style changes only when changed
         if cell.style != current_style:
           emit_style_changes(&mut buf, cell.style, current_style)
           current_style = cell.style

         // Emit foreground color if changed
         if cell.fg != current_fg:
           emit_fg_color(&mut buf, cell.fg)
           current_fg = cell.fg

         // Emit background color if changed
         if cell.bg != current_bg:
           emit_bg_color(&mut buf, cell.bg)
           current_bg = cell.bg

         // Write character (fast path: ASCII, else UTF-8)
         if char is printable ASCII (0x20-0x7E):
           buf.push(char as u8)
         else if char is control (0x00-0x1F):
           buf.push(b' ')  // Replace with space
         else:
           char.encode_utf8(&mut utf8_buf)
           buf.extend_from_slice(utf8_buf)

     // Re-enable wraparound + end sync mode
     buf.extend(b"\x1b[?7h")
     buf.extend(b"\x1b[?2026l")

  G. FLUSH
     writer.write_all(&buf)?;
     self.last_frame.copy_from_slice(&self.final_buffer)
     self.planes.clear()
     self.dirty_regions.clear()
     writer.flush()?
```

### 8.3 Blend Algorithm

```rust
fn blend_cells(dest: &mut Cell, src: &Cell, alpha: f32):
  if src.transparent || alpha <= 0: return

  if alpha >= 1.0:
    // Opaque: direct copy
    if src.bg != Reset: dest.bg = src.bg
    if src.skip:
      dest.skip = true; dest.char = ' '
    else:
      if is_braille(dest.char) && is_braille(src.char):
        dest.char = merge_braille(dest.char, src.char)  // Bitwise OR for braille overlay
      else:
        dest.char = src.char
      dest.fg = src.fg
      dest.style = src.style
      dest.skip = false
  else:
    // Transparent: alpha-blend RGB colors
    dest.bg = blend_rgb(dest.bg, src.bg, alpha)
    if !src.skip && src.char != '\0':
      dest.fg = blend_rgb(dest.fg, src.fg, alpha)
      if alpha > 0.5:
        dest.char = src.char
        dest.style = src.style

  dest.transparent = false
```

### 8.4 Color Escape Sequence Generation

```rust
// Foreground
Reset → \x1b[39m
Ansi(N) → \x1b[38;5;{N}m     // 0 ≤ N ≤ 255, 3-digit zero-padded
Rgb(r,g,b) → \x1b[38;2;{r};{g};{b}m  // 3-digit zero-padded decimal

// Background
Reset → \x1b[49m
Ansi(N) → \x1b[48;5;{N}m
Rgb(r,g,b) → \x1b[48;2;{r};{g};{b}m

// Style
BOLD on → \x1b[1m, BOLD off → \x1b[22m
ITALIC on → \x1b[3m, ITALIC off → \x1b[23m
UNDERLINE on → \x1b[4m, UNDERLINE off → \x1b[24m
DIM on → \x1b[2m, DIM off → \x1b[22m
REVERSE on → \x1b[7m, REVERSE off → \x1b[27m
BLINK on → \x1b[5m, BLINK off → \x1b[25m
HIDDEN on → \x1b[8m, HIDDEN off → \x1b[28m
STRIKETHROUGH on → \x1b[9m, STRIKETHROUGH off → \x1b[29m
```

### 8.5 Input Shield (from app.rs)

**Purpose:** Swallow all keyboard and mouse input for a configurable cooldown period after mode transitions.

**Implementation:**
```rust
pub fn shield_input(&self, duration: Duration) {
    self.input_shield_until.set(Some(Instant::now() + duration));
}

// At top of handle_event:
if let Some(until) = self.input_shield_until.get() {
    if Instant::now() < until {
        return;  // Event silently swallowed
    }
    self.input_shield_until.set(None);  // Shield expired
}
```

Resize events are NOT shielded (they must always be processed).

---

## 9. Command-Driven Architecture

### 9.1 Complete OutputParser Enum

```rust
pub enum OutputParser {
    JsonKey { key: String },                              // Extract single JSON field
    JsonPath { path: String },                            // Navigate JSON with dot-path
    JsonArray { item_key: Option<String> },               // Extract items from JSON array
    Regex { pattern: String, group: Option<usize> },      // Regex capture group extraction
    LineCount,                                            // Count output lines
    ExitCode,                                             // Map exit code to value
    SeverityLine { patterns: HashMap<String, String> },   // Log severity detection
    #[default] Plain,                                     // Raw text
}
```

### 9.2 Complete ParsedOutput Enum

```rust
pub enum ParsedOutput {
    Scalar(String),              // Single value → Gauge value, StatusBadge status
    List(Vec<String>),           // Multiple items → Table rows, List items
    Lines(Vec<LoggedLine>),      // Log lines with severity → LogViewer
    Text(String),                // Raw text → StreamingText, KeyValueGrid
    None,                        // No output (parse failure, empty command)
}

pub struct LoggedLine {
    pub text: String,
    pub severity: String,        // "red", "yellow", "default", etc.
}
```

### 9.3 CommandRunner Implementation Detail

```rust
pub fn run_sync(&self) -> (String, String, i32) {
    // Split command into program + args by whitespace
    // Spawn child process with piped stdout/stderr
    // Wait for completion
    // Return (stdout_string, stderr_string, exit_code)
}

pub fn spawn(&mut self) -> io::Result<()> {
    // Spawn child with piped stdout/stderr
    // Create mpsc channels for stdout/stderr lines
    // Spawn reader threads that send lines through channels
    // Store child_id, stdout_rx, stderr_rx
}

pub fn recv_line(&self) -> Option<String> {
    // Try to receive a line from stdout channel
    // Lines prefixed with __EXIT_CODE__ indicate process exit
}
```

### 9.4 TOML Configuration Schema

```toml
title = "Dashboard"                    # Required: window title
theme = "nord"                         # Optional: theme name
fps = 30                               # Optional: target FPS

[layout]                               # Optional: screen layout
header_height = 3
sidebar_width = 25
footer_height = 2

[[widget]]                             # Array of widgets
type = "StatusBadge"                   # Widget type name
id = 1                                 # Optional: widget ID
bind = "dracon-sync status --json"     # CLI command
parser = { type = "json_key", key = "status" }  # Output parser
refresh = 5                            # Auto-refresh interval (seconds)
confirm = "Run sync?"                  # Confirmation dialog text
label = "Sync Status"                  # Human-readable label
description = "Shows sync status"      # Description for AI enumeration

[widget.area]                          # Optional: explicit positioning
x = 0
y = 0
width = 20
height = 3
```

---

## 10. Event System

### 10.1 EventBus Internals

```rust
pub struct EventBus {
    subscribers: RefCell<HashMap<TypeId, Vec<Option<EventCallback>>>>,
    pending_tombstones: Rc<RefCell<HashSet<(TypeId, usize)>>>,
    trace: RefCell<bool>,
    history: RefCell<VecDeque<EventRecord>>,
    max_history: RefCell<usize>,
}

type EventCallback = Rc<dyn Fn(&dyn Any) + 'static>;

pub struct SubscriptionId { id: usize, type_id: TypeId }

pub struct EventRecord {
    pub timestamp: Instant,
    pub type_name: String,
    pub payload: Rc<dyn Any>,
}
```

**Publish algorithm:**
1. Trace log if enabled
2. Check tombstones (subscribe_once cleanup)
3. Find callbacks by TypeId
4. Execute each callback synchronously
5. Record in history (circular buffer, max 100)
6. Apply tombstones (remove subscribe_once callbacks)

**Subscribe algorithm:**
1. Create `EventCallback` wrapping user's closure
2. Get or create callback list for TypeId
3. Assign `SubscriptionId`
4. Return `SubscriptionId` for later `unsubscribe()`

### 10.2 Reactive<T>

```rust
pub struct Reactive<T: Clone + 'static> {
    value: RefCell<T>,
    callbacks: RefCell<Vec<Box<dyn Fn(&T)>>>,
}

impl<T: Clone + 'static> Reactive<T> {
    pub fn new(initial: T) -> Self;
    pub fn get(&self) -> T;
    pub fn set(&self, value: T);    // Updates value, fires callbacks
    pub fn subscribe<F>(&self, f: F) where F: Fn(&T) + 'static;
    pub fn map<U, F>(&self, f: F) -> Reactive<U> where F: Fn(&T) -> U + 'static;
}
```

---

## 11. Scene Router

### 11.1 SceneRouter Internal State

```rust
pub struct SceneRouter {
    scenes: HashMap<String, Box<dyn Scene>>,
    stack: Vec<String>,
    default_transition: SceneTransition,
    transition_duration: Duration,
    current_transition: Option<(SceneTransition, Instant, String, String)>,  // (animation, start_time, from_scene, to_scene)
}
```

### 11.2 Scene Lifecycle

```
push("settings"):
  1. If current scene: current.on_pause()
  2. Find "settings" in registry
  3. If found: scene.on_enter(), push to stack
  4. Start transition animation (if configured)

pop():
  1. If stack has ≥2 entries:
     a. Current scene: current.on_exit()
     b. Pop from stack
     c. New current: new.on_resume()
     d. Start transition animation
     e. Return true
  2. Else: return false

replace("about"):
  1. Current: current.on_exit()
  2. Pop from stack
  3. New: scene.on_enter(), push to stack
  4. Return true

go("home"):
  1. Clear entire stack (call on_exit for each)
  2. Push "home" scene
  3. scene.on_enter()
```

### 11.3 Transition Animation

Transitions are rendered as overlay planes on the compositor:
- Fade: over-plane with increasing opacity
- SlideLeft/SlideRight: positional offset animation
- SlideUp/SlideDown: positional offset animation
- None: instant switch (no animation plane)

---

## 12. Plugin System

### 12.1 Complete API

```rust
pub struct PluginRegistry {
    widgets: HashMap<String, WidgetFactory>,
}

impl PluginRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, name: &str, factory: WidgetFactory);
    pub fn create(&self, name: &str, id: WidgetId, theme: Theme) -> Option<Box<dyn Widget>>;
    pub fn unregister(&mut self, name: &str);
    pub fn names(&self) -> Vec<&str>;
    pub fn is_registered(&self, name: &str) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn merge(&mut self, other: PluginRegistry);   // Combine two registries
}
```

---

## 13. TextEditor (Standalone Widget)

### 13.1 File Structure

```
src/widgets/
  editor.rs              → 3,025 LOC: main implementation
  editor_search.rs       → Editor search/filter state and UI
```

### 13.2 Complete Public API

```rust
pub struct TextEditor {
    // Content
    lines: Vec<String>,
    file_path: Option<PathBuf>,
    modified: bool,

    // Cursor
    cursor_row: usize, cursor_col: usize,
    cursors: Vec<(usize, usize)>,            // Extra cursors for multi-cursor
    preferred_col: Option<u16>,              // Preferred column for vertical movement

    // Selection
    selection_start: Option<(usize, usize)>,

    // View
    offset: usize,                            // Scroll offset (lines from top)
    show_line_numbers: bool,
    word_wrap: bool,
    indent_guides: bool,
    show_status_bar: bool,

    // Syntax highlighting
    #[cfg(feature = "syntax-highlighting")]
    highlighter: Option<SyntaxHighlighter>,

    // Undo/redo
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    undo_depth: usize,
    last_edit_time: Instant,

    // Search/filter
    pub search: SearchState,

    // Theme/visual
    theme: Theme,

    // Config
    tab_width: usize,
    indent_unit: String,                       // "  " or "\t"
}

pub struct SearchState {
    pub filter_query: String,
    pub filtered_indices: Vec<usize>,
    pub mode: SearchMode,                     // Normal, Search, Replace, GotoLine
    pub mode_input: String,
    pub is_replacing: bool,
}
```

**Public methods:**
```rust
// Creation
pub fn new() -> Self
pub fn with_content(content: &str) -> Self
pub fn open(path: &Path) -> io::Result<Self>

// File I/O
pub fn save(&mut self) -> io::Result<()>
pub fn save_as(&mut self, path: &Path) -> io::Result<()>
pub fn file_path(&self) -> Option<&Path>

// Configure
pub fn with_show_line_numbers(self, show: bool) -> Self
pub fn with_word_wrap(self, wrap: bool) -> Self
pub fn with_indent_guides(self, show: bool) -> Self
pub fn with_status_bar(self, show: bool) -> Self
pub fn with_language(self, lang: &str) -> Self
pub fn with_theme(self, theme: Theme) -> Self

// Navigation
pub fn goto_line(&mut self, line: usize, area: Rect)
pub fn set_filter(&mut self, query: &str)
pub fn replace_all(&mut self, find: &str, replace: &str) -> usize
pub fn replace_next(&mut self, find: &str, replace: &str) -> bool

// Selection
pub fn get_selected_text(&self) -> Option<String>
pub fn select_all(&mut self)
pub fn select_word_at(&mut self, row: usize, col: usize)

// Multi-cursor
pub fn add_cursor(&mut self, row: usize, col: usize)
pub fn clear_extra_cursors(&mut self)

// Persistence
pub fn load_undo_stack(&mut self) -> io::Result<()>
pub fn save_undo_stack(&mut self) -> io::Result<()>
pub fn load_config(&mut self) -> io::Result<()>
pub fn save_config(&mut self) -> io::Result<()>
```

### 13.3 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Char | Insert character |
| Enter | Split line |
| Backspace | Delete before cursor |
| Delete | Delete after cursor |
| Left/Right | Move cursor |
| Up/Down | Move cursor vertically |
| Home/End | Line start/end |
| Ctrl+Home/End | Document start/end |
| PageUp/PageDown | Scroll page |
| Ctrl+Left/Right | Word boundary |
| Shift + navigation | Extend selection |
| Ctrl+A | Select all |
| Ctrl+C | Copy (requires QUIT action check) |
| Ctrl+X | Cut |
| Ctrl+V | Paste |
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Ctrl+F | Search |
| Ctrl+H | Replace |
| Ctrl+G | Goto line |
| Tab | Indent (insert indent) |
| Shift+Tab | Un-indent |

### 13.4 Undo/Redo System

```rust
enum EditAction {
    Insert { row: usize, col: usize, text: String },
    Delete { row: usize, col: usize, text: String },
    Replace { row: usize, col: usize, old: String, new: String },
    SplitLine { row: usize, col: usize },
    JoinLine { row: usize, col: usize },
    Batch { actions: Vec<EditAction> },   // Group for undo grouping
}
```

Coalescing: consecutive Insert actions at same position are merged into one `EditAction`. Same for Delete. Timer-based grouping: actions within 500ms of each other are grouped.

---

## 14. Application Patterns

### 14.1 Pattern Comparison

| Aspect | Pattern 1 (Widget Trait) | Pattern 2 (InputRouter + manual) |
|--------|-------------------------|----------------------------------|
| Render trigger | Auto via `needs_render()` | Manual via `ctx.add_plane()` in `on_tick` |
| Widget struct | Implements `Widget` | `InputRouter` widget + separate state struct |
| State mutation | Via `handle_key()`, `handle_mouse()` | Via closures with `Rc<RefCell<T>>` bridge |
| Theme cycling | `App::set_theme()` propagates | Must impl `current_theme()` for sync |
| Complexity | Lower (self-contained) | Higher (bridge pattern) |
| Use case | Simple apps, single widget | Multi-window, shared state, game loop |

### 14.2 Pattern 2 Theme Sync — Detailed Flow

```
1. App::run() dispatch_key() calls focused widget's handle_key()
2. Widget's handle_key() changes internal theme:
   fn handle_key(&mut self, key: KeyEvent) -> bool {
       if matches_theme_action(key) {
           self.app.borrow_mut().cycle_theme();
           true
       } else { false }
   }
3. After handle_key(), App checks widget.current_theme():
   if let Some(theme) = widget.current_theme() {
       if theme.name != self.theme.name {
           self.set_theme(theme);  // Propagates to all widgets
       }
   }
4. DTRON_THEME_FILE: on App::run() exit, self.theme.name is written to file
```

### 14.3 Help Overlay — Implementation Template

```rust
// In struct:
show_help: bool,

// In handle_key:
KeyCode::Char('?') if key.modifiers.is_empty() => {
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

// In render (drawn last):
if self.show_help {
    // 1. Compute centered overlay box (40×12, centered in area)
    let hw = 40u16.min(area.width.saturating_sub(4));
    let hh = 12u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;
    
    // 2. Fill background with surface_elevated
    for y in hy..hy+hh { for x in hx..hx+hw { /* set cell.bg = t.surface_elevated, cell.transparent = false */ }}
    
    // 3. Rounded corners: ╭╮╰╯
    // 4. Horizontal borders: ─
    // 5. Vertical borders: │
    // 6. Title centered: "Example Help" in t.primary + BOLD
    // 7. Shortcuts: two-column (key in primary, desc in fg)
}
```

### 14.4 Theme Return File (DTRON_THEME_FILE)

```rust
// In App::run(), before returning Ok(()):
if let Ok(path) = std::env::var("DTRON_THEME_FILE") {
    let _ = std::fs::write(&path, self.theme.name.as_bytes());
}

// In showcase launcher, after child process exits:
if let Ok(content) = std::fs::read_to_string(&theme_file_path) {
    if let Some(theme) = Theme::from_name(content.trim()) {
        ctx.set_theme(theme);
    }
}
std::fs::remove_file(&theme_file_path).ok();
```

### 14.5 SceneMode Enum Pattern

```rust
#[derive(Clone, Debug, Default)]
enum SceneMode {
    #[default]
    Normal,
    Help,
    Search,
    Confirm { message: String },
    ContextMenu { x: u16, y: u16, selected_index: Option<usize> },
}
// Benefits: exhaustive match, variant payload, impossible to have conflicting modes
```

### 14.6 Scrollbar Rendering Pattern

```rust
fn render_scrollbar(plane: &mut Plane, area: Rect, state: &ScrollState, theme: &Theme) {
    if state.content_height <= state.viewport_height { return; }
    
    let sb_x = area.width.saturating_sub(1);  // Rightmost column
    let content_h = area.height;
    let thumb_h = ((state.viewport_height as f32 / state.content_height as f32) * content_h as f32).max(1.0) as u16;
    let thumb_y = (state.fraction() * (content_h - thumb_h) as f32) as u16;
    
    // Track background
    for row in 0..content_h {
        let idx = (row * area.width + sb_x) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].bg = theme.scrollbar_track;
            plane.cells[idx].transparent = false;
        }
    }
    
    // Thumb
    for row in thumb_y..thumb_y + thumb_h {
        let idx = (row * area.width + sb_x) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '▐';
            plane.cells[idx].fg = theme.scrollbar_thumb;
            plane.cells[idx].bg = theme.scrollbar_track;
            plane.cells[idx].transparent = false;
        }
    }
}
```

---

## 15. Build Configuration & Features

### 15.1 Feature Flag Details

| Feature flag | Enable condition | What it enables | Additional deps | Impact on binary size |
|-------------|-----------------|-----------------|-----------------|----------------------|
| `system` | Default | `SystemMonitor`, `SystemData`, `ProcessInfo`, `DiskInfo` | `sysinfo` | ~+500KB |
| `syntax-highlighting` | Default | TextEditor syntax highlighting, `Theme::from_name` requires syntect theme loading | `syntect`, `regex` | ~+2MB (includes grammars) |
| `sqlite` | Optional | SQLite browser example, todo app | `rusqlite` | ~+1MB |
| `async` | Optional | Async I/O framework, network client example | `tokio`, `reqwest` | ~+3MB |
| `tracing` | Optional | Structured logging with `tracing` crate | `tracing`, `tracing-subscriber` | ~+200KB |
| `debug-events` | Optional | Key/mouse event logging to stderr | None (cfg checks) | ~+50KB |

### 15.2 Crate Dependencies (from Cargo.toml)

```toml
[dependencies]
bitflags = { version = "2.4", features = ["serde"] }       # Styles bitflags
ratatui = { version = "0.29", default-features = false }     # Rect type, Layout, integration
unicode-width = "0.1"                                        # Unicode char width
unicode-segmentation = "1.10"                                # Grapheme cluster parsing
chrono = { version = "0.4", features = ["serde", "clock"] }  # Calendar dates
signal-hook = "0.3"                                          # Signal handling
serde = { version = "1.0", features = ["derive"] }           # Serializable config
serde_json = "1.0"                                           # JSON output parsing
toml = "0.8"                                                  # TOML config parsing

[target.'cfg(not(target_os = "windows"))'.dependencies]
libc = "0.2"                                                  # POSIX ioctls

[dev-dependencies]
rand = "0.8"                                                  # Test random generation
tempfile = "3.10"                                             # Temp file creation
criterion = { version = "0.5", features = ["html_reports"] }  # Benchmarks
proptest = "1.4"                                              # Property-based testing
insta = { version = "1.40", features = ["yaml"] }             # Snapshot testing
```

### 15.3 Workspace Crates

```toml
[workspace]
members = ["crates/cargo-dracon"]  # Project scaffolding tool
```

The `extensions/` directory contains non-workspace projects:
- `extensions/lsp-server/` — Standalone LSP server (22 unwrap calls in production code)
- `extensions/vscode/` — VS Code extension for live TUI preview (TypeScript)

---

## 16. Examples & Showcase

### 16.1 Complete Example Inventory (57 Binaries)

**Group 1: Root Examples (34)**

| Example | File | Pattern | LOC (est.) | Key Features |
|---------|------|---------|------------|--------------|
| `arena` | arena.rs | Pattern 2 | 780 | Real-time game, compositor direct |
| `basic_raw` | basic_raw.rs | Raw | <100 | Minimal raw terminal demo |
| `command_dashboard` | command_dashboard.rs | Pattern 1 | ~300 | Command-driven dashboard |
| `cyberpunk_dashboard` | cyberpunk_dashboard.rs | Pattern 1 | ~300 | Themed dashboard |
| `desktop` | desktop.rs | Raw | ~400 | Window manager metaphor |
| `event_bus_demo` | event_bus_demo.rs | Pattern 2 | ~300 | Pub/sub demo |
| `form_demo` | form_demo.rs | Pattern 1 | ~400 | Form widget demo |
| `form_widget` | form_widget.rs | Pattern 1 | ~300 | Form standalone |
| `framework_chat` | framework_chat.rs | Pattern 1 | ~350 | Chat app |
| `framework_demo` | framework_demo.rs | Pattern 1 | ~300 | General demo (requires `system`) |
| `framework_file_manager` | framework_file_manager.rs | Pattern 1 | ~500 | File browser |
| `framework_widgets` | framework_widgets.rs | Pattern 1 | ~400 | Widget showcase |
| `from_toml` | from_toml.rs + .toml | TOML | ~100 | TOML-configured app |
| `game_loop` | game_loop.rs | Raw | ~300 | Game loop demo |
| `git_tui` | git_tui.rs | Pattern 1 | ~600 | Git interface |
| `god_mode` | god_mode.rs | — | ~300 | Advanced features |
| `ide` | ide.rs | Pattern 2 | ~800 | IDE with CommandPalette |
| `input_debug` | input_debug.rs | Raw | ~200 | Raw input debugging |
| `modal_demo` | modal_demo.rs | Pattern 2 | ~400 | Modal stacking + toasts |
| `network_client` | network_client.rs | Pattern 2 | ~400 | Async HTTP (requires `async`) |
| `plugin_demo` | plugin_demo.rs | Pattern 2 | ~300 | Plugin system |
| `scene_router_demo` | scene_router_demo.rs | Pattern 2 | ~350 | Scene navigation |
| `sqlite_browser` | sqlite_browser.rs | Pattern 1 | ~600 | SQLite DB (requires `sqlite`) |
| `table_widget` | table_widget.rs | Pattern 1 | ~400 | Table widget standalone |
| `text_editor_demo` | text_editor_demo.rs | Pattern 1 | ~200 | TextEditor demo |
| `theme_switcher` | theme_switcher.rs | Pattern 1 | ~300 | Theme cycling |
| `todo_app` | todo_app.rs | Pattern 2 | ~400 | SQLite todo (requires `sqlite`) |
| `tutorial_app` | tutorial_app.rs | Pattern 2 | ~350 | Tutorial/onboarding |
| `widget_tutorial` | widget_tutorial.rs | Pattern 1 | ~300 | Widget building tutorial |

**Group 2: Apps (_apps/) — 4**

| Example | File | Pattern | LOC | Key Features |
|---------|------|---------|-----|--------------|
| `system_monitor` | system_monitor.rs | Pattern 2 | ~600 | /proc data, process tree, sparklines |
| `file_manager` | file_manager.rs | Pattern 1 | ~500 | SplitPane, Breadcrumbs, file ops |
| `chat_client` | chat_client.rs | Pattern 2 | ~400 | Message list, input, history |
| `dashboard_builder` | dashboard_builder.rs | Pattern 1 | ~500 | Composable gauges, sparklines |

**Group 3: Cookbook (_cookbook/) — 19**

| Example | File | Pattern | LOC | Key Widgets |
|---------|------|---------|-----|-------------|
| `accessibility` | accessibility.rs | Pattern 1 | 567 | Focus rings, tree, log |
| `autocomplete` | autocomplete.rs | Pattern 1 | 401 | Autocomplete |
| `calendar` | calendar.rs | Pattern 1 | 400 | Calendar/DatePicker |
| `cell_pool` | cell_pool.rs | Pattern 2 | 498 | CellPool visualization |
| `command_bindings` | command_bindings.rs | Pattern 1 | ~300 | BoundCommand, parsers |
| `data_table` | data_table.rs | Pattern 1 | ~400 | Table with sorting |
| `debug_overlay` | debug_overlay.rs | Pattern 1 | 445 | DebugOverlay + gauges |
| `form_validation` | form_validation.rs | Pattern 1 | ~300 | Form validation |
| `log_monitor` | log_monitor.rs | Pattern 2 | ~350 | LogViewer |
| `menu_system` | menu_system.rs | Pattern 1 | ~300 | MenuBar, ContextMenu |
| `notification_center` | notification_center.rs | Pattern 1 | 389 | NotificationCenter |
| `plugin_demo` | plugin_demo.rs | Pattern 2 | ~300 | Plugin system |
| `rich_text` | rich_text.rs | Pattern 1 | 366 | RichText rendering |
| `scrollable_content` | scrollable_content.rs | Pattern 1 | ~300 | ScrollContainer |
| `split_resizer` | split_resizer.rs | Pattern 2 | ~300 | SplitPane drag |
| `stat_widget_plugin` | stat_widget_plugin.rs | Pattern 1 | ~200 | Plugin widget |
| `tabbed_panels` | tabbed_panels.rs | Pattern 2 | ~300 | TabBar + panels |
| `tree_navigator` | tree_navigator.rs | Pattern 1 | 330 | Tree widget |
| `widget_gallery` | widget_gallery.rs | Pattern 1 | 399 | 12 widget grid |

**Group 4: Showcase Scenes (29 embedded, in `examples/showcase/scenes/`)**

| Scene | LOC | Widgets Demonstrated |
|-------|-----|---------------------|
| widget_gallery | 482 | Workshop with 12 widgets |
| theme_switcher | 451 | Split preview, all 21 themes |
| password_input | 529 | Form, SearchInput, PasswordInput |
| notification_center | 549 | NotificationCenter, tabs |
| color_picker | 501 | ColorPicker, swatches |
| tags_input | 535 | TagsInput, cloud, stats |
| progress | 585 | ProgressBar, ProgressRing, Spinner, Gauge |
| cell_pool | 498 | CellPool visualization |
| rich_text | 548 | RichText, scrollbars |
| debug_overlay | 590 | Gauges, Profiler, DebugOverlay |
| metrics_hub | 544 | Sparklines, metrics |
| table_list | 521 | Table, List |
| navigator | 568 | Tree, file browser |
| kanban | 329 | Kanban, progress sidebar |
| (and 15 more scenes) | | |

### 16.2 Showcase Launcher Architecture

```
examples/showcase/
├── main.rs              # Entry point, app setup, binary launch
├── data.rs              # ExampleMeta definitions for all examples
├── state.rs             # Showcase struct, filtering, selection
├── render.rs            # Card rendering, preview functions
├── widget.rs            # Widget impl (render, handle_key, handle_mouse)
├── scenes/
│   ├── mod.rs           # Scene registration + shared helpers
│   ├── shared_helpers.rs # draw_text, draw_text_clipped, render_help_overlay, blit_to
│   ├── app_scenes.rs    # Application example launchers
│   ├── widget_gallery.rs
│   ├── theme_switcher.rs
│   ├── password_input.rs
│   ├── ... (25 more scene files)
│   └── workshop.rs      # Widget Workshop scene
└── tests/
    └── showcase_smoke_test.rs  # Integration smoke test (ignored by default)
```

---

## 17. Test Coverage

### 17.1 Test Inventory — Complete Per-File Breakdown

| Test File | Module | Test Functions | Coverage Area |
|-----------|--------|----------------|---------------|
| `src/framework/app.rs` (inline) | App, Ctx | 35 | App construction, builder, widget CRUD, theme, Ctx operations, split, layout, commands |
| `src/framework/command.rs` (inline) | Command | 60+ | BoundCommand builder, all 8 OutputParsers, CommandRunner sync/spawn/parse, edge cases |
| `src/framework/theme.rs` (inline) | Theme | 25+ | All 21 constructors, from_name/lookup, from_env_or, random |
| `src/framework/widget.rs` (inline) | Widget trait | 5 | WidgetId generation, sub-trait blanket impls |
| `src/framework/marquee.rs` (inline) | MarqueeState | 15 | State machine (Idle→Tracking→Active), deferred clicks, rect normalization |
| `src/framework/event_bus.rs` (inline) | EventBus | 14 | Publish/subscribe, subscribe_once, unsubscribe, history, trace |
| `src/framework/scene_router.rs` (inline) | SceneRouter | 10 | Push/pop/replace/go, lifecycle callbacks, transitions |
| `src/framework/focus.rs` (inline) | FocusManager | 14 | Register/unregister, tab cycling, focus trapping, callbacks |
| `src/framework/hitzone.rs` (inline) | HitZone | 11 | Single/double/triple click, drag, right-click, group dispatch |
| `src/framework/layout.rs` (inline) | Layout | 10+ | All constraint types, direction, spacing, margin, caching |
| `src/framework/keybindings.rs` (inline) | KeybindingSet | 8 | Parse keybinding strings, match, resolution order |
| `src/framework/scroll.rs` (inline) | ScrollState | 8 | Scroll up/down/to, max_offset, page_size, fraction |
| `src/framework/widgets/button.rs` (inline) | Button | 4 | Render, mouse events, click callback |
| `src/framework/widgets/label.rs` (inline) | Label | 3 | Render, builder methods |
| `src/framework/widgets/list_common.rs` (inline) | ListCommon | 8+ | Navigate up/down/page/home/end, wrap logic |
| `src/widgets/editor.rs` (inline) | TextEditor | 30+ | Basic editing, cursor movement, file I/O, undo/redo, search, selection |
| `src/compositor/engine.rs` (inline) | Compositor | 5 | Construction, add_plane, size, hit_test |
| `src/compositor/plane.rs` (inline) | Plane | 10+ | put_char, put_str, blit_from, fill_bg, clear, crop, Unicode handling |
| `src/compositor/filter.rs` (inline) | Filter | 5 | Dim, Invert, Scanline, Pulse, Glitch |
| `src/input/parser.rs` (inline) | Parser | 15+ | Key events, mouse events, resize, kitty protocol, edge cases |
| `src/utils.rs` (inline) | Utils | 5 | Visual width, truncate |
| `src/text.rs` (inline) | Text | 5 | Grapheme width, cluster iteration |
| `src/integration/mod.rs` (inline) | Integration | 3 | Ratatui backend conversion |

**Integration Tests (`tests/`): 43 files**

| File | Tests | What It Tests |
|------|-------|---------------|
| `widget_test.rs` | 26 | Core widget rendering, layout |
| `widget_tests.rs` | 14 | Widget interaction patterns |
| `button_test.rs` | 6 | Button click, hover, state |
| `widget_gauge_test.rs` | 12 | Gauge rendering, thresholds |
| `gauge_test.rs` | 15 | Gauge bounds, percentages |
| `label_test.rs` | 10 | Label styling, text truncation |
| `list_test.rs` | 20+ | List selection, scroll, keyboard, mouse |
| `list_common_test.rs` | 25 | ListCommon navigation, edge cases |
| `tree_widget_test.rs` | 18 | Tree expand/collapse, select, keyboard |
| `modal_widget_test.rs` | 12 | Modal show/hide, result, backdrop |
| `widget_confirm_dialog_test.rs` | 11 | Confirm yes/no/cancel, danger styling |
| `menu_test.rs` | 14 | MenuBar/ContextMenu open/close, selection |
| `form_widget_test.rs` | 16 | Form rendering, tab navigation, validation |
| `form_validation_test.rs` | 10 | All validation rule types |
| `widget_password_input_test.rs` | 15 | Masking, submit, show/hide |
| `widget_slider_test.rs` | 12 | Slider min/max/step, keyboard, mouse drag |
| `widget_status_badge_test.rs` | 10 | Status colors, bind_command |
| `widget_sparkline_test.rs` | 37 | Sparkline data bounds, braille chars |
| `widget_progress_ring_test.rs` | 38 | ProgressRing arc, percentage, sizing |
| `widget_streaming_text_test.rs` | 10 | Append, wrap, scroll |
| `widget_key_value_grid_test.rs` | 10 | Key-value rendering, JSON parsing |
| `widget_log_viewer_test.rs` | 10 | Log lines, severity colors, filter |
| `widget_snapshot_tests.rs` | 8 | Visual snapshots (List, Table, Tree) |
| `toast_test.rs` | 8 | Toast lifecycle, dismiss, animation |
| `context_menu_test.rs` | 17 | Context menu items, submenus, selection |
| `text_editor_test.rs` | 30+ | Editor operations, edge cases |
| `text_editor_adapter_test.rs` | 15 | Adapter delegation |
| `text_editor_adapter_edge_test.rs` | 8 | Adapter edge cases |
| `theme_test.rs` | 12 | Theme creation, from_name, from_env_or |
| `theme_validation_test.rs` | 5 | Theme field validation |
| `theme_propagation_test.rs` | 3 | Widget theme propagation |
| `focus_test.rs` | 14 | FocusManager Tab cycling, trapping |
| `hitzone_test.rs` | 11 | HitZone interaction patterns |
| `scroll_test.rs` | 8 | ScrollState bounds, paging |
| `multi_widget_test.rs` | 3 | Multi-widget app coordination |
| `resize_test.rs` | 4 | Terminal resize handling |
| `event_bus_test.rs` | 14 | EventBus pub/sub patterns |
| `scene_router_test.rs` | 10 | Scene lifecycle, transitions |
| `splitpane_test.rs` | 6 | SplitPane division, drag resize |
| `status_bar_test.rs` | 6 | StatusBar segments |
| `streaming_text_test.rs` | 8 | StreamingText append/clear |
| `syntax_highlighting_test.rs` | 6 | Syntect integration |
| `panel_test.rs` | 5 | Panel border rendering |
| `input_reader_test.rs` | 15 | Input parsing, buffer drain |
| `network_widget_test.rs` | 4 | Network widget (async feature) |
| `profiler_test.rs` | 5 | Profiler metrics |
| `filter_test.rs` | 8 | Visual filter compositing |
| `utils_test.rs` | 10 | Utility functions |
| `phase1_widget_test.rs` | 8 | Phase 1 widgets |
| `phase2_3_4_widget_test.rs` | 8 | Phases 2-4 widgets |
| `untested_widgets_test.rs` | 10 | Coverage gap closure tests |
| `property_tests.rs` | 6 | Property-based (proptest) |
| `widget_gallery_edge_test.rs` | — | Widget gallery edge cases |

**Benchmarks:**
| File | Benchmark Type | What's Measured |
|------|---------------|-----------------|
| `tests/framework_benchmarks.rs` | criterion | Compositor rendering, plane operations, widget rendering |
| `tests/performance_benchmarks.rs` | raw | Raw throughput, escape sequence generation |

### 17.2 Coverage by Widget

| Widget | Unit Tests | Integration Tests | Snapshot Tests | Total |
|--------|-----------|-------------------|----------------|-------|
| Autocomplete | 0 (inline) | 0 (in phase tests) | 0 | ~5 |
| Breadcrumbs | 0 (inline) | 0 (in phase tests) | 0 | ~3 |
| Button | 4 (inline) | 6 (button_test) | 0 | 10 |
| Calendar | 0 (inline) | 0 (in phase tests) | 0 | ~3 |
| Checkbox | 0 | 0 (in phase tests) | 0 | ~3 |
| ColorPicker | 0 | 0 (in phase tests) | 0 | ~2 |
| CommandPalette | 0 | 0 (in phase tests) | 0 | ~3 |
| ConfirmDialog | 5 (inline) | 11 (confirm_dialog_test) | 0 | 16 |
| ContextMenu | 3 (inline) | 17 (context_menu_test) | 0 | 20 |
| DebugOverlay | 0 | 0 | 0 | 0 |
| Divider | 0 | 0 | 0 | 0 |
| EventLogger | 0 | 0 (in phase tests) | 0 | ~2 |
| Form | 0 | 16 (form_widget_test) | 0 | 16 |
| Gauge | 0 | 27 (gauge_test + widget_gauge_test) | 0 | 27 |
| Hud | 0 | 0 | 0 | 0 |
| Kanban | 0 | 0 (in phase tests) | 0 | ~3 |
| KeyValueGrid | 0 | 10 (key_value_grid_test) | 0 | 10 |
| Label | 3 (inline) | 10 (label_test) | 0 | 13 |
| List | 5 (inline) | 20+ (list_test) | 1 (snapshot) | 26+ |
| LogViewer | 0 | 10 (log_viewer_test) | 0 | 10 |
| MenuBar | 0 | 14 (menu_test) | 0 | 14 |
| Modal | 0 | 12 (modal_widget_test) | 0 | 12 |
| NotificationCenter | 0 | 0 (in phase tests) | 0 | ~3 |
| PasswordInput | 0 | 15 (password_input_test) | 0 | 15 |
| Profiler | 0 | 5 (profiler_test) | 0 | 5 |
| ProgressBar | 0 | 0 (in phase tests) | 0 | ~3 |
| ProgressRing | 0 | 38 (progress_ring_test) | 0 | 38 |
| Radio | 0 | 0 (in phase tests) | 0 | ~3 |
| RichText | 0 | 0 (in phase tests) | 0 | ~3 |
| SearchInput | 0 | 0 (in phase tests) | 0 | ~3 |
| Select | 0 | 0 (in phase tests) | 0 | ~3 |
| Slider | 0 | 12 (slider_test) | 0 | 12 |
| Sparkline | 0 | 37 (sparkline_test) | 0 | 37 |
| Spinner | 0 | 0 | 0 | 0 |
| SplitPane | 0 | 6 (splitpane_test) | 0 | 6 |
| StatusBadge | 0 | 10 (status_badge_test) | 0 | 10 |
| StatusBar | 0 | 6 (status_bar_test) | 0 | 6 |
| StreamingText | 0 | 8 (streaming_text_test) | 0 | 8 |
| TabBar | 0 | 0 (in phase tests) | 0 | ~3 |
| Table | 5 (inline) | 0 (in phase tests) | 1 (snapshot) | 6+ |
| TagsInput | 0 | 0 (in phase tests) | 0 | ~2 |
| TextEditorAdapter | 0 | 23 (2 adapter files) | 0 | 23 |
| Toast | 0 | 8 (toast_test) | 0 | 8 |
| Toggle | 0 | 0 (in phase tests) | 0 | ~3 |
| Tooltip | 0 | 0 (in phase tests) | 0 | ~2 |
| Tree | 0 | 18 (tree_widget_test) | 1 (snapshot) | 19 |
| WidgetInspector | 0 | 0 | 0 | 0 |

---

## 18. API Surface & Prelude

### 18.1 Preliminary Re-exports (`framework::prelude`)

```rust
/// One-import entry point for all framework functionality.
pub mod prelude {
    // ── Engine Types ──
    pub use crate::compositor::{Cell, CellPool, Color, Compositor, Plane, PoolConfig, Styles};
    pub use crate::error::DraconError;
    pub use crate::Terminal;

    // ── Widget Trait ──
    pub use crate::framework::widget::{
        Commandable, Focusable, InputHandler, Renderable, Themable,
        Widget, WidgetId, WidgetState,
    };

    // ── App ──
    pub use crate::framework::app::{App, Ctx, WidgetRef, WidgetRefMut};

    // ── Framework Subsystems ──
    pub use crate::framework::animation::{Animation, AnimationManager, Easing};
    pub use crate::framework::command::{
        AppConfig, AreaConfig, BoundCommand, CommandRunner, LayoutConfig,
        LoggedLine, OutputParser, ParsedOutput, ParserConfig, WidgetConfig,
    };
    pub use crate::framework::dirty_regions::{DirtyRegion, DirtyRegionTracker};
    pub use crate::framework::dragdrop::{DragGhost, DragManager, DragPhase};
    pub use crate::framework::event_bus::{EventBus, Reactive, SubscriptionId};
    pub use crate::framework::focus::FocusManager;
    pub use crate::framework::hitzone::{DragState, HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry};
    pub use crate::framework::i18n::{tr, I18n, I18nError};
    pub use crate::framework::keybindings::{actions, resolve_keybindings, KeybindingConfig, KeybindingSet};
    pub use crate::framework::layout::{Constraint, Direction, Layout};
    pub use crate::framework::marquee::{render_marquee, MarqueeRect, MarqueeState};
    pub use crate::framework::plugin::{PluginRegistry, WidgetFactory};
    pub use crate::framework::scroll::{ScrollContainer, ScrollState};
    pub use crate::framework::scene_router::{NavigationEvent, Scene, SceneRouter};

    // ── Theme ──
    pub use crate::framework::theme::Theme;

    // ── All 47 Framework Widgets ──
    pub use crate::framework::widgets::*;

    // ── Input Events ──
    pub use crate::input::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    };

    // ── External ──
    pub use ratatui::layout::Rect;

    // ── Tracing (feature-gated) ──
    #[cfg(feature = "tracing")]
    pub use crate::frame_span;
    #[cfg(feature = "tracing")]
    pub use crate::frame_span_debug;
    #[cfg(feature = "tracing")]
    pub use tracing::instrument;
}
```

### 18.2 Crate Root Re-exports

```rust
// Compositor primitives
pub use compositor::{Cell, Color, Compositor, Plane, Styles};

// Error
pub use error::DraconError;

// Core terminal
pub use core::terminal::{Capabilities, CursorShape, Terminal};

// Input
pub use input::{InputReader, Parser};

// System (feature-gated)
#[cfg(feature = "system")]
pub use system::{DiskInfo, ProcessInfo, SystemData, SystemMonitor};

// Standalone widgets
pub use widgets::editor::TextEditor;
pub use widgets::input::TextInput;
pub use widgets::button::Button as StandaloneButton;
pub use widgets::panel::Panel;
pub use widgets::component::Component;
pub use widgets::hotkey::HotkeyHint;
pub use widgets::context_menu::ContextMenuAction;

// Framework prelude
pub use framework::prelude;
```

---

## 19. Completeness Assessment

### 19.1 Final Score: **87/100**

### 19.2 Detailed Scoring Matrix

| Category | Weight | Raw Score | Weighted | Criteria |
|----------|--------|-----------|----------|----------|
| **Core Engine** | 15% | 94/100 | 14.1 | Compositor (+60), Terminal (+20), Input (+10), Error handling (+4) |
| **Framework Architecture** | 20% | 90/100 | 18.0 | App/Ctx (+40), Widget trait (+20), subsystems (+20), lifecycle (+10) |
| **Widget Inventory** | 15% | 87/100 | 13.1 | 47 widget types (+35), widget quality/converage (+30), patterns (+22) |
| **Theme System** | 10% | 95/100 | 9.5 | 21 themes (+40), semantic fields (+30), from_name/from_env_or (+25) |
| **Input System** | 10% | 90/100 | 9.0 | SGR mouse (+25), keyboard (+25), kitty (+10), parser (+20), event types (+10) |
| **TextEditor** | 10% | 70/100 | 7.0 | Core editing (+30), syntax highlighting (+15), undo/redo (+10), search (+10), file-size (-15) |
| **Examples** | 10% | 92/100 | 9.2 | 57 binaries (+30), 29 showcase scenes (+30), patterns (+20), quality (+12) |
| **Documentation** | 5% | 60/100 | 3.0 | README (+10), AGENTS.md (+25), AI_GUIDE.md (+15), doc comments (-30), example docs (-10) |
| **Testing** | 5% | 95/100 | 4.8 | Unit tests (+30), integration (+20), snapshots (+10), benchmarks (+10), proptest (+10), 0 clippy (+15) |

**Total: 87.1 / 100**

### 19.3 Gap Analysis — 15 Items

**Critical (blocking 90+): 3 items**

| # | Gap | Impact | Effort to Fix |
|---|-----|--------|---------------|
| 1 | `TextEditor (editor.rs)` — 3,025 LOC single file | Maintainability, reviewability, merge conflicts | Medium (2-3 hours) |
| 2 | `utils.rs` — 1,217 LOC catch-all | Poor module cohesion, hard to find utilities | Medium (1-2 hours) |
| 3 | Missing `// SAFETY:` comments (11/12 `unsafe` blocks in `src/`) | Undocumented UB risk, audit friction | Low (30 min) |

**High: 5 items**

| # | Gap | Impact | Effort |
|---|-----|--------|--------|
| 4 | 13+ widgets lack mouse handlers (DebugOverlay, Divider, Gauge, Hud, Label, Profiler, ProgressBar, ProgressRing, Sparkline, Spinner, StatusBadge, StatusBar, StreamingText, Toast) | Inconsistent UX; mouse-only users can't interact with these widgets | Medium (3-5 hours) |
| 5 | 30+ widgets return `focusable() = true` default but don't implement focus behavior | Misleading: widgets claim to accept focus but do nothing with it | Medium (2-4 hours) |
| 6 | No Windows backend — `libc` gated to `cfg(not(windows))` | Excludes ~30% of target audience | High (weeks) |
| 7 | Deprecated `scrollbar_width` in Theme struct | API cruft, layout in theme violates separation of concerns | Low (30 min) |
| 8 | `App::new().unwrap()` in doc examples | Bad practice propagation; docs don't show error handling | Low (1 hour) |

**Medium: 4 items**

| # | Gap | Impact | Effort |
|---|-----|--------|--------|
| 9 | 25/30 doc-tests are `ignore` | ~83% of doc examples not compile-tested | Medium (2-3 hours) |
| 10 | CHANGELOG format drift (not strict keepachangelog) | Hard to auto-parse, inconsistent subsections | Low (30 min) |
| 11 | `dracon.toml` has no schema validation | Invalid TOML produces opaque serde errors | Low (1 hour) |
| 12 | Event bus has no benchmarks | No perf regression detection for pub/sub | Low (2 hours) |

**Low: 3 items**

| # | Gap | Impact | Effort |
|---|-----|--------|--------|
| 13 | `cargo outdated` not in CI | Outdated deps may accumulate | Low (30 min) |
| 14 | Example enrichment gaps (modal_demo 30%, tooltip 45%, tags_input 40%) | Poor first impression for those scenes | Medium (2-3 hours) |
| 15 | No cross-platform targets documented | Users assume macOS/Windows works | Low (15 min) |

### 19.4 Strengths

| Area | Strength | Evidence |
|------|----------|----------|
| Arch coherence | Clean 3-layer design (Engine → Framework → Widgets) | Module structure, data flow diagram |
| Widget breadth | 47 widget types covering most UI patterns | Full inventory in section 5 |
| Theme depth | 21 themes with 31 semantic-color fields | Theme constructors, from_name/from_env_or |
| Input coverage | SGR mouse, kitty keyboard, bracketed paste, all modifiers | Parser state machine, 60+ escape sequences |
| Compositor performance | Dirty regions, cell diffing, bulk writes, pool allocation | Optimization table in section 8.4 |
| Command-driven arch | 8 output parsers, TOML config, AI-inspectable actions | Complete parser table in section 9 |
| Example breadth | 57 examples, 29 showcase scenes, 4 patterns | Full inventory in section 16 |
| Test quality | 0 clippy warnings, 1,436 test functions, 291+ unit tests | Per-widget coverage table in section 17 |

---

## 20. Future Roadmap

### 20.1 Immediate (0.2.0)

| # | Item | Type | Priority | Effort |
|---|------|------|----------|--------|
| 1 | Split `editor.rs` into submodules | Code quality | High | 2-3h |
| 2 | Add `// SAFETY:` comments to all unsafe blocks | Safety | High | 30min |
| 3 | Split `utils.rs` into proper modules | Code quality | Medium | 1-2h |
| 4 | Widget decomposition Phase 2 — sub-traits as primary | API design | Medium | 3-5h |
| 5 | Convert 25 ignored doc-tests to compile-tested | Docs | Low | 2-3h |
| 6 | Add mouse handlers to 13+ missing widgets | UX | Medium | 3-5h |
| 7 | Remove deprecated `theme.scrollbar_width` | Cleanup | Low | 30min |
| 8 | Enforce keepachangelog CHANGELOG format | Process | Low | 1h |

### 20.2 Medium Term (0.3.0–0.5.0)

| # | Item | Description |
|---|------|-------------|
| 9 | Windows backend | Port `backend/tty.rs` to Windows console API |
| 10 | Widget focus audit | Implement proper focus for all 47 widgets |
| 11 | Schema validation for `dracon.toml` | Structural validation with error messages |
| 12 | Event bus benchmarks | Criterion benchmarks for pub/sub throughput |
| 13 | `cargo outdated` in CI | Automated dependency freshness checking |
| 14 | Scene enrichment | Fill remaining enrichment gaps (modal_demo, tooltip, tags_input) |
| 15 | Widget State serialization | Implement `WidgetState` for all major widgets |
| 16 | Accessibility audit | Verify OSC 99 announcements across all widgets |

### 20.3 Long Term (1.0+)

| # | Item | Description |
|---|------|-------------|
| 17 | Doc comment audit | Add docs for all ~30 undocumented pub fn in app.rs |
| 18 | API stabilization | Review Widget trait for 1.0 stability guarantees |
| 19 | Performance benchmarks | Full criterion suite for all subsystems |
| 20 | Release automation | GitHub Actions for crates.io publish |

### 20.4 Deferred / Explicitly Out of Scope

| Feature | Reason for Deferral |
|---------|---------------------|
| LSP integration | Requires async runtime, external processes, complex state management |
| Syntax-aware folding | Requires tree-sitter integration, per-language grammar |
| Multi-cursor enhancements | Basic multi-cursor sufficient for view/edit use case |
| Modal editing | Kakoune-style is complex, not needed for view/edit widget use case |
| Advanced text objects | vim-style text objects require deep editor integration |
| GPU-accelerated terminal | Requires custom terminal emulator, outside scope of framework |
| WebAssembly target | Browser sandbox incompatible with raw terminal access |
| ios/macOS native GUI | Deliberately terminal-only by design |

---

## 21. Appendices

### Appendix A: Environment Variable Reference

| Variable | Set By | Read By | Purpose |
|----------|--------|---------|---------|
| `DTRON_THEME` | Showcase launcher, script | `Theme::from_env_or()` | Inherit theme from parent process |
| `DTRON_THEME_FILE` | Showcase launcher | `App::run()` (auto) | Path to file for writing final theme name on exit |
| `HOME` | OS | `resolve_keybindings()` | User config directory resolution |
| `TERM` | OS | `Capabilities` detection | Terminal type identification |
| `COLORTERM` | OS | `Capabilities` detection | True color support detection |

### Appendix B: Keybinding Standard Actions — Default Mappings

| Action Constant | Default Key | Priority | Rationale |
|----------------|-------------|----------|-----------|
| `QUIT` | `Ctrl+Q` | Configurable | Never 'q' (conflicts with text input) |
| `HELP` | `F1` | Configurable | Never '?' (conflicts with text input) |
| `BACK` | `Esc` | Configurable | Universal dismiss; Backspace is delete-only |
| `THEME` | `Ctrl+T` | Configurable | Not 't' (conflicts with typing) |
| `SUBMIT` | `Enter` | Configurable | Universal confirm |
| `SEARCH` | `Ctrl+F` | Configurable | Not '/' (conflicts with typing) |
| `SAVE` | `Ctrl+S` | Configurable | Universal standard |
| `NEW` | `Ctrl+N` | Configurable | Universal standard |
| `CLOSE` | `Ctrl+W` | Configurable | Browser/IDE standard |
| `COPY` | `Ctrl+C` | Configurable | Universal (with QUIT guard) |
| `PASTE` | `Ctrl+V` | Configurable | Universal |
| `CUT` | `Ctrl+X` | Configurable | Universal |
| `DELETE` | `Delete` | Configurable | Universal |
| `REFRESH` | `F5` | Configurable | Universal |
| `PAUSE` | `Ctrl+P` | Configurable | Not 'p' (conflicts with typing) |

**Non-configurable keys (hardcoded):**
- `↑/↓/←/→`: Navigation (universal)
- `Enter`: Selection/submit (universal)
- `Tab` / `Shift+Tab`: Focus cycle (universal)
- `Backspace`: Delete character (text input primitive)
- `Char(c)`: Type character (text input primitive)

### Appendix C: Reserved Z-Index Ranges

| Range | Layer | Used By |
|-------|-------|---------|
| 0 | Background | Base widgets, background fills |
| 5 | Content | SplitPane, Panel, List, Table, Tree, Form |
| 10 | Interactive | Button, SearchInput, PasswordInput, Slider, Select |
| 50 | Overlays | Tooltip, dropdown, CommandPalette backdrop |
| 100 | Modal | Modal, ConfirmDialog |
| 500 | Notifications | Toast, NotificationCenter |
| 9000 | Drag ghost | DragGhost (reserved) |

### Appendix D: File Size Reference (Largest Files)

| File | LOC | % of Project | Notes |
|------|-----|-------------|-------|
| `src/widgets/editor.rs` | 3,025 | 7.3% | Largest file; needs splitting |
| `src/utils.rs` | 1,217 | 2.9% | Second largest; needs splitting |
| `src/framework/app.rs` | 1,591 | 3.8% | Core framework; well-structured |
| `src/framework/theme.rs` | 1,447 | 3.5% | 21 theme constructors; structurally fine |
| `src/framework/event_bus.rs` | 530 | 1.3% | Event system |

### Appendix E: Cargo Feature Matrix

| Feature | `system` | `syntax-highlighting` | `sqlite` | `async` | `tracing` | `debug-events` |
|---------|----------|----------------------|----------|---------|-----------|----------------|
| In default | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Adds dep | `sysinfo` | `syntect`, `regex` | `rusqlite` | `tokio`, `reqwest` | `tracing`, `tracing-subscriber` | — |
| Enables | SystemMonitor | TextEditor highlight | SQLite examples | Network client | Frame spans, logging | Stderr debug output |
| Excluded examples | framework_demo | All editor examples | todo_app, sqlite_browser | network_client | — | — |

### Appendix F: Test Command Reference

```bash
# Run all tests
cargo test

# Run library tests only
cargo test --lib

# Run integration tests only
cargo test --tests

# Run doc tests
cargo test --doc

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench

# Clippy checks (zero warnings required)
cargo clippy --lib --examples -- -D warnings
cargo clippy --tests -- -D warnings

# Build all examples
cargo build --examples

# Build with specific features
cargo build --features "sqlite"
cargo build --no-default-features --features "syntax-highlighting"

# Security audit
cargo audit
```

### Appendix G: Glossary

| Term | Definition |
|------|-----------|
| **Plane** | A 2D grid of `Cell` values with position, z-index, opacity, and optional filter |
| **Cell** | Single terminal character with foreground color, background color, style flags, and transparency |
| **Compositor** | Engine that composites multiple Planes into a single output using painter's algorithm |
| **Dirty region** | A rectangular area of the screen that changed since last frame |
| **Pattern 1** | Widget trait auto-render: `impl Widget` with `needs_render()` returning true |
| **Pattern 2** | InputRouter + manual render: closure-based, `ctx.add_plane()` in tick callback |
| **Bridge pattern** | `Rc<RefCell<T>>` shared state for Pattern 2 apps |
| **Input shield** | Cooldown period that swallows stale keypresses after mode transitions |
| **SGR mouse** | Terminal mouse protocol (DECSET 1006) supporting position, drag, buttons, modifiers |
| **Bracketed paste** | Terminal paste mode (DECSET 2004) wrapping pasted text in escape delimiters |
| **Kitty keyboard** | Extended keyboard protocol with Unicode key codes and press/repeat/release |
| **Sync mode 2026** | Terminal synchronized output mode for tear-free rendering |
| **DTRON_THEME** | Environment variable for theme inheritance from parent process |
| **DTRON_THEME_FILE** | Environment variable for returning theme to parent on exit |
