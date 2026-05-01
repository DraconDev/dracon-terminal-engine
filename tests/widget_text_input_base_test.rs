mod common;

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::text_input_base::BaseInput;
use ratatui::layout::Rect;

#[test]
fn test_base_input_new() {
    let id = WidgetId::new(1);
    let base = BaseInput::new(id, "placeholder");
    assert!(base.text.is_empty());
    assert_eq!(base.cursor_pos, 0);
    assert_eq!(base.placeholder, "placeholder");
    assert!(base.dirty);
}

#[test]
fn test_base_input_with_theme() {
    let id = WidgetId::new(1);
    let base = BaseInput::new(id, "placeholder").with_theme(Theme::cyberpunk());
    assert_eq!(base.theme.name, "cyberpunk");
}

#[test]
fn test_base_input_clear() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "hello".to_string();
    base.cursor_pos = 5;
    base.clear();
    assert!(base.text.is_empty());
    assert_eq!(base.cursor_pos, 0);
    assert!(base.dirty);
}

#[test]
fn test_base_input_cursor_position() {
    let id = WidgetId::new(1);
    let base = BaseInput::new(id, "placeholder");
    base.area.set(Rect::new(10, 5, 30, 1));
    let pos = base.cursor_position();
    assert!(pos.is_some());
    let (x, y) = pos.unwrap();
    assert_eq!(x, 10);
    assert_eq!(y, 5);
}

#[test]
fn test_base_input_set_area() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    let area = Rect::new(20, 10, 40, 2);
    base.set_area(area);
    assert!(base.dirty);
    let a = base.area.get();
    assert_eq!(a.x, 20);
    assert_eq!(a.y, 10);
    assert_eq!(a.width, 40);
    assert_eq!(a.height, 2);
}

#[test]
fn test_base_input_mark_dirty() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.clear_dirty();
    assert!(!base.dirty);
    base.mark_dirty();
    assert!(base.dirty);
}

#[test]
fn test_base_input_render_input_empty() {
    let id = WidgetId::new(1);
    let base = BaseInput::new(id, "placeholder");
    let plane = base.render_input(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
    assert!(plane.z_index > 0);
}

#[test]
fn test_base_input_render_input_with_text() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "test".to_string();
    base.cursor_pos = 2;
    let plane = base.render_input(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_base_input_render_input_with_mask() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "secret".to_string();
    base.mask_char = Some('*');
    base.cursor_pos = 3;
    let plane = base.render_input(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_base_input_handle_key_enter_triggers_callback() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Enter,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
}

#[test]
fn test_base_input_handle_key_char() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Char('a'),
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.text, "a");
    assert_eq!(base.cursor_pos, 1);
}

#[test]
fn test_base_input_handle_key_backspace() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abc".to_string();
    base.cursor_pos = 2;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Backspace,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.text.len(), 2);
    assert_eq!(base.cursor_pos, 1);
}

#[test]
fn test_base_input_handle_key_backspace_at_start() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abc".to_string();
    base.cursor_pos = 0;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Backspace,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.text, "abc");
}

#[test]
fn test_base_input_handle_key_left() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abc".to_string();
    base.cursor_pos = 2;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Left,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.cursor_pos, 1);
}

#[test]
fn test_base_input_handle_key_right() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abc".to_string();
    base.cursor_pos = 1;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Right,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.cursor_pos, 2);
}

#[test]
fn test_base_input_handle_key_delete() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abc".to_string();
    base.cursor_pos = 1;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Delete,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.text, "ac");
}

#[test]
fn test_base_input_handle_key_home() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abc".to_string();
    base.cursor_pos = 2;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Home,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.cursor_pos, 0);
}

#[test]
fn test_base_input_handle_key_end() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abc".to_string();
    base.cursor_pos = 1;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::End,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(result);
    assert_eq!(base.cursor_pos, 3);
}

#[test]
fn test_base_input_handle_key_repeat_ignored() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Repeat,
        code: dracon_terminal_engine::input::event::KeyCode::Char('a'),
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = base.handle_key(key);
    assert!(!result);
    assert!(base.text.is_empty());
}

#[test]
fn test_base_input_handle_mouse() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "abcdef".to_string();
    base.cursor_pos = 0;
    let result = base.handle_mouse(
        dracon_terminal_engine::input::event::MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left),
        3,
        0,
    );
    assert!(result);
    assert_eq!(base.cursor_pos, 3);
    assert!(base.dirty);
}

#[test]
fn test_base_input_handle_mouse_past_text() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "ab".to_string();
    base.cursor_pos = 0;
    let result = base.handle_mouse(
        dracon_terminal_engine::input::event::MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left),
        10,
        0,
    );
    assert!(!result);
}

#[test]
fn test_base_input_on_submit_builder() {
    let id = WidgetId::new(1);
    let base = BaseInput::new(id, "placeholder");
    let result = base.on_submit(|_text| {});
    assert!(result.on_submit.is_some());
}

#[test]
fn test_base_input_render_input_narrow_area() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "this is a very long text that should be truncated".to_string();
    let plane = base.render_input(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_base_input_render_input_unicode() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "Hello 世界".to_string();
    let plane = base.render_input(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_base_input_render_input_cursor_at_end() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "test".to_string();
    base.cursor_pos = 4;
    let plane = base.render_input(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_base_input_render_input_cursor_hidden_when_empty() {
    let id = WidgetId::new(1);
    let mut base = BaseInput::new(id, "placeholder");
    base.text = "".to_string();
    base.cursor_pos = 0;
    let plane = base.render_input(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}