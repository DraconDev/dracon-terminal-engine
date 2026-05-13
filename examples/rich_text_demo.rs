//! RichText demo standalone example.

use dracon_terminal_engine::framework::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let q = should_quit.clone();

    App::new()?
        .title("RichText Demo")
        .fps(30)
        .theme(Theme::from_env_or(Theme::nord()))
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
                q.store(true, Ordering::SeqCst);
            }
            false
        })
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();
            let theme = ctx.theme();

            // Header
            let mut plane = Plane::new(0, w, h);
            for cell in plane.cells.iter_mut() {
                cell.bg = theme.bg;
                cell.fg = theme.fg;
                cell.transparent = false;
            }

            // Title
            let title = " RichText Demo ";
            for (i, ch) in title.chars().enumerate() {
                let idx = i;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = theme.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            // Divider
            for x in 0..w {
                let idx = (w + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = theme.outline;
                }
            }

            // Render markdown content
            let content = r#"# RichText Widget

A powerful **Markdown** renderer with *styling*.

## Features

- **Bold** and *italic* text
- `inline code` support
- [Links](https://example.com)
- Unordered lists

### Code Example

```rust
fn main() {
    println!("Hello!");
}
```"#;

            let rich_text = RichText::new(content).with_theme(theme);
            let content_area = Rect::new(2, 3, w.saturating_sub(4), h.saturating_sub(6));
            let content_plane = rich_text.render(content_area);

            // Blit content
            for i in 0..content_plane.cells.len() {
                let cell = &content_plane.cells[i];
                if cell.char == '\0' || cell.transparent { continue; }
                let row = i / content_plane.width as usize;
                let col = i % content_plane.width as usize;
                let dy = content_area.y as usize + row;
                let dx = content_area.x as usize + col;
                if dy >= h as usize || dx >= w as usize { continue; }
                let idx = dy * w as usize + dx;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell.clone();
                }
            }

            // Footer
            let footer_y = h.saturating_sub(2);
            for x in 0..w {
                let idx = (footer_y * w + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = theme.outline;
                }
            }

            let footer = " q: quit | F1: help | Ctrl+T: cycle theme ";
            for (i, ch) in footer.chars().enumerate() {
                let idx = (footer_y as usize * w as usize + 2 + i) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = theme.fg_muted;
                }
            }

            ctx.add_plane(plane);

            if should_quit.load(Ordering::SeqCst) {
                ctx.request_close();
            }
        })
}