# AI Guide: Writing Dracon Terminal Engine Code

You are writing Rust code using `dracon_terminal_engine`, a terminal application **framework** — not a TUI library. One import and AI builds a complete app in minutes.

## The Two Layers

**Framework** (high-level, for building apps):
```rust
use dracon_terminal_engine::framework::prelude::*;

App::new()?
    .title("My App")
    .fps(30)
    .run(|ctx| {
        let list = List::new(vec!["Item 1", "Item 2", "Item 3"]);
        ctx.add_plane(list.render(Rect::new(0, 0, 40, 20)));
    });
```

**Engine** (low-level, for advanced use):
```rust
use dracon_terminal_engine::{Terminal, Plane, Compositor, Cell, Color};
let mut term = Terminal::new(stdout())?;
let mut plane = Plane::new(0, 40, 10);
plane.set_z_index(50);
```

---

## 1. The Golden Rule: RAII

No global state. **Always** wrap `stdout` in `Terminal` to handle Raw Mode entry/exit. When `Terminal` is dropped, terminal state is restored.

```rust
use dracon_terminal_engine::Terminal;
let mut term = Terminal::new(stdout())?;
// In raw mode. On drop, state is restored.
```

---

## 2. App — The Framework Entry Point

`App` owns terminal, compositor, input parsing, and the event loop. One call to `run()`:

```rust
App::new()?
    .title("My App")        // Sets terminal title
    .fps(30)                 // Frame rate limiter (default 30)
    .theme(Theme::dark())    // Or .light() or .cyberpunk()
    .run(|ctx| {
        // ctx.compositor() -> &Compositor
        // ctx.add_plane(plane) -> add a Plane to be rendered
        // ctx.theme() -> &Theme
        // ctx.fps() -> u64 (measured FPS, not target)
    });
```

`App::new()` returns `io::Result<App>` — handle the `?`.

---

## 3. Widgets

### List\<T\>

Vertical list with keyboard nav (Up/Down/Home/End/PageUp/PageDown) and mouse scroll. Requires `T: Clone + ToString`.

```rust
let items = vec!["Home", "Projects", "Settings", "About"];
let mut list = List::new(items);
list.set_visible_count(10);

let plane = list.render(Rect::new(0, 0, 40, 20));
ctx.add_plane(plane);

// Methods:
list.selected_index()   // usize
list.get_selected()     // Option<&T>
list.len()              // usize
list.on_select(|item| { /* called on Enter/click */ })
```

### Breadcrumbs

Clickable path segments. Constructor takes `Vec<String>`:

```rust
let crumbs = vec!["home".to_string(), "user".to_string(), "projects".to_string()];
let (plane, zones) = Breadcrumbs::new(crumbs).render(area);
// zones: Vec<HitZone<usize>> — index of clicked segment
```

Or from a `Path`:

```rust
let (plane, zones) = Breadcrumbs::from_path(path).render(area);
```

### SplitPane

H/V splits with ratio:

```rust
let split = SplitPane::new(Orientation::Vertical).ratio(0.7);
let (left_rect, right_rect) = split.split(Rect::new(0, 0, w, h));
```

### Hud

Floating overlay (z-indexed above main content):

```rust
let hud = Hud::new(100).with_size(30, 5);
let gauge = hud.render_gauge(x, y, "CPU", 45.0, 100.0, 20);
ctx.add_plane(gauge);
```

### TabBar, Modal, Table, ContextMenu

See `src/framework/widgets/` for all widgets.

---

## 4. HitZone — Declarative Interactive Regions

Hit zones track single/double/triple click, right-click, drag, and hover. T is your callback context type.

```rust
use dracon_terminal_engine::framework::hitzone::{HitZone, HitZoneGroup};

let zone = HitZone::new(id, x, y, w, h);
// Methods:
zone.contains(col, row)     // bool — was this point inside?
zone.handle_mouse(kind, col, row, modifiers)  // routes event to callbacks
```

`HitZone` accepts callbacks via builder pattern:

```rust
let zone = HitZone::new(id, x, y, w, h)
    .on_click(|kind| { /* ClickKind::Single/Double/Triple */ })
    .on_right_click(|| { /* right click */ })
    .on_drag_start(|state| { /* DragState has drag_delta() */ });
```

`HitZoneGroup` dispatches to the first matching zone by calling its `handle_mouse` method:

```rust
let mut group = HitZoneGroup::<usize>::new();
group.add(HitZone::new(0, 10, 5, 20, 1));  // id=0
group.add(HitZone::new(1, 35, 5, 20, 1));  // id=1

if let Some(id) = group.dispatch_mouse(kind, col, row, modifiers) {
    match id {
        0 => { /* first zone clicked */ }
        1 => { /* second zone clicked */ }
    }
}
```

---

## 5. Theme

Three presets: `Theme::dark()` (default), `Theme::light()`, `Theme::cyberpunk()`. Each provides bg/fg/accent/selection/border colors plus scrollbar/hover/active/input variants.

