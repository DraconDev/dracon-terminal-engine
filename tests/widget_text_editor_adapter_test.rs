//! Tests for the TextEditorAdapter widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::TextEditorAdapter;
use dracon_terminal_engine::widgets::TextEditor;

#[test]
fn test_text_editor_adapter_new() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let _ = tea.id();
}

#[test]
fn test_text_editor_adapter_id() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(42), editor);
    assert_eq!(tea.id(), WidgetId::new(42));
}

#[test]
fn test_text_editor_adapter_area() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let area = tea.area();
    assert!(area.width > 0);
}

#[test]
fn test_text_editor_adapter_on_theme_change() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    tea.on_theme_change(&Theme::nord());
    assert!(tea.needs_render());
}
