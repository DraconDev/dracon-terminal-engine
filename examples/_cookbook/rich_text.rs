use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let quit = Rc::clone(&should_quit);
    let theme = Theme::from_env_or(Theme::nord());

    let markdown = r#"# Rich Text / Markdown Demo

This widget renders **styled text** with *minimal* Markdown support.

## Features

- **Headers** — like the one above
- **Bold text** — using double asterisks
- *Italic text* — using single asterisks or underscores
- `Inline code` — using backticks
- [Links](https://github.com) — click-supported (OSC 8)
- List items — using dashes

## Example Paragraph

The quick brown fox jumps over the lazy dog. This is a longer paragraph that demonstrates word wrapping within the widget boundaries. The renderer uses `unicode-width` for accurate character measurement and breaks at word boundaries when possible.

## Code Example

You can write `let x = 42;` inline or refer to `functions` and `types`.

*Try cycling themes with Ctrl+T to see how colors adapt.*
"#;

    let rich_text = RichText::with_id(WidgetId::new(1), markdown).with_theme(theme.clone());

    let mut app = App::new()?;
    app.add_widget(Box::new(rich_text), Rect::new(2, 1, 76, 22));

    let q = quit;
    app.title("Rich Text Demo")
        .fps(30)
        .theme(theme)
        .on_input(move |key| {
            use dracon_terminal_engine::input::event::{KeyCode, KeyModifiers};
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                q.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .run(move |ctx| {
            if should_quit.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
}