```rust
let theme = Theme::cyberpunk();
// Fields:
theme.bg, theme.fg, theme.accent,
theme.selection_bg, theme.selection_fg,
theme.border, theme.scrollbar, theme.hover, theme.active, theme.input
```

Apply to widgets via `.with_theme()`:

```rust
let list = List::new(items).with_theme(Theme::dark());
```

---

## 6. The Compositor Pattern (Engine-level)

Planes have z-indices. Higher z = on top. The compositor uses painter's algorithm.

```rust
use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};

let mut compositor = Compositor::new();

// Base Layer (z=0)
let mut base = Plane::new(0, 80, 24);
base.set_z_index(0);
compositor.add_plane(base);

// Floating Modal (z=100)
let mut modal = Plane::new(1, 40, 10);
modal.set_z_index(100);
modal.set_position(20, 5);
compositor.add_plane(modal);

// Render
let frame = compositor.render();
```

**Plane creation**: `Plane::new(id, width, height)` — id is your identifier, width/height are in cells.

---

## 7. Input Handling

### Framework (App event loop)

The framework passes events to your closure. You handle them via widget methods:

```rust
App::new()?.run(|ctx| {
    let mut list = List::new(items);
    // Framework calls your closure each frame.
    // Handle input via widget.handle_key() / widget.handle_mouse()
});
```

### Engine-level

```rust
use dracon_terminal_engine::input::{InputReader, Event, MouseButton};

let mut reader = InputReader::new(stdin())?;
if let Some(Event::Mouse(me)) = reader.read()? {
    match (me.button, me.modifiers) {
        (MouseButton::Back, _) => { /* go back */ }
        (MouseButton::Left, KeyModifiers::SHIFT) => { /* shift+click */ }
        _ => {}
    }
}
```

**SGR Mouse** supports: click, drag, scroll, extra buttons (Back/Forward), shift/ctrl modifiers.

---

## 8. Color

Use `Color::Rgb(r, g, b)` for 24-bit color or `Color::Ansi(n)` for 256-color.

```rust
use dracon_terminal_engine::compositor::Color;

let c = Color::Rgb(0, 255, 136);    // bright green
let c = Color::Ansi(39);            // cyan
let c = Color::Reset;                // reset to terminal default
```

---

## 9. Ratatui Integration

Use `RatatuiBackend` for ratatui widgets combined with floating Planes:

```rust
use dracon_terminal_engine::integration::ratatui::RatatuiBackend;
use ratatui::Terminal;

let backend = RatatuiBackend::new(stdout())?;
let mut terminal = Terminal::new(backend)?;

// Access compositor to add custom layers
terminal.backend_mut().compositor_mut().add_plane(my_plane);
```

Note: `RatatuiBackend` wraps the raw-mode Terminal. When `terminal` is dropped, raw mode exits. Use `RatatuiBackend::new(writer)?` directly (no separate `Terminal::new()` call).

---

## 10. Sync Mode 2026 (Visual Polish)

For tear-free rendering, wrap output in sync mode:

```rust
use dracon_terminal_engine::visuals::sync::{begin_sync, end_sync};

begin_sync(writer)?;
terminal.write_all(frame.as_bytes())?;
end_sync(writer)?;
```

---

## 11. Unicode & Wide Characters

Characters like Kanji and Emoji take 2 columns. Use utilities to stay safe:

- `dracon_terminal_engine::utils::get_visual_width(c)` — display width
- `dracon_terminal_engine::utils::truncate_to_width(s, max_width, suffix)` — safe clipping

When a wide char occupies `(x, y)`, cell `(x+1, y)` must have `skip: true`.

---

## 12. Re-exports

Most common types are re-exported at crate root:

```rust
// Framework
use dracon_terminal_engine::framework::prelude::*;  // App, Ctx, List, Breadcrumbs, SplitPane, Hud, HitZone, Theme, ...

// Engine
use dracon_terminal_engine::{Terminal, Plane, Compositor, Cell, Color, Styles};
use dracon_terminal_engine::{InputReader, Parser, SystemMonitor};
```

---

## Summary

| Concept | Key API |
|---|---|
| Entry point | `App::new()?.title().fps().run(\|ctx\|)` |
| Add content | `ctx.add_plane(plane)` |
| List | `List::new(items).on_select(f).render(area)` |
| Breadcrumbs | `Breadcrumbs::new(crumbs).render(area)` |
| Split | `SplitPane::new(Horizontal).ratio(0.3).split(area)` |
| Interactivity | `HitZone::new(id, x, y, w, h)` |
| Theme | `Theme::dark() / .light() / .cyberpunk()` |
| Colors | `Color::Rgb(r, g, b)` or `Color::Ansi(n)` |
| Z-indexed layers | `Plane::new(id, w, h)` + `set_z_index(n)` |
| Raw mode | `Terminal::new(stdout())?` |