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

`dracon-terminal-engine` is a framework for building terminal applications. Not a TUI library — a complete runtime that owns the terminal, input, rendering, and event loop. Mouse-friendly, z-indexed planes, 41 built-in widgets, 21 themes, dirty rendering, and focus management.

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

## Framework (v29)

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

### 41 Framework Widgets
| Widget | What |
|---|---|
| [`Breadcrumbs`] | Hierarchical path display with clickable segments |
| [`Button`] | Clickable button with press state and callbacks |
| [`Checkbox`] | Two-state toggle with check mark |
| [`CommandPalette`] | Filterable command overlay with search |
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
| [`TextEditorAdapter`] | Adapter for integrating TextEditor widget |
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
| [`Theme`] | 20 built-in themes |

### App Architecture (New in v29)

**v29.11.0** — See [CHANGELOG](CHANGELOG.md) for full history.

## License

MIT or Apache-2.0, at your option.
