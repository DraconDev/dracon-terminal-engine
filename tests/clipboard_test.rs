//! Clipboard integration tests — cut/copy/paste across widgets.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::TextEditorAdapter;
use dracon_terminal_engine::input::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use dracon_terminal_engine::utils::{get_clipboard_text, set_clipboard_text};
use dracon_terminal_engine::widgets::editor::TextEditor;
use ratatui::layout::Rect;

#[test]
fn test_clipboard_set_and_get() {
    let text = "Hello, Clipboard!";
    set_clipboard_text(text);
    let result = get_clipboard_text();
    assert_eq!(result, Some(text.to_string()));
}

#[test]
fn test_clipboard_empty() {
    set_clipboard_text("");
    let result = get_clipboard_text();
    assert_eq!(result, Some("".to_string()));
}

#[test]
fn test_clipboard_multiline() {
    let text = "Line 1\nLine 2\nLine 3";
    set_clipboard_text(text);
    let result = get_clipboard_text();
    assert_eq!(result, Some(text.to_string()));
}

#[test]
fn test_clipboard_unicode() {
    let text = "Hello 世界 🌍 émojis";
    set_clipboard_text(text);
    let result = get_clipboard_text();
    assert_eq!(result, Some(text.to_string()));
}

#[test]
fn test_editor_copy_selection() {
    let mut editor = TextEditor::with_content("Hello World");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 80, 24));
    
    // Select text
    adapter.handle_key(KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
    });
    
    // Copy
    adapter.handle_key(KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
    });
    
    let clipboard = get_clipboard_text();
    assert!(clipboard.is_some());
    assert!(!clipboard.unwrap().is_empty());
}

#[test]
fn test_editor_cut_selection() {
    let mut editor = TextEditor::with_content("Hello World");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 80, 24));
    
    // Select all
    adapter.handle_key(KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
    });
    
    // Cut
    adapter.handle_key(KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
    });
    
    let clipboard = get_clipboard_text();
    assert!(clipboard.is_some());
    assert!(!clipboard.unwrap().is_empty());
}

#[test]
fn test_editor_paste() {
    set_clipboard_text("Pasted text");
    
    let mut editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 80, 24));
    
    // Paste
    adapter.handle_key(KeyEvent {
        code: KeyCode::Char('v'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
    });
    
    // Content should have been pasted
    let content = adapter.editor().get_content();
    assert!(!content.is_empty());
}

#[test]
fn test_clipboard_persists_between_operations() {
    set_clipboard_text("First");
    let first = get_clipboard_text();
    
    set_clipboard_text("Second");
    let second = get_clipboard_text();
    
    assert_eq!(first, Some("First".to_string()));
    assert_eq!(second, Some("Second".to_string()));
}

#[test]
fn test_clipboard_special_chars() {
    let text = "<&\"'\\n\\t";
    set_clipboard_text(text);
    let result = get_clipboard_text();
    assert_eq!(result, Some(text.to_string()));
}

#[test]
fn test_clipboard_long_text() {
    let text = "a".repeat(10000);
    set_clipboard_text(&text);
    let result = get_clipboard_text();
    assert_eq!(result, Some(text));
}

#[test]
fn test_editor_paste_multiline() {
    set_clipboard_text("Line 1\nLine 2\nLine 3");
    
    let mut editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 80, 24));
    
    adapter.handle_key(KeyEvent {
        code: KeyCode::Char('v'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
    });
    
    let content = adapter.editor().get_content();
    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3"));
}
