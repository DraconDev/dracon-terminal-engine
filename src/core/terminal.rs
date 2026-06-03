//! Terminal initialization, capability detection, and escape sequence utilities.
//!
//! This module provides:
//!
//! - [`Terminal`](self::Terminal) — RAII wrapper for terminal setup/teardown with raw mode
//! - [`Capabilities`](self::Capabilities) — Detects terminal features from environment variables
//! - [`CursorShape`](self::CursorShape) — Terminal cursor shape variants
//! - Escape sequences for cursor movement, visibility, bracketed paste, mouse modes
//!
//! # Example
//!
//! ```ignore
//! let mut terminal = Terminal::new()?;
//! terminal.enter_alternate_screen()?;
//! // ... render UI ...
//! drop(terminal); // Restores terminal on drop
//! ```

use crate::backend::tty::{get_terminal_attr, make_raw, set_terminal_attr, Termios};
use std::env;
use std::io::{self, Write};
use std::os::fd::{AsFd, BorrowedFd};

/// Common escape sequence to restore the terminal state:
/// end sync update, show cursor, reset cursor keys, disable synchronized update,
/// disable all mouse modes, enable line wrap, exit alt screen, disable bracketed paste.
pub const RESTORE_SEQ: &str =
    "\x1b[<u\x1b[?25h\x1b[?1l\x1b[?2026l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1007l\x1b[?7h\x1b[?1049l\x1b[?2004l";

/// Cursor shape for terminal cursor style sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    /// Block cursor (█)
    Block,
    /// Underline cursor (_)
    Underline,
    /// Bar cursor (▏)
    Bar,
    /// blinking variants
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

/// Terminal capability detection and custom escape sequence utilities.
pub struct Capabilities {
    term: String,
    colorterm: Option<String>,
    term_program: Option<String>,
    kitty_window_id: bool,
}

impl Capabilities {
    /// Detect terminal capabilities from environment variables.
    ///
    /// Checks `$TERM`, `$COLORTERM`, `$TERM_PROGRAM`, and `$KITTY_WINDOW_ID`
    /// to determine color, mouse, and feature support.
    pub fn detect() -> Self {
        let term = env::var("TERM").unwrap_or_else(|_| "dumb".to_string());
        let colorterm = env::var("COLORTERM").ok();
        let term_program = env::var("TERM_PROGRAM").ok();
        let kitty_window_id = env::var("KITTY_WINDOW_ID").is_ok();
        Self {
            term,
            colorterm,
            term_program,
            kitty_window_id,
        }
    }

    /// Returns the raw $TERM value.
    pub fn term(&self) -> &str {
        &self.term
    }

    /// Check if the terminal likely supports truecolor (24-bit color).
    ///
    /// Checks `$COLORTERM` (set to `truecolor` or `24bit` by many terminals),
    /// `$TERM_PROGRAM` (set by iTerm2, VSCode, etc.), and `$KITTY_WINDOW_ID`
    /// (confirms kitty terminal) before falling back to `$TERM` matching.
    pub fn supports_truecolor(&self) -> bool {
        if let Some(ref ct) = self.colorterm {
            let ct_lower = ct.to_lowercase();
            if ct_lower == "truecolor" || ct_lower == "24bit" {
                return true;
            }
        }

        if let Some(ref tp) = self.term_program {
            let tp_lower = tp.to_lowercase();
            if tp_lower.contains("iterm")
                || tp_lower.contains("vscode")
                || tp_lower.contains("hyper")
                || tp_lower.contains("wezterm")
                || tp_lower.contains("ghostty")
            {
                return true;
            }
        }

        if self.kitty_window_id {
            return true;
        }

        let term_lower = self.term.to_lowercase();
        if term_lower.contains("256color") || term_lower.contains("truecolor") {
            return true;
        }
        if term_lower == "dumb" || term_lower.starts_with("vt100") || term_lower == "ansi" {
            return false;
        }
        let truecolor_terms = [
            "xterm",
            "screen",
            "tmux",
            "rxvt",
            "kitty",
            "alacritty",
            "wezterm",
            "ghostty",
            "foot",
            "konsole",
            "gnome",
            "terminology",
            "eterm",
        ];
        truecolor_terms.iter().any(|t| term_lower.contains(t))
    }

    /// Check if the terminal supports SGR mouse mode (extended mouse reporting).
    ///
    /// Most modern terminals (xterm, tmux, screen, kitty, etc.) support this.
    pub fn supports_mouse(&self) -> bool {
        let term_lower = self.term.to_lowercase();
        // Known mouse-capable terminals
        if term_lower.contains("xterm")
            || term_lower.contains("screen")
            || term_lower.contains("tmux")
            || term_lower.contains("rxvt")
            || term_lower.contains("kitty")
            || term_lower.contains("wezterm")
            || term_lower.contains("foot")
            || term_lower.contains("alacritty")
            || term_lower.contains("ghostty")
            || term_lower.contains("konsole")
            || term_lower.contains("gnome")
            || term_lower.contains("terminology")
        {
            return true;
        }
        // Default to likely supported for unknown modern terminals
        !term_lower.is_empty() && term_lower != "dumb"
    }

