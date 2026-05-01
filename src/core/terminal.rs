use crate::backend::tty::{get_terminal_attr, make_raw, set_terminal_attr, Termios};
use std::io::{self, Write};
use std::os::fd::{AsFd, BorrowedFd};

/// The main RAII wrapper for the terminal.
/// When this struct is dropped, the terminal is restored to its original state.
pub struct Terminal<W: Write + AsFd> {
    original_termios: Termios,
    output: W,
}

impl<W: Write + AsFd> Drop for Terminal<W> {
    fn drop(&mut self) {
        // cleanup: show cursor, disable mouse, leave alt screen, pop kitty keyboard
        let _ = write!(
            self.output,
            "\x1b[<u\x1b[?25h\x1b[?1l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1007h\x1b[?7h\x1b[?1049l"
        );
        let _ = self.output.flush();
        // Restore terminal attributes (ignore errors for null-mode terminals)
        let _ = set_terminal_attr(self.output.as_fd(), &self.original_termios);
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

        // Safe Capture: Alt Screen, Mouse (Button Event + SGR), Kitty Keyboard, No Alt Scroll, No Wrap, No Cursor
        write!(
            writer,
            "\x1b[>1u\x1b[?1049h\x1b[?1003h\x1b[?1006h\x1b[?1007l\x1b[?7l\x1b[?25l"
        )?;
        write!(writer, "\x1b[2J\x1b[H")?;
        writer.flush()?;

        Ok(Self {
            original_termios,
            output: writer,
        })
    }

    fn new_null_mode(writer: W) -> io::Result<Self> {
        Ok(Self {
            original_termios: unsafe { std::mem::zeroed() },
            output: writer,
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
        write!(self.output, "\x1b[{};{}H", y.saturating_add(1), x.saturating_add(1)).map_err(io::Error::other)
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
