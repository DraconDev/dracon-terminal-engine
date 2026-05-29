//! Input reader for terminal stdin with signal handling.
//!
//! This module spawns a background thread to read from stdin and parse
//! input events. It handles:
//!
//! - **SIGWINCH** — Window resize signals (gracefully degrades if signal registration fails)
//! - **Non-blocking poll** — Uses `poll()` for efficient input detection
//! - **Timeout handling** — Processes incomplete escape sequences

use signal_hook::consts::signal::SIGWINCH;
use signal_hook::iterator::Signals;
use std::io::Read;
use std::os::fd::AsFd;
use std::thread;

use super::event::Event;
use super::parser::Parser;
use crate::backend::tty;

/// Buffer size for reading stdin input.
/// This size accommodates typical input bursts and escape sequences.
const READ_BUFFER_SIZE: usize = 1024;

/// Reads terminal input events from stdin and dispatches them to a callback.
pub struct InputReader;

impl InputReader {
    /// Spawns a new input reader thread that reads stdin and invokes the callback
    /// for each parsed input event (key presses, resize events, etc.).
    ///
    /// If signal registration fails (rare, e.g., in sandboxed environments),
    /// the reader will still function for keyboard input but won't receive
    /// window resize events.
    pub fn spawn<F>(mut callback: F) -> thread::JoinHandle<()>
    where
        F: FnMut(Event) + Send + 'static,
    {
        thread::spawn(move || {
            let mut parser = Parser::new();
            let mut stdin = std::io::stdin();
            let mut buffer = [0; READ_BUFFER_SIZE];

            // Register for SIGWINCH (window resize) signals.
            // This should rarely fail in practice; if it does, we simply won't
            // receive resize events, which is acceptable degradation.
            let Ok(mut signals) = Signals::new([SIGWINCH]) else {
                // Can't receive resize events, but stdin reading still works.
                // Run in a loop that only handles stdin input.
                loop {
                    let stdin_fd = stdin.as_fd();
                    let polled = loop {
                        match tty::poll_input(stdin_fd, 20) {
                            Ok(p) => break Ok(p),
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                            Err(e) => break Err(e),
                        }
                    };

                    match polled {
                        Ok(true) => match stdin.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(n) => {
                                for item in buffer.iter().take(n) {
                                    if let Some(event) = parser.advance(*item) {
                                        callback(event);
                                    }
                                }
                            }
                            Err(_) => break,
                        },
                        Ok(false) => {
                            if let Some(evt) = parser.check_timeout() {
                                callback(evt);
                            }
                        }
                        Err(_) => break,
                    }
                }
                return;
            };

            loop {
                // 1. Check Signals (Non-blocking)
                for signal in signals.pending() {
                    if signal == SIGWINCH {
                        if let Ok((w, h)) = tty::get_window_size(stdin.as_fd()) {
                            callback(Event::Resize(w, h));
                        }
                    }
                }

                // 2. Poll with 20ms timeout (retry on EINTR)
                let stdin_fd = stdin.as_fd();
                let polled = loop {
                    match tty::poll_input(stdin_fd, 20) {
                        Ok(p) => break Ok(p),
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => break Err(e),
                    }
                };

                match polled {
                    Ok(true) => {
                        match stdin.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                for item in buffer.iter().take(n) {
                                    if let Some(event) = parser.advance(*item) {
                                        callback(event);
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    Ok(false) => {
                        // Timeout - check for incomplete sequences (like Esc)
                        if let Some(evt) = parser.check_timeout() {
                            callback(evt);
                        }
                    }
                    Err(_) => break,
                }
            }
        })
    }
}
