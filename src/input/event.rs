use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// UI resize event payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UiResize {
    /// Terminal width in columns.
    pub width: u16,
    /// Terminal height in rows.
    pub height: u16,
}

/// High-level UI event, derived from input events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiEvent {
    /// Periodic tick (approx. every 250ms).
    Tick,
    /// Keyboard key pressed.
    Key {
        /// The key identifier string.
        key: Cow<'static, str>,
    },
    /// Terminal was resized.
    Resize(UiResize),
    /// Application shutdown requested.
    QuitRequested,
}

/// An input event from the terminal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Event {
    /// Keyboard key event.
    Key(KeyEvent),
    /// Mouse event.
    Mouse(MouseEvent),
    /// Terminal resized to (width, height).
    Resize(u16, u16),
    /// Bracketed paste started with the pasted text.
    Paste(String),
    /// Terminal gained focus.
    FocusGained,
    /// Terminal lost focus.
    FocusLost,
    /// Unrecognized escape sequence payload.
    Unsupported(Vec<u8>),
}

/// A keyboard event with code, modifiers, and kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    /// The key code.
    pub code: KeyCode,
    /// Active modifier keys.
    pub modifiers: KeyModifiers,
    /// Whether the key was pressed, repeated, or released.
    pub kind: KeyEventKind,
}

/// Whether a key was pressed, repeated, or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyEventKind {
    /// Key was initially pressed.
    Press,
    /// Key is repeating due to being held.
    Repeat,
    /// Key was released.
    Release,
}

/// A keyboard key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Backspace key.
    Backspace,
    /// Enter / Return key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Tab key.
    Tab,
    /// Shift+Tab key.
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Function key F1–F64.
    F(u8),
    /// A printable character.
    Char(char),
    /// Null key (Ctrl+@).
    Null,
    /// Escape key.
    Esc,
    /// Caps Lock key.
    CapsLock,
    /// Scroll Lock key.
    ScrollLock,
    /// Num Lock key.
    NumLock,
    /// Print Screen key.
    PrintScreen,
    /// Pause key.
    Pause,
    /// Application menu key.
    Menu,
    /// KeypadBegin key.
    KeypadBegin,
    /// A media key.
    Media(MediaKeyCode),
    /// A modifier key.
    Modifier(ModifierKeyCode),
}

/// A media key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaKeyCode {
    /// Play media key.
    Play,
    /// Pause media key.
    Pause,
    /// Play/Pause toggle.
    PlayPause,
    /// Reverse / search backwards.
    Reverse,
    /// Stop media.
    Stop,
    /// Fast forward.
    FastForward,
    /// Rewind.
    Rewind,
    /// Next track.
    TrackNext,
    /// Previous track.
    TrackPrevious,
    /// Record.
    Record,
    /// Lower volume.
    LowerVolume,
    /// Raise volume.
    RaiseVolume,
    /// Mute volume.
    MuteVolume,
}

/// A modifier key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKeyCode {
    /// Left Shift.
    LeftShift,
    /// Left Control.
    LeftControl,
    /// Left Alt / Option.
    LeftAlt,
    /// Left Super / Command / Win.
    LeftSuper,
    /// Left Hyper.
    LeftHyper,
    /// Left Meta.
    LeftMeta,
    /// Right Shift.
    RightShift,
    /// Right Control.
    RightControl,
    /// Right Alt / Option.
    RightAlt,
    /// Right Super / Command / Win.
    RightSuper,
    /// Right Hyper.
    RightHyper,
    /// Right Meta.
    RightMeta,
    /// ISO Level 3 Shift (AltGr).
    IsoLevel3Shift,
    /// ISO Level 5 Shift.
    IsoLevel5Shift,
}

bitflags! {
    /// Modifier keys active during an input event.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
    pub struct KeyModifiers: u8 {
        /// Shift key.
        const SHIFT = 0b0000_0001;
        /// Control key.
        const CONTROL = 0b0000_0010;
        /// Alt / Option key.
        const ALT = 0b0000_0100;
        /// Super / Command / Windows key.
        const SUPER = 0b0000_1000;
        /// Hyper key.
        const HYPER = 0b0001_0000;
        /// Meta key.
        const META = 0b0010_0000;
    }
}

/// A mouse input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseEvent {
    /// The kind of mouse event.
    pub kind: MouseEventKind,
    /// Column (0-indexed) where the event occurred.
    pub column: u16,
    /// Row (0-indexed) where the event occurred.
    pub row: u16,
    /// Modifier keys active during the event.
    pub modifiers: KeyModifiers,
}

/// The kind of mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseEventKind {
    /// Mouse button was pressed.
    Down(MouseButton),
    /// Mouse button was released.
    Up(MouseButton),
    /// Mouse button was dragged while held.
    Drag(MouseButton),
    /// Mouse was moved without any button held.
    Moved,
    /// Vertical scroll down (towards user).
    ScrollDown,
    /// Vertical scroll up (away from user).
    ScrollUp,
    /// Horizontal scroll left.
    ScrollLeft,
    /// Horizontal scroll right.
    ScrollRight,
}

/// A mouse button identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
    /// Back / Browser Back button (button 8).
    Back,
    /// Forward / Browser Forward button (button 9).
    Forward,
    /// An extended button number.
    Other(u8),
}

impl Event {
    /// Converts this event to a high-level UI event.
    pub fn to_ui_event(&self) -> Option<UiEvent> {
        match self {
            Event::Resize(width, height) => Some(UiEvent::Resize(UiResize {
                width: *width,
                height: *height,
            })),
            Event::Key(key) => Some(UiEvent::Key {
                key: Cow::Owned(format_key(key)),
            }),
            _ => None,
        }
    }
}

fn format_key(key: &KeyEvent) -> String {
    let mut out = String::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        out.push_str("ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        out.push_str("alt+");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        out.push_str("shift+");
    }
    if key.modifiers.contains(KeyModifiers::SUPER) {
        out.push_str("super+");
    }
    if key.modifiers.contains(KeyModifiers::HYPER) {
        out.push_str("hyper+");
    }
    if key.modifiers.contains(KeyModifiers::META) {
        out.push_str("meta+");
    }

    out.push_str(match key.code {
        KeyCode::Backspace => "backspace",
        KeyCode::Enter => "enter",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Up => "up",
        KeyCode::Down => "down",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "page_up",
        KeyCode::PageDown => "page_down",
        KeyCode::Tab => "tab",
        KeyCode::BackTab => "backtab",
        KeyCode::Delete => "delete",
        KeyCode::Insert => "insert",
        KeyCode::Null => "null",
        KeyCode::Esc => "esc",
        KeyCode::CapsLock => "caps_lock",
        KeyCode::ScrollLock => "scroll_lock",
        KeyCode::NumLock => "num_lock",
        KeyCode::PrintScreen => "print_screen",
        KeyCode::Pause => "pause",
        KeyCode::Menu => "menu",
        KeyCode::KeypadBegin => "keypad_begin",
        KeyCode::Char(c) => return format!("{out}{c}"),
        KeyCode::F(n) => return format!("{out}f{n}"),
        KeyCode::Media(media) => return format!("{out}media::{media:?}"),
        KeyCode::Modifier(modifier) => return format!("{out}modifier::{modifier:?}"),
    });

    match key.kind {
        KeyEventKind::Press => out.push_str(":press"),
        KeyEventKind::Repeat => out.push_str(":repeat"),
        KeyEventKind::Release => out.push_str(":release"),
    }

    out
}