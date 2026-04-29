# AI Guide: Writing Dracon Terminal Engine Code

You are writing Rust code using `dracon_terminal_engine`, the next-generation terminal engine.
Dracon Terminal Engine is **NOT** `crossterm` (Immediate Mode, Global State) and **NOT** `ratatui` (Grid Mode).
It is a **Compositor Engine** — z-indexed layers, TrueColor, SGR mouse.

## 1. The Golden Rule: RAII

Dracon Terminal Engine has NO global state. Do not send raw ANSI bytes to `stdout` unless wrapped in `Terminal`.
**Always** wrap `stdout` in `Terminal` to handle Raw Mode entry/exit.

```rust
use std::io::stdout;
use dracon_terminal_engine::core::terminal::Terminal;

let mut term = Terminal::new(stdout())?;
// term is now in Raw Mode. When dropped, it restores terminal state.
```

## 2. The Compositor Pattern (Layers)

Do not draw generic text. Use **Planes** with z-indices.
The compositor uses the **Painter's Algorithm** (higher z-index = on top).

### Creating a Floating Window

```rust
use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};
use dracon_terminal_engine::compositor::filter::Dim;

let mut compositor = Compositor::new();

// Base Layer (Background)
let mut base = Plane::new(0, 80, 24);
base.set_z_index(0);
compositor.add_plane(base);

// Floating Modal (Foreground)
let mut modal = Plane::new(1, 40, 10);
modal.set_z_index(100);
modal.set_position(20, 5);
modal.set_filter(Box::new(Dim));
compositor.add_plane(modal);

// Render
let frame = compositor.render();
```

## 3. Input Handling

Supports SGR Mouse (including side buttons Back/Forward, shift/ctrl modifiers).
Use `dracon_terminal_engine::input::{InputReader, Parser}`.

```rust
use dracon_terminal_engine::input::{InputEvent, InputReader};
use std::io::stdin;

let mut reader = InputReader::new(stdin())?;
if let Some(InputEvent::Mouse(me)) = reader.read()? {
    match (me.button, me.modifiers) {
        (MouseButton::Back, _) => { /* Go Back */ }
        (MouseButton::Forward, _) => { /* Go Forward */ }
        _ => {}
    }
}
```

## 4. Ratatui Integration

Use `ratatui` with `RatatuiBackend` for standard widgets (Block, Paragraph) combined with floating Planes.

```rust
use dracon_terminal_engine::integration::ratatui::RatatuiBackend;
use ratatui::Terminal;

let backend = RatatuiBackend::new(stdout())?;
let mut terminal = Terminal::new(backend)?;

// Access the underlying compositor to add custom layers
terminal.backend_mut().compositor_mut().add_plane(my_plane);
```

## 5. Visual Polish

Use **Synchronized Updates** (Mode 2026) for non-trivial renders to prevent tearing.
Call `visuals::sync::begin_sync()` before and `end_sync()` after rendering.

```rust
use dracon_terminal_engine::visuals::sync::{begin_sync, end_sync};

begin_sync(writer)?;
terminal.write_all(frame.as_bytes())?;
end_sync(writer)?;
```

## 6. Unicode & Wide Character Handling

Dracon Terminal Engine is **width-aware**. Characters like Kanji and Emoji take **2 columns**.
If not handled correctly, this breaks borders and overlaps adjacent content.

### The "Skip" Flag Pattern
When a character has width 2, cell `(x, y)` contains the character, and cell `(x+1, y)` **MUST** be marked `skip = true`.
- **Renderer**: Skips cells with `skip: true`
- **Compositor**: `blend_cells` propagates the `skip` flag

### Utilities
- `dracon_terminal_engine::utils::get_visual_width(c)` — character display width
- `dracon_terminal_engine::utils::truncate_to_width(s, max_width, suffix)` — safe string clipping

## Re-exports

The crate re-exports the most common types at the top level for convenience:
```rust
use dracon_terminal_engine::{Terminal, Plane, Compositor, Cell, Color, Styles};
use dracon_terminal_engine::{InputReader, Parser};
```

## Summary

- **Structs**: `Terminal`, `Compositor`, `Plane`, `Cell`
- **Backend**: `RatatuiBackend` for ratatui integration
- **Z-Index**: Use it for overlapping UI
- **No Macros**: Use struct methods, not `crossterm::queue!` style