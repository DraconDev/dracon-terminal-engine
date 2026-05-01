//! Ratatui backend integration for the terminal engine.
//!
//! This module provides a [`Backend`] implementation that bridges
//! [`Ratatui`](ratatui) with the terminal engine's compositor.

use crate::compositor::engine::{map_color, Compositor};
use crate::compositor::plane::Plane;
use crate::core::terminal::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::{Position, Size};
use std::io::{self, Write};
use std::os::fd::AsFd;
use unicode_width::UnicodeWidthStr;

/// A ratatui backend that renders to the terminal compositor.
pub struct RatatuiBackend<W: io::Write + std::os::fd::AsFd> {
    inner: Terminal<W>,
    compositor: Compositor,
    last_size_check: std::time::Instant,
}

impl<W: io::Write + std::os::fd::AsFd> RatatuiBackend<W> {
    /// Creates a new `RatatuiBackend` wrapping the given writer.
    pub fn new(writer: W) -> io::Result<Self> {
        let size = crate::backend::tty::get_window_size(writer.as_fd()).unwrap_or((80, 24));
        let mut compositor = Compositor::new(size.0, size.1);

        let base_plane = Plane::new(0, size.0, size.1);
        compositor.add_plane(base_plane);

        Ok(Self {
            inner: Terminal::new(writer)?,
            compositor,
            last_size_check: std::time::Instant::now(),
        })
    }

    /// Returns a mutable reference to the underlying compositor.
    pub fn compositor_mut(&mut self) -> &mut Compositor {
        &mut self.compositor
    }
}

impl<W: io::Write + std::os::fd::AsFd> Backend for RatatuiBackend<W> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        if self.last_size_check.elapsed() > std::time::Duration::from_millis(100) {
            self.last_size_check = std::time::Instant::now();
            if let Ok((w, h)) = crate::backend::tty::get_window_size(self.inner.as_fd()) {
                let (cw, ch) = self.compositor.size();
                if w != cw || h != ch {
                    self.compositor.resize(w, h);
                    if let Some(plane) = self.compositor.planes.first_mut() {
                        *plane = Plane::new(0, w, h);
                    }
                }
            }
        }

        if let Some(plane) = self.compositor.planes.first_mut() {
            for (x, y, cell) in content {
                let fg = map_color(cell.fg);
                let bg = map_color(cell.bg);
                let mut style = crate::compositor::plane::Styles::empty();
                if cell.modifier.contains(ratatui::style::Modifier::BOLD) {
                    style.insert(crate::compositor::plane::Styles::BOLD);
                }

                let sym = cell.symbol();
                let width = sym.width();

                if width == 0 && !sym.is_empty() {
                    plane.set_skip(x, y, true);
                    continue;
                }

                if width > 0 || !sym.is_empty() || bg != crate::compositor::plane::Color::Reset {
                    plane.set_style(x, y, fg, bg, style);
                    let c = sym.chars().next().unwrap_or(' ');
                    plane.put_char(x, y, c);
                    if width > 1 {
                        for i in 1..width {
                            plane.set_skip(x + i as u16, y, true);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.inner, "\x1b[?25l")
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.inner, "\x1b[?25h")
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position { x: 0, y: 0 })
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, pos: P) -> io::Result<()> {
        let pos = pos.into();
        write!(self.inner, "\x1b[{};{}H", pos.y + 1, pos.x + 1)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.compositor.force_clear();
        write!(self.inner, "\x1b[48;2;0;0;0m\x1b[2J")
    }

    fn size(&self) -> io::Result<Size> {
        let (w, h) = crate::backend::tty::get_window_size(self.inner.as_fd())?;
        Ok(Size {
            width: w,
            height: h,
        })
    }
    fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
        let (w, h) = crate::backend::tty::get_window_size(self.inner.as_fd())?;
        Ok(ratatui::backend::WindowSize {
            columns_rows: ratatui::layout::Size {
                width: w,
                height: h,
            },
            pixels: ratatui::layout::Size {
                width: 0,
                height: 0,
            },
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.compositor.render(self.inner.inner())?;
        self.inner.flush()
    }
}
