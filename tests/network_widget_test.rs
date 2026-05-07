//! Network widget tests — chat client message sending/receiving.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Label, List, TextEditorAdapter};
use dracon_terminal_engine::widgets::editor::TextEditor;
use ratatui::layout::Rect;

#[test]
fn test_chat_message_label() {
    let label = Label::new("Alice: Hello!");
    let plane = label.render(Rect::new(0, 0, 40, 1));
    assert_eq!(plane.width, 40);
}

#[test]
fn test_chat_message_list() {
    let messages = vec![
        "Alice: Hello!".to_string(),
        "Bob: Hi there!".to_string(),
        "Alice: How are you?".to_string(),
    ];
    let list = List::new_with_id(WidgetId::new(1), messages);
    let plane = list.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.height, 10);
}

#[test]
fn test_chat_input_editor() {
    let editor = TextEditor::new();
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = adapter.render(Rect::new(0, 0, 80, 3));
    assert_eq!(plane.height, 3);
}

#[test]
fn test_chat_message_too_long() {
    let long_msg = "A".repeat(200);
    let label = Label::new(&long_msg);
    let plane = label.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_chat_list_scrollable() {
    let messages: Vec<String> = (0..50)
        .map(|i| format!("User{}: Message {}", i, i))
        .collect();
    let list = List::new_with_id(WidgetId::new(1), messages);
    let plane = list.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.height, 20);
}

#[test]
fn test_chat_system_message() {
    let label = Label::new("*** User joined the channel ***");
    let plane = label.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_chat_empty_message() {
    let label = Label::new("");
    let plane = label.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_chat_unicode_messages() {
    let messages = vec![
        "世界: 你好".to_string(),
        "🌍: Hello".to_string(),
        "αβγ: Test".to_string(),
    ];
    let list = List::new_with_id(WidgetId::new(1), messages);
    let plane = list.render(Rect::new(0, 0, 80, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_chat_input_multiline() {
    let editor = TextEditor::with_content("Line 1\nLine 2\nLine 3");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = adapter.render(Rect::new(0, 0, 80, 5));
    assert_eq!(plane.height, 5);
}

#[test]
fn test_chat_theme_colors() {
    let label = Label::new("Test").with_theme(Theme::nord());
    let plane = label.render(Rect::new(0, 0, 80, 1));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}
