#![allow(missing_docs)]

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
//! - **Widgets** — `widgets::TextEditor` provides a full-featured code editor with syntax
//!   highlighting via syntect. `widgets::TextInput` is a single-line text input widget.
//! - **Integration** — [`ratatui`] bridge lets you drop in any ratatui widget.
//! - **Visuals** — `visuals::icons` for file-type icons, `visuals::osc` for clipboard,
//!   hyperlinks, bell, and notifications. `begin_sync`/`end_sync` implement
//!   terminal mode 2026 for synchronized tear-free output.
//! - **Backend** — `backend::tty` wraps low-level POSIX terminal ioctls.
//! - **System** — [`SystemMonitor`] collects CPU, memory, disk, and process
//!   metrics.
//!
//! ## Example
//!
//! ```no_run
//! use dracon_terminal_engine::framework::prelude::*;
//! use dracon_terminal_engine::framework::widget::Widget;
//! use ratatui::layout::Rect;
//!
//! App::new().unwrap()
//!     .title("My App")
//!     .fps(30)
//!     .on_tick(|ctx, _tick| {
//!         // Called every 250ms by default
//!     })
//!     .run(|ctx| {
//!         let (w, h) = ctx.compositor().size();
//!         let area = Rect::new(0, 0, w, h);
//!         let list = List::new(vec!["Item 1", "Item 2", "Item 3"]);
//!         ctx.add_plane(list.render(area));
//!     });
//! ```
//!
//! ## Version
//!
//! v27.0.5

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
#[doc = "System monitoring (CPU, memory, disk, processes)."]
pub mod system;

#[doc = "General utilities (visual width, truncate, formatting helpers)."]
pub mod utils;
#[doc = "Visual helpers: icons, OSC strings (clipboard, hyperlink, bell), sync mode 2026."]
pub mod visuals;
#[doc = "Built-in widgets (TextEditor with syntect highlighting, TextInput)."]
pub mod widgets;

pub use compositor::{Cell, Color, Compositor, Plane, Styles};
pub use core::terminal::Terminal;
pub use framework::prelude; // App, Ctx, List, HitZone, etc.
pub use input::{InputReader, Parser};
pub use system::{DiskInfo, ProcessInfo, SystemData, SystemMonitor};
