```text
  _______   ______   .______      .___  ___.      ___
 |       | |   ___|  |   _  \     |   \/   |     /   \
 |.|   | | |  |__    |  |_)  |    |  \  /  |    /  ^  \
   |   |   |   __|   |      /     |  |\/|  |   /  /_\  \
   |   |   |  |____  |  |\  \----.|  |  |  |  /  _____  \
   |___|   |_______| | _| `._____||__|  |__| /__/     \__\

```

> **THE UNIVERSAL RUNTIME FOR THE SOVEREIGN INTERFACE.**

---

## ⚡ The Manifesto

The terminal is not a legacy artifact. It is the **Sovereign Interface**.

For decades, we have accepted TUI libraries that treat the terminal as a stream of text. They give you widgets. They give you print statements. They give you constraints.

**Terma gives you a Game Engine.**

We do not "print" to the screen. We **inhabit** it. Terma is a high-performance, z-indexed, event-driven runtime designed to build interfaces that feel closer to _Cyberpunk_ than _Curses_.

---

## � Core Architecture

### 1. The Compositor (Z-Index)

Stop thinking in "rows and columns." Think in **Layers**.
Terma implements a full composition engine. Spawn a `Plane`, set its Z-Index to 50, and float it above your application.

- **Layer 0**: Background / Wallpaper
- **Layer 10**: Main Application
- **Layer 100**: Modal Dialogs & Toasts
- **Layer 9000**: Debug Overlays

### 2. God-Tier Input

Standard terminals merge `TAB` and `Ctrl+I`. They can't tell `Ctrl+Shift+A` from `Ctrl+A`.
**Terma knows.**

- **Full Kitty Keyboard Protocol**: We detect chords, modifiers, and release events.
- **Discrete Mouse**: Tracking click, drag, scroll, and extra buttons (Side/Forward).

### 3. Visual Supremacy

- **Images**: Render high-res PNG/JPGs directly in the terminal (Kitty Protocol).
- **Procedural Geometry**: Draw rounded rectangles, circles, and gradients.
- **TrueColor**: 24-bit color support by default.

### 4. The Editor (Not just an Input)

The `TextEditor` widget is a power-user's dream:

- **Syntax Highlighting**: Powered by `syntect` with "Cyberpunk" & "GitHub" themes.
- **Smart Filters**: Live fuzzy-finding that narrows down content while preserving lines.
- **Unlimited Undo/Redo**: Because mistakes happen.
- **Multi-Selection**: Batch edits with Shift+Arrows.

---

## 📦 Installation

```toml
[dependencies]
terma = { git = "https://github.com/DraconDev/terma" }
```

---

## 💻 The Runtime

```rust
use terma::prelude::*;

fn main() -> Result<()> {
    // 1. Initialize the Engine
    let mut engine = Engine::new();

    // 2. Create a Floating Layer
    let mut hud = Plane::new(40, 10);
    hud.set_z_index(50);
    hud.set_style(Style::new().bg(Color::Rgb(20, 20, 25)).fg(Color::Cyan));
    hud.perimeter(Border::Neons);
    hud.put_str(2, 2, "SYSTEM ONLINE");

    // 3. Mount & Run
    engine.compositor_mut().add_plane(hud);
    engine.run_loop()?;

    Ok(())
}
```

---

## 🚀 System Boot Sequence

- [x] **Core Runtime**: Event Loop & TTY Management.
- [x] **Compositor**: Z-Indexed Rendering.
- [x] **Input**: Enhanced Keyboard & Mouse Protocols.
- [x] **Hyper-Editor**: Syntax Highlighting & Rich Editing.

---

> _"We are the ghost in the shell."_
>
> Built by **Dracon** for sovereign terminal systems.