    /// Check if the terminal supports Unicode wide character rendering.
    ///
    /// Most modern terminals support this. Returns true for known supportive
    /// terminals, false for dumb/minimal terminals.
    pub fn supports_unicode_width(&self) -> bool {
        let term_lower = self.term.to_lowercase();
        term_lower != "dumb" && !term_lower.starts_with("vt100")
    }

    /// Check if the terminal supports setting window title via OSC sequences.
    ///
    /// OSC 0 (icon name + window title) and OSC 21 (window title only) are
    /// supported by most terminal emulators.
    pub fn supports_title(&self) -> bool {
        let term_lower = self.term.to_lowercase();
        // Dumb terminals and some very old ones don't support titles
        if term_lower == "dumb" || term_lower.starts_with("vt100") {
            return false;
        }
        true
    }
}

/// The main RAII wrapper for the terminal.
/// When this struct is dropped, the terminal is restored to its original state.
pub struct Terminal<W: Write + AsFd> {
    original_termios: Option<Termios>,
    output: W,
    capabilities: Capabilities,
    is_null_mode: bool,
}

impl<W: Write + AsFd> Drop for Terminal<W> {
    fn drop(&mut self) {
        let _ = write!(self.output, "{}", RESTORE_SEQ);
        let _ = self.output.flush();
        // Restore terminal attributes (skip in null mode)
        if !self.is_null_mode {
            if let Some(ref termios) = self.original_termios {
                let _ = set_terminal_attr(self.output.as_fd(), termios);
            }
        }
    }
}

impl<W: Write + AsFd> Terminal<W> {
    /// Enter "God Mode" (Raw Mode + Alternate Screen).
    ///
    /// Falls back to null mode (no-op) when `writer` is not a TTY
    /// (e.g., when stdout is piped in a test environment).
    pub fn new(mut writer: W) -> io::Result<Self> {
        let fd = writer.as_fd();
        let original_termios = match get_terminal_attr(fd) {
            Ok(t) => t,
            Err(e) if e.raw_os_error() == Some(25) => {
                return Self::new_null_mode(writer);
            }
            Err(e) => return Err(e),
        };

        let mut termios = original_termios;
        make_raw(&mut termios);
        set_terminal_attr(fd, &termios)?;

        // Safe Capture: Alt Screen, Mouse (Button Event + SGR), Kitty Keyboard, No Alt Scroll, No Wrap, No Cursor, Bracketed Paste
        write!(
            writer,
            "\x1b[>1u\x1b[?1049h\x1b[?1003h\x1b[?1006h\x1b[?1007l\x1b[?7l\x1b[?25l\x1b[?2004h"
        )?;
        write!(writer, "\x1b[2J\x1b[H")?;
        writer.flush()?;

        Ok(Self {
            original_termios: Some(original_termios),
            output: writer,
            capabilities: Capabilities::detect(),
            is_null_mode: false,
        })
    }

    fn new_null_mode(writer: W) -> io::Result<Self> {
        Ok(Self {
            original_termios: None,
            output: writer,
            capabilities: Capabilities::detect(),
            is_null_mode: true,
        })
    }

    /// Access the underlying writer (e.g., to flush)
    pub fn inner(&mut self) -> &mut W {
        &mut self.output
    }

