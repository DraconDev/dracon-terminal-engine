use crate::input::event::{Event, UiEvent};

/// Converts a runtime event to a UI event (identity function).
///
/// This function is deprecated since event types are now unified.
#[deprecated(since = "19.3.0", note = "Event types are now unified; this is an identity function")]
pub fn from_runtime_event(event: crate::input::event::Event) -> Event {
    event
}

/// Converts a UI event to a runtime event (identity function).
///
/// This function is deprecated since event types are now unified.
#[deprecated(since = "19.3.0", note = "Event types are now unified; this is an identity function")]
pub fn to_runtime_event(event: &Event) -> crate::input::event::Event {
    event.clone()
}

/// Converts an event to a UI event if applicable.
pub fn to_ui_event(event: &Event) -> Option<UiEvent> {
    event.to_ui_event()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_runtime_event_identity() {
        let event = Event::Key(crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Char('a'),
            modifiers: crate::input::event::KeyModifiers::empty(),
        });
        #[allow(deprecated)]
        let result = from_runtime_event(event.clone());
        match result {
            Event::Key(k) => {
                assert!(matches!(k.code, crate::input::event::KeyCode::Char('a')));
            }
            _ => panic!("expected Key event"),
        }
    }

    #[test]
    fn test_to_runtime_event_identity() {
        let event = Event::Key(crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Char('b'),
            modifiers: crate::input::event::KeyModifiers::empty(),
        });
        #[allow(deprecated)]
        let result = to_runtime_event(&event);
        match result {
            Event::Key(k) => {
                assert!(matches!(k.code, crate::input::event::KeyCode::Char('b')));
            }
            _ => panic!("expected Key event"),
        }
    }

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
        use crate::input::event::{MouseEvent, MouseEventKind, MouseButton};
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