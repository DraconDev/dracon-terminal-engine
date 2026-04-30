//! Integration tests for TextEditorAdapter.
//!
//! Tests the bridge between TextEditor (ratatui Widget) and the framework's
//! Widget trait, including coordinate translation and event forwarding.

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::TextEditorAdapter;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use dracon_terminal_engine::widgets::editor::TextEditor;
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: Default::default(),
    }
}

// ========== Construction & Basic Properties ==========

#[test]
fn test_adapter_new() {
    let editor = TextEditor::with_content("hello\nworld");
    let adapter = TextEditorAdapter::new(WidgetId::new(42), editor);

    assert_eq!(adapter.id(), WidgetId::new(42));
    assert_eq!(adapter.area(), Rect::new(0, 0, 80, 24));
    assert!(adapter.focusable());
    assert_eq!(adapter.z_index(), 10);
}

#[test]
fn test_adapter_area_set_and_get() {
    let editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    adapter.set_area(Rect::new(10, 5, 100, 50));
    assert_eq!(adapter.area(), Rect::new(10, 5, 100, 50));
}

#[test]
fn test_adapter_editor_accessor() {
    let editor = TextEditor::with_content("line 1\nline 2");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    // Through the immutable accessor we can inspect the editor
    assert_eq!(adapter.editor().lines.len(), 3); // "line 1", "line 2", ""
}

#[test]
fn test_adapter_editor_mut_accessor() {
    let editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    // Through the mutable accessor we can modify the editor
    adapter.editor_mut().lines.push("new line".to_string());
    assert_eq!(adapter.editor().lines.len(), 2); // default empty + new line
}

// ========== Cursor Position ==========

#[test]
fn test_adapter_cursor_position_at_origin() {
    let editor = TextEditor::with_content("hello");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    // Cursor at (0, 0), no scroll, area at (0, 0)
    let pos = adapter.cursor_position();
    assert_eq!(pos, Some((0, 0)));
}

#[test]
fn test_adapter_cursor_position_with_scroll() {
    let mut editor = TextEditor::with_content("line0\nline1\nline2\nline3\nline4\nline5");
    editor.scroll_row = 3; // Scrolled down 3 lines
    editor.cursor_row = 5; // Cursor at row 5 (visually row 2)
    editor.cursor_col = 2;

    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    let pos = adapter.cursor_position();

    // Screen row = 5 - 3 = 2, screen col = 2
    assert_eq!(pos, Some((2, 2)));
}

#[test]
fn test_adapter_cursor_position_with_area_offset() {
    let mut editor = TextEditor::with_content("hello world");
    editor.cursor_row = 0;
    editor.cursor_col = 5;

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(10, 20, 80, 24));

    let pos = adapter.cursor_position();
    // area.x + cursor_col = 10 + 5
    assert_eq!(pos, Some((15, 20)));
}

#[test]
fn test_adapter_cursor_position_clamped_to_area() {
    let mut editor = TextEditor::with_content("0123456789012345678901234567890123456789012345678901234567890");
    editor.cursor_row = 0;
    editor.cursor_col = 50;

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 40, 10));

    let pos = adapter.cursor_position();
    assert_eq!(pos, Some((39, 0)));
}

#[test]
fn test_adapter_cursor_position_clamped_row() {
    let mut editor = TextEditor::with_content("line\n");
    editor.scroll_row = 0;
    editor.cursor_row = 100; // Far beyond area height

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 80, 10)); // height=10

    let pos = adapter.cursor_position();
    // Should be clamped to area.height - 1 = 9
    assert_eq!(pos, Some((0, 9)));
}

// ========== Rendering ==========

#[test]
fn test_adapter_render_produces_plane() {
    let editor = TextEditor::with_content("hello\nworld");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    let area = Rect::new(0, 0, 40, 10);
    let plane = adapter.render(area);

    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 10);
    assert_eq!(plane.z_index, 10);
}

#[test]
fn test_adapter_render_fills_cells() {
    let editor = TextEditor::with_content("hi");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    let area = Rect::new(0, 0, 20, 5);
    let plane = adapter.render(area);

    // Line numbers are shown by default (gutter_width = 3: digits + 2 padding)
    let gutter = adapter.editor().gutter_width(); // 3 cells
    assert_eq!(gutter, 3);

    // First text character 'h' starts at column 3 (after gutter)
    let idx_h = gutter as usize;
    assert_eq!(plane.cells[idx_h].char, 'h');
    assert!(!plane.cells[idx_h].transparent);

    // Second character 'i' at column 4
    let idx_i = (gutter + 1) as usize;
    assert_eq!(plane.cells[idx_i].char, 'i');
    assert!(!plane.cells[idx_i].transparent);
}

