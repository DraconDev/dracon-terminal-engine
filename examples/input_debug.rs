#![allow(missing_docs)]
//! Input Debugger — Raw terminal input event inspector with color coding.
//!
//! Shows parsed events alongside raw bytes for debugging terminal input.
//!
//! Controls:
//!   q        — quit
//!   ?        — toggle help overlay
//!   c        — clear history
//!   ↑/↓      — scroll history

use dracon_terminal_engine::core::terminal::Terminal;
use dracon_terminal_engine::input::event::{Event, KeyCode, KeyEvent, MouseEventKind};
use dracon_terminal_engine::input::parser::Parser;
use std::collections::VecDeque;
use std::io::{self, Read, Write};

struct InputDebugger {
    history: VecDeque<(Vec<u8>, String)>,
    scroll_offset: usize,
    max_history: usize,
    show_help: bool,
    raw_buffer: Vec<u8>,
    event_count: usize,
    start_time: std::time::Instant,
}

impl InputDebugger {
    fn new() -> Self {
        Self {
            history: VecDeque::new(),
            scroll_offset: 0,
            max_history: 100,
            show_help: false,
            raw_buffer: Vec::new(),
            event_count: 0,
            start_time: std::time::Instant::now(),
        }
    }

    fn format_event(&self, event: &Event) -> String {
        match event {
            Event::Key(key) => {
                let mut parts = vec!["KEY".to_string()];
                if key.kind != dracon_terminal_engine::input::event::KeyEventKind::Press {
                    parts.push(format!("{:?}", key.kind));
                }
                parts.push(format!("{:?}", key.code));
                if key.modifiers.bits() != 0 {
                    parts.push(format!("{:?}", key.modifiers));
                }
                parts.join(" ")
            }
            Event::Mouse(mouse) => {
                let btn = match mouse.kind {
                    MouseEventKind::Down(b) => format!("Down({:?})", b),
                    MouseEventKind::Up(b) => format!("Up({:?})", b),
                    MouseEventKind::Drag(b) => format!("Drag({:?})", b),
                    MouseEventKind::Moved => "Moved".to_string(),
                    MouseEventKind::ScrollDown => "ScrollDown".to_string(),
                    MouseEventKind::ScrollUp => "ScrollUp".to_string(),
                };
                format!("MOUSE {} at {}, {}", btn, mouse.column, mouse.row)
            }
            Event::FocusGained => "FOCUS Gained".to_string(),
            Event::FocusLost => "FOCUS Lost".to_string(),
            Event::Paste(text) => format!("PASTE {} chars", text.len()),
            Event::Resize(w, h) => format!("RESIZE {}x{}", w, h),
        }
    }

    fn format_raw_bytes(&self, bytes: &[u8]) -> String {
        if bytes.len() <= 8 {
            bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
        } else {
            let head: String = bytes[..4].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            let tail: String = bytes[bytes.len()-4..].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            format!("{} … {} ({}B)", head, tail, bytes.len())
        }
    }

    fn add_event(&mut self, bytes: Vec<u8>, event: &Event) {
        let formatted = self.format_event(event);
        self.history.push_back((bytes, formatted));
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        self.event_count += 1;
    }

    fn render(&self, term: &mut Terminal<io::StdoutLock<'_>>) -> io::Result<()> {
        if self.show_help {
            return self.render_help(term);
        }

        let (w, h) = (80usize, 24usize);
        write!(term, "\x1b[2J\x1b[H")?;

        // Header bar
        let elapsed = self.start_time.elapsed().as_secs();
        let header = format!(
            " 󰌌 Input Debugger │ {} events │ {}s │ q:quit ?:help c:clear ",
            self.event_count, elapsed
        );
        write!(term, "\x1b[7m{: <width$}\x1b[0m\r\n", header, width = w)?;

        // Column headers
        write!(term, "\x1b[1m{: <24} │ {: <48}\x1b[0m\r\n", "RAW BYTES", "EVENT")?;
        write!(term, "{:-<80}\r\n", "")?;

        // History entries
        let visible_rows = h - 6;
        let total = self.history.len();
        let start = if total > visible_rows {
            total.saturating_sub(visible_rows).min(self.scroll_offset)
        } else {
            0
        };

        for i in 0..visible_rows {
            let idx = start + i;
            if let Some((bytes, event)) = self.history.get(idx) {
                let color = if event.starts_with("KEY") {
                    "\x1b[36m" // Cyan
                } else if event.starts_with("MOUSE") {
                    "\x1b[33m" // Yellow
                } else if event.starts_with("FOCUS") {
                    "\x1b[35m" // Magenta
                } else if event.starts_with("PASTE") {
                    "\x1b[32m" // Green
                } else {
                    "\x1b[37m" // White
                };
                let raw = self.format_raw_bytes(bytes);
                write!(term, "\x1b[90m{: <24}\x1b[0m │ {}{: <48}\x1b[0m\r\n", raw, color, event)?;
            } else {
                write!(term, "\r\n")?;
            }
        }

        // Status bar
        let scroll_info = if total > visible_rows {
            format!("[{}/{}]", start + visible_rows.min(total - start), total)
        } else {
            format!("[{}]", total)
        };
        let status = format!(" ↑/↓ scroll {} │ Press keys/mouse to see events ", scroll_info);
        write!(term, "\x1b[7m{: <width$}\x1b[0m", status, width = w)?;

        term.flush()
    }

