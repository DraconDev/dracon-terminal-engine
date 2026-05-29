#![allow(missing_docs)]

//! Terminal backend for low-level TTY operations.
//!
//! This module provides bindings to POSIX terminal ioctls for:
//! - Getting and setting terminal attributes (raw mode)
//! - Querying terminal window size
//! - Polling stdin for input
//!
//! All unsafe operations have documented safety invariants.

#[doc = "POSIX terminal ioctls (window size, raw mode, sigwinch)."]
pub mod tty;
