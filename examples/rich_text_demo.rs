//! RichText demo standalone example.

use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> std::io::Result<()> {
    let kb_config = resolve_keybindings();
    let keybindings = KeybindingSet::from_config(&kb_config);
    let should_quit = Arc::new(AtomicBool::new(false));
    let q = should_quit.clone();
    let show_help = Arc::new(AtomicBool::new(false));
    let show_help_input = Arc::clone(&show_help);
    let show_help_render = Arc::clone(&show_help);

    let mut app = App::new()?
        .title("RichText Demo")
        .fps(30)
        .set_theme(Theme::from_env_or(Theme::nord()));
    app.on_input(move |key| {
        if keybindings.matches(actions::QUIT, &key) {
            q.store(true, Ordering::SeqCst);
            return true;
        }
        if keybindings.matches(actions::HELP, &key) {
            show_help_input.store(!show_help_input.load(Ordering::SeqCst), Ordering::SeqCst);
            return true;
        }
        if keybindings.matches(actions::THEME, &key) {
            return true;
        }
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Esc if show_help_input.load(Ordering::SeqCst) => {
                show_help_input.store(false, Ordering::SeqCst);
                true
            }
            _ => false,
        }
    })
    .on_tick(move |ctx, _| {
        if should_quit.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(move |ctx| {
        let (w, h) = ctx.compositor().size();
        let theme = ctx.theme().clone();

        let mut plane = Plane::new(0, w, h);
        for cell in plane.cells.iter_mut() {
            cell.bg = theme.bg;
            cell.fg = theme.fg;
            cell.transparent = false;
        }

        let title = " RichText Demo ";
        for (i, ch) in title.chars().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        for x in 0..w {
            let idx = (w + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = theme.outline;
            }
        }

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

        let rich_text = RichText::new(content).with_theme(theme.clone());
        let content_area = Rect::new(2, 3, w.saturating_sub(4), h.saturating_sub(6));
        let content_plane = rich_text.render(content_area);

        for i in 0..content_plane.cells.len() {
            let cell = &content_plane.cells[i];
            if cell.char == '\0' || cell.transparent {
                continue;
            }
            let row = i / content_plane.width as usize;
            let col = i % content_plane.width as usize;
            let dy = content_area.y as usize + row;
            let dx = content_area.x as usize + col;
            if dy >= h as usize || dx >= w as usize {
                continue;
            }
            let idx = dy * w as usize + dx;
            if idx < plane.cells.len() {
                plane.cells[idx] = *cell;
            }
        }

        let footer_y = h.saturating_sub(2);
        for x in 0..w {
            let idx = (footer_y * w + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = theme.outline;
            }
        }

        let footer = " Ctrl+Q: quit | F1: help | Ctrl+T: cycle theme ";
        for (i, ch) in footer.chars().enumerate() {
            let idx = footer_y as usize * w as usize + 2 + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = theme.fg_muted;
            }
        }

        if show_help_render.load(Ordering::SeqCst) {
            let hw = 42u16.min(w.saturating_sub(4));
            let hh = 10u16.min(h.saturating_sub(4));
            let hx = (w - hw) / 2;
            let hy = (h - hh) / 2;

            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * w + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = theme.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            for x in hx + 1..hx + hw - 1 {
                let top = (hy * w + x) as usize;
                let bot = ((hy + hh - 1) * w + x) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = theme.outline;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = theme.outline;
                }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * w + hx) as usize;
                let right = (y * w + hx + hw - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = theme.outline;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = theme.outline;
                }
            }
            let corners = [
                ('╭', hx, hy),
                ('╮', hx + hw - 1, hy),
                ('╰', hx, hy + hh - 1),
                ('╯', hx + hw - 1, hy + hh - 1),
            ];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * w + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = theme.outline;
                }
            }

            let help_title = "RichText Demo Help";
            let tx = hx + (hw - help_title.len() as u16) / 2;
            for (i, c) in help_title.chars().enumerate() {
                let idx = ((hy + 1) * w + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = theme.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            let shortcuts = [
                ("Ctrl+T", "Cycle theme"),
                ("F1 / ?", "Toggle help"),
                ("Esc", "Dismiss help"),
                ("Ctrl+Q", "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * w + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = theme.primary;
                    }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * w + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = theme.fg;
                    }
                }
            }
        }

        ctx.add_plane(plane);
    })
}