    fn render_help(&self, term: &mut Terminal<io::StdoutLock<'_>>) -> io::Result<()> {
        write!(term, "\x1b[2J\x1b[H")?;
        write!(term, "╭────────────────────────────────────────────────────────────╮\r\n")?;
        write!(term, "│                Input Debugger Help                         │\r\n")?;
        write!(term, "├────────────────────────────────────────────────────────────┤\r\n")?;
        write!(term, "│  \x1b[1mq\x1b[0m        — Quit                                         │\r\n")?;
        write!(term, "│  \x1b[1m?\x1b[0m        — Toggle this help                            │\r\n")?;
        write!(term, "│  \x1b[1mc\x1b[0m        — Clear event history                         │\r\n")?;
        write!(term, "│  \x1b[1m↑/↓\x1b[0m      — Scroll history                              │\r\n")?;
        write!(term, "├────────────────────────────────────────────────────────────┤\r\n")?;
        write!(term, "│  Events are color-coded:                                   │\r\n")?;
        write!(term, "│    \x1b[36mKEY\x1b[0m      — Keyboard input                               │\r\n")?;
        write!(term, "│    \x1b[33mMOUSE\x1b[0m    — Mouse clicks, drags, scroll                   │\r\n")?;
        write!(term, "│    \x1b[35mFOCUS\x1b[0m    — Window focus in/out                         │\r\n")?;
        write!(term, "│    \x1b[32mPASTE\x1b[0m    — Bracketed paste                             │\r\n")?;
        write!(term, "│    \x1b[37mRESIZE\x1b[0m   — Terminal resize                              │\r\n")?;
        write!(term, "╰────────────────────────────────────────────────────────────╯\r\n")?;
        term.flush()
    }
}

fn main() -> io::Result<()> {
    println!("Preparing to enter Raw Mode...");
    println!("Type 'q' to quit, '?' for help.");
    std::thread::sleep(std::time::Duration::from_secs(1));

    let stdout = io::stdout();
    let mut term = Terminal::new(stdout)?;

    // Enable all input modes
    write!(term, "\x1b[?1000h\x1b[?1006h\x1b[>1u\x1b[?1004h\x1b[?2004h")?;

    let mut parser = Parser::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = [0u8; 128];
    let mut debugger = InputDebugger::new();

    // Initial render
    debugger.render(&mut term)?;

    loop {
        let n = handle.read(&mut buf)?;
        if n == 0 {
            break;
        }

        // Accumulate raw bytes for the event
        debugger.raw_buffer.extend_from_slice(&buf[..n]);

        for &byte in &buf[..n] {
            if let Some(event) = parser.advance(byte) {
                let raw_bytes = std::mem::take(&mut debugger.raw_buffer);

                match &event {
                    Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                        return Ok(());
                    }
                    Event::Key(KeyEvent { code: KeyCode::Char('?'), .. }) => {
                        debugger.show_help = !debugger.show_help;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Char('c'), .. }) => {
                        debugger.history.clear();
                        debugger.scroll_offset = 0;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                        if debugger.scroll_offset > 0 {
                            debugger.scroll_offset -= 1;
                        }
                    }
                    Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                        let max = debugger.history.len().saturating_sub(18);
                        if debugger.scroll_offset < max {
                            debugger.scroll_offset += 1;
                        }
                    }
                    _ => {
                        debugger.add_event(raw_bytes, &event);
                    }
                }

                debugger.render(&mut term)?;
            }
        }
    }

    Ok(())
}
