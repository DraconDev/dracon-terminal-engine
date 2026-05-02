#![allow(missing_docs)]
//! TextEditor example — demonstrates TextEditor widget in App context.
//!
//! Opens a file and displays it in an editable text editor within the App.
//! Keyboard: type to edit, arrows to navigate, Ctrl+G for goto line,
//!           Ctrl+F to search, Ctrl+S to save.
//!
//! Usage:
//!     cargo run --example text_editor_demo -- [FILE_PATH]

use dracon_terminal_engine::backend::tty;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::TextEditorAdapter;
use dracon_terminal_engine::widgets::editor::TextEditor;
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    let file_path = std::env::args().nth(1).map(PathBuf::from);

    let mut editor = if let Some(ref path) = file_path {
        if path.exists() {
            TextEditor::open(path).unwrap_or_else(|_| TextEditor::with_content(""))
        } else {
            TextEditor::with_content("")
        }
    } else {
        TextEditor::with_content("// Start typing...\n")
    };

    editor.with_show_line_numbers(true);
    editor.with_indent_guides(true);
    editor.with_status_bar(true);
    if let Some(ref path) = file_path {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                editor.with_language(ext_str);
            }
        }
    }

    let mut app = App::new()?.title("TextEditor Demo").fps(30).theme(theme);

    // Query terminal size so the editor fills the screen from startup.
    // The framework does not auto-resize widget areas on terminal resize;
    // each widget's area is fixed after add_widget() unless explicitly updated.
    let (w, h) = tty::get_window_size(std::io::stdout().as_fd()).unwrap_or((80, 24));

    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    app.add_widget(Box::new(adapter), Rect::new(0, 0, w, h));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    app = app
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        });
    app.run(move |ctx| {
        let (w, h) = ctx.compositor().size();
        ctx.hide_cursor().ok();
        ctx.mark_dirty(0, 0, w, h);
    })
}
