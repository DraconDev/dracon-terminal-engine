//! Edge case tests for TextEditorAdapter.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::TextEditorAdapter;
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use dracon_terminal_engine::widgets::editor::TextEditor;
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        kind: KeyEventKind::Press,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    }
}

fn rect(x: u16, y: u16, w: u16, h: u16) -> Rect {
    Rect::new(x, y, w, h)
}

// === cursor_position edge cases ===

#[test]
fn test_cursor_position_origin_inside_scrolled_view() {
    let mut editor = TextEditor::with_content("0123456789");
    editor.scroll_col = 3;
    editor.cursor_row = 0;
    editor.cursor_col = 0;

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(5, 5, 20, 10));

    let pos = adapter.cursor_position();
    assert_eq!(pos, Some((5, 5)));
}

#[test]
fn test_cursor_position_both_scroll_and_area_offset() {
    let mut editor = TextEditor::with_content("0123456789012345678901234567890123456789");
    editor.scroll_row = 2;
    editor.scroll_col = 5;
    editor.cursor_row = 5;
    editor.cursor_col = 10;

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(10, 20, 30, 15));

    let pos = adapter.cursor_position();
    assert!(pos.is_some());
    let (x, y) = pos.unwrap();

    assert!(
        (10..10 + 30).contains(&x),
        "cursor x should be within area x range"
    );
    assert!(
        (20..20 + 15).contains(&y),
        "cursor y should be within area y range"
    );
}

#[test]
fn test_cursor_position_area_0x0() {
    let editor = TextEditor::with_content("hello");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 0, 0));

    let pos = adapter.cursor_position();
    assert_eq!(pos, Some((0, 0)));
}

#[test]
fn test_cursor_position_nonzero_area_offset() {
    let mut editor = TextEditor::with_content("hello");
    editor.cursor_row = 0;
    editor.cursor_col = 2;

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(7, 3, 20, 10));

    let pos = adapter.cursor_position();
    let (x, y) = pos.unwrap();
    assert_eq!(y, 3);
    assert!(x >= 7);
}

#[test]
fn test_cursor_position_scroll_row_beyond_content() {
    let mut editor = TextEditor::with_content("a\nb\nc");
    editor.scroll_row = 100;
    editor.cursor_row = 2;
    editor.cursor_col = 0;

    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 10));

    let pos = adapter.cursor_position();
    assert!(pos.is_some());
}

// === render edge cases ===

#[test]
fn test_render_area_0x0_does_not_panic() {
    let editor = TextEditor::with_content("hello");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 0, 0));

    let plane = adapter.render(rect(0, 0, 0, 0));
    assert_eq!(plane.cells.len(), 0);
}

#[test]
fn test_render_with_line_numbers() {
    let mut editor = TextEditor::with_content("line1\nline2\nline3");
    editor.with_show_line_numbers(true);

    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = adapter.render(rect(0, 0, 40, 10));

    assert!(!plane.cells.is_empty());
    let non_space: Vec<_> = plane.cells.iter().filter(|c| c.char != ' ').collect();
    assert!(
        !non_space.is_empty(),
        "rendered plane should have visible content"
    );
}

#[test]
fn test_render_cell_colors_are_mapped() {
    let editor = TextEditor::with_content("X");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = adapter.render(rect(0, 0, 20, 5));

    let x_cell = plane.cells.iter().find(|c| c.char == 'X');
    assert!(
        x_cell.is_some(),
        "rendered plane should have the character X"
    );

    for cell in &plane.cells {
        assert!(
            !matches!(cell.fg, Color::Reset) || cell.char == ' ',
            "visible cells should have non-Reset fg (or be transparent space)"
        );
    }
}

#[test]
fn test_render_plane_z_index_is_10() {
    let editor = TextEditor::with_content("test");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = adapter.render(rect(0, 0, 20, 5));

    assert_eq!(plane.z_index, 10);
}

#[test]
fn test_render_plane_area_matches() {
    let editor = TextEditor::with_content("hello world this is a long line");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    let plane = adapter.render(rect(3, 7, 30, 12));

    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 12);
}

// === handle_key key kinds ===

#[test]
fn test_handle_key_press_is_forwarded() {
    let editor = TextEditor::with_content("abc");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 5));

    let result = adapter.handle_key(make_key(KeyCode::Right));

    assert!(!result, "plain arrow keys are not consumed (return false)");
}

