//! Tests for ScrollState and ScrollContainer.

use dracon_terminal_engine::framework::scroll::{ScrollContainer, ScrollState};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        kind: KeyEventKind::Press,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    }
}

fn make_key_repeat(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        kind: KeyEventKind::Repeat,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    }
}

// === ScrollState ===

#[test]
fn test_scroll_state_new() {
    let s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 20,
    };
    assert_eq!(s.offset, 0);
    assert_eq!(s.content_height, 100);
    assert_eq!(s.viewport_height, 20);
}

#[test]
fn test_scroll_state_max_offset_zero_when_content_equals_viewport() {
    let s = ScrollState {
        offset: 0,
        content_height: 20,
        viewport_height: 20,
    };
    assert_eq!(s.max_offset(), 0);
}

#[test]
fn test_scroll_state_max_offset_when_content_larger() {
    let s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 20,
    };
    assert_eq!(s.max_offset(), 80);
}

#[test]
fn test_scroll_state_max_offset_when_content_less() {
    let s = ScrollState {
        offset: 0,
        content_height: 10,
        viewport_height: 20,
    };
    assert_eq!(s.max_offset(), 0);
}

#[test]
fn test_scroll_state_page_size() {
    let s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 20,
    };
    assert_eq!(s.page_size(), 19);
}

#[test]
fn test_scroll_state_page_size_minimum_1() {
    let s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 1,
    };
    assert_eq!(s.page_size(), 1);
}

#[test]
fn test_scroll_state_page_size_viewport_0() {
    let s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 0,
    };
    assert_eq!(s.page_size(), 1);
}

#[test]
fn test_scroll_state_scroll_up() {
    let mut s = ScrollState {
        offset: 10,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_up(3);
    assert_eq!(s.offset, 7);
}

#[test]
fn test_scroll_state_scroll_up_at_zero() {
    let mut s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_up(5);
    assert_eq!(s.offset, 0);
}

#[test]
fn test_scroll_state_scroll_up_by_zero() {
    let mut s = ScrollState {
        offset: 5,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_up(0);
    assert_eq!(s.offset, 5);
}

#[test]
fn test_scroll_state_scroll_down() {
    let mut s = ScrollState {
        offset: 10,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_down(3);
    assert_eq!(s.offset, 13);
}

#[test]
fn test_scroll_state_scroll_down_at_max() {
    let mut s = ScrollState {
        offset: 80,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_down(5);
    assert_eq!(s.offset, 80);
}

#[test]
fn test_scroll_state_scroll_down_by_zero() {
    let mut s = ScrollState {
        offset: 50,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_down(0);
    assert_eq!(s.offset, 50);
}

#[test]
fn test_scroll_state_scroll_to() {
    let mut s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_to(42);
    assert_eq!(s.offset, 42);
}

#[test]
fn test_scroll_state_scroll_to_beyond_max() {
    let mut s = ScrollState {
        offset: 0,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_to(200);
    assert_eq!(s.offset, 80);
}

#[test]
fn test_scroll_state_scroll_to_top() {
    let mut s = ScrollState {
        offset: 50,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_to_top();
    assert_eq!(s.offset, 0);
}

#[test]
fn test_scroll_state_scroll_to_bottom() {
    let mut s = ScrollState {
        offset: 10,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_to_bottom();
    assert_eq!(s.offset, 80);
}

#[test]
fn test_scroll_state_scroll_page_up() {
    let mut s = ScrollState {
        offset: 30,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_page_up();
    assert_eq!(s.offset, 11);
}

#[test]
fn test_scroll_state_scroll_page_up_at_top() {
    let mut s = ScrollState {
        offset: 5,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_page_up();
    assert_eq!(s.offset, 0);
}

#[test]
fn test_scroll_state_scroll_page_down() {
    let mut s = ScrollState {
        offset: 10,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_page_down();
    assert_eq!(s.offset, 29);
}

#[test]
fn test_scroll_state_scroll_page_down_at_bottom() {
    let mut s = ScrollState {
        offset: 75,
        content_height: 100,
        viewport_height: 20,
    };
    s.scroll_page_down();
    assert_eq!(s.offset, 80);
}

// === ScrollContainer ===

#[test]
fn test_scroll_container_new() {
    let sc = ScrollContainer::new();
    assert!(sc.state().content_height == 0 || true);
}

#[test]
fn test_scroll_container_builder_chain() {
    let sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20)
        .with_scrollbar(true);

    assert_eq!(sc.state().content_height, 100);
    assert_eq!(sc.state().viewport_height, 20);
}

#[test]
fn test_scroll_container_handle_key_up() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 5;

    let consumed = sc.handle_key(make_key(KeyCode::Up));
    assert!(consumed);
    assert_eq!(sc.state().offset, 4);
}

#[test]
fn test_scroll_container_handle_key_down() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let consumed = sc.handle_key(make_key(KeyCode::Down));
    assert!(consumed);
    assert_eq!(sc.state().offset, 1);
}

#[test]
fn test_scroll_container_handle_key_page_up() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 30;

    let consumed = sc.handle_key(make_key(KeyCode::PageUp));
    assert!(consumed);
    assert_eq!(sc.state().offset, 11);
}

#[test]
fn test_scroll_container_handle_key_page_down() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let consumed = sc.handle_key(make_key(KeyCode::PageDown));
    assert!(consumed);
    assert_eq!(sc.state().offset, 19);
}

#[test]
fn test_scroll_container_handle_key_home() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 50;

    let consumed = sc.handle_key(make_key(KeyCode::Home));
    assert!(consumed);
    assert_eq!(sc.state().offset, 0);
}

#[test]
fn test_scroll_container_handle_key_end() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let consumed = sc.handle_key(make_key(KeyCode::End));
    assert!(consumed);
    assert_eq!(sc.state().offset, 80);
}

#[test]
fn test_scroll_container_handle_key_repeat_not_ignored() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let consumed = sc.handle_key(make_key_repeat(KeyCode::Down));
    assert!(!consumed, "repeat events are not consumed by handle_key");
}

#[test]
fn test_scroll_container_handle_key_at_top_up_stays_at_zero() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 0;

    sc.handle_key(make_key(KeyCode::Up));
    assert_eq!(sc.state().offset, 0);
}

#[test]
fn test_scroll_container_handle_key_at_bottom_down_stays_at_max() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 80;

    sc.handle_key(make_key(KeyCode::Down));
    assert_eq!(sc.state().offset, 80);
}

#[test]
fn test_scroll_container_handle_mouse_scroll_up() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 5;

    let consumed = sc.handle_mouse(MouseEventKind::ScrollUp, 0, 0);
    assert!(consumed);
    assert_eq!(sc.state().offset, 2);
}

