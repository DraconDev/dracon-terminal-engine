//! Input handling module for terminal input processing.

#[doc = "Input events (KeyEvent, MouseEvent, Event, UiEvent, KeyCode, MouseButton, etc.)."]
pub mod event;
#[doc = "Kitty keyboard protocol parser."]
pub mod kitty_key;
#[doc = "Key combination mapping helpers."]
pub mod mapping;
#[doc = "ANSI escape sequence parser (SGR mouse, chords, bracketed paste)."]
pub mod parser;
#[doc = "Input reader wrapper."]
pub mod reader;

#[cfg(feature = "tokio")]
pub mod async_reader;

pub use parser::Parser;
pub use reader::InputReader;
