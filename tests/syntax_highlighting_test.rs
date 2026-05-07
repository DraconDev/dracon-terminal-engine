//! Syntax highlighting tests — language detection, color accuracy.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Label, List, Table};
use dracon_terminal_engine::utils::highlight_code;
use ratatui::layout::Rect;

#[test]
fn test_highlight_code_rust() {
    let code = r#"fn main() { println!("Hello"); }"#;
    let lines = highlight_code(code, "rs");
    assert!(!lines.is_empty());
}

#[test]
fn test_highlight_code_python() {
    let code = "def hello():\n    print('world')";
    let lines = highlight_code(code, "py");
    assert!(!lines.is_empty());
}

#[test]
fn test_highlight_code_javascript() {
    let code = "function hello() { return 'world'; }";
    let lines = highlight_code(code, "js");
    assert!(!lines.is_empty());
}

#[test]
fn test_highlight_code_empty() {
    let lines = highlight_code("", "rs");
    assert!(lines.is_empty());
}

#[test]
fn test_highlight_code_unknown_extension() {
    let code = "some text here";
    let lines = highlight_code(code, "unknown");
    // Should not panic, may return unhighlighted text
    assert!(!lines.is_empty());
}

#[test]
fn test_highlight_code_multiline() {
    let code = "line1\nline2\nline3";
    let lines = highlight_code(code, "txt");
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_highlight_code_comments() {
    let code = "// This is a comment\nfn main() {}";
    let lines = highlight_code(code, "rs");
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_highlight_code_strings() {
    let code = r#"let s = "hello world";"#;
    let lines = highlight_code(code, "rs");
    assert!(!lines.is_empty());
}

#[test]
fn test_highlight_code_keywords() {
    let code = "fn let mut const if else match";
    let lines = highlight_code(code, "rs");
    assert!(!lines.is_empty());
}

#[test]
fn test_label_render_with_highlighted_text() {
    let mut label = Label::new("Hello World");
    label.on_theme_change(&Theme::nord());
    let plane = label.render(Rect::new(0, 0, 20, 1));

    // Should have visible text
    let has_chars = plane.cells.iter().any(|c| c.char != ' ' && c.char != '\0');
    assert!(has_chars);
}

#[test]
fn test_list_render_with_highlighted_items() {
    let items = vec!["fn main() {}".to_string(), "let x = 42;".to_string()];
    let mut list = List::new_with_id(WidgetId::new(1), items);
    list.on_theme_change(&Theme::nord());
    let plane = list.render(Rect::new(0, 0, 30, 5));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_theme_primary_color_visible() {
    let theme = Theme::nord();
    assert_ne!(theme.primary, Color::Reset);
    assert_ne!(theme.primary, theme.bg);
}

#[test]
fn test_theme_success_color_visible() {
    let theme = Theme::nord();
    assert_ne!(theme.success, Color::Reset);
}

#[test]
fn test_theme_warning_color_visible() {
    let theme = Theme::nord();
    assert_ne!(theme.warning, Color::Reset);
}

#[test]
fn test_theme_error_color_visible() {
    let theme = Theme::nord();
    assert_ne!(theme.error, Color::Reset);
}
