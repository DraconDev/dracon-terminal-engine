# Migration Guide: 0.1.x → 1.0

This guide documents breaking changes when upgrading from Dracon Terminal Engine 0.1.x to 1.0.

## Table of Contents

- [Widget Trait Changes](#widget-trait-changes)
- [App Builder Changes](#app-builder-changes)
- [Theme Field Renames](#theme-field-renames)
- [Widget API Changes](#widget-api-changes)

---

## Widget Trait Changes

### `needs_render()` Signature

The `needs_render()` method now requires `&self` instead of `&mut self`:

```rust
// 0.1.x (broken)
fn needs_render(&mut self) -> bool;

// 1.0 (correct)
fn needs_render(&self) -> bool;
```

**Migration:** Remove `mut` from all `needs_render()` implementations. Store dirty state internally:

```rust
// 1.0 Example
struct MyWidget {
    dirty: bool,
}

impl Widget for MyWidget {
    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}
```

### `render()` Signature

The `render()` method signature is unchanged, but the return type changed:

```rust
// 0.1.x
fn render(&mut self, area: Rect) -> Plane;

// 1.0
fn render(&self, area: Rect) -> Plane;
```

**Migration:** Move any mutation logic to `handle_key()`, `handle_mouse()`, or `mark_dirty()`.

---

## App Builder Changes

### `on_tick` Callback Signature

The `on_tick` callback now receives `Ctx` instead of `&mut App`:

```rust
// 0.1.x (removed API)
app.on_tick(|app, tick| {
    let mut app = app.borrow_mut();
    app.tick();
});

// 1.0
app.on_tick(|ctx, tick| {
    let (w, h) = ctx.compositor().size();
    // use ctx.add_plane() for rendering
});
```

**Migration:** Use `Ctx.compositor().size()` for dimensions, `Ctx.add_plane()` for rendering.

### Removed Methods

The following `App` builder methods were removed:

| Removed | Replacement |
|---------|-------------|
| `app.handle_key(key)` | Handle keys in `handle_key()` widget method |
| `app.handle_mouse(kind, col, row)` | Handle mouse in `handle_mouse()` widget method |
| `app.set_theme(theme)` | Use `ctx.set_theme()` in callbacks |
| `app.theme()` | Use `ctx.theme()` in callbacks |
| `app.fps(fps)` | Use `App::new().fps(fps)` in builder |
| `app.last_frame_duration()` | Use `ctx.compositor()` for timing |

---

## Theme Field Renames

All theme colors were normalized for consistency:

| 0.1.x Name | 1.0 Name |
|------------|----------|
| `fg` | `fg` (unchanged) |
| `bg` | `bg` (unchanged) |
| `selection_bg` | `selection_bg` (unchanged) |
| `selection_fg` | `selection_fg` (unchanged) |
| `hover_bg` | `hover_bg` (unchanged) |
| `primary` | `primary` (unchanged) |
| `secondary` | `secondary` (unchanged) |
| `accent` | `accent` (unchanged) |
| `muted` | `muted` (unchanged) |
| `dim` | `dim` (unchanged) |
| `outline` | `outline` (unchanged) |
| `error` | `error` (unchanged) |
| `warning` | `warning` (unchanged) |
| `success` | `success` (unchanged) |
| `info` | `info` (unchanged) |
| `link` | `link` (unchanged) |
| `visited_link` | `visited_link` (unchanged) |
| `scrollbar_bg` | `scrollbar_bg` (unchanged) |
| `scrollbar_thumb` | `scrollbar_thumb` (unchanged) |
| `surface_elevated` | `surface_elevated` (unchanged) |
| `surface_flat` | `surface_flat` (unchanged) |

**Note:** Theme fields are unchanged in 1.0. This table documents that no renaming occurred.

---

## Widget API Changes

### List Widget

#### Selection Callback

```rust
// 0.1.x
list.on_select(|item| { /* ... */ });

// 1.0
list.on_select(|selected: &T| { /* ... */ });
```

The callback now receives a reference instead of consuming.

#### Multi-Selection

```rust
// 0.1.x
list.multi_select(true);
list.get_selected_items()

// 1.0
list.with_multi_select(true);
list.get_selected_items() // returns Vec<&T>
```

### Table Widget

#### Header Click Callback

```rust
// 0.1.x
table.on_header_click(|col| { /* ... */ });

// 1.0
table.on_header_click(|col: usize| { /* ... */ });
```

Column index is now 0-based and typed as `usize`.

### Tree Widget

#### Node Selection

```rust
// 0.1.x
tree.select_node(path);
tree.get_selected()

// 1.0
tree.set_selected_path(path);
tree.selected_path() // returns Option<Vec<usize>>
```

### SearchInput Widget

```rust
// 0.1.x
search.on_search(|query| { /* ... */ });

// 1.0
search.on_search(|query: &str| { /* ... */ });
```

Callback receives a reference to the query string.

### CommandPalette Widget

```rust
// 0.1.x
palette.on_execute(|cmd| { /* ... */ });

// 1.0
palette.on_execute(|cmd: &str| { /* ... */ });
```

Command ID is passed as `&str` reference.

---

## Event Handling Changes

### KeyEvent Structure

```rust
// 0.1.x
KeyEvent {
    code: KeyCode,
    kind: KeyEventKind,
    modifiers: KeyModifiers,
}

// 1.0 (unchanged)
KeyEvent {
    code: KeyCode,
    kind: KeyEventKind,
    modifiers: KeyModifiers,
}
```

No structural changes, but some `KeyCode` variants were renamed:

| 0.1.x | 1.0 |
|-------|-----|
| `KeyCode::Esc` | `KeyCode::Esc` (unchanged) |
| `KeyCode::Enter` | `KeyCode::Enter` (unchanged) |

### MouseEvent Handling

```rust
// 0.1.x
fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool;

// 1.0 (unchanged)
fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool;
```

No changes to mouse event handling signature.

---

## Dependency Changes

### Required Rust Version

- **0.1.x:** Rust 1.70+
- **1.0:** Rust 1.75+

### Removed Dependencies

The following optional dependencies were consolidated:

| Removed | Notes |
|---------|-------|
| `libc` feature flag | Always enabled on non-Windows |
| `tokio` feature | Renamed to `async` |

### New Dependencies

| Added | Version | Notes |
|-------|---------|-------|
| `unicode-segmentation` | 1.10+ | Required for grapheme clusters |

---

## Pattern Changes

### Pattern 1: Widget Trait (Recommended)

```rust
// 1.0 - Full widget pattern
use dracon_terminal_engine::prelude::*;
use ratatui::layout::Rect;

struct MyApp {
    theme: Theme,
    list: List<String>,
    dirty: bool,
}

impl Widget for MyApp {
    fn id(&self) -> WidgetId { WidgetId::new() }
    fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
    fn needs_render(&self) -> bool { self.dirty }
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        // ... render list
        plane
    }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.list.handle_key(key)
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.list.handle_mouse(kind, col, row)
    }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn on_theme_change(&mut self, theme: &Theme) { self.theme = *theme; }
}
```

### Pattern 2: Closure-Based (App Callbacks)

```rust
// 1.0 - Closure-based pattern
use dracon_terminal_engine::prelude::*;

App::new().unwrap()
    .title("My App")
    .fps(30)
    .on_tick(|ctx, _tick| {
        let (w, h) = ctx.compositor().size();
        let mut plane = Plane::new(0, w, h);
        plane.fill_bg(ctx.theme().bg);
        // ... render content
        ctx.add_plane(plane);
    })
    .on_input(|key| {
        // handle input
        false
    })
    .run();
```

---

## Deprecation Notes

The following APIs are deprecated but still functional:

### Deprecated: Direct `App` State Access

```rust
// Deprecated (still works, but ctx.theme() preferred)
let theme = app.theme();

// Preferred
let theme = ctx.theme();
```

### Deprecated: Manual Plane Management

```rust
// Deprecated (still works)
let mut app = app.borrow_mut();
app.set_theme(new_theme);

// Preferred
ctx.set_theme(new_theme);
```

---

## Getting Help

If you encounter issues not covered by this guide:

1. Check the [examples directory](./examples/) for working code patterns
2. Review the [API documentation](https://docs.rs/dracon-terminal-engine)
3. Open an issue on [GitHub](https://github.com/DraconDev/dracon-terminal-engine)