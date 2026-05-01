//! Tests for input/mapping.rs deprecated functions and kitty_key.rs parser.

mod common;

use dracon_terminal_engine::input::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use dracon_terminal_engine::input::kitty_key::parse_kitty_keyboard;
use std::io;

#[test]
fn test_from_runtime_event_identity() {
    let event = Event::Key(dracon_terminal_engine::input::event::KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
    });
    #[allow(deprecated)]
    let result = dracon_terminal_engine::input::mapping::from_runtime_event(event.clone());
    match (&event, &result) {
        (Event::Key(a), Event::Key(b)) => {
            assert_eq!(a.code, b.code);
            assert_eq!(a.kind, b.kind);
        }
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_to_runtime_event_identity() {
    let event = Event::Key(dracon_terminal_engine::input::event::KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
    });
    #[allow(deprecated)]
    let result = dracon_terminal_engine::input::mapping::to_runtime_event(&event);
    match (&event, &result) {
        (Event::Key(a), Event::Key(b)) => {
            assert_eq!(a.code, b.code);
            assert_eq!(a.kind, b.kind);
        }
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_to_ui_event_key() {
    let event = Event::Key(dracon_terminal_engine::input::event::KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
    });
    let result = dracon_terminal_engine::input::mapping::to_ui_event(&event);
    assert!(result.is_some());
}

#[test]
fn test_kitty_key_parse_empty() {
    let result = parse_kitty_keyboard(&[]);
    assert!(result.is_none());
}

#[test]
fn test_kitty_key_parse_escape() {
    let result = parse_kitty_keyboard(&["27"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Esc),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_enter() {
    let result = parse_kitty_keyboard(&["13"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Enter),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_tab() {
    let result = parse_kitty_keyboard(&["9"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Tab),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_backspace() {
    let result = parse_kitty_keyboard(&["127"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Backspace),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_with_modifiers() {
    let result = parse_kitty_keyboard(&["13", "2"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => {
            assert_eq!(k.code, KeyCode::Enter);
            assert!(k.modifiers.contains(KeyModifiers::SHIFT));
        }
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_with_alt_modifier() {
    let result = parse_kitty_keyboard(&["13", "3"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => {
            assert!(k.modifiers.contains(KeyModifiers::ALT));
        }
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_with_control_modifier() {
    let result = parse_kitty_keyboard(&["13", "5"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => {
            assert!(k.modifiers.contains(KeyModifiers::CONTROL));
        }
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_repeat_kind() {
    let result = parse_kitty_keyboard(&["13", "1", "2"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.kind, KeyEventKind::Repeat),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_release_kind() {
    let result = parse_kitty_keyboard(&["13", "1", "3"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.kind, KeyEventKind::Release),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_f1() {
    let result = parse_kitty_keyboard(&["57364"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::F(1)),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_f12() {
    let result = parse_kitty_keyboard(&["57375"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::F(12)),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_cursor_up() {
    let result = parse_kitty_keyboard(&["57358"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Up),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_cursor_down() {
    let result = parse_kitty_keyboard(&["57359"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Down),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_cursor_left() {
    let result = parse_kitty_keyboard(&["57360"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Left),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_cursor_right() {
    let result = parse_kitty_keyboard(&["57361"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Right),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_page_up() {
    let result = parse_kitty_keyboard(&["57362"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::PageUp),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_page_down() {
    let result = parse_kitty_keyboard(&["57363"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::PageDown),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_insert() {
    let result = parse_kitty_keyboard(&["57345"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Insert),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_delete() {
    let result = parse_kitty_keyboard(&["57346"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Delete),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_home() {
    let result = parse_kitty_keyboard(&["57347"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Home),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_end() {
    let result = parse_kitty_keyboard(&["57348"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::End),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_unicode_char() {
    let result = parse_kitty_keyboard(&["97"]);
    assert!(result.is_some());
    let event = result.unwrap();
    match event {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Char('a')),
        _ => panic!("expected Key event"),
    }
}

#[test]
fn test_kitty_key_parse_invalid_code() {
    let result = parse_kitty_keyboard(&["999999999"]);
    assert!(result.is_none());
}

#[test]
fn test_kitty_key_parse_invalid_modifier_type() {
    let result = parse_kitty_keyboard(&["13", "xyz"]);
    assert!(result.is_some());
}
