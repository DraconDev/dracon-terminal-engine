#![forbid(unsafe_code)]

use std::borrow::Cow;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// Terminal UI resize event containing the new dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiResize {
    /// New terminal width in columns.
    pub width: u16,
    /// New terminal height in rows.
    pub height: u16,
}

/// Terminal UI events from the runtime environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiEvent {
    /// Periodic tick event for rendering updates.
    Tick,
    /// Keyboard key event.
    Key {
        /// The key value.
        key: Cow<'static, str>,
    },
    /// Terminal resize event with new dimensions.
    Resize(UiResize),
    /// Request to quit the application.
    QuitRequested,
}

/// Input events from user interaction with the terminal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputEvent {
    /// Keyboard input event.
    Key(KeyEvent),
    /// Mouse input event.
    Mouse(MouseEvent),
    /// Terminal resize event with width and height.
    Resize(u16, u16),
    /// Pasted text content.
    Paste(String),
    /// Terminal gained focus.
    FocusGained,
    /// Terminal lost focus.
    FocusLost,
    /// Unsupported input event raw bytes.
    Unsupported(Vec<u8>),
}

/// Keyboard input event with key code, modifiers, and event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    /// The key code that was pressed or released.
    pub code: KeyCode,
    /// Modifier keys active during the event.
    pub modifiers: KeyModifiers,
    /// The kind of key event (press, repeat, release).
    pub kind: KeyEventKind,
}

/// The type of keyboard event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyEventKind {
    /// Key was pressed down.
    Press,
    /// Key was held and auto-repeated.
    Repeat,
    /// Key was released.
    Release,
}

/// The key code representing which key was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Backspace key.
    Backspace,
    /// Enter key.
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
    /// Backward Tab key (Shift+Tab).
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Function key F followed by number (0-12).
    F(u8),
    /// Printable character key.
    Char(char),
    /// Null key.
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
    /// Menu key.
    Menu,
    /// Keypad begin key.
    KeypadBegin,
    /// Media key with specific media key code.
    Media(MediaKeyCode),
    /// Modifier key with specific modifier key code.
    Modifier(ModifierKeyCode),
}

/// Media key codes for multimedia keyboard keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaKeyCode {
    /// Play media key.
    Play,
    /// Pause media key.
    Pause,
    /// Play/Pause toggle key.
    PlayPause,
    /// Reverse media key.
    Reverse,
    /// Stop media key.
    Stop,
    /// Fast forward media key.
    FastForward,
    /// Rewind media key.
    Rewind,
    /// Next track media key.
    TrackNext,
    /// Previous track media key.
    TrackPrevious,
    /// Record media key.
    Record,
    /// Lower volume media key.
    LowerVolume,
    /// Raise volume media key.
    RaiseVolume,
    /// Mute volume media key.
    MuteVolume,
}

/// Modifier key codes for keyboard modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKeyCode {
    /// Left Shift key.
    LeftShift,
    /// Left Control key.
    LeftControl,
    /// Left Alt key.
    LeftAlt,
    /// Left Super (Windows/Command) key.
    LeftSuper,
    /// Left Hyper key.
    LeftHyper,
    /// Left Meta key.
    LeftMeta,
    /// Right Shift key.
    RightShift,
    /// Right Control key.
    RightControl,
    /// Right Alt key.
    RightAlt,
    /// Right Super (Windows/Command) key.
    RightSuper,
    /// Right Hyper key.
    RightHyper,
    /// Right Meta key.
    RightMeta,
    /// ISO Level 3 Shift key.
    IsoLevel3Shift,
    /// ISO Level 5 Shift key.
    IsoLevel5Shift,
}

bitflags! {
    /// Bitflags representing active keyboard modifier keys.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
    pub struct KeyModifiers: u8 {
        /// Shift modifier key.
        const SHIFT = 0b0000_0001;
        /// Control modifier key.
        const CONTROL = 0b0000_0010;
        /// Alt modifier key.
        const ALT = 0b0000_0100;
        /// Super (Windows/Command) modifier key.
        const SUPER = 0b0000_1000;
        /// Hyper modifier key.
        const HYPER = 0b0001_0000;
        /// Meta modifier key.
        const META = 0b0010_0000;
    }
}

/// Mouse input event with position, type, and active modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseEvent {
    /// The type of mouse event.
    pub kind: MouseEventKind,
    /// Column position where the event occurred.
    pub column: u16,
    /// Row position where the event occurred.
    pub row: u16,
    /// Modifier keys active during the event.
    pub modifiers: KeyModifiers,
}

/// The type of mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseEventKind {
    /// Mouse button pressed down.
    Down(MouseButton),
    /// Mouse button released.
    Up(MouseButton),
    /// Mouse button dragged while held.
    Drag(MouseButton),
    /// Mouse moved without button press.
    Moved,
    /// Scroll wheel moved down.
    ScrollDown,
    /// Scroll wheel moved up.
    ScrollUp,
    /// Scroll wheel moved left.
    ScrollLeft,
    /// Scroll wheel moved right.
    ScrollRight,
}

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
    /// Back mouse button.
    Back,
    /// Forward mouse button.
    Forward,
    /// Other mouse button with vendor-specific code.
    Other(u8),
}

/// Trait for rendering the UI state to the terminal.
pub trait UiRenderer<State> {
    /// Error type returned when rendering fails.
    type Error;

    /// Renders the given state to the terminal.
    fn render(&mut self, state: &State) -> Result<(), Self::Error>;
}

/// Trait for polling UI events from the runtime environment.
pub trait UiEventSource {
    /// Error type returned when event fetching fails.
    type Error;

    /// Returns the next UI event, or None if no event is available.
    fn next_event(&mut self) -> Result<Option<UiEvent>, Self::Error>;
}

/// Trait for the main UI runtime loop that coordinates rendering and events.
pub trait UiRuntime<State> {
    /// Error type returned when the runtime fails.
    type Error;

    /// Runs the main event loop, rendering the state and processing events.
    fn run<R, E>(
        &mut self,
        renderer: &mut R,
        events: &mut E,
        state: &mut State,
    ) -> Result<(), Self::Error>
    where
        R: UiRenderer<State>,
        E: UiEventSource;
}
