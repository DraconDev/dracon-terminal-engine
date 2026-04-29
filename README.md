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
use ratatui::layout::Rect;

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

## Framework (v26)

The `framework` module provides the complete application runtime:

| Widget | What |
|---|---|
| [`App`] | Event loop, terminal, compositor — one call to run |
| [`Ctx`] | Per-frame context: add planes, access compositor/theme/FPS |
| [`App::on_tick`] | Periodic callback (every N milliseconds) |
| [`App::tick_interval`] | Set the tick interval in ms |
| [`Ctx::split_h`] | Horizontal split into two panes |
| [`Ctx::split_v`] | Vertical split into two panes |
| [`List<T>`] | Vertical list with keyboard nav + mouse scroll + selection |
| [`Table<T>`] | Sortable table with column headers + row click |
| [`TabBar`] | Horizontal tab strip, click or arrow-key to switch |
| [`Breadcrumbs`] | Clickable path segments (from `Path` or `Vec<String>`) |
| [`SplitPane`] | H/V splits with drag-resize divider |
| [`Modal`] | Auto-centered popup with button hit zones |
| [`ContextMenu`] | Right-click popup menu |
| [`Hud`] | Floating layer (z-indexed overlay) with gauge/text |
| [`HitZone<T>`] | Declarative interactive region (click/double/drag/hover) |
| [`HitZoneGroup<T>`] | Batch of hit zones, auto-dispatched |
| [`ScopedZone<T>`] | Lightweight geometry-only zone for per-frame dispatch |
| [`ScopedZoneRegistry<T>`] | Registry that clears per frame |
| [`DragManager<T>`] | Drag-and-drop state machine with ghost rendering |
| [`ScrollContainer`] | Scrollable container with offset management + scrollbar |
| [`Theme`] | Dark / light / cyberpunk presets |

---

## Engine (Core)

The framework is built on these primitives — available directly when needed:

| Module | What |
|---|---|
| `compositor` | `Plane`, `Compositor`, `Cell`, `Color`, `Styles` — z-indexed layer rendering |
| `input::parser` | SGR mouse + chord parsing |
| `input::reader` | Non-blocking input reader with EINTR retry |
| `widgets::editor` | Code editor with syntax highlighting |
| `widgets::input` | Text input widget |
| `integration::ratatui` | Ratatui bridge |
| `backend::tty` | Low-level terminal control |
| `visuals` | Icons, OSC commands (clipboard, hyperlink, bell, notify), sync mode |
| `system` | `SystemMonitor`, `SystemData`, `DiskInfo`, `ProcessInfo` |

---

## Installation

```toml
[dependencies]
dracon-terminal-engine = "26.0.1"
```

Or from git:

```toml
[dependencies]
dracon-terminal-engine = { git = "https://github.com/DraconDev/dracon-terminal-engine", tag = "v26.0.1" }
```

## Quick Start (Framework)

```rust
use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;

App::new().unwrap()
    .title("My App")
    .fps(30)
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

# Engine examples — raw compositor usage
cargo run --example basic_raw             # minimal Terminal usage
cargo run --example god_mode              # Ratatui + compositor overlay
cargo run --example input_debug           # SGR mouse + keyboard parsing
```

## Version

**v26.0.1** — See [CHANGELOG](CHANGELOG.md) for full history.

## License

MIT