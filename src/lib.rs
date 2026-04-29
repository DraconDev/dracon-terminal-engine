#![warn(missing_docs)]

//! # Dracon Terminal Engine
//!
//! A z-indexed, event-driven terminal compositor runtime written in Rust.
//!
//! ## Architecture
//!
//! The engine is organized into several layers:
//!
//! - **Core** — [`Terminal`] wraps the terminal in raw mode with RAII cleanup.
//! - **Compositor** — [`Plane`] layers are composited via [`Compositor`] into a
//!   single frame. Supports TrueColor, style flags, opacity, and per-plane
//!   visual filters (Dim, Invert, Scanline, Pulse, Glitch).
//! - **Input** — [`InputReader`] and [`Parser`] decode SGR mouse events and
//!   extended keyboard sequences (chords, modifiers, extra buttons).
//! - **Widgets** — [`Editor`] provides a full-featured code editor with syntax
//!   highlighting via syntect. [`Input`] is a single-line text input widget.
//! - **Integration** — [`ratatui`] bridge lets you drop in any ratatui widget.
//! - **Visuals** — [`icons`] for file-type icons, [`osc`] for clipboard,
//!   hyperlinks, bell, and notifications. [`begin_sync`]/[`end_sync`] implement
//!   terminal mode 2026 for synchronized tear-free output.
//! - **Backend** — [`tty`] wraps low-level POSIX terminal ioctls.
//! - **System** — [`SystemMonitor`] collects CPU, memory, disk, and process
//!   metrics.
//!
//! ## Example
//!
//! ```no_run
//! use dracon_terminal_engine::core::terminal::Terminal;
//! use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
//!
//! let mut terminal = Terminal::new(std::io::stdout()).unwrap();
//! let mut hud = Plane::new(0, 40, 10);
//! hud.set_z_index(50);
//!
//! let cell = Cell {
//!     char: ' ',
//!     fg: Color::Rgb(0, 255, 136),
//!     bg: Color::Rgb(0, 30, 20),
//!     style: Styles::empty(),
//!     transparent: false,
//!     skip: false,
//! };
//! hud.fill(cell);
//! hud.put_str(1, 1, "SYSTEM ONLINE");
//! terminal.write_all(hud.render().as_bytes()).unwrap();
//! ```
//!
//! ## Version
//!
//! v19.2.2

#[doc = "Terminal backend (POSIX tty ioctls, raw mode setup)."]
pub mod backend;
#[doc = "Z-indexed layer compositor (Plane, Compositor, Cell, Color, Styles, filters)."]
pub mod compositor;
#[doc = "Input contract types (UiRenderer, UiRuntime traits — used internally)."]
pub mod contracts;
#[doc = "Core terminal wrapper (RAII raw mode + alt screen)."]
pub mod core;
#[doc = "Framework: App, HitZone, ScrollContainer, widgets, theme — the one-import entry point."]
pub mod framework;
#[doc = "Input reader (InputReader) + SGR mouse / chord parser."]
pub mod input;
#[doc = "Ratatui integration bridge."]
pub mod integration;
#[doc = "Layout helpers (grid, border, padding utilities)."]
pub mod layout;
pub(crate) mod system;

#[doc = "General utilities (visual width, truncate, formatting helpers)."]
pub mod utils;
#[doc = "Visual helpers: icons, OSC strings (clipboard, hyperlink, bell), sync mode 2026."]
pub mod visuals;
#[doc = "Built-in widgets (Editor with syntect highlighting, TextInput)."]
pub mod widgets;

pub use compositor::{Cell, Color, Compositor, Plane, Styles};
pub use core::terminal::Terminal;
pub use framework::prelude;
pub use input::{InputReader, Parser};