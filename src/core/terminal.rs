use crate::backend::tty::{get_terminal_attr, make_raw, set_terminal_attr, Termios};
use std::env;
use std::io::{self, Write};
use std::os::fd::{AsFd, BorrowedFd};

/// Common escape sequence to restore the terminal state:
/// end sync update, show cursor, reset cursor keys, disable synchronized update,
/// disable all mouse modes, enable line wrap, exit alt screen, disable bracketed paste.
const RESTORE_SEQ: &str =
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
}

impl Capabilities {
    /// Detect terminal capabilities from $TERM environment variable.
    pub fn detect() -> Self {
        let term = env::var("TERM").unwrap_or_else(|_| "dumb".to_string());
        Self { term }
    }

    /// Returns the raw $TERM value.
    pub fn term(&self) -> &str {
        &self.term
    }

    /// Check if the terminal likely supports truecolor (24-bit color).
    ///
    /// Terminals known to support truecolor: xterm-256color, screen-256color,
    /// tmux-256color, iterm2, konsole, gnome-terminal, etc.
    pub fn supports_truecolor(&self) -> bool {
        let term_lower = self.term.to_lowercase();
        // Known truecolor-capable terminals
        if term_lower.contains("256color") || term_lower.contains("truecolor") {
            return true;
        }
        // Known non-truecolor or limited terminals
        if term_lower == "dumb" || term_lower.starts_with("vt100") || term_lower == "ansi" {
            return false;
        }
        // Common modern terminals
        let truecolor_terms = [
            "xterm", "screen", "tmux", "rxvt", "kitty", "alacritty", "wezterm",
            "ghostty", "foot", "konsole", "gnome", "terminology", "eterm",
        ];
        truecolor_terms.iter().any(|t| term_lower.contains(t))
    }

    /// Check if the terminal supports SGR mouse mode (extended mouse reporting).
    ///
    /// Most modern terminals (xterm, tmux, screen, kitty, etc.) support this.
    pub fn supports_mouse(&self) -> bool {
        let term_lower = self.term.to_lowercase();
        // Known mouse-capable terminals
        if term_lower.contains("xterm") || term_lower.contains("screen")
            || term_lower.contains("tmux") || term_lower.contains("rxvt")
            || term_lower.contains("kitty") || term_lower.contains("wezterm")
            || term_lower.contains("foot") || term_lower.contains("alacritty")
            || term_lower.contains("ghostty") || term_lower.contains("konsole")
            || term_lower.contains("gnome") || term_lower.contains("terminology") {
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
        // cleanup: show cursor, disable mouse, exit sync update, leave alt screen, pop kitty keyboard
        let _ = write!(
            self.output,
            "\x1b[<u\x1b[?25h\x1b[?1l\x1b[?2026l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1007l\x1b[?7h\x1b[?1049l\x1b[?2004l"
        );
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
        let _ = write!(
            self.output,
            "\x1b[<u\x1b[?25h\x1b[?1l\x1b[?2026l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1007l\x1b[?7h\x1b[?1049l\x1b[?2004l"
        );
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
        self.output.write_all(seq.as_bytes()).map_err(io::Error::other)
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
