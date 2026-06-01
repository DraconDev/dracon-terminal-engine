//! Tests for the TextEditorAdapter widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId, WidgetState};
use dracon_terminal_engine::framework::widgets::context_menu::{ContextMenu, ContextMenuItem};
use dracon_terminal_engine::framework::widgets::TextEditorAdapter;
use dracon_terminal_engine::widgets::TextEditor;
use ratatui::layout::Rect;
use serde_json::json;

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

#[test]
fn test_text_editor_adapter_set_id() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    tea.set_id(WidgetId::new(99));
    assert_eq!(tea.id(), WidgetId::new(99));
}

#[test]
fn test_text_editor_adapter_with_theme_constructor() {
    let theme = Theme::cyberpunk();
    let tea = TextEditorAdapter::with_theme(theme);
    assert!(tea.id().0 > 0);
    assert!(tea.needs_render());
}

#[test]
fn test_text_editor_adapter_with_context_menu() {
    let menu = ContextMenu::new(vec![ContextMenuItem::new("cut", "Cut")]);
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor).with_context_menu(menu);
    let _ = tea.id();
}

#[test]
fn test_text_editor_adapter_dirty_lifecycle() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    assert!(tea.needs_render());
    tea.clear_dirty();
    assert!(!tea.needs_render());
    tea.mark_dirty();
    assert!(tea.needs_render());
}

#[test]
fn test_text_editor_adapter_set_area_marks_dirty() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    tea.clear_dirty();
    <TextEditorAdapter as Widget>::set_area(&mut tea, Rect::new(10, 5, 40, 12));
    assert_eq!(tea.area(), Rect::new(10, 5, 40, 12));
    assert!(tea.needs_render());
}

#[test]
fn test_text_editor_adapter_set_area_via_method() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    tea.set_area(Rect::new(0, 0, 50, 20));
    assert_eq!(tea.area(), Rect::new(0, 0, 50, 20));
}

#[test]
fn test_text_editor_adapter_z_index_is_10() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    assert_eq!(tea.z_index(), 10);
}

#[test]
fn test_text_editor_adapter_focusable() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    assert!(tea.focusable());
}

#[test]
fn test_text_editor_adapter_editor_accessor() {
    let editor = TextEditor::with_content("hello");
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let content = tea.editor().get_content();
    assert!(content.starts_with("hello"));
}

#[test]
fn test_text_editor_adapter_editor_mut_modifies() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    tea.editor_mut().insert_string("mutated");
    let content = tea.editor().get_content();
    assert!(content.starts_with("mutated"));
}

#[test]
fn test_text_editor_adapter_focus_and_blur() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    tea.on_focus();
    tea.on_blur();
    tea.on_focus();
    tea.on_blur();
}

#[test]
fn test_text_editor_adapter_render_zero_area_does_not_panic() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = tea.render(Rect::new(0, 0, 0, 0));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_text_editor_adapter_render_normal_area() {
    let editor = TextEditor::with_content("hi");
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = tea.render(Rect::new(0, 0, 20, 5));
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 5);
    assert_eq!(plane.cells.len(), 100);
}

#[test]
fn test_text_editor_adapter_render_plane_z_index() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = tea.render(Rect::new(0, 0, 10, 3));
    assert_eq!(plane.z_index, 10);
}

#[test]
fn test_text_editor_adapter_cursor_position_inside_area() {
    let mut editor = TextEditor::with_content("abc");
    editor.cursor_row = 0;
    editor.cursor_col = 1;
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let pos = tea.cursor_position();
    assert!(pos.is_some());
    let (col, row) = pos.unwrap();
    assert_eq!(col, 1);
    assert_eq!(row, 0);
}

#[test]
fn test_text_editor_adapter_widget_state_id() {
    let editor = TextEditor::new();
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    assert_eq!(tea.state_id(), Some("text_editor"));
}

#[test]
fn test_text_editor_adapter_to_json() {
    let mut editor = TextEditor::with_content("hello\nworld");
    editor.cursor_row = 1;
    editor.cursor_col = 2;
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let state = tea.to_json();
    let content = state["content"].as_str().unwrap();
    assert!(content.starts_with("hello"));
    assert!(content.contains("world"));
    assert_eq!(state["cursor_row"], 1);
    assert_eq!(state["cursor_col"], 2);
}

#[test]
fn test_text_editor_adapter_apply_json() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let state = json!({
        "content": "restored",
        "cursor_row": 0,
        "cursor_col": 4,
        "scroll_row": 0,
        "scroll_col": 0
    });
    tea.apply_json(&state).unwrap();
    assert_eq!(tea.editor().get_content(), "restored");
    assert!(tea.needs_render());
}

#[test]
fn test_text_editor_adapter_apply_json_empty_content() {
    let editor = TextEditor::with_content("old");
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let state = json!({ "content": "" });
    tea.apply_json(&state).unwrap();
    let content = tea.editor().get_content();
    assert!(content.is_empty() || content == "\n");
}

#[test]
fn test_text_editor_adapter_state_roundtrip() {
    let mut editor = TextEditor::with_content("roundtrip\nline2");
    editor.cursor_row = 1;
    editor.cursor_col = 3;
    let tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let state = tea.to_json();

    let editor2 = TextEditor::new();
    let mut tea2 = TextEditorAdapter::new(WidgetId::new(2), editor2);
    tea2.apply_json(&state).unwrap();
    let content = tea2.editor().get_content();
    assert!(content.starts_with("roundtrip"));
    assert!(content.contains("line2"));
    assert_eq!(tea2.editor().cursor_row, 1);
    assert_eq!(tea2.editor().cursor_col, 3);
}

#[test]
fn test_text_editor_adapter_apply_json_invalid() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    let state = json!({ "content": null });
    let result = tea.apply_json(&state);
    assert!(result.is_ok());
}

#[test]
fn test_text_editor_adapter_handle_key_dirty() {
    let editor = TextEditor::new();
    let mut tea = TextEditorAdapter::new(WidgetId::new(1), editor);
    tea.clear_dirty();
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    let key = KeyEvent {
        code: KeyCode::Char('a'),
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::empty(),
    };
    let _ = tea.handle_key(key);
    assert!(tea.needs_render());
}