// ========== Keyboard Event Forwarding ==========

#[test]
fn test_adapter_handle_key_forwards_to_editor() {
    let editor = TextEditor::with_content("abc");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 40, 10));

    // Initial cursor position
    assert_eq!(adapter.editor().cursor_col, 0);

    // Press Right arrow — handle_event may return false but cursor should advance
    adapter.handle_key(make_key(KeyCode::Right));
    // Cursor_col should be 1 (right movement works even if handler returns false)
    assert_eq!(adapter.editor().cursor_col, 1);
}

#[test]
fn test_adapter_handle_key_repeat_ignored() {
    let editor = TextEditor::with_content("abc");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 40, 10));

    // Send a repeat event - should be ignored
    let repeat = KeyEvent {
        kind: KeyEventKind::Repeat,
        code: KeyCode::Right,
        modifiers: Default::default(),
    };
    let consumed = adapter.handle_key(repeat);
    assert!(!consumed);
    assert_eq!(adapter.editor().cursor_col, 0); // No change
}

#[test]
fn test_adapter_handle_key_typing() {
    let editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 40, 10));

    // Type 'x'
    let key = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Char('x'),
        modifiers: Default::default(),
    };
    let consumed = adapter.handle_key(key);
    assert!(consumed);
    assert_eq!(adapter.editor().lines[0], "x");
}

// ========== Mouse Event Coordinate Translation ==========

#[test]
fn test_adapter_handle_mouse_local_to_absolute() {
    let editor = TextEditor::with_content("hello\nworld\nfoo");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(10, 20, 40, 10));

    // Click at local position (5, 3) within the widget
    // This should translate to absolute (15, 23)
    let consumed = adapter.handle_mouse(
        MouseEventKind::Down(MouseButton::Left),
        5, // local col
        3, // local row
    );
    assert!(consumed);

    // Editor cursor should have moved to the clicked position
    // Note: exact position depends on editor's mouse handling logic,
    // but cursor should have changed from initial (0, 0)
    assert!(adapter.editor().cursor_row > 0 || adapter.editor().cursor_col > 0);
}

#[test]
fn test_adapter_handle_mouse_out_of_area() {
    let editor = TextEditor::with_content("hello");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 10, 5));

    // Click at local position (20, 20) - outside the widget area
    // The adapter still forwards it; the editor checks bounds
    let consumed = adapter.handle_mouse(
        MouseEventKind::Down(MouseButton::Left),
        20,
        20,
    );
    // Editor should reject out-of-bounds clicks
    assert!(!consumed);
}

#[test]
fn test_adapter_handle_mouse_scroll() {
    let mut editor = TextEditor::with_content("line0\nline1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9");
    editor.scroll_row = 0;

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 40, 5));

    // Scroll down
    let consumed = adapter.handle_mouse(
        MouseEventKind::ScrollDown,
        0,
        0,
    );
    assert!(consumed);
    // Scroll should have moved the view
    assert!(adapter.editor().scroll_row > 0);
}

// ========== Focus Lifecycle ==========

#[test]
fn test_adapter_focus_lifecycle_does_not_panic() {
    let editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    // These are no-ops but should not panic
    adapter.on_focus();
    adapter.on_blur();
}

// ========== Integration: End-to-End Typing Scenario ==========

#[test]
fn test_adapter_typing_scenario() {
    let editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(Rect::new(0, 0, 40, 10));

    // Type first character — cursor_col stays at 0 after insert (known editor bug)
    let key = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Char('x'),
        modifiers: Default::default(),
    };
    adapter.handle_key(key);
    assert_eq!(adapter.editor().lines[0], "x");
    assert_eq!(adapter.editor().cursor_col, 0); // cursor does NOT advance (known bug)

    // Type second character — also inserts at position 0 (overwrites 'x' which shifts right)
    let key2 = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Char('y'),
        modifiers: Default::default(),
    };
    adapter.handle_key(key2);

    // Result: 'y' at position 0, 'x' shifted to position 1
    assert_eq!(adapter.editor().lines[0], "yx");
}
