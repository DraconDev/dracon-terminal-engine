```text
  _______   ______   .______      .___  ___.      ___
 |       | |   ___|  |   _  \     |   \/   |     /   \
 |.|   | | |  |__    |  |_)  |    |  \  /  |    /  ^  \
   |   |   |   __|   |      /     |  |\/|  |   /  /_\  \
   |   |   |  |____  |  |\  \----.|  |  |  |  /  _____  \
   |___|   |_______| | _| `._____||__|  |__| /__/     \__\

```

> **A terminal compositor engine for Rust.**

---

## What It Is

`dracon-terminal-engine` is a z-indexed, event-driven terminal runtime. Not a "TUI library" — an engine that owns the terminal, renders compositing layers, parses advanced input protocols, and ships with built-in widgets.

**Self-contained.** Input parsing and visual filters are baked in — no external contract crates needed.

---

## Core

### 1. Compositor (Z-Indexed Layers)

Think in **layers**, not rows/columns. Spawn a `Plane`, set its Z-Index, float it above your app.

- **Layer 0**: Background / wallpaper
- **Layer 10**: Main application
- **Layer 100**: Modal dialogs & toasts
- **Layer 9000**: Debug overlays

### 2. Input

- **SGR Mouse**: Click, drag, scroll, extra buttons (Button4/5, Shift+Click, Ctrl+Click)
- **Chord support**: Alt+2, Ctrl+Shift+Arrow, etc.
- **Contract types**: `InputEvent`, `KeyCode`, `KeyEvent`, `MouseEvent` — all in `input::mapping`

### 3. Visuals

- **TrueColor**: 24-bit ANSI SGR color support
- **Visual filters**: Dim, Invert, Scanline, Pulse, Glitch
- **Synchronized output**: Terminal mode 2026 for tear-free rendering

### 4. Editor Widget

- **Syntax highlighting**: `syntect` with built-in themes
- **Smart filters**: Live fuzzy-finding
- **Unlimited undo/redo**
- **Multi-selection**: Shift+Arrows batch edits

### 5. Ratatui Bridge

Drop-in `ratatui` integration via `integration::ratatui`.

---

## Installation

```toml
[dependencies]
dracon-terminal-engine = { git = "https://github.com/DraconDev/dracon-terminal-engine", tag = "v19.2.2" }
```

## Quick Start

```rust
use dracon_terminal_engine::core::terminal::Terminal;
use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};

fn main() -> std::io::Result<()> {
    let mut terminal = Terminal::new(std::io::stdout())?;

    // Create a floating layer (id=0, width=40, height=10 at 0,0)
    let mut hud = Plane::new(0, 40, 10);
    hud.set_z_index(50);

    let cell = Cell {
        char: 'S',
        fg: Color::Rgb(0, 255, 136),
        bg: Color::Reset,
        style: Styles::BOLD,
        transparent: false,
        skip: false,
    };
    hud.put_cell(2, 2, cell);
    hud.put_str(2, 3, "SYSTEM ONLINE");

    terminal.write_all(hud.render().as_bytes())?;
    terminal.flush()?;

    // Keep terminal alive (Drop restores state on drop)
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}
```

## End-to-End Loop

```rust
use dracon_terminal_engine::core::terminal::Terminal;
use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};
use dracon_terminal_engine::input::{InputEvent, InputReader};
use std::io::{stdin, stdout};

fn main() -> std::io::Result<()> {
    let mut terminal = Terminal::new(stdout())?;
    let mut reader = InputReader::new(stdin())?;
    let mut compositor = Compositor::new();

    // Layer 10: main content
    let mut app = Plane::new(0, 80, 24);
    app.set_z_index(10);
    app.put_str(0, 0, "Dracon Terminal Engine v19.2.2");
    compositor.add_plane(app);

    // Layer 50: HUD floating above the app
    let mut hud = Plane::new(1, 20, 3);
    hud.set_z_index(50);
    let banner = Cell {
        char: ' ',
        fg: Color::Rgb(0, 255, 136),
        bg: Color::Rgb(0, 30, 20),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    hud.fill(banner);
    hud.put_str(1, 1, "HUD ACTIVE");
    compositor.add_plane(hud);

    loop {
        // Render composited frame
        let frame = compositor.render();
        terminal.write_all(frame.as_bytes())?;
        terminal.flush()?;

        // Check for quit signal
        if let Some(InputEvent::Key(key)) = reader.read()? {
            if key.code == 3 {
                // Ctrl+C
                break;
            }
        }
    }

    Ok(())
}
```

## Modules

| Module | What |
|---|---|
| `core::terminal` | RAII terminal wrapper (raw mode, alt screen, cleanup) |
| `compositor` | Z-indexed layer engine (`Plane`, `Compositor`, `Cell`, `Color`, `Styles`) |
| `input::parser` | SGR mouse + chord parsing |
| `input::reader` | Non-blocking input reader with EINTR retry |
| `widgets::editor` | Code editor with syntax highlighting |
| `widgets::input` | Text input widget |
| `integration::ratatui` | Ratatui bridge |
| `backend::tty` | Low-level terminal control |
| `visuals` | Icons, OSC commands (clipboard, hyperlink, bell, notify), sync mode |
| `system` | System metrics (CPU, memory, disks, processes) |

## Version

**v19.2.2** — See [CHANGELOG](CHANGELOG.md) for full history.

## License

MIT