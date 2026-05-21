# Research Tasks — Dracon Terminal Engine

Created from full code research (2026-05-21)

---

## High Priority

### FN-082: Replace input parser panic!() calls
**Files:** `src/input/parser.rs`, `src/input/mapping.rs`  
**Panics:** 4 locations  
**Fix:** Return `None` or log warning instead of panicking

```rust
// Current (panic):
panic!("Did not parse SGR Back Button event");

// Option 1: Return None
fn parse_back_button(...) -> Option<MouseEvent> {
    None  // Unknown button
}

// Option 2: Log warning
eprintln!("Unknown SGR Back Button: {:?}", raw_data);
```

### FN-083: Widget trait sub-trait blanket implementations
**Breaking change for 0.2.0**  
**Sub-traits:** `Renderable`, `Focusable`, `Themable`  
**All 50+ widgets need zero changes with blanket impls**

```rust
// Blanket impl needed:
impl<T: Widget> Renderable for T {}

impl<T: Widget> Focusable for T {}

impl<T: Widget> Themable for T {}
```

---

## Medium Priority

### FN-084: Structured logging with tracing crate
**Feature:** `#[cfg(feature = "tracing")]`  
**Hot paths:** input parsing, plane blitting, render cycles

```rust
#[cfg(feature = "tracing")]
use tracing::{info, warn, error, instrument};

#[instrument(skip_all, fields(event_kind = ?key.code))]
fn handle_key(&mut self, key: KeyEvent) -> bool { ... }
```

### FN-085: UI snapshot tests with insta
**Tool:** `insta` crate (in dev-dependencies)  
**Targets:** 10 core widgets (Button, Checkbox, List, Table, Tree, etc.)  
**Macro:** `insta::assert_snapshot!()`

```rust
#[test]
fn test_button_render() {
    let btn = Button::new(\"Click me\");
    let plane = btn.render(Rect::new(0, 0, 20, 3));
    insta::assert_snapshot!(plane_to_string(&plane));
}
```

---

## Low Priority

### FN-086: Check transitive dependency updates
**Command:** `cargo outdated`  
**Note:** Some updates may require code changes

### FN-087: Add runtime metrics collection
**Areas:** frame timing, event loop latency, widget render counts, CellPool usage

```rust
static RENDER_COUNT: AtomicU64 = AtomicU64::new(0);

fn render(&self, area: Rect) -> Plane {
    RENDER_COUNT.fetch_add(1, Ordering::Relaxed);
    // ...
}
```

### FN-088: Add missing widget doc examples
**Target widgets:** tooltip, divider, label, event_logger, hud

```rust
/// # Example
/// ```
/// let tooltip = Tooltip::new(\"Hover text\");
/// ```
```

---

## Status Summary

| Priority | Tasks | Status |
|----------|-------|--------|
| High | 2 | TODO |
| Medium | 2 | TODO |
| Low | 3 | TODO |
| **Total** | **7** | |

---

*Tasks tracked in Fusion board (FN-082 through FN-088)*