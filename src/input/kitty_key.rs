//! Kitty keyboard protocol parser.
//!
//! Parses the Kitty keyboard protocol extension sequences.

use super::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Kitty Private Use Area start codepoint for keyboard extensions.
/// The Kitty protocol maps special keys to codepoints in the Private Use Area
/// (U+E0000-U+EFFFF) to avoid conflicts with standard Unicode.
const KITTY_PUA_START: u32 = 57344; // U+E0000

/// Kitty Private Use Area end codepoint (inclusive).
const KITTY_PUA_END: u32 = 63743; // U+EFFFF

/// Kitty modifier bit values.
mod modifier {
    /// Shift modifier.
    pub const SHIFT: u8 = 1;
    /// Alt modifier.
    pub const ALT: u8 = 2;
    /// Control modifier.
    pub const CTRL: u8 = 4;
    /// Super (Windows/Command) modifier.
    pub const SUPER: u8 = 8;
    /// Hyper modifier.
    pub const HYPER: u8 = 16;
    /// Meta modifier.
    pub const META: u8 = 32;
}

/// Kitty key event types.
mod event_type {
    /// Key press.
    pub const PRESS: u8 = 1;
    /// Key repeat (held down).
    pub const REPEAT: u8 = 2;
    /// Key release.
    pub const RELEASE: u8 = 3;
}

/// Specific Kitty PUA codepoints for special keys.
mod key_codes {
    /// Escape key.
    pub const ESC: u32 = 57344;
    /// Insert key.
    pub const INSERT: u32 = 57345;
    /// Delete key.
    pub const DELETE: u32 = 57346;
    /// Home key.
    pub const HOME: u32 = 57347;
    /// End key.
    pub const END: u32 = 57348;
    /// Page Up key.
    pub const PAGE_UP: u32 = 57362;
    /// Page Down key.
    pub const PAGE_DOWN: u32 = 57363;
    /// Up Arrow key.
    pub const UP: u32 = 57358;
    /// Down Arrow key.
    pub const DOWN: u32 = 57359;
    /// Left Arrow key.
    pub const LEFT: u32 = 57360;
    /// Right Arrow key.
    pub const RIGHT: u32 = 57361;
    /// F1 key.
    pub const F1: u32 = 57364;
    /// F2 key.
    pub const F2: u32 = 57365;
    /// F3 key.
    pub const F3: u32 = 57366;
    /// F4 key.
    pub const F4: u32 = 57367;
    /// F5 key.
    pub const F5: u32 = 57368;
    /// F6 key.
    pub const F6: u32 = 57369;
    /// F7 key.
    pub const F7: u32 = 57370;
    /// F8 key.
    pub const F8: u32 = 57371;
    /// F9 key.
    pub const F9: u32 = 57372;
    /// F10 key.
    pub const F10: u32 = 57373;
    /// F11 key.
    pub const F11: u32 = 57374;
    /// F12 key.
    pub const F12: u32 = 57375;
}

/// Parses a Kitty keyboard event from its component parts.
///
/// Returns `Some(Event::Key(...))` on success, or `None` if the parts
/// do not represent a valid Kitty keyboard sequence.
pub fn parse_kitty_keyboard(parts: &[&str]) -> Option<Event> {
    if parts.is_empty() {
        return None;
    }

    let code_val: u32 = parts[0].parse().ok()?;
    let mut modifiers = KeyModifiers::empty();
    let mut kind = KeyEventKind::Press; // Default

    // Second param is modifiers (1-based bitmask)
    if parts.len() > 1 {
        if let Ok(mod_val) = parts[1].parse::<u8>() {
            let m = mod_val.saturating_sub(1);
            if (m & modifier::SHIFT) != 0 {
                modifiers.insert(KeyModifiers::SHIFT);
            }
            if (m & modifier::ALT) != 0 {
                modifiers.insert(KeyModifiers::ALT);
            }
            if (m & modifier::CTRL) != 0 {
                modifiers.insert(KeyModifiers::CONTROL);
            }
            if (m & modifier::SUPER) != 0 {
                modifiers.insert(KeyModifiers::SUPER);
            }
            if (m & modifier::HYPER) != 0 {
                modifiers.insert(KeyModifiers::HYPER);
            }
            if (m & modifier::META) != 0 {
                modifiers.insert(KeyModifiers::META);
            }
        }
    }

    // Third param is event type (Press, Repeat, Release)
    if parts.len() > 2 {
        if let Ok(type_val) = parts[2].parse::<u8>() {
            match type_val {
                event_type::PRESS => kind = KeyEventKind::Press,
                event_type::REPEAT => kind = KeyEventKind::Repeat,
                event_type::RELEASE => kind = KeyEventKind::Release,
                _ => {}
            }
        }
    }

    // Map Kitty Functional Keys (PUA range)
    let key_code = if (KITTY_PUA_START..=KITTY_PUA_END).contains(&code_val) {
        // PUA mapping
        map_kitty_pua(code_val)
    } else {
        // Standard ASCII / Unicode
        match code_val {
            27 => KeyCode::Esc,
            13 => KeyCode::Enter,
            9 => KeyCode::Tab,
            127 => KeyCode::Backspace,
            _ => {
                if let Some(c) = std::char::from_u32(code_val) {
                    KeyCode::Char(c)
                } else {
                    return None;
                }
            }
        }
    };

    Some(Event::Key(KeyEvent {
        code: key_code,
        modifiers,
        kind,
    }))
}

fn map_kitty_pua(code: u32) -> KeyCode {
    // Basic mapping, incomplete
    match code {
        key_codes::F1 => KeyCode::F(1),
        key_codes::F2 => KeyCode::F(2),
        key_codes::F3 => KeyCode::F(3),
        key_codes::F4 => KeyCode::F(4),
        key_codes::F5 => KeyCode::F(5),
        key_codes::F6 => KeyCode::F(6),
        key_codes::F7 => KeyCode::F(7),
        key_codes::F8 => KeyCode::F(8),
        key_codes::F9 => KeyCode::F(9),
        key_codes::F10 => KeyCode::F(10),
        key_codes::F11 => KeyCode::F(11),
        key_codes::F12 => KeyCode::F(12),
        // Cursor Keys
        key_codes::UP => KeyCode::Up,
        key_codes::DOWN => KeyCode::Down,
        key_codes::LEFT => KeyCode::Left,
        key_codes::RIGHT => KeyCode::Right,
        key_codes::PAGE_UP => KeyCode::PageUp,
        key_codes::PAGE_DOWN => KeyCode::PageDown,
        key_codes::ESC => KeyCode::Esc,
        key_codes::INSERT => KeyCode::Insert,
        key_codes::DELETE => KeyCode::Delete,
        key_codes::HOME => KeyCode::Home,
        key_codes::END => KeyCode::End,
        _ => {
            // Unknown PUA codepoint — no meaningful key mapping.
            // KeyCode::Null should only be used for the actual null key (code 0).
            // Using Unsupported preserves the raw codepoint for debugging without
            // producing Null events that widgets may interpret as valid input.
            KeyCode::Unsupported(code)
        }
    }
}
