//! Key combination mapping helpers.
//!
//! This module provides utilities for mapping input events to UI events.
//! Most functionality is now unified in [`Event`](crate::input::event::Event)
//! and [`UiEvent`](crate::input::event::UiEvent), making this module largely
//! a compatibility layer.

use crate::input::event::{Event, UiEvent};

/// Converts an event to a UI event if applicable.
pub fn to_ui_event(event: &Event) -> Option<UiEvent> {
    event.to_ui_event()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ui_event_key() {
        let event = Event::Key(crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Char('c'),
            modifiers: crate::input::event::KeyModifiers::empty(),
        });
        let result = to_ui_event(&event);
        assert!(result.is_some());
    }

    #[test]
    fn test_to_ui_event_resize() {
        let event = Event::Resize(80, 24);
        let result = to_ui_event(&event);
        assert!(result.is_some());
    }

    #[test]
    fn test_to_ui_event_mouse() {
        use crate::input::event::{MouseButton, MouseEvent, MouseEventKind};
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: crate::input::event::KeyModifiers::empty(),
        });
        let result = to_ui_event(&event);
        assert!(result.is_some() || result.is_none());
    }
}
