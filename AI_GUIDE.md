# AI Guide: Writing Terma Code 🤖

**Context for AI Agents:**
You are writing Rust code using `terma`, the next-generation terminal engine.
Terma is **NOT** `crossterm` (Immediate Mode, Global State) and **NOT** `ratatui` (Grid Mode).
Terma is a **Compositor Engine**.

## 1. The Golden Rule: RAII

Terma has NO global state. Do not send raw ANSI bytes to `stdout` unless wrapped in `Terma`.
**Always** wrap `stdout` in `Terma` to handle Raw Mode entry/exit.

```rust
use std::io::stdout;
use terma::Terma;

// CORRECT
let mut term = Terma::new(stdout())?;
// term is now in Raw Mode. When dropped, it resets.
```

## 2. The God Mode Pattern (Compositor)

Do not draw generic text. Use **Planes**.
Terma uses the **Painter's Algorithm**.

### Creating a Floating Window

```rust
use terma::compositor::{Compositor, Plane};
use terma::compositor::filter::Dim;

// 1. Initialize Compositor (Dynamic Size)
let (w, h) = terma::core::tty::get_window_size(std::io::stdout().as_fd())?;
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

Terma natively supports Side Buttons (Back/Forward).
Use `terma::input::parser::Parser`.

```rust
use terma::input::parser::{Parser, Event, MouseButton};

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

If the user asks for a UI, use `ratatui` with `TermaBackend`.
This allows using standard Widgets (Block, Paragraph) AND floating Planes.

```rust
use terma::integration::ratatui::TermaBackend;
use ratatui::Terminal;

let backend = TermaBackend::new(stdout())?;
let mut terminal = Terminal::new(backend)?;

// Access the underlying compositor to add custom layers
terminal.backend_mut().compositor_mut().add_plane(my_plane);
```

## 5. Visual Polish

Always suggest **Synchronized Updates** (Mode 2026) for non-trivial renders to prevent tearing.
`TermaBackend` handles this automatically on flush.

## 6. Unicode & Wide Character Handling 🇯🇵

Terma is **width-aware**. Some characters (like Kanji, Emoji) take **2 columns** instead of 1.
If not handled correctly, this breaks borders and overlaps adjacent content.

### The "Skip" Flag Pattern
When a character has a width of 2, the cell at `(x, y)` contains the character, and the cell at `(x+1, y)` **MUST** be marked with `skip = true`.
-   **Renderer**: Skips any cell with `skip: true`, preventing it from overwriting the second half of a wide character with a space.
-   **Compositor**: `blend_cells` propagates the `skip` flag from source to destination.

### Utilities
Use `terma::utils::get_visual_width(c)` instead of `c.len_utf8()`.
Use `terma::utils::truncate_to_width(s, max_width, suffix)` for safe string clipping.

---

**Summary for Code Generation:**

-   **Structs**: `Terma`, `Compositor`, `Plane`.
-   **Z-Index**: Use it for overlapping UI.
-   **No Macros**: Avoid `crossterm::queue!` style macros. Use struct methods.
