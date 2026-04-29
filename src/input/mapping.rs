use crate::input::event::{Event, UiEvent};

#[deprecated(since = "19.3.0", note = "Event types are now unified; this is an identity function")]
pub fn from_runtime_event(event: crate::input::event::Event) -> Event {
    event
}

#[deprecated(since = "19.3.0", note = "Event types are now unified; this is an identity function")]
pub fn to_runtime_event(event: &Event) -> crate::input::event::Event {
    event.clone()
}

pub fn to_ui_event(event: &Event) -> Option<UiEvent> {
    event.to_ui_event()
}