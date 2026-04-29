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

App::new()?
    .title("My App")
    .fps(30)
    .run(|ctx| {
        ctx.split_h(|left, right| {
            left.list(vec!["Files", "Search", "Git"], |item| { });
            right.text("Hello, world!");
        });
    });
```

---

## Framework (v25)

The `framework` module provides the complete application runtime:

| Widget | What |
|---|---|
| [`App`] | Event loop, terminal, compositor — one call to run |
| [`Ctx`] | Per-frame context: add planes, access compositor/theme/FPS |
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
| [`ScrollContainer`] | Scrolleable container with offset management |
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
dracon-terminal-engine = { git = "https://github.com/DraconDev/dracon-terminal-engine" }
```

## Quick Start (Framework)

```rust
use dracon_terminal_engine::framework::prelude::*;

App::new()?
    .title("My App")
    .fps(30)
    .run(|ctx| {
        let items = vec!["Home", "Projects", "Settings", "About"];
        let mut list = List::new(items).on_select(|item| {
            println!("selected: {item}");
        });
        let (w, _) = ctx.compositor().size();
        let plane = list.render(Rect::new(0, 0, w, 10));
        ctx.add_plane(plane);
    });
```

## Quick Start (Engine-level)

```rust
use dracon_terminal_engine::compositor::{Color, Plane, Styles};
use dracon_terminal_engine::Terminal;

let mut terminal = Terminal::new(std::io::stdout())?;
let mut hud = Plane::new(0, 40, 10);
hud.set_z_index(50);

let cell = Cell {
    char: ' ',
    fg: Color::Rgb(0, 255, 136),
    bg: Color::Rgb(0, 30, 20),
    style: Styles::BOLD,
    transparent: false,
    skip: false,
};
hud.fill(cell);
hud.put_str(1, 1, "SYSTEM ONLINE");
terminal.write_all(hud.render().as_bytes())?;
std::thread::sleep(std::time::Duration::from_secs(2));
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

**v26.0.0** — See [CHANGELOG](CHANGELOG.md) for full history.

## License

MIT