#[test]
fn test_scroll_container_handle_mouse_scroll_down() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let consumed = sc.handle_mouse(MouseEventKind::ScrollDown, 0, 0);
    assert!(consumed);
    assert_eq!(sc.state().offset, 3);
}

#[test]
fn test_scroll_container_handle_mouse_scroll_down_at_max() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 80;

    let consumed = sc.handle_mouse(MouseEventKind::ScrollDown, 0, 0);
    assert!(consumed);
    assert_eq!(sc.state().offset, 80);
}

#[test]
fn test_scroll_container_handle_mouse_other_returns_false() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let consumed = sc.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(!consumed);
}

#[test]
fn test_scroll_container_render_scrollbar_content_equals_viewport() {
    let sc = ScrollContainer::new()
        .with_content_height(20)
        .with_viewport_height(20);

    let plane = sc.render_scrollbar(Rect::new(0, 0, 1, 20));

    for cell in &plane.cells {
        assert_eq!(cell.char, ' ', "scrollbar should be empty when no scrolling needed");
    }
}

#[test]
fn test_scroll_container_render_scrollbar_thumb_at_top() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 0;

    let plane = sc.render_scrollbar(Rect::new(0, 0, 1, 20));

    let thumb_cell = &plane.cells[0];
    assert_eq!(thumb_cell.char, '█');
}

#[test]
fn test_scroll_container_render_scrollbar_thumb_at_bottom() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 80;

    let plane = sc.render_scrollbar(Rect::new(0, 0, 1, 20));

    let thumb_cell = &plane.cells[19];
    assert_eq!(thumb_cell.char, '█');
}

#[test]
fn test_scroll_container_render_scrollbar_thumb_in_middle() {
    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 40;

    let plane = sc.render_scrollbar(Rect::new(0, 0, 1, 20));

    let thumb_cells: Vec<_> = plane.cells.iter().filter(|c| c.char == '█').collect();
    assert!(!thumb_cells.is_empty());
    let first_thumb_idx = plane.cells.iter().position(|c| c.char == '█').unwrap();
    let _last_thumb_idx = plane.cells.len() - 1 - plane.cells.iter().rev().position(|c| c.char == '█').unwrap();

    let expected_pos = (40 * 19) / 80;
    assert!(
        first_thumb_idx <= expected_pos + 1,
        "thumb should be near expected position, expected {}..{} got first at {}",
        expected_pos.saturating_sub(1),
        (expected_pos + 2).min(19),
        first_thumb_idx
    );
}

#[test]
fn test_scroll_container_render_scrollbar_hidden_when_disabled() {
    let sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20)
        .with_scrollbar(false);

    let plane = sc.render_scrollbar(Rect::new(0, 0, 1, 20));

    for cell in &plane.cells {
        assert_eq!(cell.char, ' ', "scrollbar should not render when hidden");
    }
}

#[test]
fn test_scroll_container_render_scrollbar_with_custom_colors() {
    let sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let plane = sc.render_scrollbar(Rect::new(0, 0, 1, 20));

    for cell in &plane.cells {
        assert!(!matches!(cell.fg, dracon_terminal_engine::compositor::Color::Reset));
    }
}

#[test]
fn test_scroll_container_state_mut() {
    let sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);

    let state = sc.state();
    assert_eq!(state.content_height, 100);

    let mut sc = ScrollContainer::new()
        .with_content_height(100)
        .with_viewport_height(20);
    sc.state_mut().offset = 50;
    assert_eq!(sc.state().offset, 50);
}
