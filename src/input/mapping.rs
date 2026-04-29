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