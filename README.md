# Dracon Terminal Engine

[![crates.io](https://img.shields.io/crates/v/dracon-terminal-engine.svg)](https://crates.io/crates/dracon-terminal-engine)
[![docs.rs](https://img.shields.io/docsrs/dracon-terminal-engine)](https://docs.rs/dracon-terminal-engine)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

```
  _______   ______   .______      .___ ___.      ___
 |       | |   ___|  |   _  \     |   \/   |     /   \
 |.|   | | |  |__    |  |_)  |    |  \  /  |    /  ^  \
   |   |   |   __|   |      /     |  |\/|  |   /  /_\  \
   |   |   |  |____  |  |\  \----.|  |  |  |  /  _____  \
   |___|   |_______| | _| `._____||__|  |__| /__/     \__\
```

> **A terminal application framework for Rust — one import, AI builds a complete app.**

## What It Is

`dracon-terminal-engine` is a framework for building terminal applications. Not a TUI library — a complete runtime that owns the terminal, input, rendering, and event loop. Mouse-friendly, z-indexed planes, 35 built-in widgets, 15 themes, dirty rendering, and focus management.

**Command-driven architecture** — every widget binds a CLI command, AI can enumerate all actions via `ctx.available_commands()` and trigger them via `ctx.run_command()`.

**One import to rule them all:**

```rust
use dracon_terminal_engine::framework::prelude::*;

App::new().unwrap()
    .title("My App")
    .fps(30)
    .theme(Theme::cyberpunk())
    .on_tick(|_ctx, _tick| { /* called every 250ms */ })
    .run(|ctx| {
        let items = vec!["Files", "Search", "Git", "Settings"];
        let list = List::new(items);
        let (w, h) = ctx.compositor().size();
        ctx.add_plane(list.render(Rect::new(0, 0, w, h)));
    });
```

## Framework (v27)

The `framework` module provides the complete application runtime:

Every widget can bind a CLI command. AI can enumerate all actions via `ctx.available_commands()` and trigger them via `ctx.run_command()`:

```rust
// In a tick callback, AI can:
let cmds = ctx.available_commands();  // List all 50+ available commands
for cmd in cmds {
    println!("{}: {}", cmd.label, cmd.description);
}

// Trigger any action:
let (stdout, stderr, code) = ctx.run_command("dracon-sync status --json");
```
| Widget | What |
|---|---|
| [`HitZone<T>`] | Declarative interactive region (click/double/drag/hover) |
| [`HitZoneGroup<T>`] | Batch of hit zones, auto-dispatched |
| [`ScopedZone<T>`] | Lightweight geometry-only zone for per-frame dispatch |
| [`ScopedZoneRegistry<T>`] | Registry that clears per frame |
| [`DragManager<T>`] | Drag-and-drop state machine with ghost rendering |
| [`FocusManager`] | Tab-order focus ring with keyboard navigation |
| [`ScrollContainer`] | Scrollable container with offset management + scrollbar |

### 35 Framework Widgets
| Widget | What |
|---|---|
| [`Breadcrumbs`] | Hierarchical path display with clickable segments |
| [`Button`] | Clickable button with press state and callbacks |
| [`Checkbox`] | Two-state toggle with check mark |
| [`ConfirmDialog`] | Modal yes/no dialog with optional danger styling |
| [`ContextMenu`] | Right-click popup menu with nested submenus |
| [`DebugOverlay`] | FPS, widget count, and debug info overlay |
| [`EventLogger`] | Scrollable event log panel |
| [`Form`] | Multi-field form container with validation |
| [`Gauge`] | Filled progress bar with warn/crit thresholds |
| [`Hud`] | Top-right HUD with system metrics |
| [`KeyValueGrid`] | Key-value display from JSON/Scalar CLI output |
| [`Label`] | Static text label |
| [`List`] | Scrollable list with keyboard/touch navigation |
| [`LogViewer`] | Auto-scrolling log with severity detection |
| [`MenuBar`] | Top menu bar with dropdown menus |
| [`Modal`] | Modal dialog overlay with backdrop |
| [`PasswordInput`] | Single-line password input with masking |
| [`ProgressBar`] | Animated progress indicator |
| [`Profiler`] | Frame timing profiler with bar chart |
| [`Radio`] | Radio button group (single selection) |
| [`SearchInput`] | Text input with search/filter behavior |
| [`Select`] | Dropdown select/combobox |
| [`Slider`] | Horizontal slider with value display |
| [`Spinner`] | Animated loading spinner |
| [`SplitPane`] | Split view with draggable divider |
| [`StatusBadge`] | Colored OK/WARN/ERROR badge from CLI status |
| [`StatusBar`] | Bottom status bar |
| [`StreamingText`] | Live-updating text with word-wrap |
| [`TabBar`] | Tab navigation bar |
| [`Table`] | Multi-column table with sorting |
| [`Toast`] | Temporary notification toast messages |
| [`Toggle`] | Two-state on/off toggle switch |
| [`Tooltip`] | Hover tooltip overlay |
| [`Tree`] | Expandable/collapsible tree view |
| [`WidgetInspector`] | Widget tree inspector |

### Utilities
| Module | What |
|---|---|
| [`DirtyRegionTracker`] | Efficient partial screen updates |
| [`AnimationManager`] | Tweening animations with easing curves |
| [`Layout`] | Constraint-based layout engine (percentage, fixed, min, max, ratio) |
| [`Theme`] | 15 built-in themes |

### 15 Built-in Themes
`dark` · `light` · `cyberpunk` · `dracula` · `nord` · `catppuccin_mocha` · `gruvbox_dark` · `tokyo_night` · `solarized_dark` · `solarized_light` · `one_dark` · `rose_pine` · `kanagawa` · `everforest` · `monokai`

---

## Engine (Core)

The framework is built on these primitives — available directly when needed:

| Module | What |
|---|---|
| `compositor` | `Plane`, `Compositor`, `Cell`, `Color`, `Styles` — z-indexed layer rendering |
| `input::parser` | SGR mouse + chord parsing |
| `input::reader` | Non-blocking input reader with EINTR retry |
| `widgets::editor` | Code editor with syntax highlighting, undo, filter, multi-cursor |
| `widgets::input` | Text input widget |
| `integration::ratatui` | Ratatui bridge |
| `backend::tty` | Low-level terminal control |
| `visuals` | Icons, OSC commands (clipboard, hyperlink, bell, notify), sync mode |
| `system` | `SystemMonitor`, `SystemData`, `DiskInfo`, `ProcessInfo` |

---

## Installation

```toml
[dependencies]
dracon-terminal-engine = "27.0.5"
```

Or from git:

```toml
[dependencies]
dracon-terminal-engine = { git = "https://github.com/DraconDev/dracon-terminal-engine", tag = "v27.0.5" }
```

## Quick Start (Framework)

```rust
use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;

App::new().unwrap()
    .title("My App")
    .fps(30)
    .theme(Theme::cyberpunk())
    .on_tick(|_ctx, _tick| {
        // Called every 250ms by default
    })
    .run(|ctx| {
        let items = vec!["Home", "Projects", "Settings", "About"];
        let list = List::new(items);
        let (w, h) = ctx.compositor().size();
        let area = Rect::new(0, 0, w, h);
        ctx.add_plane(list.render(area));
    });
```

## Examples

```bash
# Framework examples — the recommended path
cargo run --example framework_demo        # App + List + Breadcrumbs + SplitPane + Hud + SystemMonitor
cargo run --example framework_file_manager # File browser with List + Breadcrumbs + SplitPane
cargo run --example framework_chat        # Chat UI: message list + input bar + theme
cargo run --example framework_widgets     # Showcase all 35 framework widgets
cargo run --example command_dashboard    # Command-driven dashboard with auto-refresh
cargo run --example from_toml            # TOML-driven app configuration
cargo run --example text_editor_demo      # Standalone TextEditor with theme switching

# Engine examples — raw compositor usage
cargo run --example basic_raw             # minimal Terminal usage
cargo run --example god_mode              # Ratatui + compositor overlay
cargo run --example input_debug           # SGR mouse + keyboard parsing
cargo run --example cyberpunk_dashboard   # Ratatui chart dashboard with simulation
cargo run --example demo                  # General engine demo with charts
cargo run --example desktop              # Desktop-style window management
cargo run --example game_loop             # 60fps game loop with mouse tracking
```

## Testing

```bash
# All tests (unit + integration)
cargo test

# Specific test suites
cargo test --lib               # Unit tests
cargo test --test phase1_widget_test    # Widget integration tests
cargo test --test theme_propagation_test # Theme propagation tests
cargo test --test scroll_test          # Scroll behavior tests
```

## Version

**v27.0.5** — See [CHANGELOG](CHANGELOG.md) for full history.

## License

MIT or Apache-2.0, at your option.