    /// Shows the terminal cursor.
    pub fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.output, "\x1b[?25h").map_err(io::Error::other)
    }

    /// Hides the terminal cursor.
    pub fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.output, "\x1b[?25l").map_err(io::Error::other)
    }

    /// Sets the cursor position (1-indexed, as terminals expect).
    pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        write!(
            self.output,
            "\x1b[{};{}H",
            y.saturating_add(1),
            x.saturating_add(1)
        )
        .map_err(io::Error::other)
    }

    /// Temporarily restore terminal to normal mode for child processes.
    pub fn suspend(&mut self) -> io::Result<()> {
        let _ = write!(self.output, "{}", RESTORE_SEQ);
        let _ = self.output.flush();
        if !self.is_null_mode {
            if let Some(ref termios) = self.original_termios {
                let _ = set_terminal_attr(self.output.as_fd(), termios);
            }
        }
        Ok(())
    }

    /// Re-enter raw mode + alternate screen after suspend().
    pub fn resume(&mut self) -> io::Result<()> {
        if !self.is_null_mode {
            let fd = self.output.as_fd();
            if let Some(ref termios) = self.original_termios {
                let mut raw = *termios;
                make_raw(&mut raw);
                set_terminal_attr(fd, &raw)?;
            }
        }
        write!(
            self.output,
            "\x1b[>1u\x1b[?1049h\x1b[?1003h\x1b[?1006h\x1b[?1007l\x1b[?7l\x1b[?25l\x1b[?2004h"
        )?;
        write!(self.output, "\x1b[2J\x1b[H")?;
        self.output.flush()?;
        Ok(())
    }

    // ── Capabilities ────────────────────────────────────────────────────────

    /// Returns the terminal capabilities detected from $TERM.
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    // ── Custom Escape Sequences ────────────────────────────────────────────

    /// Emit a raw OSC/DCS sequence directly to the terminal.
    ///
    /// Use this for advanced terminal control that isn't covered by
    /// other methods. The sequence should not include the final ST or BEL.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Set window title
    /// terminal.emit("\x1b]0;My Title\x07")?;
    /// // Set foreground color
    /// terminal.emit("\x1b]10;#FF0000\x07")?;
    /// ```
    pub fn emit(&mut self, seq: &str) -> io::Result<()> {
        self.output
            .write_all(seq.as_bytes())
            .map_err(io::Error::other)
    }

    /// Set the terminal window title.
    ///
    /// Uses OSC 0 (icon name + window title) or OSC 21 (window title only)
    /// depending on the sequence. Falls back gracefully if the terminal
    /// doesn't support titles.
    pub fn set_title(&mut self, title: &str) -> io::Result<()> {
        if !self.capabilities.supports_title() {
            return Ok(());
        }
        write!(self.output, "\x1b]0;{title}\x07").map_err(io::Error::other)
    }

    /// Set the terminal icon name (usually shown in taskbar/dock).
    ///
    /// Uses OSC 1. Only supported by some terminal emulators.
    pub fn set_icon(&mut self, icon: &str) -> io::Result<()> {
        write!(self.output, "\x1b]1;{icon}\x07").map_err(io::Error::other)
    }

    /// Set the cursor shape/style.
    ///
    /// Uses the DECSCUSR sequence (CSI Ps SP q) with values:
    /// 0=blinking block, 1=blinking block (default), 2=steady block,
    /// 3=blinking underline, 4=steady underline, 5=blinking bar, 6=steady bar.
    ///
    /// Note: Not all terminals support all shapes.
    pub fn set_cursor_style(&mut self, shape: CursorShape) -> io::Result<()> {
        let code = match shape {
            CursorShape::Block => 2,
            CursorShape::Underline => 4,
            CursorShape::Bar => 6,
            CursorShape::BlinkingBlock => 1,
            CursorShape::BlinkingUnderline => 3,
            CursorShape::BlinkingBar => 5,
        };
        write!(self.output, "\x1b[{code} q").map_err(io::Error::other)
    }
}

impl<W: Write + AsFd> Write for Terminal<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

impl<W: Write + AsFd> AsFd for Terminal<W> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.output.as_fd()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_shape_variants() {
        // Test that all cursor shapes exist
        assert_eq!(CursorShape::Block, CursorShape::Block);
        assert_eq!(CursorShape::Underline, CursorShape::Underline);
        assert_eq!(CursorShape::Bar, CursorShape::Bar);
        assert_eq!(CursorShape::BlinkingBlock, CursorShape::BlinkingBlock);
        assert_eq!(
            CursorShape::BlinkingUnderline,
            CursorShape::BlinkingUnderline
        );
        assert_eq!(CursorShape::BlinkingBar, CursorShape::BlinkingBar);
    }

    #[test]
    fn test_cursor_shape_debug() {
        let shape = CursorShape::Block;
        let debug_str = format!("{:?}", shape);
        assert!(debug_str.contains("Block"));
    }

    #[test]
    fn test_capabilities_detect() {
        let caps = Capabilities::detect();
        // Should have some term value
        assert!(!caps.term().is_empty() || caps.term() == "dumb");
    }

    #[test]
    fn test_capabilities_term() {
        let caps = Capabilities::detect();
        let term = caps.term();
        // term should return a valid string
        assert_eq!(term, caps.term());
    }

    #[test]
    fn test_capabilities_supports_title() {
        let caps = Capabilities::detect();
        // Dumb terminals should not support title
        if caps.term() == "dumb" {
            assert!(!caps.supports_title());
        }
    }

    #[test]
    fn test_capabilities_supports_unicode_width() {
        let caps = Capabilities::detect();
        // Dumb terminals should not support unicode width
        if caps.term() == "dumb" {
            assert!(!caps.supports_unicode_width());
        }
    }

    #[test]
    fn test_capabilities_supports_truecolor() {
        let caps = Capabilities::detect();
        // Should return a boolean without panicking
        let _ = caps.supports_truecolor();
    }

    #[test]
    fn test_capabilities_supports_mouse() {
        let caps = Capabilities::detect();
        // Should return a boolean without panicking
        let _ = caps.supports_mouse();
    }

    #[test]
    fn test_restore_seq() {
        // Verify RESTORE_SEQ contains expected sequences
        assert!(RESTORE_SEQ.contains("\x1b[?25h")); // Show cursor
        assert!(RESTORE_SEQ.contains("\x1b[?1049l")); // Exit alt screen
        assert!(RESTORE_SEQ.contains("\x1b[?2004l")); // Disable bracketed paste
    }
}
