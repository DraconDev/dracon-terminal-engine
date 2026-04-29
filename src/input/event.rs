use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UiResize {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiEvent {
    Tick,
    Key { key: Cow<'static, str> },
    Resize(UiResize),
    QuitRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Paste(String),
    FocusGained,
    FocusLost,
    Unsupported(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyEventKind {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
    Media(MediaKeyCode),
    Modifier(ModifierKeyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaKeyCode {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKeyCode {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollDown,
    ScrollUp,
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u8),
}

impl Event {
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