#[test]
fn test_handle_key_repeat_is_forwarded() {
    let editor = TextEditor::with_content("abc");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 5));

    let repeat_key = KeyEvent {
        code: KeyCode::Right,
        kind: KeyEventKind::Repeat,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = adapter.handle_key(repeat_key);

    assert!(!result, "repeat events are not consumed");
}

#[test]
fn test_handle_key_release_returns_false() {
    let editor = TextEditor::with_content("abc");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 5));

    let release_key = KeyEvent {
        code: KeyCode::Right,
        kind: KeyEventKind::Release,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = adapter.handle_key(release_key);

    assert!(
        !result,
        "Release key kind should not be consumed (event ignored by editor)"
    );
}

#[test]
fn test_handle_key_typing_char_advances() {
    let editor = TextEditor::with_content("");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 5));

    adapter.handle_key(make_key(KeyCode::Char('h')));
    adapter.handle_key(make_key(KeyCode::Char('i')));

    assert_eq!(adapter.editor().lines.len(), 1);
    assert!(adapter.editor().lines[0].contains('h') || adapter.editor().lines[0].contains('i'));
}

// === handle_mouse edge cases ===

#[test]
fn test_handle_mouse_left_press_at_origin() {
    let editor = TextEditor::with_content("hello");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 5));

    let result = adapter.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(result, "left press should be consumed");
}

#[test]
fn test_handle_mouse_out_of_bounds_coords() {
    let editor = TextEditor::with_content("hello");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 5));

    let result = adapter.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100);
    assert!(!result, "mouse far outside area should not be consumed");
}

#[test]
fn test_handle_mouse_scroll_propagates() {
    let editor = TextEditor::with_content("line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 20, 5));

    let initial_scroll = adapter.editor().scroll_row;
    adapter.handle_mouse(MouseEventKind::ScrollDown, 5, 2);
    let new_scroll = adapter.editor().scroll_row;

    assert_ne!(
        initial_scroll, new_scroll,
        "scroll mouse event should change scroll_row"
    );
}

#[test]
fn test_handle_mouse_left_drag() {
    let editor = TextEditor::with_content("hello world");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(0, 0, 30, 5));

    let result = adapter.handle_mouse(MouseEventKind::Drag(MouseButton::Left), 5, 0);
    assert!(result, "left drag should be consumed");
}

// === on_focus / on_blur edge cases ===

#[test]
fn test_on_focus_called_multiple_times_does_not_panic() {
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), TextEditor::new());
    adapter.set_area(rect(0, 0, 20, 5));

    adapter.on_focus();
    adapter.on_focus();
    adapter.on_focus();
}

#[test]
fn test_on_blur_called_multiple_times_does_not_panic() {
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), TextEditor::new());
    adapter.set_area(rect(0, 0, 20, 5));

    adapter.on_blur();
    adapter.on_blur();
    adapter.on_blur();
}

#[test]
fn test_on_focus_and_blur_sequence_does_not_panic() {
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), TextEditor::new());
    adapter.set_area(rect(0, 0, 20, 5));

    adapter.on_focus();
    adapter.on_blur();
    adapter.on_focus();
    adapter.on_blur();
}

// === editor accessor ===

#[test]
fn test_editor_accessor_returns_editor() {
    let editor = TextEditor::with_content("hello");
    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor.clone());
    assert_eq!(
        adapter.editor().lines.len(),
        2,
        "with_content adds trailing newline"
    );
}

#[test]
fn test_editor_mut_accessor_modifies_editor() {
    let editor = TextEditor::with_content("hello");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.editor_mut().cursor_row = 0;
    adapter.editor_mut().cursor_col = 2;

    assert_eq!(adapter.editor().cursor_col, 2);
}

// === area set/get ===

#[test]
fn test_set_area_during_render() {
    let editor = TextEditor::with_content("test");
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);

    adapter.set_area(rect(0, 0, 80, 24));
    let plane1 = adapter.render(rect(0, 0, 80, 24));

    adapter.set_area(rect(0, 0, 40, 10));
    let plane2 = adapter.render(rect(0, 0, 40, 10));

    assert_ne!(
        plane1.width, plane2.width,
        "different area should produce different plane sizes"
    );
}

#[test]
fn test_area_stored_correctly() {
    let editor = TextEditor::new();
    let mut adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    adapter.set_area(rect(1, 2, 3, 4));
    assert_eq!(adapter.area(), Rect::new(1, 2, 3, 4));
}
