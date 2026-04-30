```text
  _______   ______   .______      .___ ___.      ___
 |       | |   ___|  |   _  \     |   \/   |     /   \
 |.|   | | |  |__    |  |_)  |    |  \  /  |    /  ^  \
   |   |   |   __|   |      /     |  |\/|  |   /  /_\  \
   |   |   |  |____  |  |\  \----.|  |  |  |  /  _____  \
   |___|   |_______| | _| `._____||__|  |__| /__/     \__\

```

> **A terminal application framework for Rust — one import, AI builds a complete app.**

---

## What It Is

`dracon-terminal-engine` is a framework for building terminal applications. Not a TUI library — a complete runtime that owns the terminal, input, rendering, and event loop.

**One import to rule them all:**

```rust
use dracon_terminal_engine::framework::prelude::*;

App::new().unwrap()
    .title("My App")
    .fps(30)
    .on_tick(|ctx, tick| { /* called every 250ms */ })
    .run(|ctx| {
        let (w, h) = ctx.compositor().size();
        let area = Rect::new(0, 0, w, h);
        let list = List::new(vec!["Files", "Search", "Git"]);
        ctx.add_plane(list.render(area));
    });
```

---

## Framework (v27)

The `framework` module provides the complete application runtime:

### Core
| Widget | What |
|---|---|
| [`App`] | Event loop, terminal, compositor — one call to run |
| [`Ctx`] | Per-frame context: add planes, compositor, theme, animations, dirty tracking |
| [`App::add_widget`] | Register a widget with lifecycle callbacks |
| [`App::set_theme`] | Switch theme and propagate to all widgets |
| [`App::on_tick`] | Periodic callback (every N milliseconds) |
| [`App::tick_interval`] | Set the tick interval in ms |

### Widgets
| Widget | What |
|---|---|
| [`List<T>`] | Virtual list with keyboard nav, mouse scroll, selection |
| [`Table<T>`] | Sortable table with column headers + row click |
| [`TabBar`] | Horizontal tab strip, click or arrow-key to switch |
| [`Breadcrumbs`] | Clickable path segments |
| [`SplitPane`] | H/V splits with drag-resize divider |
| [`Modal`] | Auto-centered popup with button hit zones |
| [`ContextMenu`] | Right-click popup menu |
| [`Hud`] | Floating layer (z-indexed overlay) with gauge/text |
| [`Button`] | Clickable button with press state |
| [`Checkbox`] | Toggle checkbox with label |
| [`Toggle`] | On/off toggle switch |
| [`Radio`] | Radio button group |
| [`Select`] | Dropdown select list |
| [`Slider`] | Horizontal value slider |
| [`ProgressBar`] | Animated progress bar |
| [`Spinner`] | Animated loading indicator |
| [`Label`] | Static text label |
| [`StatusBar`] | Status bar with segments |
| [`TextEditorAdapter`] | Full TextEditor wrapped as a framework widget |
| [`TextInputBase`] | Shared text input state |
| [`SearchInput`] | Search box with clear button |
| [`PasswordInput`] | Password input with masked characters |
| [`Tree`] | Recursive tree with expand/collapse |
| [`Form`] | Form with labeled fields |
| [`MenuBar`] | Horizontal menu bar |
| [`Toast`] | Auto-dismiss notification |
| [`Tooltip`] | Hover tooltip |
| [`EventLogger`] | Debug event log display |
| [`Profiler`] | Performance metrics display |
| [`DebugOverlay`] | Debug overlay |
| [`WidgetInspector`] | Widget tree inspector |

### Utilities
| Module | What |
|---|---|
| [`HitZone<T>`] | Declarative interactive region (click/double/drag/hover) |
| [`HitZoneGroup<T>`] | Batch of hit zones, auto-dispatched |
| [`ScopedZone<T>`] | Lightweight geometry-only zone for per-frame dispatch |
| [`ScopedZoneRegistry<T>`] | Registry that clears per frame |
| [`DragManager<T>`] | Drag-and-drop state machine with ghost rendering |
| [`ScrollContainer`] | Scrollable container with offset management + scrollbar |
| [`FocusManager`] | Tab-order focus ring with keyboard navigation |
| [`DirtyRegionTracker`] | Efficient partial screen updates |
| [`AnimationManager`] | Tweening animations with easing curves |
| [`Layout`] | Constraint-based layout engine |
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
dracon-terminal-engine = "27.0.0"
```

Or from git:

```toml
[dependencies]
dracon-terminal-engine = { git = "https://github.com/DraconDev/dracon-terminal-engine", tag = "v27.0.0" }
```

## Quick Start (Framework)

```rust
use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;

App::new().unwrap()
    .title("My App")
    .fps(30)
    .theme(Theme::cyberpunk())
    .on_tick(|ctx, tick| {
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
cargo run --example framework_widgets     # Showcase all 23+ framework widgets
cargo run --example text_editor_demo      # Standalone TextEditor with theme switching

# Engine examples — raw compositor usage
cargo run --example basic_raw             # minimal Terminal usage
cargo run --example god_mode              # Ratatui + compositor overlay
cargo run --example input_debug           # SGR mouse + keyboard parsing
```

## Version

**v27.0.0** — See [CHANGELOG](CHANGELOG.md) for full history.

## License

MIT
