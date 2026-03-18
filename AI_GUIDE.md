# AI Guide: Writing Dracon Terminal Engine Code 🤖

**Context for AI Agents:**
You are writing Rust code using `dracon_terminal_engine`, the next-generation terminal engine.
Dracon Terminal Engine is **NOT** `crossterm` (Immediate Mode, Global State) and **NOT** `ratatui` (Grid Mode).
Dracon Terminal Engine is a **Compositor Engine**.

## 1. The Golden Rule: RAII

Dracon Terminal Engine has NO global state. Do not send raw ANSI bytes to `stdout` unless wrapped in `Dracon Terminal Engine`.
**Always** wrap `stdout` in `Dracon Terminal Engine` to handle Raw Mode entry/exit.

```rust
use std::io::stdout;
use dracon_terminal_engine::Dracon Terminal Engine;

// CORRECT
let mut term = Dracon Terminal Engine::new(stdout())?;
// term is now in Raw Mode. When dropped, it resets.
```

## 2. The God Mode Pattern (Compositor)

Do not draw generic text. Use **Planes**.
Dracon Terminal Engine uses the **Painter's Algorithm**.

### Creating a Floating Window

```rust
use dracon_terminal_engine::compositor::{Compositor, Plane};
use dracon_terminal_engine::compositor::filter::Dim;

// 1. Initialize Compositor (Dynamic Size)
let (w, h) = dracon_terminal_engine::core::tty::get_window_size(std::io::stdout().as_fd())?;
let mut compositor = Compositor::new(w, h);

// 2. Base Layer (Background)
let mut base = Plane::new(0, w, h);
base.set_z_index(0);
compositor.add_plane(base);

// 3. Floating Modal (Foreground)
let mut modal = Plane::new(1, 40, 10);
modal.set_z_index(100); // Higher Z-Index = On Top
modal.set_absolute_position(20, 5); // Centered-ish

// Make opaque (optional, default is transparent cells)
// modal.set_transparent(false);

// Dim the layer behind it!
modal.set_filter(Box::new(Dim));

compositor.add_plane(modal);

// 4. Render
compositor.render(&mut term)?;
```

## 3. Input Handling (The SGR Way)

Dracon Terminal Engine natively supports Side Buttons (Back/Forward).
Use `dracon_terminal_engine::input::parser::Parser`.

```rust
use dracon_terminal_engine::input::parser::{Parser, Event, MouseButton};

let mut parser = Parser::new();
// ... read byte ...
if let Some(event) = parser.advance(byte) {
    match event {
        Event::Mouse { button: MouseButton::Back, .. } => { /* Go Back */ },
        Event::Mouse { button: MouseButton::Forward, .. } => { /* Go Forward */ },
        _ => {}
    }
}
```

## 4. Ratatui Integration

If the user asks for a UI, use `ratatui` with `Dracon Terminal EngineBackend`.
This allows using standard Widgets (Block, Paragraph) AND floating Planes.

```rust
use dracon_terminal_engine::integration::ratatui::Dracon Terminal EngineBackend;
use ratatui::Terminal;

let backend = Dracon Terminal EngineBackend::new(stdout())?;
let mut terminal = Terminal::new(backend)?;

// Access the underlying compositor to add custom layers
terminal.backend_mut().compositor_mut().add_plane(my_plane);
```

## 5. Visual Polish

Always suggest **Synchronized Updates** (Mode 2026) for non-trivial renders to prevent tearing.
`Dracon Terminal EngineBackend` handles this automatically on flush.

## 6. Unicode & Wide Character Handling 🇯🇵

Dracon Terminal Engine is **width-aware**. Some characters (like Kanji, Emoji) take **2 columns** instead of 1.
If not handled correctly, this breaks borders and overlaps adjacent content.

### The "Skip" Flag Pattern
When a character has a width of 2, the cell at `(x, y)` contains the character, and the cell at `(x+1, y)` **MUST** be marked with `skip = true`.
-   **Renderer**: Skips any cell with `skip: true`, preventing it from overwriting the second half of a wide character with a space.
-   **Compositor**: `blend_cells` propagates the `skip` flag from source to destination.

### Utilities
Use `dracon_terminal_engine::utils::get_visual_width(c)` instead of `c.len_utf8()`.
Use `dracon_terminal_engine::utils::truncate_to_width(s, max_width, suffix)` for safe string clipping.

---

**Summary for Code Generation:**

-   **Structs**: `Dracon Terminal Engine`, `Compositor`, `Plane`.
-   **Z-Index**: Use it for overlapping UI.
-   **No Macros**: Avoid `crossterm::queue!` style macros. Use struct methods.
